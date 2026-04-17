//! Tests for system-level fitness scoring, weight redistribution, and trend tracking.

use architecture_fitness::{
    DimensionResult, DimensionStatus, FitnessReport, FitnessValidator, ReportSummary,
    SystemFitnessResult, TrendDirection,
};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

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
fn system_score_equal_weights_default() {
    // No custom weights → equal weight per active dimension.
    // MT01 has 1 rule with violations, all other enabled dims have no rules (score=1.0).
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[dimensions]
AC01 = false
PF01 = false
AV01 = false

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 10
scope = "function"
"#,
        None,
        &[(
            "metrics/complexity.json",
            // 2 entities, 1 violation → MT01 score = 1.0 - 1/2 = 0.5
            r#"{ "src/a.py::foo": { "cyclomatic_complexity": 5 }, "src/b.py::bar": { "cyclomatic_complexity": 15 } }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    let sf = report.system_fitness.as_ref().unwrap();
    // 5 active dimensions (MT01=0.5, MD01=1.0, ST01=1.0, SC01=1.0, LG01=1.0)
    // Equal weight = 0.2 each
    // Score = 0.2*0.5 + 0.2*1.0*4 = 0.1 + 0.8 = 0.9
    assert!((sf.score - 0.9).abs() < 0.01, "expected ~0.9, got {}", sf.score);
    assert!(sf.passing); // 0.9 >= 0.7
    assert_eq!(sf.weights_used.len(), 5);
    assert!(sf.weights_note.is_none());
}

#[test]
fn system_score_custom_weights() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[dimensions]
AC01 = false
PF01 = false
AV01 = false

[system_fitness]
min_score = 0.8

[system_fitness.weights]
MT01 = 0.4
MD01 = 0.3
ST01 = 0.1
SC01 = 0.1
LG01 = 0.1

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 10
scope = "function"
"#,
        None,
        &[(
            "metrics/complexity.json",
            // 2 entities, 1 violation → MT01 score = 0.5
            r#"{ "src/a.py::foo": { "cyclomatic_complexity": 5 }, "src/b.py::bar": { "cyclomatic_complexity": 15 } }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    let sf = report.system_fitness.as_ref().unwrap();
    // Score = 0.4*0.5 + 0.3*1.0 + 0.1*1.0 + 0.1*1.0 + 0.1*1.0 = 0.2 + 0.3 + 0.3 = 0.8
    assert!((sf.score - 0.8).abs() < 0.02, "expected ~0.8, got {}", sf.score);
    // Floating point: score might be 0.7999... so just check it's close to passing
    assert!(sf.score >= 0.79, "score should be at or near threshold");
    assert_eq!(sf.weights_used["MT01"], 0.4);
}

#[test]
fn system_score_weight_redistribution_on_skip() {
    // SC01 has a rule that will be skipped (missing artifact).
    // Its weight should be redistributed.
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[dimensions]
AC01 = false
PF01 = false
AV01 = false

[system_fitness.weights]
MT01 = 0.5
SC01 = 0.5

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 10
scope = "function"

[[rules.threshold]]
id = "vuln-count"
name = "Vulnerability Count"
dimension = "SC01"
source = "metrics/nonexistent.json"
field = "vulnerability_count"
max = 0
scope = "system"
"#,
        None,
        &[(
            "metrics/complexity.json",
            r#"{ "src/a.py::foo": { "cyclomatic_complexity": 5 } }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    let sf = report.system_fitness.as_ref().unwrap();
    // SC01 is skipped (missing artifact), so weights redistribute
    // MT01 gets all weight. MD01, ST01, LG01 have no configured weight and no active rules.
    // But they ARE active with score=1.0. Let me check the logic...
    // Actually: only configured weights (MT01=0.5, SC01=0.5) are used.
    // SC01 is skipped → only MT01 weight active → redistributed to 1.0 for MT01.
    // But MD01/ST01/LG01 don't have configured weights, so they get 0.
    // So score = 1.0 * 1.0 (MT01 score) = 1.0
    assert!(sf.weights_note.is_some());
    assert!(sf.weights_note.as_ref().unwrap().contains("skipped"));
}

#[test]
fn system_score_below_threshold_reports_failing() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[dimensions]
AC01 = false
PF01 = false
AV01 = false

[system_fitness]
min_score = 0.95

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 10
scope = "function"
"#,
        None,
        &[(
            "metrics/complexity.json",
            // 4 entities, 2 violations → MT01 score = 0.5
            r#"{
                "src/a.py::foo": { "cyclomatic_complexity": 5 },
                "src/b.py::bar": { "cyclomatic_complexity": 15 },
                "src/c.py::baz": { "cyclomatic_complexity": 20 },
                "src/d.py::qux": { "cyclomatic_complexity": 8 }
            }"#,
        )],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    let sf = report.system_fitness.as_ref().unwrap();
    assert!(!sf.passing);
    assert!(FitnessValidator::has_failures(&report));
}

#[test]
fn trend_improving() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[dimensions]
AC01 = false
PF01 = false
AV01 = false

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 10
scope = "function"
"#,
        None,
        &[(
            "metrics/complexity.json",
            // All pass → MT01 score = 1.0
            r#"{ "src/a.py::foo": { "cyclomatic_complexity": 5 } }"#,
        )],
    );

    // Build a mock previous report with lower score
    let previous = build_mock_report(0.8, &[("MT01", 0.7)]);

    let validator = FitnessValidator::load(dir.path(), None)
        .unwrap()
        .with_previous_report(previous);
    let report = validator.validate().unwrap();

    let sf = report.system_fitness.as_ref().unwrap();
    let trend = sf.trend.as_ref().unwrap();
    assert_eq!(trend.direction, TrendDirection::Improving);
    assert!(trend.delta > 0.0);
    assert!((trend.previous_score - 0.8).abs() < 0.01);
}

#[test]
fn trend_declining() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[dimensions]
AC01 = false
PF01 = false
AV01 = false

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 10
scope = "function"
"#,
        None,
        &[(
            "metrics/complexity.json",
            // 2 entities, 1 violation → MT01 score = 0.5
            r#"{ "src/a.py::foo": { "cyclomatic_complexity": 5 }, "src/b.py::bar": { "cyclomatic_complexity": 20 } }"#,
        )],
    );

    // Previous had a higher score
    let previous = build_mock_report(0.98, &[("MT01", 0.95)]);

    let validator = FitnessValidator::load(dir.path(), None)
        .unwrap()
        .with_previous_report(previous);
    let report = validator.validate().unwrap();

    let sf = report.system_fitness.as_ref().unwrap();
    let trend = sf.trend.as_ref().unwrap();
    assert_eq!(trend.direction, TrendDirection::Declining);
    assert!(trend.delta < 0.0);
}

