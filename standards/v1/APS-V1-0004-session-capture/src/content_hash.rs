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
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use sha2::{Digest, Sha256};

use crate::SessionEnvelope;

/// The largest integer exactly representable as an IEEE-754 double.
///
/// JCS renders every number as a double (RFC 8785 section 3.2.2.3). An integer
/// beyond this magnitude therefore loses precision during canonicalization, and
/// two distinct values collapse to the same canonical form and the same digest.
const MAX_EXACT_INTEGER: i128 = 9_007_199_254_740_992; // 2^53

/// Errors from computing a content hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentHashError {
    /// An object contained a duplicate member name.
    ///
    /// I-JSON requires unique names, and JCS has no defined ordering for
    /// duplicates. Note that most JSON parsers silently keep the last value, so
    /// `{"a":1,"a":2}` and `{"a":2}` would otherwise produce the same digest
    /// despite being different documents.
    DuplicateMemberName(String),
    /// A number was too large to survive canonicalization exactly.
    ///
    /// Beyond 2^53 the double conversion JCS mandates is lossy, so
    /// `9007199254740993` and `9007199254740992` would canonicalize identically
    /// and receive the same digest.
    NumberNotExactlyRepresentable(String),
    /// The value could not be canonicalized per RFC 8785 for any other reason.
    NotCanonicalizable,
}

impl std::fmt::Display for ContentHashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentHashError::DuplicateMemberName(name) => write!(
                f,
                "duplicate object member name `{name}`: I-JSON requires unique names, so this has no RFC 8785 canonical form"
            ),
            ContentHashError::NumberNotExactlyRepresentable(value) => write!(
                f,
                "number {value} exceeds 2^53 and cannot survive the IEEE-754 conversion RFC 8785 requires, so its digest would collide with a different value"
            ),
            ContentHashError::NotCanonicalizable => write!(
                f,
                "value is not I-JSON compatible, so it has no RFC 8785 canonical form and therefore no conforming content_hash"
            ),
        }
    }
}

impl std::error::Error for ContentHashError {}

// ─── I-JSON parsing ─────────────────────────────────────────────────────────

/// A `serde_json::Value` parsed under I-JSON rules.
struct IJson(serde_json::Value);

struct IJsonVisitor;

