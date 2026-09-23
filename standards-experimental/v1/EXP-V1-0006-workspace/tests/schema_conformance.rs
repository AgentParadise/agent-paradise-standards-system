use serde_json::Value;
use workspace_standard::LaunchManifest;

const SCHEMA: &str = include_str!("../schemas/workspace-launch.schema.json");
const EXAMPLE: &str = include_str!("../examples/minimal/workspace-launch.json");

#[test]
fn minimal_example_conforms_to_schema_and_rust_contract() {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let instance: Value = serde_json::from_str(EXAMPLE).unwrap();
    assert!(validator.is_valid(&instance));

    let manifest: LaunchManifest = serde_json::from_value(instance).unwrap();
    manifest.validate().unwrap();
    assert_eq!(
        serde_json::to_value(&manifest).unwrap()["schema"],
        "apss.workspace-launch/v1"
    );
}

#[test]
fn schema_rejects_unknown_fields() {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let mut instance: Value = serde_json::from_str(EXAMPLE).unwrap();
    instance["provider_secret"] = Value::String("must-not-cross-boundary".into());
    assert!(!validator.is_valid(&instance));
}
