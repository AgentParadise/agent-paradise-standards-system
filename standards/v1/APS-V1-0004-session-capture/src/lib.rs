//! Session Capture Standard (APS-V1-0004): definitional envelope types.
//!
//! This crate is DEFINITIONAL. It provides the session-envelope schema types,
//! their structural validation, the embedded JSON Schema, and the reconstitution
//! registry ([`reconstitution`]), shared by all three conformance profiles
//! (Source, Exporter, Reconstitutor). The behavioural reference implementation
//! (Source adapters, the exporter and CLI, the server-side sanitizer and
//! per-provider parsers, the Reconstitutor client) lives in the SeshMagic
//! repository, not here.
//!
//! Consumers depend on this crate rather than reimplementing the envelope, so a
//! divergence from the standard surfaces as a build or test failure instead of an
//! undetected drift. See `docs/01_spec.md` section 1.3.
//!
//! # The envelope
//!
//! A thin, universal header wrapped around a provider-native transcript that is
//! preserved verbatim in [`SessionEnvelope::raw`]. The standard never parses or
//! reshapes `raw`; it is opaque. See `docs/01_spec.md` section 4.
//!
//! Only the header fields and `raw` are required. `parent_session_id` and
//! `metadata` are optional. `metadata` is freeform and first-class for search.
//!
//! # Example
//!
//! ```
//! use session_capture::SessionEnvelope;
//!
//! let json = r#"{
//!   "scs_version": "1.0",
//!   "origin": { "host": "macbook-neural", "environment": "local" },
//!   "agent": "ClaudeCode",
//!   "source_format": "claude-jsonl-v1",
//!   "session_id": "abc-123",
//!   "started_at": "2026-05-02T14:03:11Z",
//!   "last_activity_at": "2026-05-02T15:20:44Z",
//!   "content_hash": "sha256:deadbeef",
//!   "raw": { "messages": [] }
//! }"#;
//!
//! let envelope: SessionEnvelope = serde_json::from_str(json).unwrap();
//! assert!(envelope.validate().is_ok());
//! assert_eq!(envelope.origin.environment, "local");
//! ```

use serde::{Deserialize, Serialize};

pub mod reconstitution;

/// The envelope schema version this crate implements (section 4.2).
///
/// `MAJOR.MINOR`, additive-only within a major. Note the form: it is `"1.0"`,
/// NOT a prefixed string such as `"scs/1"`, which does not match the schema's
/// `scs_version` pattern and is therefore non-conformant.
pub const SCS_VERSION: &str = "1.0";

/// The standard's JSON Schema for a session envelope, embedded verbatim.
///
/// Consumers validate against this rather than shipping their own copy, so a
/// drift from the standard is a test failure instead of an undetected
/// divergence (section 1.3).
pub const SESSION_ENVELOPE_SCHEMA: &str = include_str!("../schemas/session-envelope.schema.json");

/// Parse [`SESSION_ENVELOPE_SCHEMA`] into a [`serde_json::Value`].
///
/// # Panics
///
/// Panics if the embedded schema is not valid JSON, which would mean the
/// shipped package is corrupt. A freshness test in `tests/` covers this.
pub fn session_envelope_schema() -> serde_json::Value {
    serde_json::from_str(SESSION_ENVELOPE_SCHEMA).expect("embedded schema must be valid JSON")
}

