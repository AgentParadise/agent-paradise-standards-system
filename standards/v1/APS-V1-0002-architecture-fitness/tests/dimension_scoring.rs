//! Tests for per-dimension scoring and dimension result computation.

use architecture_fitness::{DimensionStatus, FitnessValidator};
use std::fs;
use tempfile::TempDir;

/// Create a test fixture: fitness.toml + topology artifacts + optional exceptions.
fn setup_fixture(
    rules_toml: &str,
    exceptions_toml: Option<&str>,
    artifacts: &[(&str, &str)],
) -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    fs::write(root.join("fitness.toml"), rules_toml).unwrap();

    if let Some(exc) = exceptions_toml {
        fs::write(root.join("fitness-exceptions.toml"), exc).unwrap();
    }

    let topo_dir = root.join(".topology");
    fs::create_dir_all(topo_dir.join("metrics")).unwrap();
    for (path, content) in artifacts {
        let full_path = topo_dir.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full_path, content).unwrap();
    }

    dir
}

#[test]
fn dimension_scores_computed_correctly() {
    // Two rules in MT01: one passes, one has violations.
    // 3 entities total, 1 unexcepted violation → score = 1.0 - 1/3 ≈ 0.667
    let dir = setup_fixture(
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
        None,
        &[
            (
                "metrics/complexity.json",
                r#"{ "src/a.py::foo": { "cyclomatic_complexity": 5 }, "src/b.py::bar": { "cyclomatic_complexity": 20 } }"#,
            ),
            (
                "metrics/loc.json",
                r#"{ "src/main.py": { "lines_of_code": 100 } }"#,
            ),
        ],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    let mt01 = &report.dimensions["MT01"];
    assert_eq!(mt01.status, DimensionStatus::Active);
    assert_eq!(mt01.rules_evaluated, 2);
    assert_eq!(mt01.rules_passed, 1); // max-loc passes
    assert_eq!(mt01.rules_failed, 1); // max-cc fails
    assert_eq!(mt01.total_violations, 1);
    assert_eq!(mt01.excepted_violations, 0);
    // score = 1.0 - 1/3 (1 unexcepted violation, 3 total entities across both rules)
    let score = mt01.score.unwrap();
    assert!((score - 0.6667).abs() < 0.01, "expected ~0.667, got {score}");
}

#[test]
fn dimension_with_no_violations_scores_1() {
    let dir = setup_fixture(
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
"#,
        None,
        &[(
            "metrics/complexity.json",
            r#"{ "src/a.py::foo": { "cyclomatic_complexity": 5 }, "src/b.py::bar": { "cyclomatic_complexity": 10 } }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    let mt01 = &report.dimensions["MT01"];
    assert_eq!(mt01.score, Some(1.0));
    assert_eq!(mt01.rules_passed, 1);
    assert_eq!(mt01.total_violations, 0);
}

#[test]
fn disabled_dimension_excluded_from_scoring() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[dimensions]
AC01 = false
PF01 = false
AV01 = false
"#,
        None,
        &[],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    // AC01 is disabled (opt-in, no reason needed)
    let ac01 = &report.dimensions["AC01"];
    assert_eq!(ac01.status, DimensionStatus::Disabled);
    assert_eq!(ac01.score, None);

    // MT01 is enabled by default, no rules → active with score 1.0
    let mt01 = &report.dimensions["MT01"];
    assert_eq!(mt01.status, DimensionStatus::Active);
    assert_eq!(mt01.score, Some(1.0));
}

#[test]
fn skipped_dimension_when_all_artifacts_missing() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
dimension = "MT01"
source = "metrics/nonexistent.json"
field = "cyclomatic_complexity"
max = 15
scope = "function"
"#,
        None,
        &[],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    let mt01 = &report.dimensions["MT01"];
    assert_eq!(mt01.status, DimensionStatus::Skipped);
    assert_eq!(mt01.score, None);
    assert_eq!(mt01.rules_evaluated, 0);
}

#[test]
fn dimension_field_appears_in_rule_results() {
    let dir = setup_fixture(
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
"#,
        None,
        &[(
            "metrics/complexity.json",
            r#"{ "src/a.py::foo": { "cyclomatic_complexity": 5 } }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results[0].dimension.as_deref(), Some("MT01"));
    assert_eq!(report.results[0].total_entities, Some(1));
}

#[test]
fn backward_compat_report_dimensions_with_v1_config() {
    // V1-style config (no dimension field on rules).
    // Dimensions should still appear in report, but all rules have dimension=None.
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 15
scope = "function"
"#,
        None,
        &[(
            "metrics/complexity.json",
            r#"{ "src/a.py::foo": { "cyclomatic_complexity": 5 } }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    // Rule has no dimension
    assert_eq!(report.results[0].dimension, None);

    // Default-enabled dimensions are active (no rules assigned, so score=1.0)
    assert_eq!(report.dimensions["MT01"].status, DimensionStatus::Active);
    assert_eq!(report.dimensions["MT01"].score, Some(1.0));

    // Summary includes skipped count
    assert_eq!(report.summary.skipped, 0);
}

#[test]
fn dimension_score_with_exceptions() {
    // 2 entities, 2 violations, 1 excepted → 1 unexcepted → score = 1.0 - 1/2 = 0.5
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 10
scope = "function"
"#,
        Some(
            r##"
[max-cc."src/a.py::foo"]
value = 25
issue = "#100"
"##,
        ),
        &[(
            "metrics/complexity.json",
            r#"{ "src/a.py::foo": { "cyclomatic_complexity": 20 }, "src/b.py::bar": { "cyclomatic_complexity": 15 } }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    let mt01 = &report.dimensions["MT01"];
    assert_eq!(mt01.total_violations, 2);
    assert_eq!(mt01.excepted_violations, 1);
    // score = 1.0 - 1/2 = 0.5
    let score = mt01.score.unwrap();
    assert!((score - 0.5).abs() < 0.01, "expected 0.5, got {score}");
}

#[test]
fn report_json_includes_dimensions() {
    let dir = setup_fixture(
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
"#,
        None,
        &[(
            "metrics/complexity.json",
            r#"{ "src/a.py::foo": { "cyclomatic_complexity": 5 } }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    // Serialize to JSON and verify dimensions appear
    let json = serde_json::to_string_pretty(&report).unwrap();
    assert!(json.contains("\"MT01\""), "JSON should contain MT01 dimension");
    assert!(json.contains("\"Maintainability\""), "JSON should contain dimension name");

    // Roundtrip
    let parsed: architecture_fitness::FitnessReport = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.dimensions["MT01"].status, DimensionStatus::Active);
    assert_eq!(parsed.dimensions["MT01"].score, Some(1.0));
}
