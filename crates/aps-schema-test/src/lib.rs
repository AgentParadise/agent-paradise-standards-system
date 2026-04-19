//! Shared helpers for schema-drift tests.
//!
//! Lives in a tooling crate (not a shippable standard) because `jsonschema`
//! transitively pulls ~50 crates (icu_*, fancy-regex, fraction, url, etc.).
//! Scoping all schema-validation to this one crate keeps the shippable
//! standards' dep trees clean.

use jsonschema::Validator;
use serde_json::Value;

/// Compile a JSON Schema source string into a validator.
pub fn compile(schema_str: &str) -> Validator {
    let schema: Value = serde_json::from_str(schema_str).expect("schema parses");
    jsonschema::options()
        .build(&schema)
        .expect("schema compiles")
}

/// Collect validation errors with instance paths. Empty vec ⇒ valid.
pub fn collect_errors(validator: &Validator, value: &Value) -> Vec<String> {
    validator
        .iter_errors(value)
        .map(|e| format!("at {}: {}", e.instance_path(), e))
        .collect()
}

/// Convert a TOML source string into `serde_json::Value` — the JSON shape a
/// non-Rust consumer would see when reading the same TOML.
pub fn toml_to_json(toml_str: &str) -> Value {
    let raw: toml::Value = toml::from_str(toml_str).expect("toml parses");
    serde_json::to_value(raw).expect("toml → json conversion")
}