/// A single Session Capture envelope (APS-V1-0004 section 4).
///
/// The header fields and [`raw`](SessionEnvelope::raw) are required.
/// `parent_session_id` and `metadata` are optional.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEnvelope {
    /// SemVer of the envelope schema (`MAJOR.MINOR`). Additive-only within a major.
    pub scs_version: String,
    /// Where the envelope came from. Keeps a multi-source corpus attributable.
    pub origin: Origin,
    /// The producing runtime, for example `ClaudeCode`, `Codex`, `Cursor`.
    pub agent: String,
    /// Provenance tag for `raw`: how a server-side parser should read it.
    pub source_format: String,
    /// Session identifier, stable per source. Half of the idempotency key.
    pub session_id: String,
    /// For a subagent session, the parent's id. Reserved in v1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_session_id: Option<String>,
    /// Real session start time (RFC3339), derived from the transcript.
    pub started_at: String,
    /// Real time of the last activity (RFC3339).
    pub last_activity_at: String,
    /// Content hash (`algo:hexdigest`) over the captured content, computed by
    /// the store at ingest before sanitizing (section 4.2.3).
    ///
    /// `None` while in flight: the store's value is authoritative, so producers
    /// omit it. `Some` once stored. Use [`SessionEnvelope::validate_stored`] to
    /// require it.
    ///
    /// This is a dedup identity, NOT an integrity check over the stored bytes:
    /// the stored form is sanitized and may be re-sanitized later, while this
    /// digest deliberately does not move.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    /// Freeform, non-load-bearing metadata; first-class for search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// The provider-native transcript, preserved verbatim. Opaque to the standard.
    pub raw: serde_json::Value,
}

/// Where an envelope came from (APS-V1-0004 section 4.2.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Origin {
    /// Human-meaningful host identity.
    pub host: String,
    /// Environment class: `local`, `vps`, `container`, or `workflow`.
    /// Consumers MUST tolerate unknown values (section 8.3), so this stays a
    /// `String` rather than a closed enum.
    pub environment: String,
}

