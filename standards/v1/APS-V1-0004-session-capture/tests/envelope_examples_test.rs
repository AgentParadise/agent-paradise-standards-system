//! Integration tests: the shipped example envelopes parse and validate against
//! the definitional crate, and the example batch is well-formed.
//!
//! Both examples model envelopes IN FLIGHT (spec 4.2.3), so neither carries a
//! `content_hash`. That is the state a producer actually emits, and the examples
//! exist to show producers what to build.

use session_capture::reconstitution::Registry;
use session_capture::{SessionEnvelope, ValidationError};

const SOURCE_EXAMPLE: &str = include_str!("../examples/claude-source-envelope.json");
const BATCH_EXAMPLE: &str = include_str!("../examples/workflow-exporter-batch.json");

fn batch_envelopes() -> Vec<SessionEnvelope> {
    let body: serde_json::Value =
        serde_json::from_str(BATCH_EXAMPLE).expect("batch example must be valid JSON");
    body["envelopes"]
        .as_array()
        .expect("batch body must carry an `envelopes` array")
        .iter()
        .map(|v| serde_json::from_value(v.clone()).expect("each envelope must parse"))
        .collect()
}

fn all_examples() -> Vec<SessionEnvelope> {
    let mut all = vec![serde_json::from_str(SOURCE_EXAMPLE).expect("source example must parse")];
    all.extend(batch_envelopes());
    all
}

#[test]
fn source_example_parses_and_validates() {
    let env: SessionEnvelope =
        serde_json::from_str(SOURCE_EXAMPLE).expect("source example must be a valid envelope");
    env.validate().expect("source example must pass validation");
    assert_eq!(env.agent, "ClaudeCode");
    assert_eq!(env.origin.environment, "local");
}

#[test]
fn batch_example_envelopes_all_validate_and_share_workflow_id() {
    let envelopes = batch_envelopes();
    assert!(
        envelopes.len() >= 2,
        "multi-agent run has one envelope per agent"
    );

    let mut workflow_ids = std::collections::BTreeSet::new();
    for env in envelopes {
        env.validate().expect("each envelope must validate");
        assert_eq!(env.origin.environment, "workflow");
        let md = env.metadata.expect("workflow envelopes carry metadata");
        workflow_ids.insert(md.workflow_id.expect("workflow_id groups the run"));
    }
    // Every per-agent envelope in one run shares a single workflow_id.
    assert_eq!(workflow_ids.len(), 1, "one run == one shared workflow_id");
}

/// Producers do not compute `content_hash` (spec 4.2.3), so no shipped example
/// may carry one. An example that did would teach producers to send a value the
/// store is required to discard.
#[test]
fn no_example_carries_a_producer_supplied_content_hash() {
    for env in all_examples() {
        assert!(
            env.content_hash.is_none(),
            "in-flight example must not carry content_hash (session {})",
            env.session_id
        );
        assert_eq!(
            env.idempotency_key(),
            None,
            "an in-flight envelope has no dedup key yet"
        );
        assert_eq!(
            env.validate_stored(),
            Err(ValidationError::MissingStoredContentHash),
            "in-flight examples are not stored envelopes"
        );
    }
}

/// Every example claims a `source_format` that appears in the reconstitution
/// registry, so each MUST carry a string `raw` (spec 4.3.1) and a `session_id`
/// the registry accepts. This is what stops the examples drifting into a shape
/// the standard forbids.
#[test]
fn examples_are_reconstitutable_as_the_registry_requires() {
    let registry = Registry::shipped();
    for env in all_examples() {
        let entry = registry
            .entry(&env.source_format)
            .unwrap_or_else(|e| panic!("example uses an unregistered source_format: {e}"));
        assert!(
            env.raw.is_string(),
            "{}: a registered source_format requires a verbatim string raw, got {}",
            env.source_format,
            if env.raw.is_object() {
                "object"
            } else {
                "array"
            }
        );
        entry
            .validate_session_id(&env.session_id)
            .unwrap_or_else(|e| {
                panic!(
                    "{}: example session_id `{}` is rejected by the registry: {e}",
                    env.source_format, env.session_id
                )
            });
    }
}

/// `metadata.source_path` is the field a Reconstitutor prefers, so the examples
/// must model a value that passes the untrusted-input check.
#[test]
fn example_source_paths_are_safe() {
    for env in all_examples() {
        let md = env.metadata.expect("examples carry metadata");
        let source_path = md
            .source_path
            .expect("examples model source_path so producers copy it");
        assert!(
            session_capture::reconstitution::is_safe_source_path(&source_path),
            "example source_path must pass the containment check: {source_path}"
        );
    }
}
