//! The ST01 dependency-graph contract: the engine reads the graph
//! code-topology writes (`graphs/dependency-graph.json`, schema 2.x, nodes
//! keyed by `path`), and every way a rule could test nothing fails closed:
//! a missing or malformed graph, an empty matcher, an invalid glob.

use architecture_fitness::{FitnessValidator, RuleStatus};
use std::fs;
use tempfile::TempDir;

const HEADER: &str = r#"
[config]
topology_dir = ".topology"
"#;

/// The 2.0.0 graph code-topology writes for a crate where `domain::model`
/// (src/domain/model.rs) imports `infra::db` (src/infra/db.rs).
const V2_GRAPH: &str = r#"{
  "schema_version": "2.0.0",
  "level": "module",
  "nodes": [
    { "id": "domain::model", "type": "module", "path": "src/domain/model.rs" },
    { "id": "infra::db", "type": "module", "path": "src/infra/db.rs" }
  ],
  "edges": [
    { "from": "domain::model", "to": "infra::db", "type": "imports", "weight": 1 }
  ]
}"#;

fn forbidden_rule(from: &str, to: &str, extra: &str) -> String {
    format!(
        r#"
[[rules.structural]]
id = "domain-not-infra"
name = "Domain must not import infra"
pattern = "forbidden_import"
from = {{ path = "{from}" }}
to = {{ path = "{to}" }}
{extra}
"#
    )
}

fn fixture(rules: &str, graph: Option<&str>) -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("fitness.toml"), format!("{HEADER}{rules}")).unwrap();
    let graphs = dir.path().join(".topology/graphs");
    fs::create_dir_all(&graphs).unwrap();
    if let Some(g) = graph {
        fs::write(graphs.join("dependency-graph.json"), g).unwrap();
    }
    dir
}

fn report(dir: &TempDir) -> architecture_fitness::FitnessReport {
    FitnessValidator::load(dir.path(), None)
        .unwrap()
        .validate()
        .unwrap()
}

#[test]
fn v2_graph_forbidden_edge_fails_on_the_file_path() {
    let dir = fixture(
        &forbidden_rule("**/domain/**", "**/infra/**", ""),
        Some(V2_GRAPH),
    );
    let r = report(&dir);
    assert_eq!(r.results[0].status, RuleStatus::Fail);
    assert_eq!(r.results[0].violations.len(), 1);
    assert_eq!(r.results[0].violations[0].entity, "src/domain/model.rs");
}

#[test]
fn v2_graph_without_the_edge_passes() {
    let graph = V2_GRAPH.replace(
        r#"{ "from": "domain::model", "to": "infra::db", "type": "imports", "weight": 1 }"#,
        "",
    );
    let dir = fixture(
        &forbidden_rule("**/domain/**", "**/infra/**", ""),
        Some(&graph),
    );
    let r = report(&dir);
    assert_eq!(r.results[0].status, RuleStatus::Pass);
}

#[test]
fn module_ids_are_not_matched_by_path_globs() {
    // `domain::model` is an id, not a path: a rule written against ids
    // matches no node, so the vacuity guard fails it rather than passing.
    let dir = fixture(&forbidden_rule("domain::*", "infra::*", ""), Some(V2_GRAPH));
    let r = report(&dir);
    assert_eq!(r.results[0].status, RuleStatus::Fail);
    assert_eq!(
        r.results[0].violations[0].entity,
        "RULE_MATCHED_NO_NODE:from"
    );
}

#[test]
fn empty_to_matcher_fails() {
    let dir = fixture(
        &forbidden_rule("**/domain/**", "**/nowhere/**", ""),
        Some(V2_GRAPH),
    );
    let r = report(&dir);
    assert_eq!(r.results[0].status, RuleStatus::Fail);
    assert_eq!(r.results[0].violations[0].entity, "RULE_MATCHED_NO_NODE:to");
}

