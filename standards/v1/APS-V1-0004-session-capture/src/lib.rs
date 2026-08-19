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
//! An envelope as a producer emits it: no `content_hash` (the store computes it
//! at ingest, section 4.2.3), and `raw` as the transcript's verbatim bytes, which
//! is what a reconstitutable `source_format` requires (section 4.3.1).
//!
//! ```
//! use session_capture::{SessionEnvelope, content_hash_for};
//!
//! let json = r#"{
//!   "scs_version": "1.0",
//!   "origin": { "host": "macbook-neural", "environment": "local" },
//!   "agent": "ClaudeCode",
//!   "source_format": "claude-jsonl-v1",
//!   "session_id": "019973e4-58a9-7b83-9f21-2b6c4d0a1e77",
//!   "started_at": "2026-05-02T14:03:11Z",
//!   "last_activity_at": "2026-05-02T15:20:44Z",
//!   "raw": "{\"type\":\"user\",\"message\":\"hello\"}\n"
//! }"#;
//!
//! let mut envelope: SessionEnvelope = serde_json::from_str(json).unwrap();
//! envelope.validate().unwrap();
//! assert_eq!(envelope.origin.environment, "local");
//!
//! // In flight there is no dedup key yet.
//! assert!(envelope.idempotency_key().is_none());
//!
//! // What a store does at ingest, before sanitizing.
//! envelope.content_hash = Some(content_hash_for(&envelope).unwrap());
//! envelope.validate_stored().unwrap();
//! ```

use serde::{Deserialize, Serialize};

pub mod cli;
pub mod content_hash;
pub mod reconstitution;

pub use content_hash::{ContentHashError, content_hash_for};

/// Standard identifier.
pub const ID: &str = "APS-V1-0004";

/// CLI dispatch slug.
pub const SLUG: &str = "session-capture";

/// Human-readable name.
pub const NAME: &str = "Session Capture Standard";

/// Short description.
pub const DESCRIPTION: &str = "One capture contract so agent sessions from any runtime back up uniformly, stay searchable, and can be resumed on another machine";

/// Version of this standard, kept in step with `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Register this standard with a composing CLI binary (APS-V1-0000.DI01).
pub fn register(registry: &mut dyn apss_core::registry::StandardRegistry) {
    registry.register(
        apss_core::registry::RegisteredStandard {
            id: ID.to_string(),
            slug: SLUG.to_string(),
            name: NAME.to_string(),
            description: DESCRIPTION.to_string(),
            version: VERSION.to_string(),
            commands: cli::COMMAND_NAMES.iter().map(|s| s.to_string()).collect(),
        },
        Box::new(cli::SessionCaptureCommandHandler::new()),
    );
}

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
///
/// `#[non_exhaustive]`: construct with [`Origin::new`] and the `with_*`
/// builders rather than a struct literal. Section 8 lets this standard add
/// OPTIONAL `origin` fields in a minor version, and a public struct whose
/// literal construction is exhaustive turns every such addition into a
/// source-breaking change for every downstream crate. Taking that break once,
/// here in 2.0.0, is what keeps the next optional field additive in Rust as
/// well as on the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Origin {
    /// Human-meaningful host identity.
    pub host: String,
    /// Environment class: `local`, `vps`, `container`, or `workflow`.
    /// Consumers MUST tolerate unknown values (section 8.3), so this stays a
    /// `String` rather than a closed enum.
    ///
    /// This is the CLASS of runtime, not which deployment it was. For that,
    /// see [`Origin::deployment`].
    pub environment: String,
    /// OPTIONAL deployment identity: which concrete deployment produced the
    /// session. Convention is `<app>__<tier>` (double underscore), e.g.
    /// `syntropic137__dev`, so one field carries both and a consumer MAY split
    /// on the first `__` to roll up app -> tier -> host.
    ///
    /// Absent means a single deployment with no tier, which is the common
    /// local case. Added in 2.0.0; consumers on 1.x ignore it on the wire (section 8.3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment: Option<String>,
}

impl Origin {
    /// A new origin with the two REQUIRED fields (section 4.2.1).
    ///
    /// `environment` is the CLASS of runtime (`local`, `vps`, `container`,
    /// `workflow`), NOT which deployment produced the session. Use
    /// [`Origin::with_deployment`] for that.
    pub fn new(host: impl Into<String>, environment: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            environment: environment.into(),
            deployment: None,
        }
    }

    /// Set the OPTIONAL deployment identity, conventionally `<app>__<tier>`.
    #[must_use]
    pub fn with_deployment(mut self, deployment: impl Into<String>) -> Self {
        self.deployment = Some(deployment.into());
        self
    }

    /// The application segment of [`Origin::deployment`]: everything before the
    /// FIRST `__`, or the whole value when it carries none.
    ///
    /// Returns `None` only when `deployment` is absent, so a caller can
    /// distinguish "no deployment stamped" from "deployment with no tier".
    #[must_use]
    pub fn deployment_app(&self) -> Option<&str> {
        self.deployment
            .as_deref()
            .map(|d| d.split_once("__").map_or(d, |(app, _)| app))
    }

    /// The tier segment of [`Origin::deployment`]: everything after the FIRST
    /// `__`. `None` when `deployment` is absent or carries no `__`.
    #[must_use]
    pub fn deployment_tier(&self) -> Option<&str> {
        self.deployment
            .as_deref()
            .and_then(|d| d.split_once("__"))
            .map(|(_, tier)| tier)
    }
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
                    "a stored envelope must carry a content_hash computed by the store over the captured content"
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

