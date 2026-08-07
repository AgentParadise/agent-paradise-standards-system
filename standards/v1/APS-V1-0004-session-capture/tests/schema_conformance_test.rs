//! Conformance tests: the shipped examples validate against the shipped JSON
//! Schema, and the schema stays in step with the Rust types.
//!
//! These are the tests that make APS-V1-0004 executable rather than aspirational
//! (spec section 1.3). A consumer that depends on this crate inherits them.

use session_capture::{SessionEnvelope, session_envelope_schema};

const SOURCE_EXAMPLE: &str = include_str!("../examples/claude-source-envelope.json");
const BATCH_EXAMPLE: &str = include_str!("../examples/workflow-exporter-batch.json");

fn compiled_schema() -> jsonschema::Validator {
    let schema = session_envelope_schema();
    jsonschema::validator_for(&schema).expect("the shipped schema must compile")
}

fn assert_schema_valid(validator: &jsonschema::Validator, instance: &serde_json::Value) {
    if let Err(error) = validator.validate(instance) {
        panic!("instance must satisfy the session-envelope schema: {error}");
    }
}

#[test]
fn embedded_schema_is_valid_and_compiles() {
    let schema = session_envelope_schema();
    assert_eq!(
        schema["$id"], "urn:apss:schema:aps-v1-0004:session-envelope:1.0.0",
        "schema $id must carry the ratified standard id"
    );
    let _ = compiled_schema();
}

#[test]
fn source_example_satisfies_the_schema() {
    let validator = compiled_schema();
    let instance: serde_json::Value = serde_json::from_str(SOURCE_EXAMPLE).unwrap();
    assert_schema_valid(&validator, &instance);
}

#[test]
fn every_batch_envelope_satisfies_the_schema() {
    let validator = compiled_schema();
    let body: serde_json::Value = serde_json::from_str(BATCH_EXAMPLE).unwrap();
    let envelopes = body["envelopes"]
        .as_array()
        .expect("batch carries envelopes");
    for envelope in envelopes {
        assert_schema_valid(&validator, envelope);
    }
}

/// The schema and the Rust `validate()` must agree on what is conformant.
/// If they drift, one of them is lying to consumers.
#[test]
fn schema_and_rust_validation_agree_on_scs_version() {
    let validator = compiled_schema();
    let mut instance: serde_json::Value = serde_json::from_str(SOURCE_EXAMPLE).unwrap();

    // The real-world drift: `scs/1` instead of `1.0`. Both layers must reject it.
    instance["scs_version"] = serde_json::json!("scs/1");
    assert!(
        validator.validate(&instance).is_err(),
        "schema must reject a prefixed scs_version"
    );
    let envelope: SessionEnvelope = serde_json::from_value(instance.clone()).unwrap();
    assert!(
        envelope.validate().is_err(),
        "Rust validation must reject a prefixed scs_version"
    );

    // And both must accept the canonical form.
    instance["scs_version"] = serde_json::json!("1.0");
    assert_schema_valid(&validator, &instance);
    let envelope: SessionEnvelope = serde_json::from_value(instance).unwrap();
    envelope
        .validate()
        .expect("canonical scs_version is conformant");
}

/// Every field the schema marks required must be non-optional in practice:
/// dropping any one of them must fail schema validation.
#[test]
fn every_required_field_is_actually_enforced() {
    let validator = compiled_schema();
    let schema = session_envelope_schema();
    let required: Vec<String> = schema["required"]
        .as_array()
        .expect("schema declares required fields")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(
        required.contains(&"raw".to_string()),
        "raw is the payload; it must be required"
    );

    for field in &required {
        let mut instance: serde_json::Value = serde_json::from_str(SOURCE_EXAMPLE).unwrap();
        instance.as_object_mut().unwrap().remove(field);
        assert!(
            validator.validate(&instance).is_err(),
            "schema must reject an envelope missing required field `{field}`"
        );
    }
}

/// Unknown keys must be tolerated so additive evolution cannot break existing
/// stores (spec section 8.3).
#[test]
fn additive_evolution_is_tolerated() {
    let validator = compiled_schema();
    let mut instance: serde_json::Value = serde_json::from_str(SOURCE_EXAMPLE).unwrap();
    instance["a_field_from_a_later_minor"] = serde_json::json!("value");
    instance["metadata"]["an_unknown_metadata_key"] = serde_json::json!(42);
    instance["origin"]["an_unknown_origin_key"] = serde_json::json!(true);
    assert_schema_valid(&validator, &instance);

    let envelope: SessionEnvelope = serde_json::from_value(instance).unwrap();
    envelope
        .validate()
        .expect("unknown keys must not make an envelope non-conformant");
}
