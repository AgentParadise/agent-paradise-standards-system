//! Tests for `evaluate_structural_rule` - the ST01 substandard's three
//! documented pattern types (forbidden_import, required_import,
//! layer_enforcement). The evaluator delegates to the dependency-graph path
//! so it shares the same correctness story as `[[rules.dependency]]`.

use architecture_fitness::{FitnessValidator, RuleStatus};
use std::fs;
use tempfile::TempDir;

fn setup_struct_fixture(rules_toml: &str, graph_json: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    fs::write(root.join("fitness.toml"), rules_toml).unwrap();
    let topo_dir = root.join(".topology");
    fs::create_dir_all(topo_dir.join("graphs")).unwrap();
    fs::write(topo_dir.join("graphs/dependency-graph.json"), graph_json).unwrap();

    dir
}

#[test]
fn forbidden_import_pattern_fails_when_edge_exists() {
    let dir = setup_struct_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.structural]]
id = "controllers-no-repos"
name = "Controllers MUST NOT import Repositories"
dimension = "ST01"
pattern = "forbidden_import"
from = { path = "src/controllers" }
to = { path = "src/repositories" }
severity = "error"
"#,
        r#"{
            "nodes": ["src/controllers", "src/services", "src/repositories"],
            "edges": [
                ["src/controllers", "src/services"],
                ["src/controllers", "src/repositories"],
                ["src/services", "src/repositories"]
            ]
        }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results.len(), 1);
    let result = &report.results[0];
    assert_eq!(result.status, RuleStatus::Fail);
    assert_eq!(result.dimension.as_deref(), Some("ST01"));
    assert_eq!(result.violations.len(), 1);
    assert_eq!(result.violations[0].entity, "src/controllers");
}

#[test]
fn forbidden_import_pattern_passes_when_no_edge() {
    let dir = setup_struct_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.structural]]
id = "controllers-no-repos"
name = "Controllers MUST NOT import Repositories"
dimension = "ST01"
pattern = "forbidden_import"
from = { path = "src/controllers" }
to = { path = "src/repositories" }
"#,
        r#"{
            "nodes": ["src/controllers", "src/services", "src/repositories"],
            "edges": [
                ["src/controllers", "src/services"],
                ["src/services", "src/repositories"]
            ]
        }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].status, RuleStatus::Pass);
}

#[test]
fn required_import_pattern_fails_when_missing_edge() {
    let dir = setup_struct_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.structural]]
id = "controllers-must-use-service-bus"
name = "Controllers MUST import the service bus"
dimension = "ST01"
pattern = "required_import"
from = { path = "src/controllers" }
to = { path = "src/service-bus" }
severity = "error"
"#,
        r#"{
            "nodes": ["src/controllers", "src/repositories", "src/service-bus"],
            "edges": [["src/controllers", "src/repositories"]]
        }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].status, RuleStatus::Fail);
    assert_eq!(report.results[0].violations[0].entity, "src/controllers");
}

#[test]
fn layer_enforcement_aliases_forbidden_import() {
    // `layer_enforcement` is documented in the ST01 pattern catalog as a
    // synonym for `forbidden_import` aimed at layered architectures
    // (e.g., controllers MUST NOT import repositories directly).
    let dir = setup_struct_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.structural]]
id = "no-skip-layers"
name = "Controllers MUST NOT skip the service layer"
dimension = "ST01"
pattern = "layer_enforcement"
from = { path = "src/controllers" }
to = { path = "src/repositories" }
severity = "error"
"#,
        r#"{
            "nodes": ["src/controllers", "src/repositories"],
            "edges": [["src/controllers", "src/repositories"]]
        }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].status, RuleStatus::Fail);
}

#[test]
fn unknown_pattern_fails_with_invalid_structural_pattern() {
    let dir = setup_struct_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.structural]]
id = "garbage"
name = "Garbage pattern"
dimension = "ST01"
pattern = "not_a_real_pattern"
from = { path = "src" }
to = { path = "src" }
"#,
        r#"{ "nodes": [], "edges": [] }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results.len(), 1);
    let result = &report.results[0];
    assert_eq!(result.status, RuleStatus::Fail);
    assert_eq!(result.violations.len(), 1);
    assert!(
        result.violations[0]
            .entity
            .starts_with("INVALID_STRUCTURAL_PATTERN:")
    );
}

#[test]
fn missing_from_to_fails_fast() {
    let dir = setup_struct_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.structural]]
id = "incomplete"
name = "Missing from / to"
dimension = "ST01"
pattern = "forbidden_import"
"#,
        r#"{ "nodes": [], "edges": [] }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results.len(), 1);
    let result = &report.results[0];
    assert_eq!(result.status, RuleStatus::Fail);
    assert_eq!(result.violations[0].field, "from/to");
}
