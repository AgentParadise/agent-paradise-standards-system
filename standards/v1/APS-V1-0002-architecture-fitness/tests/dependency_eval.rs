//! Tests for dependency rule evaluation including Tarjan SCC cycle detection.

use architecture_fitness::{FitnessValidator, RuleStatus};
use std::fs;
use tempfile::TempDir;

/// Create a test fixture with a dependency graph and optional threshold artifacts.
fn setup_dep_fixture(
    rules_toml: &str,
    exceptions_toml: Option<&str>,
    graph_json: &str,
) -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    fs::write(root.join("fitness.toml"), rules_toml).unwrap();

    if let Some(exc) = exceptions_toml {
        fs::write(root.join("fitness-exceptions.toml"), exc).unwrap();
    }

    let topo_dir = root.join(".topology");
    fs::create_dir_all(topo_dir.join("graphs")).unwrap();
    fs::write(topo_dir.join("graphs/dependency-graph.json"), graph_json).unwrap();

    dir
}

#[test]
fn forbidden_dependency_detected() {
    let dir = setup_dep_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.dependency]]
id = "no-api-to-infra"
name = "API must not import Infra"
type = "forbidden"
from = { path = "src/api" }
to = { path = "src/infra" }
severity = "error"
"#,
        None,
        r#"{
            "nodes": ["src/api", "src/domain", "src/infra"],
            "edges": [["src/api", "src/domain"], ["src/api", "src/infra"], ["src/domain", "src/infra"]]
        }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].status, RuleStatus::Fail);
    assert_eq!(report.results[0].violations.len(), 1);
    assert_eq!(report.results[0].violations[0].entity, "src/api");
}

#[test]
fn forbidden_no_violation_when_no_edge() {
    let dir = setup_dep_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.dependency]]
id = "no-api-to-infra"
name = "API must not import Infra"
type = "forbidden"
from = { path = "src/api" }
to = { path = "src/infra" }
"#,
        None,
        r#"{
            "nodes": ["src/api", "src/domain", "src/infra"],
            "edges": [["src/api", "src/domain"], ["src/domain", "src/infra"]]
        }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results[0].status, RuleStatus::Pass);
    assert!(report.results[0].violations.is_empty());
}

#[test]
fn forbidden_with_path_not_exclusion() {
    let dir = setup_dep_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.dependency]]
id = "no-cross-boundary"
name = "No cross-boundary imports"
type = "forbidden"
from = { path = "src/*", path_not = "src/shared" }
to = { path = "src/*", path_not = "src/shared" }
"#,
        None,
        r#"{
            "nodes": ["src/api", "src/domain", "src/shared"],
            "edges": [["src/api", "src/shared"], ["src/api", "src/domain"]]
        }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    // src/api→src/shared is excluded (path_not), src/api→src/domain is forbidden
    assert_eq!(report.results[0].violations.len(), 1);
    assert_eq!(report.results[0].violations[0].entity, "src/api");
}

#[test]
fn required_dependency_missing() {
    let dir = setup_dep_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.dependency]]
id = "must-use-domain"
name = "Services must depend on Domain"
type = "required"
from = { path = "src/services/*" }
to = { path = "src/domain" }
severity = "error"
"#,
        None,
        r#"{
            "nodes": ["src/services/auth", "src/services/billing", "src/domain"],
            "edges": [["src/services/auth", "src/domain"]]
        }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    // billing has no edge to domain → violation
    assert_eq!(report.results[0].status, RuleStatus::Fail);
    assert_eq!(report.results[0].violations.len(), 1);
    assert_eq!(
        report.results[0].violations[0].entity,
        "src/services/billing"
    );
}

#[test]
fn circular_dependency_detected_tarjan() {
    let dir = setup_dep_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.dependency]]