/// Days in a month, accounting for leap years per the proleptic Gregorian rule.
fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            // `%` rather than `is_multiple_of`, which is newer than the
            // workspace MSRV of 1.85.
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Check that a timestamp is RFC3339 (section 4.2.2).
///
/// Validates the full grammar including real month lengths and leap years, so
/// `2026-02-31T12:00:00Z` is rejected rather than merely being implausible.
/// Second 60 is accepted only at `23:59:60` in the value's own offset, which is
/// where RFC3339 permits a leap second.
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
    let (year, month, day) = (num(0..4), num(5..7), num(8..10));
    if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
        return false;
    }
    // 24:00:00 is not permitted by RFC3339. Second 60 is a leap second, valid
    // only at the final second of a day (23:59:60 in the stated offset).
    let (hour, minute, second) = (num(11..13), num(14..16), num(17..19));
    if hour > 23 || minute > 59 || second > 60 {
        return false;
    }
    // A leap second occurs at 23:59:60 UTC, which in a non-zero offset is written
    // as the offset-equivalent local time. RFC3339 section 5.8 gives
    // `1990-12-31T15:59:60-08:00` as a valid example, so requiring local 23:59
    // would reject conformant timestamps. Defer the check until the offset is
    // known, below.
    let is_leap_second = second == 60;

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
    let (offset_hours, offset_minutes, offset_sign) = if rest == "Z" || rest == "z" {
        (0i32, 0i32, 1i32)
    } else {
        let offset = rest.as_bytes();
        if offset.len() != 6 || (offset[0] != b'+' && offset[0] != b'-') || offset[3] != b':' {
            return false;
        }
        if !offset[1..3].iter().all(u8::is_ascii_digit)
            || !offset[4..6].iter().all(u8::is_ascii_digit)
        {
            return false;
        }
        let hours = rest[1..3].parse::<i32>().unwrap_or(i32::MAX);
        let minutes = rest[4..6].parse::<i32>().unwrap_or(i32::MAX);
        if hours > 23 || minutes > 59 {
            return false;
        }
        (hours, minutes, if offset[0] == b'-' { -1 } else { 1 })
    };

    if is_leap_second {
        // Convert the stated local time to UTC minutes-of-day; a leap second is
        // only valid at 23:59:60 UTC.
        let local_minutes = (hour as i32) * 60 + minute as i32;
        let offset_total = offset_sign * (offset_hours * 60 + offset_minutes);
        let utc_minutes = (local_minutes - offset_total).rem_euclid(24 * 60);
        if utc_minutes != 23 * 60 + 59 {
            return false;
        }
    }
    true
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
    // Exactly `sha256:` plus 64 lowercase hex digits. Within scs_version 1.x the
    // algorithm is fixed (section 4.2.3): allowing others would let two stores
    // produce different identities for identical content. Casing is fixed for
    // the same reason, since dedup compares these as strings.
    let Some(digest) = hash.strip_prefix("sha256:") else {
        return false;
    };
    digest.len() == 64
        && digest
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

#[cfg(test)]
mod tests {
    use super::Origin;

    /// The `<app>__<tier>` split is a documented convention that two
    /// independent implementations must agree on, so its edges are pinned here
    /// rather than left to each consumer's `split` call.
    #[test]
    fn deployment_split_is_unambiguous_at_every_edge() {
        // Absent: distinguishable from "present but no tier".
        let bare = Origin::new("host-a", "local");
        assert_eq!(bare.deployment_app(), None);
        assert_eq!(bare.deployment_tier(), None);

        // No `__`: the whole label is the app, and it is conformant.
        let flat = Origin::new("host-a", "local").with_deployment("laptop");
        assert_eq!(flat.deployment_app(), Some("laptop"));
        assert_eq!(flat.deployment_tier(), None);

        // The ordinary case.
        let normal = Origin::new("ws-1", "workflow").with_deployment("syntropic137__dev");
        assert_eq!(normal.deployment_app(), Some("syntropic137"));
        assert_eq!(normal.deployment_tier(), Some("dev"));

        // Multiple `__`: split on the FIRST, so the tier keeps the remainder
        // rather than being silently truncated.
        let multi = Origin::new("ws-1", "workflow").with_deployment("app__dev__eu");
        assert_eq!(multi.deployment_app(), Some("app"));
        assert_eq!(multi.deployment_tier(), Some("dev__eu"));

        // Leading `__`: empty app segment. Preserved, not coerced - a consumer
        // that wants to reject it can, but the parse must be predictable.
        let leading = Origin::new("ws-1", "workflow").with_deployment("__dev");
        assert_eq!(leading.deployment_app(), Some(""));
        assert_eq!(leading.deployment_tier(), Some("dev"));

        // Trailing `__`: empty tier segment, likewise preserved.
        let trailing = Origin::new("ws-1", "workflow").with_deployment("app__");
        assert_eq!(trailing.deployment_app(), Some("app"));
        assert_eq!(trailing.deployment_tier(), Some(""));
    }