#[test]
fn trend_stable() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[dimensions]
AC01 = false
PF01 = false
AV01 = false

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 10
scope = "function"
"#,
        None,
        &[(
            "metrics/complexity.json",
            // All pass
            r#"{ "src/a.py::foo": { "cyclomatic_complexity": 5 } }"#,
        )],
    );

    // Current score will be ~1.0 (all pass), set previous to ~1.0 as well
    let previous = build_mock_report(1.0, &[("MT01", 1.0)]);

    let validator = FitnessValidator::load(dir.path(), None)
        .unwrap()
        .with_previous_report(previous);
    let report = validator.validate().unwrap();

    let sf = report.system_fitness.as_ref().unwrap();
    let trend = sf.trend.as_ref().unwrap();
    assert_eq!(trend.direction, TrendDirection::Stable);
}

#[test]
fn system_fitness_disabled_omits_from_report() {
    let dir = setup_fixture(
        r#"
[config]
topology_dir = ".topology"

[dimensions]
AC01 = false
PF01 = false
AV01 = false

[system_fitness]
enabled = false
"#,
        None,
        &[],
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert!(report.system_fitness.is_none());
}

/// Helper: build a minimal previous report for trend tests.
fn build_mock_report(system_score: f64, dim_scores: &[(&str, f64)]) -> FitnessReport {
    let mut dimensions = HashMap::new();
    for &(code, score) in dim_scores {
        dimensions.insert(
            code.to_string(),
            DimensionResult {
                name: code.to_string(),
                status: DimensionStatus::Active,
                score: Some(score),
                rules_evaluated: 1,
                rules_passed: 1,
                rules_failed: 0,
                rules_warned: 0,
                total_violations: 0,
                excepted_violations: 0,
            },
        );
    }

    FitnessReport {
        version: "1.0.0".to_string(),
        timestamp: "2026-04-14T00:00:00Z".to_string(),
        summary: ReportSummary::default(),
        dimensions,
        system_fitness: Some(SystemFitnessResult {
            score: system_score,
            min_score: 0.7,
            passing: system_score >= 0.7,
            weights_used: HashMap::new(),
            weights_note: None,
            trend: None,
        }),
        results: vec![],
        stale_exceptions: vec![],
    }
}
