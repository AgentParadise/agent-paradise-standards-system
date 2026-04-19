//! architecture-fitness `FitnessReport` ↔ fitness-report.schema.json drift guard.

use aps_schema_test::{collect_errors, compile};
use architecture_fitness::FitnessValidator;
use std::fs;
use tempfile::TempDir;

const SCHEMA: &str = include_str!(
    "../../../standards/v1/APS-V1-0002-architecture-fitness/schemas/fitness-report.schema.json"
);

#[test]
fn fitness_report_matches_schema() {
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

    let schema = compile(SCHEMA);
    let errors = collect_errors(&schema, &report_value);
    assert!(
        errors.is_empty(),
        "schema errors: {errors:#?}\nreport: {}",
        serde_json::to_string_pretty(&report_value).unwrap()
    );
}