id = "no-circular"
name = "No Circular Deps"
type = "forbidden"
from = { path = "src/*" }
to = { path = "src/*" }
circular = true
severity = "error"
"#,
        None,
        r#"{
            "nodes": ["src/a", "src/b", "src/c", "src/d"],
            "edges": [["src/a", "src/b"], ["src/b", "src/c"], ["src/c", "src/a"], ["src/d", "src/a"]]
        }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results[0].status, RuleStatus::Fail);
    // a→b→c→a forms a cycle of 3 nodes
    assert_eq!(report.results[0].violations.len(), 3);

    let violation_entities: Vec<&str> = report.results[0]
        .violations
        .iter()
        .map(|v| v.entity.as_str())
        .collect();
    assert!(violation_entities.contains(&"src/a"));
    assert!(violation_entities.contains(&"src/b"));
    assert!(violation_entities.contains(&"src/c"));
    // src/d is NOT in the cycle
    assert!(!violation_entities.contains(&"src/d"));
}

#[test]
fn no_circular_when_dag() {
    let dir = setup_dep_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.dependency]]
id = "no-circular"
name = "No Circular Deps"
type = "forbidden"
from = { path = "src/*" }
to = { path = "src/*" }
circular = true
"#,
        None,
        r#"{
            "nodes": ["src/a", "src/b", "src/c"],
            "edges": [["src/a", "src/b"], ["src/b", "src/c"]]
        }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results[0].status, RuleStatus::Pass);
    assert!(report.results[0].violations.is_empty());
}

#[test]
fn dependency_exception_makes_violation_pass() {
    let dir = setup_dep_fixture(
        r#"
[config]
topology_dir = ".topology"

[[rules.dependency]]
id = "no-api-to-infra"
name = "API must not import Infra"
type = "forbidden"
from = { path = "src/api" }
to = { path = "src/infra" }
"#,
        Some(
            r##"
[no-api-to-infra."src/api"]
targets = ["src/infra"]
issue = "#300"
"##,
        ),
        r#"{
            "nodes": ["src/api", "src/infra"],
            "edges": [["src/api", "src/infra"]]
        }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results[0].status, RuleStatus::Pass);
    assert_eq!(report.results[0].violations.len(), 1);
    assert!(report.results[0].violations[0].excepted);
    assert_eq!(report.results[0].exceptions_used, 1);
}

#[test]
fn dependency_rule_skipped_when_graph_missing() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    fs::write(
        root.join("fitness.toml"),
        r#"
[config]
topology_dir = ".topology"

[[rules.dependency]]
id = "no-circular"
name = "No Circular Deps"
type = "forbidden"
from = { path = "src/**" }
to = { path = "src/**" }
circular = true
"#,
    )
    .unwrap();

    // Create topology dir but NO graphs/dependency-graph.json
    fs::create_dir_all(root.join(".topology")).unwrap();

    let validator = FitnessValidator::load(root, None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results[0].status, RuleStatus::Skip);
}

#[test]
fn dependency_violations_contribute_to_dimension_score() {
    let dir = setup_dep_fixture(
        r#"
[config]
topology_dir = ".topology"

[dimensions]
AC01 = false
PF01 = false
AV01 = false

[[rules.dependency]]
id = "no-circular"
name = "No Circular Deps"
dimension = "MD01"
type = "forbidden"
from = { path = "src/*" }
to = { path = "src/*" }
circular = true
severity = "error"
"#,
        None,
        r#"{
            "nodes": ["src/a", "src/b", "src/c"],
            "edges": [["src/a", "src/b"], ["src/b", "src/c"], ["src/c", "src/a"]]
        }"#,
    );

    let validator = FitnessValidator::load(dir.path(), None).unwrap();
    let report = validator.validate().unwrap();

    let md01 = &report.dimensions["MD01"];
    assert_eq!(md01.rules_failed, 1);
    assert_eq!(md01.total_violations, 3); // 3 nodes in cycle
    // score = 1.0 - 3/3 = 0.0 (all entities violate)
    assert!((md01.score.unwrap() - 0.0).abs() < 0.01);
}
