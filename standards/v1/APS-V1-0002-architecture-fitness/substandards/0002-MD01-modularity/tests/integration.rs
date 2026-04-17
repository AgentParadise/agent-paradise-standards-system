//! Integration tests for MD01 default rules.
//!
//! Drives the APS-V1-0002 fitness engine against fixture `coupling.json`
//! artifacts to pin the behavior of each default rule.

use architecture_fitness::{FitnessValidator, RuleStatus};
use architecture_fitness_md01::{DEFAULT_RULES_TOML, DIMENSION_CODE};
use std::fs;
use std::path::Path;

fn write_coupling_fixture(topology_dir: &Path, json: &str) {
    let metrics_dir = topology_dir.join("metrics");
    fs::create_dir_all(&metrics_dir).unwrap();
    fs::write(metrics_dir.join("coupling.json"), json).unwrap();
}

fn write_fitness_config(repo_root: &Path, extra_rules: &str) {
    let config = format!(
        r#"
[config]
topology_dir = ".topology"
severity_default = "error"

[dimensions]
MT01 = false
MD01 = true
ST01 = false
SC01 = false
LG01 = false

[dimensions.reasons]
MT01 = "test isolates MD01"
ST01 = "test isolates MD01"
SC01 = "test isolates MD01"
LG01 = "test isolates MD01"

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
fn default_rules_flag_high_efferent_coupling() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_fitness_config(root, DEFAULT_RULES_TOML);
    write_coupling_fixture(
        &root.join(".topology"),
        r#"{
  "schema_version": "1.0.0",
  "modules": [
    {
      "id": "hub",
      "path": "src/hub",
      "afferent_coupling": 1,
      "efferent_coupling": 25,
      "instability": 0.96,
      "abstractness": 0.0,
      "distance_from_main_sequence": 0.04
    },
    {
      "id": "healthy",
      "path": "src/healthy",
      "afferent_coupling": 3,
      "efferent_coupling": 5,
      "instability": 0.625,
      "abstractness": 0.3,
      "distance_from_main_sequence": 0.075
    }
  ]
}"#,
    );

    let validator = FitnessValidator::load(root, None).expect("load");
    let report = validator.validate().expect("validate");

    let ce_rule = find_rule_result(&report.results, "md01-max-efferent-coupling");
    assert_eq!(ce_rule.status, RuleStatus::Fail);
    assert_eq!(ce_rule.violations.len(), 1);
    assert_eq!(ce_rule.violations[0].entity, "hub");
    assert_eq!(ce_rule.violations[0].actual, 25.0);
    assert_eq!(ce_rule.violations[0].threshold, 20.0);
    assert_eq!(ce_rule.dimension.as_deref(), Some(DIMENSION_CODE));
}

#[test]
fn default_rules_flag_instability_extremes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_fitness_config(root, DEFAULT_RULES_TOML);
    write_coupling_fixture(
        &root.join(".topology"),
        r#"{
  "schema_version": "1.0.0",
  "modules": [
    {
      "id": "rigid",
      "path": "src/rigid",
      "afferent_coupling": 10,
      "efferent_coupling": 0,
      "instability": 0.0,
      "abstractness": 0.0,
      "distance_from_main_sequence": 1.0
    },
    {
      "id": "volatile",
      "path": "src/volatile",
      "afferent_coupling": 0,
      "efferent_coupling": 10,
      "instability": 1.0,
      "abstractness": 0.0,
      "distance_from_main_sequence": 0.0
    }
  ]
}"#,
    );

    let validator = FitnessValidator::load(root, None).expect("load");
    let report = validator.validate().expect("validate");

    let instability_rule = find_rule_result(&report.results, "md01-instability-balance");
    assert_eq!(instability_rule.status, RuleStatus::Warn);
    assert_eq!(instability_rule.violations.len(), 2);
    let entities: Vec<_> = instability_rule
        .violations
        .iter()
        .map(|v| v.entity.as_str())
        .collect();
    assert!(entities.contains(&"rigid"));
    assert!(entities.contains(&"volatile"));
}

#[test]
fn default_rules_flag_main_sequence_distance() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_fitness_config(root, DEFAULT_RULES_TOML);
    write_coupling_fixture(
        &root.join(".topology"),
        r#"{
  "schema_version": "1.0.0",
  "modules": [
    {
      "id": "zone-of-pain",
      "path": "src/painful",
      "afferent_coupling": 20,
      "efferent_coupling": 0,
      "instability": 0.0,
      "abstractness": 0.05,
      "distance_from_main_sequence": 0.95
    }
  ]
}"#,
    );

    let validator = FitnessValidator::load(root, None).expect("load");
    let report = validator.validate().expect("validate");

    let distance_rule = find_rule_result(&report.results, "md01-max-main-sequence-distance");
    assert_eq!(distance_rule.status, RuleStatus::Fail);
    assert_eq!(distance_rule.violations.len(), 1);
    assert_eq!(distance_rule.violations[0].entity, "zone-of-pain");
    assert_eq!(distance_rule.violations[0].actual, 0.95);
}

#[test]
fn clean_module_passes_all_default_rules() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_fitness_config(root, DEFAULT_RULES_TOML);
    write_coupling_fixture(
        &root.join(".topology"),
        r#"{
  "schema_version": "1.0.0",
  "modules": [
    {
      "id": "balanced",
      "path": "src/balanced",
      "afferent_coupling": 4,
      "efferent_coupling": 6,
      "instability": 0.6,
      "abstractness": 0.3,
      "distance_from_main_sequence": 0.1
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
            "rule {} should pass for balanced module",
            rule.rule_id
        );
        assert!(rule.violations.is_empty());
    }
}