    /// `environment` carries the runtime CLASS and `deployment` the identity.
    /// Conflating them is the drift this field exists to correct, so the two
    /// stay independent.
    #[test]
    fn environment_and_deployment_are_independent() {
        let o = Origin::new("ws-1", "workflow").with_deployment("syntropic137__prod");
        assert_eq!(o.environment, "workflow");
        assert_eq!(o.deployment.as_deref(), Some("syntropic137__prod"));
    }

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
          "content_hash": "sha256:deadbeef01000000000000000000000000000000000000000000000000000000",
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
            Some((
                "abc-123",
                "sha256:deadbeef01000000000000000000000000000000000000000000000000000000"
            ))
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
          "content_hash": "sha256:00ff000000000000000000000000000000000000000000000000000000000000",
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
          "content_hash": "sha256:abcd000000000000000000000000000000000000000000000000000000000000",
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
          "content_hash": "sha256:00ff000000000000000000000000000000000000000000000000000000000000",
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
    fn timestamps_are_validated_against_the_real_calendar() {
        let mut env: SessionEnvelope = serde_json::from_str(valid_json()).unwrap();
        for good in [
            "2026-05-02T14:03:11Z",
            "2026-05-02t14:03:11z",
            "2024-02-29T00:00:00Z",      // leap year
            "2026-05-02T14:03:11.123Z",  // fractional seconds
            "2026-05-02T14:03:11+02:00", // offset
            "2026-05-02T14:03:11-08:00",
            "2016-12-31T23:59:60Z", // leap second at end of UTC day
            // RFC3339 section 5.8's own example: the same leap-second instant
            // expressed in a non-zero offset. Requiring local 23:59 would
            // wrongly reject this.
            "1990-12-31T15:59:60-08:00",
            "1990-12-31T23:59:60+00:00",
        ] {
            env.started_at = good.to_string();
            assert!(env.validate().is_ok(), "must accept RFC3339 {good}");
        }
        for bad in [
            "2026-02-31T12:00:00Z",      // February has no 31st
            "2025-02-29T12:00:00Z",      // 2025 is not a leap year
            "1900-02-29T12:00:00Z",      // century non-leap year
            "2026-04-31T12:00:00Z",      // April has 30 days
            "2026-01-01T12:00:60Z",      // leap second not at end of UTC day
            "1990-12-31T15:59:60-07:00", // offset does not land on 23:59 UTC
            "2026-01-01T24:00:00Z",      // hour 24 is not RFC3339
            "2026-13-01T12:00:00Z",      // month 13
            "2026-00-01T12:00:00Z",      // month 0
            "2026-01-00T12:00:00Z",      // day 0
            "2026-05-02 14:03:11Z",      // space instead of T
            "2026-05-02T14:03:11",       // offset is required
            "2026-05-02T14:03:11.Z",     // empty fraction
            "2026-05-02T14:03:11+2:00",
            "2026-05-02T14:03:11+02:60",
            "not-a-timestamp",
        ] {
            env.started_at = bad.to_string();
            assert_eq!(
                env.validate(),
                Err(ValidationError::MalformedTimestamp("started_at")),
                "must reject non-RFC3339 {bad}"
            );
        }
    }

    #[test]
    fn scalar_raw_is_rejected() {
        let mut env: SessionEnvelope = serde_json::from_str(valid_json()).unwrap();
        for scalar in [
            serde_json::json!(42),
            serde_json::json!(true),
            serde_json::Value::Null,
        ] {
            env.raw = scalar.clone();
            assert_eq!(
                env.validate(),
                Err(ValidationError::InvalidRawType),
                "a scalar is not a transcript: {scalar}"
            );
        }
        for ok in [
            serde_json::json!({}),
            serde_json::json!([]),
            serde_json::Value::String(String::new()),
        ] {
            env.raw = ok;
            assert!(env.validate().is_ok());
        }
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

        // Stored: the store has computed it over the captured content.
        env.content_hash = Some(
            "sha256:deadbeef01000000000000000000000000000000000000000000000000000000".to_string(),
        );
        env.validate_stored()
            .expect("a stored envelope with a hash is conformant");
    }
}
