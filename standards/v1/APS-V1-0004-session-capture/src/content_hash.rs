//! The canonical `content_hash` computation (APS-V1-0004 section 4.2.3).
//!
//! `content_hash` is the dedup identity for a session. Two independent stores
//! MUST derive the same digest for the same captured content, or the same
//! session stored in two places is two different sessions and dedup does not
//! interoperate. That makes this the most interoperability-sensitive rule in the
//! standard, which is why the implementation ships here rather than being left
//! to prose.
//!
//! # What is hashed
//!
//! Exactly three members, taken from the envelope as RECEIVED, before the store
//! sanitizes:
//!
//! ```json
//! { "raw": ..., "session_format": "<source_format>", "session_id": "<session_id>" }
//! ```
//!
//! `content_hash` is excluded by construction, since a structure containing its
//! own digest cannot be hashed. Timestamps, `origin`, `agent`, and `metadata` are
//! excluded because they describe the capture event rather than the content and
//! can differ between two captures of the same session.
//!
//! The object is serialized with RFC 8785 JSON Canonicalization Scheme, then
//! hashed with SHA-256, then rendered `sha256:<lowercase hex>`.
//!
//! # Why before sanitization
//!
//! Hashing the sanitized form would make the sanitizer ruleset part of session
//! identity: every secret-detection improvement would move the digest of content
//! that never changed. See section 3.9. The pre-sanitization value is hashed in
//! memory and never persisted (section 5.4).

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::SessionEnvelope;

/// Errors from computing a content hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentHashError {
    /// `raw` was not I-JSON compatible, so RFC 8785 cannot canonicalize it.
    ///
    /// JCS requires input serializable per I-JSON: numbers must be
    /// representable as IEEE-754 doubles, and object member names must be
    /// unique. A `raw` carrying an arbitrary-precision literal such as `1e400`
    /// has no conforming canonical form, so it has no conforming digest.
    NotCanonicalizable,
}

impl std::fmt::Display for ContentHashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentHashError::NotCanonicalizable => write!(
                f,
                "raw is not I-JSON compatible, so it has no RFC 8785 canonical form and therefore no conforming content_hash"
            ),
        }
    }
}

impl std::error::Error for ContentHashError {}

/// The hash input (section 4.2.3).
///
/// Field order here is irrelevant to the result: JCS sorts member names. It is
/// written in sorted order anyway so the code reads like the spec.
#[derive(Serialize)]
struct HashInput<'a> {
    raw: &'a serde_json::Value,
    session_format: &'a str,
    session_id: &'a str,
}

/// Compute the canonical `content_hash` for captured content.
///
/// Takes the three inputs directly so a store can hash at ingest without first
/// constructing an envelope. See [`content_hash_for`] for the envelope form.
pub fn content_hash(
    session_id: &str,
    source_format: &str,
    raw: &serde_json::Value,
) -> Result<String, ContentHashError> {
    let input = HashInput {
        raw,
        session_format: source_format,
        session_id,
    };
    let canonical = serde_json_canonicalizer::to_vec(&input)
        .map_err(|_| ContentHashError::NotCanonicalizable)?;
    let digest = Sha256::digest(&canonical);
    // Lowercase hex is REQUIRED: two implementations that agree on the digest
    // bytes but disagree on casing would produce unequal dedup keys.
    Ok(format!("sha256:{digest:x}"))
}

