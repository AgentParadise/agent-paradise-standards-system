//! Wire messages for optional inventory replication.
use super::{InventoryRecord, InventoryRevision, InventoryValidationError};
use serde::{Deserialize, Serialize};

/// Durable acknowledgement of a qualified capture, independent of inventory publication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct CaptureReceipt {
    pub storage_key: String,
    pub content_hash: String,
    pub stored_content_hash: String,
    pub duplicate: bool,
}

impl CaptureReceipt {
    /// Clients must bind an acknowledgement to the exact submitted identity and
    /// original-content hash before retiring durable work.
    pub fn validates_capture(
        &self,
        identity: &super::QualifiedTranscript,
        content_hash: &str,
    ) -> bool {
        self.storage_key == identity.storage_key()
            && self.content_hash == content_hash
            && [&self.content_hash, &self.stored_content_hash]
                .iter()
                .all(|hash| {
                    hash.strip_prefix("sha256:").is_some_and(|hex| {
                        hex.len() == 64
                            && hex
                                .bytes()
                                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    })
                })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct InventoryReceipt {
    pub duplicate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct InventoryManifestBatch {
    pub revision: InventoryRevision,
    pub start: i64,
    pub record_ids: Vec<String>,
}
impl InventoryManifestBatch {
    pub fn validate(&self) -> Result<(), InventoryValidationError> {
        self.revision.validate()?;
        if self.start < 0
            || self.record_ids.is_empty()
            || self.record_ids.len() > 500
            || self
                .start
                .checked_add(self.record_ids.len() as i64)
                .is_none_or(|end| end > self.revision.expected_record_count)
            || self
                .record_ids
                .iter()
                .any(|id| id.trim().is_empty() || id.contains('\0') || id.chars().count() > 2048)
        {
            return Err(InventoryValidationError::Invalid(
                "invalid manifest batch bounds or record identifiers".into(),
            ));
        }
        Ok(())
    }
}

/// Durable exporter input, shared by producers and transport implementations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(
    tag = "operation",
    content = "body",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum InventoryOperation {
    Record(InventoryRecord),
    Stage(InventoryRevision),
    Manifest(InventoryManifestBatch),
    Publish(InventoryRevision),
}
impl InventoryOperation {
    pub fn validate(&self) -> Result<(), InventoryValidationError> {
        match self {
            Self::Record(record) => record.validate(),
            Self::Stage(revision) | Self::Publish(revision) => revision.validate(),
            Self::Manifest(batch) => batch.validate(),
        }
    }
}