#[test]
fn allow_empty_reason_makes_the_empty_match_a_visible_exception() {
    let rule = forbidden_rule(
        "**/domain/**",
        "**/nowhere/**",
        r#"allow_empty_reason = "written ahead of the nowhere crate""#,
    );
    let dir = fixture(&rule, Some(V2_GRAPH));
    let r = report(&dir);
    let result = &r.results[0];
    assert_eq!(result.status, RuleStatus::Pass);
    assert_eq!(result.exceptions_used, 1);
    assert!(result.violations[0].excepted);
    assert!(
        result.violations[0]
            .message
            .as_deref()
            .unwrap()
            .contains("written ahead of the nowhere crate")
    );
}

#[test]
fn blank_allow_empty_reason_is_a_config_error() {
    let rule = forbidden_rule(
        "**/domain/**",
        "**/infra/**",
        r#"allow_empty_reason = "  ""#,
    );
    let dir = fixture(&rule, Some(V2_GRAPH));
    let Err(err) = FitnessValidator::load(dir.path(), None) else {
        panic!("load must fail");
    };
    assert!(err.to_string().contains("allow_empty_reason"), "{err}");
}

#[test]
fn invalid_glob_is_a_config_error() {
    let dir = fixture(
        &forbidden_rule("src/[domain", "**/infra/**", ""),
        Some(V2_GRAPH),
    );
    let Err(err) = FitnessValidator::load(dir.path(), None) else {
        panic!("load must fail");
    };
    assert!(err.to_string().contains("invalid from glob"), "{err}");
}

#[test]
fn edge_to_an_unknown_node_fails_the_rule() {
    let graph = V2_GRAPH.replace(r#""to": "infra::db""#, r#""to": "infra::gone""#);
    let dir = fixture(
        &forbidden_rule("**/domain/**", "**/infra/**", ""),
        Some(&graph),
    );
    let r = report(&dir);
    assert_eq!(r.results[0].status, RuleStatus::Fail);
    let v = &r.results[0].violations[0];
    assert_eq!(
        v.entity,
        "DEPENDENCY_GRAPH_INVALID:graphs/dependency-graph.json"
    );
    assert!(v.message.as_deref().unwrap().contains("infra::gone"));
}

#[test]
fn node_without_a_path_fails_the_rule() {
    let graph = V2_GRAPH.replace(r#", "path": "src/infra/db.rs""#, "");
    let dir = fixture(
        &forbidden_rule("**/domain/**", "**/infra/**", ""),
        Some(&graph),
    );
    let r = report(&dir);
    assert_eq!(r.results[0].status, RuleStatus::Fail);
    assert!(
        r.results[0].violations[0]
            .message
            .as_deref()
            .unwrap()
            .contains("path")
    );
}

#[test]
fn unsupported_schema_version_fails_the_rule() {
    let graph = V2_GRAPH.replace(r#""2.0.0""#, r#""1.0.0""#);
    let dir = fixture(
        &forbidden_rule("**/domain/**", "**/infra/**", ""),
        Some(&graph),
    );
    let r = report(&dir);
    assert_eq!(r.results[0].status, RuleStatus::Fail);
    assert!(
        r.results[0].violations[0]
            .message
            .as_deref()
            .unwrap()
            .contains("unsupported schema_version 1.0.0")
    );
}

#[test]
fn unparseable_graph_on_a_dependency_rule_is_an_error() {
    let rules = r#"
[[rules.dependency]]
id = "no-cycles"
name = "No cycles"
type = "forbidden"
from = { path = "src/**" }
to = { path = "src/**" }
circular = true
"#;
    let dir = fixture(rules, Some("{ not json"));
    let Err(err) = FitnessValidator::load(dir.path(), None).unwrap().validate() else {
        panic!("validate must fail");
    };
    assert!(err.to_string().contains("dependency-graph.json"), "{err}");
}

#[test]
fn missing_graph_skips_only_on_an_incubating_dimension() {
    let rule = format!(
        "{}dimension = \"PF01\"\n",
        forbidden_rule("**/domain/**", "**/infra/**", "")
    );
    let dir = fixture(&rule, None);
    let r = report(&dir);
    assert_eq!(r.results[0].status, RuleStatus::Skip);
}
