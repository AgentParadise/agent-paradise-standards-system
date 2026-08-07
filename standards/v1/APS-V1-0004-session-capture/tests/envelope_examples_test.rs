//! Integration tests: the shipped example envelopes parse and validate against
//! the definitional crate, and the example batch is well-formed.

use session_capture::SessionEnvelope;

const SOURCE_EXAMPLE: &str = include_str!("../examples/claude-source-envelope.json");
const BATCH_EXAMPLE: &str = include_str!("../examples/workflow-exporter-batch.json");

#[test]
fn source_example_parses_and_validates() {
    let env: SessionEnvelope =
        serde_json::from_str(SOURCE_EXAMPLE).expect("source example must be a valid envelope");
    env.validate().expect("source example must pass validation");
    assert_eq!(env.agent, "ClaudeCode");
    assert_eq!(env.origin.environment, "local");
    // `raw` is preserved verbatim and is not empty.
    assert!(env.raw.is_object());
}

#[test]
fn batch_example_envelopes_all_validate_and_share_workflow_id() {
    let body: serde_json::Value =
        serde_json::from_str(BATCH_EXAMPLE).expect("batch example must be valid JSON");
    let envelopes = body
        .get("envelopes")
        .and_then(|v| v.as_array())
        .expect("batch body must carry an `envelopes` array");
    assert!(
        envelopes.len() >= 2,
        "multi-agent run has one envelope per agent"
    );

    let mut workflow_ids = std::collections::BTreeSet::new();
    for value in envelopes {
        let env: SessionEnvelope =
            serde_json::from_value(value.clone()).expect("each envelope must parse");
        env.validate().expect("each envelope must validate");
        assert_eq!(env.origin.environment, "workflow");
        let md = env.metadata.expect("workflow envelopes carry metadata");
        workflow_ids.insert(md.workflow_id.expect("workflow_id groups the run"));
    }
    // Every per-agent envelope in one run shares a single workflow_id.
    assert_eq!(workflow_ids.len(), 1, "one run == one shared workflow_id");
}

#[test]
fn idempotency_key_is_session_id_and_content_hash() {
    let env: SessionEnvelope = serde_json::from_str(SOURCE_EXAMPLE).unwrap();
    // The examples show STORED envelopes, so the hash is present.
    env.validate_stored()
        .expect("the source example is a stored envelope");
    let (session_id, content_hash) = env.idempotency_key().expect("stored envelope has a key");
    assert_eq!(session_id, env.session_id);
    assert_eq!(Some(content_hash), env.content_hash.as_deref());
}
