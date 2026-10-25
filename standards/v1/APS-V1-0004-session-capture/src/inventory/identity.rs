//! Qualified identities preserve native IDs while separating installations and harnesses.
//!
//! These are lookup keys, never authorization credentials. Legacy bare-ID callers
//! must resolve uniqueness before reading a qualified record.

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidIdentifier {
    limit: usize,
}

fn validate(value: &str, limit: usize) -> Result<(), InvalidIdentifier> {
    if value.trim().is_empty() || value.contains('\0') || value.chars().count() > limit {
        return Err(InvalidIdentifier { limit });
    }
    Ok(())
}

fn routing<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let value = String::deserialize(deserializer)?;
    validate(&value, 128).map_err(serde::de::Error::custom)?;
    Ok(value)
}

fn native<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let value = String::deserialize(deserializer)?;
    validate(&value, 2048).map_err(serde::de::Error::custom)?;
    Ok(value)
}

/// A workflow execution in one durable source installation.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualifiedRun {
    #[serde(deserialize_with = "routing")]
    source_instance_id: String,
    #[serde(deserialize_with = "routing")]
    execution_id: String,
}

impl QualifiedRun {
    pub fn new(
        source_instance_id: String,
        execution_id: String,
    ) -> Result<Self, InvalidIdentifier> {
        validate(&source_instance_id, 128)?;
        validate(&execution_id, 128)?;
        Ok(Self {
            source_instance_id,
            execution_id,
        })
    }

    pub fn source_instance_id(&self) -> &str {
        &self.source_instance_id
    }
    pub fn execution_id(&self) -> &str {
        &self.execution_id
    }
}

/// Native transcript identity, independent of body revisions or run memberships.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualifiedTranscript {
    #[serde(deserialize_with = "routing")]
    source_instance_id: String,
    #[serde(deserialize_with = "native")]
    harness: String,
    #[serde(deserialize_with = "native")]
    native_session_id: String,
}

impl QualifiedTranscript {
    pub fn new(
        source_instance_id: String,
        harness: String,
        native_session_id: String,
    ) -> Result<Self, InvalidIdentifier> {
        validate(&source_instance_id, 128)?;
        validate(&harness, 2048)?;
        validate(&native_session_id, 2048)?;
        Ok(Self {
            source_instance_id,
            harness,
            native_session_id,
        })
    }

    pub fn source_instance_id(&self) -> &str {
        &self.source_instance_id
    }
    pub fn harness(&self) -> &str {
        &self.harness
    }
    pub fn native_session_id(&self) -> &str {
        &self.native_session_id
    }

    /// Opaque storage locator, never an envelope ID or authorization credential.
    /// Stores must also retain and compare the original qualified tuple.
    pub fn storage_key(&self) -> String {
        let mut hash = Sha256::new();
        hash.update(b"session-inventory/1:transcript\0");
        for value in [
            &self.source_instance_id,
            &self.harness,
            &self.native_session_id,
        ] {
            hash.update((value.len() as u64).to_be_bytes());
            hash.update(value.as_bytes());
        }
        format!("qts1:{:x}", hash.finalize())
    }
}

impl std::fmt::Display for InvalidIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "identifier must be nonblank, NUL-free, and at most {} characters",
            self.limit
        )
    }
}
impl std::error::Error for InvalidIdentifier {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn collisions_across_installations_and_harnesses_remain_distinct() {
        let keys: HashSet<_> = [("one", "claude"), ("two", "claude"), ("one", "codex")]
            .into_iter()
            .map(|(source, harness)| {
                QualifiedTranscript::new(source.into(), harness.into(), "same-native-id".into())
                    .unwrap()
            })
            .collect();
        assert_eq!(keys.len(), 3);
        assert_eq!(
            QualifiedRun::new("one".into(), "run".into())
                .unwrap()
                .execution_id(),
            "run"
        );
        assert_ne!(
            QualifiedRun::new("one".into(), "run".into()),
            QualifiedRun::new("two".into(), "run".into())
        );
    }

    #[test]
    fn wire_round_trip_preserves_native_spelling_and_unicode() {
        let original = QualifiedTranscript::new(
            "source".into(),
            "harness".into(),
            " ../native/雪 %2F ".into(),
        )
        .unwrap();
        let restored: QualifiedTranscript =
            serde_json::from_str(&serde_json::to_string(&original).unwrap()).unwrap();
        assert_eq!(restored, original);
        assert_eq!(restored.native_session_id(), " ../native/雪 %2F ");
        assert_eq!(restored.source_instance_id(), "source");
        assert_eq!(restored.harness(), "harness");
    }

    #[test]
    fn invalid_identifiers_cannot_bypass_constructors_through_json() {
        for invalid in [
            "".to_string(),
            " \n".to_string(),
            "a\0b".to_string(),
            "雪".repeat(129),
        ] {
            assert!(QualifiedRun::new(invalid.clone(), "run".into()).is_err());
            let wire = serde_json::json!({"source_instance_id": invalid, "execution_id": "run"});
            assert!(serde_json::from_value::<QualifiedRun>(wire).is_err());
        }
        for invalid in ["".to_string(), "a\0b".to_string(), "a".repeat(2049)] {
            let wire = serde_json::json!({"source_instance_id": "source", "harness": "codex", "native_session_id": invalid});
            assert!(serde_json::from_value::<QualifiedTranscript>(wire).is_err());
        }
        let wire = serde_json::json!({"source_instance_id": "source", "execution_id": "run", "extra": true});
        assert!(serde_json::from_value::<QualifiedRun>(wire).is_err());
    }
}
