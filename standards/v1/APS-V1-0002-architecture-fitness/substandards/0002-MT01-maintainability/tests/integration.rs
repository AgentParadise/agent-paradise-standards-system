//! Integration tests for MT01 default rules.
//!
//! Drives the APS-V1-0002 fitness engine against fixture `functions.json`
//! artifacts to pin the behavior of each default rule.

use architecture_fitness::{FitnessValidator, RuleStatus};
use architecture_fitness_mt01::{DEFAULT_RULES_TOML, DIMENSION_CODE};
use std::fs;
use std::path::Path;

fn write_functions_fixture(topology_dir: &Path, json: &str) {
    let metrics_dir = topology_dir.join("metrics");
    fs::create_dir_all(&metrics_dir).unwrap();
    fs::write(metrics_dir.join("functions.json"), json).unwrap();
}

fn write_fitness_config(repo_root: &Path, extra_rules: &str) {
    let config = format!(
        r#"
[config]
topology_dir = ".topology"
severity_default = "error"

[dimensions]
MT01 = true
MD01 = false
ST01 = false
SC01 = false
LG01 = false

[dimensions.reasons]
MD01 = "test isolates MT01"
ST01 = "test isolates MT01"
SC01 = "test isolates MT01"
LG01 = "test isolates MT01"

[system_fitness]
enabled = false

{extra_rules}
"#
    );
    fs::write(repo_root.join("fitness.toml"), config).unwrap();
}

fn find_rule_result<'a>(
    results: &'a [architecture_fitness::RuleResult],
    id: &str,
) -> &'a architecture_fitness::RuleResult {
    results
        .iter()
        .find(|r| r.rule_id == id)
        .unwrap_or_else(|| panic!("rule {id} not in results"))
}

#[test]
fn default_rules_flag_high_cyclomatic() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_fitness_config(root, DEFAULT_RULES_TOML);
    write_functions_fixture(
        &root.join(".topology"),
        r#"{
  "schema_version": "1.0.0",
  "functions": [
    {
      "id": "rust:complex::validate",
      "name": "validate",
      "module": "complex",
      "file": "src/complex/validate.rs",
      "line": 10,
      "metrics": {
        "cyclomatic": 18,
        "cognitive": 8,
        "halstead": {
          "vocabulary": 20,
          "length": 50,
          "volume": 200.0,
          "difficulty": 5.0,
          "effort": 1000.0
        },
        "loc": 40
      }
    },
    {
      "id": "rust:simple::add",
      "name": "add",
      "module": "simple",
      "file": "src/simple/add.rs",
      "line": 3,
      "metrics": {
        "cyclomatic": 1,
        "cognitive": 0,
        "halstead": {
          "vocabulary": 5,
          "length": 10,
          "volume": 20.0,
          "difficulty": 1.0,
          "effort": 20.0
        },
        "loc": 3
      }
    }
  ]
}"#,
    );

    let validator = FitnessValidator::load(root, None).expect("load");
    let report = validator.validate().expect("validate");

    let cc_rule = find_rule_result(&report.results, "mt01-max-cyclomatic");
    assert_eq!(cc_rule.status, RuleStatus::Fail);
    assert_eq!(cc_rule.violations.len(), 1);
    assert_eq!(cc_rule.violations[0].entity, "rust:complex::validate");
    assert_eq!(cc_rule.violations[0].actual, 18.0);
    assert_eq!(cc_rule.violations[0].threshold, 10.0);
    assert_eq!(cc_rule.dimension.as_deref(), Some(DIMENSION_CODE));
}

#[test]
fn default_rules_flag_high_cognitive() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_fitness_config(root, DEFAULT_RULES_TOML);
    write_functions_fixture(
        &root.join(".topology"),
        r#"{
  "schema_version": "1.0.0",
  "functions": [
    {
      "id": "rust:mod::nested",
      "name": "nested",
      "module": "mod",
      "file": "src/mod/nested.rs",
      "line": 1,
      "metrics": {
        "cyclomatic": 5,
        "cognitive": 25,
        "halstead": {
          "vocabulary": 15,
          "length": 40,
          "volume": 150.0,
          "difficulty": 3.0,
          "effort": 450.0
        },
        "loc": 30
      }
    }
  ]
}"#,
    );

    let validator = FitnessValidator::load(root, None).expect("load");
    let report = validator.validate().expect("validate");

    let cog_rule = find_rule_result(&report.results, "mt01-max-cognitive");
    assert_eq!(cog_rule.status, RuleStatus::Fail);
    assert_eq!(cog_rule.violations.len(), 1);
    assert_eq!(cog_rule.violations[0].actual, 25.0);
}

#[test]
fn default_rules_flag_high_halstead_volume() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_fitness_config(root, DEFAULT_RULES_TOML);
    write_functions_fixture(
        &root.join(".topology"),
        r#"{
  "schema_version": "1.0.0",
  "functions": [
    {
      "id": "rust:heavy::process",
      "name": "process",
      "module": "heavy",
      "file": "src/heavy/process.rs",
      "line": 1,
      "metrics": {
        "cyclomatic": 8,
        "cognitive": 10,
        "halstead": {
          "vocabulary": 80,
          "length": 300,
          "volume": 1800.0,
          "difficulty": 20.0,
          "effort": 36000.0
        },
        "loc": 60
      }
    }
  ]
}"#,
    );

    let validator = FitnessValidator::load(root, None).expect("load");
    let report = validator.validate().expect("validate");

    let vol_rule = find_rule_result(&report.results, "mt01-max-halstead-volume");
    assert_eq!(vol_rule.status, RuleStatus::Warn);
    assert_eq!(vol_rule.violations.len(), 1);
    assert_eq!(vol_rule.violations[0].actual, 1800.0);
}

#[test]
fn clean_function_passes_all_default_rules() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_fitness_config(root, DEFAULT_RULES_TOML);
    write_functions_fixture(
        &root.join(".topology"),
        r#"{
  "schema_version": "1.0.0",
  "functions": [
    {
      "id": "rust:clean::small",
      "name": "small",
      "module": "clean",
      "file": "src/clean/small.rs",
      "line": 1,
      "metrics": {
        "cyclomatic": 3,
        "cognitive": 4,
        "halstead": {
          "vocabulary": 15,
          "length": 30,
          "volume": 120.0,
          "difficulty": 2.0,
          "effort": 240.0
        },
        "loc": 12
      }
    }
  ]
}"#,
    );

    let validator = FitnessValidator::load(root, None).expect("load");
    let report = validator.validate().expect("validate");

    for rule in &report.results {
        assert_eq!(
            rule.status,
            RuleStatus::Pass,
            "rule {} should pass for clean function",
            rule.rule_id
        );
        assert!(rule.violations.is_empty());
    }
}