/// Freeform, first-class-for-search metadata (APS-V1-0004 section 4.4).
///
/// Every field is optional. Unknown keys are preserved in [`Metadata::extra`]
/// so additive evolution does not lose data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_remote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,
    /// Original transcript path on the capturing machine, relative to the
    /// harness session root (section 4.4). Preferred by a Reconstitutor over a
    /// derived path.
    ///
    /// UNTRUSTED producer input used to choose a filesystem write location.
    /// Resolve it with [`reconstitution::resolve_within_root`], which performs
    /// both the lexical and the filesystem-aware containment checks;
    /// [`reconstitution::is_safe_source_path`] alone cannot see symlinks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    /// Unknown metadata keys, tolerated per section 8.3.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// A structural validation error for an envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// A required header field was empty.
    EmptyField(&'static str),
    /// `content_hash` did not match the `algo:hexdigest` shape.
    MalformedContentHash,
    /// A stored envelope had no `content_hash` (section 4.2.3).
    ///
    /// Legal in flight, non-conformant once persisted: the store must compute
    /// it during ingest.
    MissingStoredContentHash,
    /// `raw` was a scalar (number, boolean, or null) rather than an object,
    /// array, or string (section 4.3).
    ///
    /// A scalar cannot be a transcript. The schema restricts the same three
    /// types; this keeps the Rust type domain from being wider than the schema's.
    InvalidRawType,
    /// A timestamp was not RFC3339 (section 4.2.2).
    ///
    /// Carries the field name, since both `started_at` and `last_activity_at`
    /// are checked.
    MalformedTimestamp(&'static str),
    /// `scs_version` was not `MAJOR.MINOR` (section 4.2).
    ///
    /// This exists to catch a specific real drift: a producer emitting a
    /// prefixed form such as `"scs/1"` instead of `"1.0"`. That value does not
    /// match the schema's `scs_version` pattern, so every envelope carrying it
    /// is non-conformant.
    MalformedScsVersion,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyField(name) => {
                write!(f, "required field `{name}` must not be empty")
            }
            ValidationError::MalformedContentHash => {
                write!(f, "content_hash must be of the form `algo:hexdigest`")
            }
            ValidationError::InvalidRawType => {
                write!(
                    f,
                    "raw must be an object, array, or string; a scalar is not a transcript"
                )
            }
            ValidationError::MalformedTimestamp(name) => {
                write!(
                    f,
                    "`{name}` must be an RFC3339 date-time, for example 2026-05-02T14:03:11Z"
                )
            }
            ValidationError::MissingStoredContentHash => {
                write!(
                    f,
                    "a stored envelope must carry a content_hash computed by the store over the sanitized form"
                )
            }
            ValidationError::MalformedScsVersion => {
                write!(
                    f,
                    "scs_version must be `MAJOR.MINOR` (for example `1.0`), not a prefixed form such as `scs/1`"
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

impl SessionEnvelope {
    /// Structurally validate the envelope beyond what deserialization enforces.
    ///
    /// Deserialization already guarantees the required fields are present and
    /// typed. This checks that required string fields are non-empty and that
    /// `content_hash` follows the `algo:hexdigest` shape. It deliberately does
    /// NOT inspect `raw`, which is opaque to the standard.
    pub fn validate(&self) -> Result<(), ValidationError> {
        for (name, value) in [
            ("scs_version", &self.scs_version),
            ("agent", &self.agent),
            ("source_format", &self.source_format),
            ("session_id", &self.session_id),
            ("started_at", &self.started_at),
            ("last_activity_at", &self.last_activity_at),
            ("origin.host", &self.origin.host),
            ("origin.environment", &self.origin.environment),
        ] {
            if value.trim().is_empty() {
                return Err(ValidationError::EmptyField(name));
            }
        }
        if !is_valid_scs_version(&self.scs_version) {
            return Err(ValidationError::MalformedScsVersion);
        }
        for (name, value) in [
            ("started_at", &self.started_at),
            ("last_activity_at", &self.last_activity_at),
        ] {
            if !is_rfc3339(value) {
                return Err(ValidationError::MalformedTimestamp(name));
            }
        }
        if !matches!(
            self.raw,
            serde_json::Value::Object(_)
                | serde_json::Value::Array(_)
                | serde_json::Value::String(_)
        ) {
            return Err(ValidationError::InvalidRawType);
        }
        if let Some(hash) = &self.content_hash
            && !is_valid_content_hash(hash)
        {
            return Err(ValidationError::MalformedContentHash);
        }
        Ok(())
    }

    /// Validate an envelope that has been persisted by a store (section 4.2.3).
    ///
    /// Everything [`validate`](SessionEnvelope::validate) checks, plus the
    /// requirement that `content_hash` is present. A store MUST populate it
    /// during ingest, so an envelope read back without one is non-conformant.
    pub fn validate_stored(&self) -> Result<(), ValidationError> {
        self.validate()?;
        if self.content_hash.is_none() {
            return Err(ValidationError::MissingStoredContentHash);
        }
        Ok(())
    }

    /// The idempotency key: `(session_id, content_hash)` (section 5.3).
    ///
    /// `None` for an in-flight envelope, whose hash the store has not yet
    /// computed. Dedup is therefore a store-side operation (4.2.3).
    pub fn idempotency_key(&self) -> Option<(&str, &str)> {
        self.content_hash
            .as_deref()
            .map(|hash| (self.session_id.as_str(), hash))
    }
}

/// Check that a timestamp is RFC3339 (section 4.2.2).
///
/// Validates shape and calendar-plausible ranges, not exact month lengths or
/// leap years: the point is to reject capture-time placeholders and obviously
/// malformed values, which is what the schema's `format: date-time` also aims
/// at. Full date arithmetic would require a date dependency this definitional
/// crate deliberately avoids.
fn is_rfc3339(value: &str) -> bool {
    let bytes = value.as_bytes();
    // Minimum: 1970-01-01T00:00:00Z
    if bytes.len() < 20 {
        return false;
    }
    let digits = |range: std::ops::Range<usize>| bytes[range].iter().all(u8::is_ascii_digit);
    if !(digits(0..4) && bytes[4] == b'-' && digits(5..7) && bytes[7] == b'-' && digits(8..10)) {
        return false;
    }
    if bytes[10] != b'T' && bytes[10] != b't' {
        return false;
    }
    if !(digits(11..13)
        && bytes[13] == b':'
        && digits(14..16)
        && bytes[16] == b':'
        && digits(17..19))
    {
        return false;
    }
    let num = |range: std::ops::Range<usize>| value[range].parse::<u32>().unwrap_or(u32::MAX);
    if !(1..=12).contains(&num(5..7)) || !(1..=31).contains(&num(8..10)) {
        return false;
    }
    // 24:00:00 is not permitted by RFC3339; leap second 60 is.
    if num(11..13) > 23 || num(14..16) > 59 || num(17..19) > 60 {
        return false;
    }

    let mut rest = &value[19..];
    // Optional fractional seconds.
    if let Some(stripped) = rest.strip_prefix('.') {
        let frac_len = stripped.chars().take_while(char::is_ascii_digit).count();
        if frac_len == 0 {
            return false;
        }
        rest = &stripped[frac_len..];
    }
    // Offset is REQUIRED: `Z`, or +/-HH:MM.
    if rest == "Z" || rest == "z" {
        return true;
    }
    let offset = rest.as_bytes();
    if offset.len() != 6 || (offset[0] != b'+' && offset[0] != b'-') || offset[3] != b':' {
        return false;
    }
    if !offset[1..3].iter().all(u8::is_ascii_digit) || !offset[4..6].iter().all(u8::is_ascii_digit)
    {
        return false;
    }
    rest[1..3].parse::<u32>().unwrap_or(u32::MAX) <= 23
        && rest[4..6].parse::<u32>().unwrap_or(u32::MAX) <= 59
}

/// Check that `scs_version` is `MAJOR.MINOR`, matching the schema's pattern
/// `^[0-9]+\.[0-9]+$`.
fn is_valid_scs_version(version: &str) -> bool {
    match version.split_once('.') {
        Some((major, minor)) => {
            !major.is_empty()
                && !minor.is_empty()
                && major.chars().all(|c| c.is_ascii_digit())
                && minor.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

/// Check that a content hash is of the form `algo:hexdigest`.
fn is_valid_content_hash(hash: &str) -> bool {
    match hash.split_once(':') {
        Some((algo, digest)) => {
            !algo.is_empty()
                && algo
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                && !digest.is_empty()
                && digest.chars().all(|c| c.is_ascii_hexdigit())
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_json() -> &'static str {
        r#"{
          "scs_version": "1.0",
          "origin": { "host": "macbook-neural", "environment": "local" },
          "agent": "ClaudeCode",
          "source_format": "claude-jsonl-v1",
          "session_id": "abc-123",
          "parent_session_id": null,
          "started_at": "2026-05-02T14:03:11Z",
          "last_activity_at": "2026-05-02T15:20:44Z",
          "content_hash": "sha256:deadbeef01",
          "metadata": { "repo": "owner/name", "tags": ["a", "b"], "message_count": 42 },
          "raw": { "messages": [] }
        }"#
    }

    #[test]
    fn parses_and_validates_a_full_envelope() {
        let env: SessionEnvelope = serde_json::from_str(valid_json()).unwrap();
        assert!(env.validate().is_ok());
        assert_eq!(
            env.idempotency_key(),
            Some(("abc-123", "sha256:deadbeef01"))
        );
        let md = env.metadata.unwrap();
        assert_eq!(md.repo.as_deref(), Some("owner/name"));
        assert_eq!(md.tags, vec!["a", "b"]);
        assert_eq!(md.message_count, Some(42));
    }

    #[test]
    fn minimal_envelope_without_optional_fields_is_valid() {
        let json = r#"{
          "scs_version": "1.0",
          "origin": { "host": "h", "environment": "vps" },
          "agent": "Codex",
          "source_format": "codex-turns-v1",
          "session_id": "s1",
          "started_at": "2026-05-02T14:03:11Z",
          "last_activity_at": "2026-05-02T15:20:44Z",
          "content_hash": "sha256:00ff",
          "raw": "opaque string transcript"
        }"#;
        let env: SessionEnvelope = serde_json::from_str(json).unwrap();
        assert!(env.validate().is_ok());
        assert!(env.metadata.is_none());
        assert!(env.parent_session_id.is_none());
    }

    #[test]
    fn raw_is_preserved_verbatim_and_untouched() {
        let env: SessionEnvelope = serde_json::from_str(valid_json()).unwrap();
        // The standard never reshapes `raw`; it round-trips as-is.
        assert_eq!(env.raw, serde_json::json!({ "messages": [] }));
    }

    #[test]
    fn unknown_metadata_keys_are_tolerated() {
        let json = r#"{
          "scs_version": "1.0",
          "origin": { "host": "h", "environment": "container" },
          "agent": "Cursor",
          "source_format": "cursor-bubbles-v1",
          "session_id": "s2",
          "started_at": "2026-05-02T14:03:11Z",
          "last_activity_at": "2026-05-02T15:20:44Z",
          "content_hash": "sha256:abcd",
          "metadata": { "repo": "o/n", "future_field": 123 },
          "raw": {}
        }"#;
        let env: SessionEnvelope = serde_json::from_str(json).unwrap();
        let md = env.metadata.unwrap();
        assert_eq!(md.extra.get("future_field"), Some(&serde_json::json!(123)));
    }

    #[test]
    fn empty_required_field_is_rejected() {
        let mut env: SessionEnvelope = serde_json::from_str(valid_json()).unwrap();
        env.agent = String::new();
        assert_eq!(env.validate(), Err(ValidationError::EmptyField("agent")));
    }

    #[test]
    fn the_shipped_scs_version_constant_is_conformant() {
        assert_eq!(SCS_VERSION, "1.0");
        assert!(is_valid_scs_version(SCS_VERSION));
    }

    #[test]
    fn prefixed_scs_version_is_rejected() {
        let mut env: SessionEnvelope = serde_json::from_str(valid_json()).unwrap();
        // The exact drift this catches: a producer emitting `scs/1`.
        env.scs_version = "scs/1".to_string();
        assert_eq!(env.validate(), Err(ValidationError::MalformedScsVersion));

        for bad in ["1", "v1.0", "1.0.0", "1.", ".0"] {
            env.scs_version = bad.to_string();
            assert_eq!(
                env.validate(),
                Err(ValidationError::MalformedScsVersion),
                "must reject scs_version {bad:?}"
            );
        }

        // An empty version is caught by the earlier emptiness check, which is a
        // more precise diagnostic than "malformed".
        env.scs_version = String::new();
        assert_eq!(
            env.validate(),
            Err(ValidationError::EmptyField("scs_version"))
        );
    }

    #[test]
    fn source_path_round_trips_through_metadata() {
        let json = r#"{
          "scs_version": "1.0",
          "origin": { "host": "h", "environment": "local" },
          "agent": "Codex",
          "source_format": "codex-turns-v1",
          "session_id": "s1",
          "started_at": "2026-05-02T14:03:11Z",
          "last_activity_at": "2026-05-02T15:20:44Z",
          "content_hash": "sha256:00ff",
          "metadata": { "source_path": "2026/06/04/rollout-x.jsonl" },
          "raw": {}
        }"#;
        let env: SessionEnvelope = serde_json::from_str(json).unwrap();
        let md = env.metadata.unwrap();
        assert_eq!(
            md.source_path.as_deref(),
            Some("2026/06/04/rollout-x.jsonl")
        );
    }

    #[test]
    fn malformed_content_hash_is_rejected() {
        let mut env: SessionEnvelope = serde_json::from_str(valid_json()).unwrap();
        env.content_hash = Some("not-a-hash".to_string());
        assert_eq!(env.validate(), Err(ValidationError::MalformedContentHash));

        env.content_hash = Some("sha256:xyz".to_string()); // non-hex digest
        assert_eq!(env.validate(), Err(ValidationError::MalformedContentHash));
    }

    #[test]
    fn in_flight_envelope_omits_the_hash_but_a_stored_one_requires_it() {
        let mut env: SessionEnvelope = serde_json::from_str(valid_json()).unwrap();

        // In flight: the producer cannot compute the hash (section 4.2.3).
        env.content_hash = None;
        env.validate()
            .expect("an in-flight envelope without a hash is conformant");
        assert_eq!(
            env.idempotency_key(),
            None,
            "no hash means no dedup key yet"
        );
        assert_eq!(
            env.validate_stored(),
            Err(ValidationError::MissingStoredContentHash),
            "but a stored envelope must carry one"
        );

        // Stored: the store has computed it over the sanitized form.
        env.content_hash = Some("sha256:deadbeef01".to_string());
        env.validate_stored()
            .expect("a stored envelope with a hash is conformant");
    }
}
