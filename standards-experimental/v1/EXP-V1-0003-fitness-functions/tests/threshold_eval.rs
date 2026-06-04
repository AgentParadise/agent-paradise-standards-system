//! Tests for threshold rule evaluation against mock topology JSON.

use fitness_functions::{FitnessValidator, RuleStatus, StaleReason};
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

    // Write fitness.toml
    fs::write(root.join("fitness.toml"), rules_toml).unwrap();

    // Write exceptions if provided
    if let Some(exc) = exceptions_toml {
        fs::write(root.join("fitness-exceptions.toml"), exc).unwrap();
    }

    // Create topology dir and artifacts
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
fn all_pass_when_under_threshold() {
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
            r#"{
                "src/main.py::run": { "cyclomatic_complexity": 5 },
                "src/utils.py::helper": { "cyclomatic_complexity": 10 }
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.summary.total_rules, 1);
    assert_eq!(report.summary.passed, 1);
    assert_eq!(report.summary.failed, 0);
    assert!(report.results[0].violations.is_empty());
    assert_eq!(report.results[0].status, RuleStatus::Pass);
}

#[test]
fn fail_when_over_threshold() {
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
            r#"{
                "src/main.py::run": { "cyclomatic_complexity": 5 },
                "src/engine.py::execute": { "cyclomatic_complexity": 42 }
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.summary.failed, 1);
    assert_eq!(report.results[0].status, RuleStatus::Fail);
    assert_eq!(report.results[0].violations.len(), 1);

    let v = &report.results[0].violations[0];
    assert_eq!(v.entity, "src/engine.py::execute");
    assert_eq!(v.actual, 42.0);
    assert_eq!(v.threshold, 15.0);
    assert!(!v.excepted);
    assert!(FitnessValidator::has_failures(&report));
}