impl<'de> Visitor<'de> for IJsonVisitor {
    type Value = IJson;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("an I-JSON value")
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<IJson, E> {
        Ok(IJson(serde_json::Value::Bool(v)))
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<IJson, E> {
        if (v as i128).abs() > MAX_EXACT_INTEGER {
            return Err(E::custom(format!("number-not-exact:{v}")));
        }
        Ok(IJson(serde_json::Value::from(v)))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<IJson, E> {
        if v as i128 > MAX_EXACT_INTEGER {
            return Err(E::custom(format!("number-not-exact:{v}")));
        }
        Ok(IJson(serde_json::Value::from(v)))
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<IJson, E> {
        // Already a double, so canonicalization is lossless. Non-finite values
        // cannot appear in parsed JSON.
        Ok(IJson(serde_json::Value::from(v)))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<IJson, E> {
        Ok(IJson(serde_json::Value::String(v.to_owned())))
    }

    fn visit_unit<E: de::Error>(self) -> Result<IJson, E> {
        Ok(IJson(serde_json::Value::Null))
    }

    fn visit_none<E: de::Error>(self) -> Result<IJson, E> {
        Ok(IJson(serde_json::Value::Null))
    }

    fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<IJson, D::Error> {
        d.deserialize_any(IJsonVisitor)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<IJson, A::Error> {
        let mut items = Vec::new();
        while let Some(IJson(value)) = access.next_element()? {
            items.push(value);
        }
        Ok(IJson(serde_json::Value::Array(items)))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<IJson, A::Error> {
        let mut map = serde_json::Map::new();
        while let Some(key) = access.next_key::<String>()? {
            let IJson(value) = access.next_value()?;
            // The whole reason this visitor exists: serde_json's own Value
            // deserializer overwrites here instead of complaining.
            if map.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate-member:{key}")));
            }
            map.insert(key, value);
        }
        Ok(IJson(serde_json::Value::Object(map)))
    }
}

impl<'de> Deserialize<'de> for IJson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(IJsonVisitor)
    }
}

/// Map a visitor error string back to a typed error.
fn classify(message: &str) -> ContentHashError {
    if let Some(name) = message
        .split_once("duplicate-member:")
        .map(|(_, rest)| rest.split(" at line").next().unwrap_or(rest).trim())
    {
        return ContentHashError::DuplicateMemberName(name.to_string());
    }
    if let Some(value) = message
        .split_once("number-not-exact:")
        .map(|(_, rest)| rest.split(" at line").next().unwrap_or(rest).trim())
    {
        return ContentHashError::NumberNotExactlyRepresentable(value.to_string());
    }
    ContentHashError::NotCanonicalizable
}

/// Parse JSON text under I-JSON rules, rejecting what RFC 8785 cannot
/// canonicalize (section 4.2.3).
///
/// A store MUST use this, or an equivalent check, on the ORIGINAL request bytes.
/// Parsing first and validating afterwards does not work: duplicate member names
/// are gone by the time a `serde_json::Value` exists, because the parser keeps
/// the last one silently.
pub fn parse_ijson(text: &str) -> Result<serde_json::Value, ContentHashError> {
    match serde_json::from_str::<IJson>(text) {
        Ok(IJson(value)) => Ok(value),
        Err(error) => Err(classify(&error.to_string())),
    }
}

/// Check an already-parsed value for the I-JSON violations still detectable.
///
/// Numbers can be checked here; duplicate member names CANNOT, because parsing
/// has already discarded them. Use [`parse_ijson`] on the original text when the
/// text is available.
pub fn check_ijson_value(value: &serde_json::Value) -> Result<(), ContentHashError> {
    match value {
        serde_json::Value::Number(number) => {
            let exceeds = number
                .as_u64()
                .map(|v| v as i128 > MAX_EXACT_INTEGER)
                .or_else(|| {
                    number
                        .as_i64()
                        .map(|v| (v as i128).abs() > MAX_EXACT_INTEGER)
                })
                .unwrap_or(false);
            if exceeds {
                return Err(ContentHashError::NumberNotExactlyRepresentable(
                    number.to_string(),
                ));
            }
            Ok(())
        }
        serde_json::Value::Array(items) => items.iter().try_for_each(check_ijson_value),
        serde_json::Value::Object(map) => map.values().try_for_each(check_ijson_value),
        _ => Ok(()),
    }
}

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
    // Reject what canonicalization would silently coerce. Without this, two
    // different `raw` values receive the same digest and dedup merges two
    // distinct sessions into one.
    check_ijson_value(raw)?;

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

    /// Regression: these two documents differ, but JCS renders every number as a
    /// double, so beyond 2^53 they canonicalize identically. Before the I-JSON
    /// check both hashed to the same digest, which would have merged two
    /// distinct sessions into one during dedup.
    #[test]
    fn integers_beyond_2_53_are_rejected_rather_than_colliding() {
        let at_limit: serde_json::Value =
            serde_json::from_str(r#"{"n":9007199254740992}"#).unwrap();
        let past_limit: serde_json::Value =
            serde_json::from_str(r#"{"n":9007199254740993}"#).unwrap();

        // 2^53 is exactly representable, so it remains hashable.
        content_hash("s", "f", &at_limit).expect("2^53 is exact");

        // 2^53 + 1 is not, and must be refused rather than silently coerced.
        assert_eq!(
            content_hash("s", "f", &past_limit).unwrap_err(),
            ContentHashError::NumberNotExactlyRepresentable("9007199254740993".to_string())
        );
    }

    #[test]
    fn negative_integers_beyond_2_53_are_also_rejected() {
        let value: serde_json::Value = serde_json::from_str(r#"{"n":-9007199254740993}"#).unwrap();
        assert!(matches!(
            content_hash("s", "f", &value).unwrap_err(),
            ContentHashError::NumberNotExactlyRepresentable(_)
        ));
    }

    #[test]
    fn nested_out_of_range_numbers_are_found() {
        let value: serde_json::Value =
            serde_json::from_str(r#"{"a":[{"deep":9007199254740993}]}"#).unwrap();
        assert!(matches!(
            content_hash("s", "f", &value).unwrap_err(),
            ContentHashError::NumberNotExactlyRepresentable(_)
        ));
    }

    /// Regression: `{"a":1,"a":2}` and `{"a":2}` are different documents, but
    /// serde_json keeps only the last member, so both previously produced the
    /// same digest. Duplicates must be caught at PARSE time; by the time a
    /// `Value` exists the evidence is gone.
    #[test]
    fn duplicate_member_names_are_rejected_at_parse_time() {
        assert_eq!(
            parse_ijson(r#"{"a":1,"a":2}"#).unwrap_err(),
            ContentHashError::DuplicateMemberName("a".to_string())
        );
        // The distinct document that it would have collided with still parses.
        assert!(parse_ijson(r#"{"a":2}"#).is_ok());
    }

    #[test]
    fn duplicate_member_names_are_found_when_nested() {
        assert!(matches!(
            parse_ijson(r#"{"outer":{"a":1,"a":2}}"#).unwrap_err(),
            ContentHashError::DuplicateMemberName(_)
        ));
        assert!(matches!(
            parse_ijson(r#"[{"a":1,"a":2}]"#).unwrap_err(),
            ContentHashError::DuplicateMemberName(_)
        ));
    }

    #[test]
    fn parse_ijson_rejects_out_of_range_numbers_too() {
        assert!(matches!(
            parse_ijson(r#"{"n":9007199254740993}"#).unwrap_err(),
            ContentHashError::NumberNotExactlyRepresentable(_)
        ));
    }

    #[test]
    fn parse_ijson_accepts_ordinary_documents_unchanged() {
        for text in [
            r#"{"a":1,"b":[1,2,3],"c":{"d":null},"e":true,"f":1.5,"g":"x"}"#,
            r#"[]"#,
            r#""just a string""#,
            r#"{"n":9007199254740992}"#,
        ] {
            let parsed = parse_ijson(text).unwrap_or_else(|e| panic!("{text} should parse: {e}"));
            let expected: serde_json::Value = serde_json::from_str(text).unwrap();
            assert_eq!(parsed, expected, "parsing must not alter the value");
        }
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
        second.content_hash =
            Some("sha256:deadbeef00000000000000000000000000000000000000000000000000000000".into());
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