/// Compute the canonical `content_hash` for an envelope's captured content.
///
/// Ignores any `content_hash` already present, which is the required behaviour:
/// a store MUST NOT honour a producer-supplied value (section 4.2.3).
pub fn content_hash_for(envelope: &SessionEnvelope) -> Result<String, ContentHashError> {
    content_hash(&envelope.session_id, &envelope.source_format, &envelope.raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw_string() -> serde_json::Value {
        serde_json::Value::String("{\"a\":1}\n{\"b\":2}\n".to_string())
    }

    /// Known-answer test. The digest must be reproducible by any implementer
    /// from the spec text alone, so it is pinned here as a conformance vector.
    ///
    /// This value was computed INDEPENDENTLY of this crate, by canonicalizing
    /// the input per RFC 8785 and hashing it in a separate implementation, and
    /// only then pinned. Pinning whatever the code happened to emit would make
    /// the test circular and would prove nothing about interoperability.
    ///
    /// Canonical form of the input:
    ///
    /// ```text
    /// {"raw":"{\"a\":1}\n{\"b\":2}\n","session_format":"claude-jsonl-v1","session_id":"abc-123"}
    /// ```
    ///
    /// A change to this value is a BREAKING change to dedup across every store
    /// and every already-captured session. This test is what makes that
    /// impossible to do by accident.
    #[test]
    fn known_answer_is_pinned() {
        let hash = content_hash("abc-123", "claude-jsonl-v1", &raw_string()).unwrap();
        assert_eq!(
            hash, "sha256:08499b689727559e3b984f5cd14d91444b3a74adb5852978df8a267ecb09b77f",
            "the canonical digest is a wire-visible constant; changing it breaks \
             dedup interoperability with every already-stored session"
        );
    }

    #[test]
    fn digest_is_lowercase_hex_with_algorithm_prefix() {
        let hash = content_hash("s", "f", &raw_string()).unwrap();
        let digest = hash.strip_prefix("sha256:").expect("algorithm prefix");
        assert_eq!(digest.len(), 64);
        assert!(
            digest
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "hex must be lowercase so equal digests compare equal as strings"
        );
    }

    /// Member ordering in the source object must not change the digest: JCS
    /// sorts. This is what lets two stores with different struct layouts agree.
    #[test]
    fn member_order_of_raw_does_not_change_the_digest() {
        let a: serde_json::Value = serde_json::from_str(r#"{"b":2,"a":1}"#).unwrap();
        let b: serde_json::Value = serde_json::from_str(r#"{"a":1,"b":2}"#).unwrap();
        assert_eq!(
            content_hash("s", "f", &a).unwrap(),
            content_hash("s", "f", &b).unwrap()
        );
    }

    /// Whitespace in a STRING raw is content and must change the digest, which
    /// is exactly why a verbatim string raw supports byte-exact reconstitution.
    #[test]
    fn whitespace_in_string_raw_changes_the_digest() {
        let a = serde_json::Value::String("{\"a\":1}".into());
        let b = serde_json::Value::String("{ \"a\": 1 }".into());
        assert_ne!(
            content_hash("s", "f", &a).unwrap(),
            content_hash("s", "f", &b).unwrap()
        );
    }

    #[test]
    fn every_hashed_member_affects_the_digest() {
        let base = content_hash("s", "f", &raw_string()).unwrap();
        assert_ne!(base, content_hash("s2", "f", &raw_string()).unwrap());
        assert_ne!(base, content_hash("s", "f2", &raw_string()).unwrap());
        assert_ne!(
            base,
            content_hash("s", "f", &serde_json::Value::String("other".into())).unwrap()
        );
    }

    /// Fields outside the hash input must NOT affect the digest, or two captures
    /// of the same session would fail to deduplicate.
    #[test]
    fn capture_event_fields_do_not_affect_the_digest() {
        let json = r#"{
          "scs_version": "1.0",
          "origin": { "host": "h1", "environment": "local" },
          "agent": "ClaudeCode",
          "source_format": "claude-jsonl-v1",
          "session_id": "abc-123",
          "started_at": "2026-05-02T14:03:11Z",
          "last_activity_at": "2026-05-02T15:20:44Z",
          "raw": "transcript"
        }"#;
        let first: SessionEnvelope = serde_json::from_str(json).unwrap();

        // A second capture of the same session from a different host, at a
        // different time, with different metadata, and carrying a bogus
        // producer-supplied hash.
        let mut second = first.clone();
        second.origin.host = "a-different-machine".into();
        second.origin.environment = "vps".into();
        second.started_at = "2026-06-01T00:00:00Z".into();
        second.last_activity_at = "2026-06-01T01:00:00Z".into();
        second.agent = "SomethingElse".into();
        second.content_hash = Some("sha256:deadbeef".into());
        second.metadata = Some(crate::Metadata {
            repo: Some("owner/name".into()),
            ..Default::default()
        });

        assert_eq!(
            content_hash_for(&first).unwrap(),
            content_hash_for(&second).unwrap(),
            "the same session captured twice must deduplicate"
        );
    }

    #[test]
    fn a_producer_supplied_hash_is_ignored() {
        let json = r#"{
          "scs_version": "1.0",
          "origin": { "host": "h", "environment": "local" },
          "agent": "ClaudeCode",
          "source_format": "claude-jsonl-v1",
          "session_id": "abc-123",
          "started_at": "2026-05-02T14:03:11Z",
          "last_activity_at": "2026-05-02T15:20:44Z",
          "content_hash": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
          "raw": "transcript"
        }"#;
        let envelope: SessionEnvelope = serde_json::from_str(json).unwrap();
        let computed = content_hash_for(&envelope).unwrap();
        assert_ne!(
            Some(computed.as_str()),
            envelope.content_hash.as_deref(),
            "the store's computation must not be influenced by what the producer sent"
        );
    }

    /// The computed hash must satisfy the crate's own validator and the schema's
    /// pattern, or a store would produce envelopes its own standard rejects.
    #[test]
    fn computed_hash_passes_envelope_validation() {
        let json = r#"{
          "scs_version": "1.0",
          "origin": { "host": "h", "environment": "local" },
          "agent": "ClaudeCode",
          "source_format": "claude-jsonl-v1",
          "session_id": "abc-123",
          "started_at": "2026-05-02T14:03:11Z",
          "last_activity_at": "2026-05-02T15:20:44Z",
          "raw": "transcript"
        }"#;
        let mut envelope: SessionEnvelope = serde_json::from_str(json).unwrap();
        envelope.content_hash = Some(content_hash_for(&envelope).unwrap());
        envelope
            .validate_stored()
            .expect("a store-computed hash must be a conformant stored envelope");
    }
}