#[test]
fn exception_makes_violation_pass() {
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
        Some(
            r##"
[max-cc."src/engine.py::execute"]
value = 42
issue = "#138"
"##,
        ),
        &[(
            "metrics/complexity.json",
            r#"{
                "src/engine.py::execute": { "cyclomatic_complexity": 42 }
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.summary.passed, 1);
    assert_eq!(report.summary.failed, 0);
    assert_eq!(report.summary.excepted_violations, 1);
    assert_eq!(report.results[0].status, RuleStatus::Pass);
    assert!(report.results[0].violations[0].excepted);
}

#[test]
fn exception_budget_exceeded_still_fails() {
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
        Some(
            r##"
[max-cc."src/engine.py::execute"]
value = 40
issue = "#138"
"##,
        ),
        &[(
            "metrics/complexity.json",
            r#"{
                "src/engine.py::execute": { "cyclomatic_complexity": 45 }
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    // Value 45 exceeds budget of 40, so exception is insufficient
    assert_eq!(report.summary.failed, 1);
    assert!(!report.results[0].violations[0].excepted);
}

#[test]
fn stale_exception_detected_now_passing() {
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
        Some(
            r##"
[max-cc."src/engine.py::execute"]
value = 42
issue = "#138"
"##,
        ),
        &[(
            "metrics/complexity.json",
            r#"{
                "src/engine.py::execute": { "cyclomatic_complexity": 10 }
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.stale_exceptions.len(), 1);
    assert_eq!(report.stale_exceptions[0].rule_id, "max-cc");
    assert_eq!(report.stale_exceptions[0].entity, "src/engine.py::execute");
    assert_eq!(report.stale_exceptions[0].reason, StaleReason::NowPassing);
}

#[test]
fn stale_exception_detected_entity_not_found() {
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
        Some(
            r##"
[max-cc."src/deleted_module.py::old_func"]
value = 42
issue = "#138"
"##,
        ),
        &[(
            "metrics/complexity.json",
            r#"{
                "src/main.py::run": { "cyclomatic_complexity": 5 }
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.stale_exceptions.len(), 1);
    assert_eq!(
        report.stale_exceptions[0].entity,
        "src/deleted_module.py::old_func"
    );
    assert_eq!(
        report.stale_exceptions[0].reason,
        StaleReason::EntityNotFound
    );
}

#[test]
fn warning_severity_does_not_cause_failure() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-loc"
name = "Max LOC"
source = "metrics/file_metrics.json"
field = "lines_of_code"
max = 500
scope = "file"
severity = "warning"
"#,
        None,
        &[(
            "metrics/file_metrics.json",
            r#"{
                "src/big_file.py": { "lines_of_code": 800 }
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.summary.warned, 1);
    assert_eq!(report.summary.failed, 0);
    assert_eq!(report.results[0].status, RuleStatus::Warn);
    assert!(!FitnessValidator::has_failures(&report));
}

#[test]
fn skip_when_artifact_missing() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
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

    assert_eq!(report.results[0].status, RuleStatus::Skip);
}

#[test]
fn exclude_patterns_skip_entities() {
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
exclude = ["**/test_*"]
"#,
        None,
        &[(
            "metrics/complexity.json",
            r#"{
                "src/main.py::run": { "cyclomatic_complexity": 5 },
                "tests/test_main.py::test_run": { "cyclomatic_complexity": 25 }
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.summary.passed, 1);
    assert!(report.results[0].violations.is_empty());
}

#[test]
fn min_threshold_violation() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "min-instability"
name = "Min Instability"
source = "metrics/coupling.json"
field = "instability"
min = 0.1
scope = "module"
"#,
        None,
        &[(
            "metrics/coupling.json",
            r#"{
                "src/core": { "instability": 0.05 },
                "src/api": { "instability": 0.5 }
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results[0].violations.len(), 1);
    let v = &report.results[0].violations[0];
    assert_eq!(v.entity, "src/core");
    assert_eq!(v.actual, 0.05);
    assert_eq!(v.threshold, 0.1);
}

#[test]
fn array_format_artifact() {
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
            r#"[
                { "path": "src/main.py::run", "cyclomatic_complexity": 5 },
                { "path": "src/engine.py::execute", "cyclomatic_complexity": 20 }
            ]"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results[0].violations.len(), 1);
    assert_eq!(
        report.results[0].violations[0].entity,
        "src/engine.py::execute"
    );
}

#[test]
fn missing_config_returns_error() {
    let dir = TempDir::new().unwrap();
    let result = FitnessValidator::load(dir.path(), None);
    assert!(result.is_err());
}

#[test]
fn missing_topology_dir_returns_error() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("fitness.toml"),
        r#"
[config]
topology_dir = ".topology"
"#,
    )
    .unwrap();
    let result = FitnessValidator::load(dir.path(), None);
    assert!(result.is_err());
}

#[test]
fn report_json_roundtrip() {
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
            r#"{ "src/main.py::run": { "cyclomatic_complexity": 20 } }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    // Serialize to JSON and back
    let json = serde_json::to_string_pretty(&report).unwrap();
    let parsed: fitness_functions::FitnessReport = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.summary.failed, 1);
    assert_eq!(parsed.results[0].violations[0].entity, "src/main.py::run");
}

#[test]
fn wrapped_functions_format() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-cognitive"
name = "Max Cognitive Complexity"
source = "metrics/functions.json"
field = "metrics.cognitive"
max = 15
scope = "function"
"#,
        None,
        &[(
            "metrics/functions.json",
            r#"{
                "functions": [
                    { "id": "python:mod::safe_func", "metrics": { "cognitive": 5, "cyclomatic": 2 } },
                    { "id": "python:mod::complex_func", "metrics": { "cognitive": 20, "cyclomatic": 8 } }
                ]
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.summary.failed, 1);
    assert_eq!(report.results[0].violations.len(), 1);
    assert_eq!(
        report.results[0].violations[0].entity,
        "python:mod::complex_func"
    );
    assert_eq!(report.results[0].violations[0].actual, 20.0);
    assert_eq!(report.results[0].violations[0].threshold, 15.0);
}

#[test]
fn wrapped_modules_format() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-ce"
name = "Max Efferent Coupling"
source = "metrics/modules.json"
field = "metrics.martin.ce"
max = 30
scope = "module"
"#,
        None,
        &[(
            "metrics/modules.json",
            r#"{
                "modules": [
                    { "id": "packages.syn-domain.core", "metrics": { "martin": { "ce": 10, "ca": 5 } } },
                    { "id": "packages.syn-adapters.heavy", "metrics": { "martin": { "ce": 35, "ca": 2 } } }
                ]
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.summary.failed, 1);
    assert_eq!(report.results[0].violations.len(), 1);
    assert_eq!(
        report.results[0].violations[0].entity,
        "packages.syn-adapters.heavy"
    );
    assert_eq!(report.results[0].violations[0].actual, 35.0);
}

#[test]
fn wrapped_slices_with_extra_keys() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-loc"
name = "Max LOC per Slice"
source = "metrics/slices.json"
field = "metrics.lines_of_code"
max = 1000
scope = "slice"
"#,
        None,
        &[(
            "metrics/slices.json",
            r#"{
                "schema_version": "1.0",
                "metadata": { "generated_at": "2026-03-12" },
                "slices": [
                    { "id": "orchestration.execute_workflow", "metrics": { "lines_of_code": 500 } },
                    { "id": "orchestration.provision_workspace", "metrics": { "lines_of_code": 1200 } }
                ]
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.summary.failed, 1);
    assert_eq!(report.results[0].violations.len(), 1);
    assert_eq!(
        report.results[0].violations[0].entity,
        "orchestration.provision_workspace"
    );
    assert_eq!(report.results[0].violations[0].actual, 1200.0);
}

#[test]
fn dot_path_field_traversal() {
    // Flat object format (no wrapper) but with nested field path
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-difficulty"
name = "Max Halstead Difficulty"
source = "metrics/complexity.json"
field = "halstead.difficulty"
max = 20.0
scope = "function"
"#,
        None,
        &[(
            "metrics/complexity.json",
            r#"{
                "src/main.py::run": { "halstead": { "difficulty": 8.5, "effort": 100.0 } },
                "src/engine.py::execute": { "halstead": { "difficulty": 25.0, "effort": 500.0 } }
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.summary.failed, 1);
    assert_eq!(report.results[0].violations.len(), 1);
    assert_eq!(
        report.results[0].violations[0].entity,
        "src/engine.py::execute"
    );
    assert_eq!(report.results[0].violations[0].actual, 25.0);
}

#[test]
fn insufficient_budget_exception_not_reported_as_stale() {
    // Regression: an exception whose budget is exceeded (metric > budget) should NOT
    // be flagged as EntityNotFound stale. The entity exists and is matched — it just
    // needs a tighter budget. Only "now passing" or truly absent entities are stale.
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
        Some(
            r##"
[max-cc."src/engine.py::execute"]
value = 30
issue = "#99"
"##,
        ),
        &[(
            "metrics/complexity.json",
            // entity exceeds threshold (20 > 15) AND exceeds budget (20 ≤ 30 is false — wait,
            // 20 <= 30 is true, so excepted = true. Let's use 35 > 30 to trigger budget exceeded.
            r#"{
                "src/engine.py::execute": { "cyclomatic_complexity": 35 }
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    // The exception exists but the budget is insufficient (35 > 30), so it fails.
    assert_eq!(report.summary.failed, 1);
    assert!(!report.results[0].violations[0].excepted);
    // But the exception should NOT be reported as stale — the entity is still there.
    assert_eq!(
        report.stale_exceptions.len(),
        0,
        "insufficient-budget exception must not be flagged as EntityNotFound"
    );
}
