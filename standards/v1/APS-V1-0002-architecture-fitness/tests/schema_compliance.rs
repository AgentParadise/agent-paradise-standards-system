//! Round-trip validation: produced artifacts and canonical examples must
//! conform to their published schemas.
//!
//! These tests fail if any of:
//!   - `FitnessValidator::validate()` emits a report whose shape drifts from
//!     `schemas/fitness-report.schema.json`
//!   - `examples/fitness.toml` stops being a valid instance of
//!     `schemas/fitness-config.schema.json`
//!   - `examples/fitness-exceptions.toml` stops being a valid instance of
//!     `schemas/fitness-exceptions.schema.json`
//!
//! Producer and contract stay in lockstep.

use architecture_fitness::FitnessValidator;
use jsonschema::Validator;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

const REPORT_SCHEMA: &str = include_str!("../schemas/fitness-report.schema.json");
const CONFIG_SCHEMA: &str = include_str!("../schemas/fitness-config.schema.json");
const EXCEPTIONS_SCHEMA: &str = include_str!("../schemas/fitness-exceptions.schema.json");

const EXAMPLE_FITNESS_TOML: &str = include_str!("../examples/fitness.toml");
const EXAMPLE_EXCEPTIONS_TOML: &str = include_str!("../examples/fitness-exceptions.toml");

fn compile(schema_str: &str) -> Validator {
    let schema: Value = serde_json::from_str(schema_str).expect("schema parses");
    jsonschema::options()
        .build(&schema)
        .expect("schema compiles")
}

fn format_errors(validator: &Validator, value: &Value) -> Vec<String> {
    validator
        .iter_errors(value)
        .map(|e| format!("at {}: {}", e.instance_path(), e))
        .collect()
}

/// Convert parsed TOML directly to the JSON shape that the JSON Schema expects,
/// bypassing the Rust structs. This is what a non-Rust consumer would see.
fn toml_to_json(toml_str: &str) -> Value {
    let raw: toml::Value = toml::from_str(toml_str).expect("toml parses");
    serde_json::to_value(raw).expect("toml → json conversion")
}

#[test]
fn fitness_report_matches_schema() {
    // Minimal fixture: two MT01 rules, one failing — exercises dimensions,
    // system_fitness, summary, results, and a violation record.
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::write(
        root.join("fitness.toml"),
        r#"
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 15
scope = "function"

[[rules.threshold]]
id = "max-loc"
name = "Max LOC"
dimension = "MT01"
source = "metrics/loc.json"
field = "lines_of_code"
max = 500
scope = "file"
"#,
    )
    .unwrap();

    let topo = root.join(".topology/metrics");
    fs::create_dir_all(&topo).unwrap();
    fs::write(
        topo.join("complexity.json"),
        r#"{ "src/a.py::foo": { "cyclomatic_complexity": 5 }, "src/b.py::bar": { "cyclomatic_complexity": 20 } }"#,
    )
    .unwrap();
    fs::write(
        topo.join("loc.json"),
        r#"{ "src/main.py": { "lines_of_code": 100 } }"#,
    )
    .unwrap();

    let validator = FitnessValidator::load(root, None).unwrap();
    let report = validator.validate().unwrap();
    let report_value = serde_json::to_value(&report).expect("report serializes");

    let schema = compile(REPORT_SCHEMA);
    let errors = format_errors(&schema, &report_value);
    assert!(
        errors.is_empty(),
        "schema errors: {errors:#?}\nreport: {}",
        serde_json::to_string_pretty(&report_value).unwrap()
    );
}

#[test]
fn example_fitness_toml_matches_config_schema() {
    let value = toml_to_json(EXAMPLE_FITNESS_TOML);
    let schema = compile(CONFIG_SCHEMA);
    let errors = format_errors(&schema, &value);
    assert!(errors.is_empty(), "schema errors: {errors:#?}");
    // Sanity — make sure the example path we think we're using is valid.
    assert!(
        Path::new("examples/fitness.toml").is_relative(),
        "example path shape"
    );
}

#[test]
fn example_fitness_exceptions_toml_matches_schema() {
    let value = toml_to_json(EXAMPLE_EXCEPTIONS_TOML);
    let schema = compile(EXCEPTIONS_SCHEMA);
    let errors = format_errors(&schema, &value);
    assert!(errors.is_empty(), "schema errors: {errors:#?}");
}
