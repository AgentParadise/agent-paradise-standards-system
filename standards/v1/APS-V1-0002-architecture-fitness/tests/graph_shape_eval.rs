//! Tests for the ST01 graph-shape patterns `max_dependency_depth` and
//! `max_fan_out` (with composition roots). One test per clause of the ST01
//! spec §3.3 and §3.4.

use architecture_fitness::{FitnessReport, FitnessValidator, RuleResult, RuleStatus, StaleReason};
use std::fs;
use tempfile::TempDir;

const HEADER: &str = "[config]\ntopology_dir = \".topology\"\n\n";

fn fixture(rules: &str, graph: &str, exceptions: Option<&str>) -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::write(root.join("fitness.toml"), format!("{HEADER}{rules}")).unwrap();
    let topo = root.join(".topology/graphs");
    fs::create_dir_all(&topo).unwrap();
    fs::write(topo.join("dependency-graph.json"), graph).unwrap();
    if let Some(exc) = exceptions {
        fs::write(root.join("fitness-exceptions.toml"), exc).unwrap();
    }
    dir
}

fn run(rules: &str, graph: &str, exceptions: Option<&str>) -> FitnessReport {
    let dir = fixture(rules, graph, exceptions);
    FitnessValidator::load(dir.path(), None)
        .unwrap()
        .validate()
        .unwrap()
}

fn only(report: &FitnessReport) -> &RuleResult {
    assert_eq!(report.results.len(), 1);
    &report.results[0]
}

fn load_err(rules: &str) -> String {
    let dir = fixture(rules, r#"{"nodes": [], "edges": []}"#, None);
    match FitnessValidator::load(dir.path(), None) {
        Ok(_) => panic!("config should be rejected"),
        Err(e) => e.to_string(),
    }
}

fn depth_rule(max: u64, extra: &str) -> String {
    format!(
        r#"[[rules.structural]]
id = "depth"
name = "Depth"
pattern = "max_dependency_depth"
from = {{ path = "c/*" }}
max = {max}
{extra}
"#
    )
}

fn fan_rule(max: u64, extra: &str) -> String {
    format!(
        r#"[[rules.structural]]
id = "fan"
name = "Fan"
pattern = "max_fan_out"
from = {{ path = "c/*" }}
max = {max}
{extra}
"#
    )
}

const CHAIN: &str = r#"{
    "nodes": ["c/a", "c/b", "c/c", "c/d"],
    "edges": [["c/a", "c/b"], ["c/b", "c/c"], ["c/c", "c/d"]]
}"#;

// ─── max_dependency_depth ───────────────────────────────────────────────────

#[test]
fn depth_counts_edges_on_the_longest_path_and_names_it() {
    let report = run(&depth_rule(2, ""), CHAIN, None);
    let r = only(&report);
    assert_eq!(r.status, RuleStatus::Fail);
    assert_eq!(r.dimension.as_deref(), Some("ST01"));
    assert_eq!(r.total_entities, Some(4));
    assert_eq!(r.violations.len(), 1, "only c/a has depth 3 > 2");
    let v = &r.violations[0];
    assert_eq!(v.entity, "c/a");
    assert_eq!(v.field, "dependency_depth");
    assert_eq!(v.actual, 3.0);
    assert_eq!(v.threshold, 2.0);
    assert_eq!(
        v.message.as_deref(),
        Some("longest path: c/a -> c/b -> c/c -> c/d")
    );
}

#[test]
fn depth_at_max_passes_and_a_leaf_has_depth_zero() {
    let report = run(&depth_rule(3, ""), CHAIN, None);
    assert_eq!(only(&report).status, RuleStatus::Pass);
    let leaf = run(
        &depth_rule(0, ""),
        r#"{"nodes": ["c/a"], "edges": []}"#,
        None,
    );
    assert_eq!(only(&leaf).status, RuleStatus::Pass);
}

#[test]
fn depth_path_may_pass_through_nodes_outside_from() {
    // c/a -> x/mid -> c/z: x/mid is outside `from` but the path counts it.
    let report = run(
        &depth_rule(1, ""),
        r#"{"nodes": ["c/a", "x/mid", "c/z"],
            "edges": [["c/a", "x/mid"], ["x/mid", "c/z"]]}"#,
        None,
    );
    let r = only(&report);
    assert_eq!(r.violations.len(), 1);
    assert_eq!(r.violations[0].entity, "c/a");
    assert_eq!(r.violations[0].actual, 2.0);
    assert_eq!(r.total_entities, Some(2), "x/mid is not judged");
}

#[test]
fn depth_counts_an_edge_only_when_its_target_is_in_to() {
    // to = c/*: the edge into ext/lib does not count, so c/a has depth 1.
    let report = run(
        &depth_rule(1, r#"to = { path = "c/*" }"#),
        r#"{"nodes": ["c/a", "c/b", "ext/lib"],
            "edges": [["c/a", "c/b"], ["c/b", "ext/lib"]]}"#,
        None,
    );
    assert_eq!(only(&report).status, RuleStatus::Pass);
}

#[test]
fn depth_treats_a_cycle_as_one_node_shared_by_its_members() {
    let report = run(
        &depth_rule(0, ""),
        r#"{"nodes": ["c/a", "c/b", "c/c"],
            "edges": [["c/a", "c/b"], ["c/b", "c/a"], ["c/b", "c/c"]]}"#,
        None,
    );
    let r = only(&report);
    let mut got: Vec<(&str, f64, &str)> = r
        .violations
        .iter()
        .map(|v| (v.entity.as_str(), v.actual, v.message.as_deref().unwrap()))
        .collect();
    got.sort_by(|a, b| a.0.cmp(b.0));
    assert_eq!(
        got,
        vec![
            ("c/a", 1.0, "longest path: {c/a, c/b} -> c/c"),
            ("c/b", 1.0, "longest path: {c/a, c/b} -> c/c"),
        ]
    );
}

#[test]
fn depth_breaks_ties_by_lexicographic_node_order() {
    // c/a reaches two equally long branches; c/m sorts before c/z.
    let graph = r#"{"nodes": ["c/a", "c/z", "c/m", "c/z2", "c/m2"],
        "edges": [["c/a", "c/z"], ["c/z", "c/z2"], ["c/a", "c/m"], ["c/m", "c/m2"]]}"#;
    for _ in 0..8 {
        let report = run(&depth_rule(1, ""), graph, None);
        let v = &only(&report).violations[0];
        assert_eq!(
            v.message.as_deref(),
            Some("longest path: c/a -> c/m -> c/m2")
        );
    }
}

#[test]
fn depth_ignores_self_edges_and_collapses_parallel_edges() {
    let report = run(
        &depth_rule(1, ""),
        r#"{"nodes": ["c/a", "c/b"],
            "edges": [["c/a", "c/a"], ["c/a", "c/b"], ["c/a", "c/b"], ["c/b", "c/b"]]}"#,
        None,
    );
    assert_eq!(only(&report).status, RuleStatus::Pass);
}

#[test]
fn depth_exception_value_is_a_ratchet_ceiling() {
    let fits = run(
        &depth_rule(2, ""),
        CHAIN,
        Some("[depth.\"c/a\"]\nvalue = 3\nissue = \"#1\"\n"),
    );
    let r = only(&fits);
    assert_eq!(r.status, RuleStatus::Pass);
    assert!(r.violations[0].excepted);
    assert_eq!(r.exceptions_used, 1);

    let over = run(
        &depth_rule(2, ""),
        CHAIN,
        Some("[depth.\"c/a\"]\nvalue = 2\nissue = \"#1\"\n"),
    );
    assert_eq!(only(&over).status, RuleStatus::Fail);
}

#[test]
fn depth_reports_stale_exceptions() {
    let report = run(
        &depth_rule(3, ""),
        CHAIN,
        Some(
            "[depth.\"c/a\"]\nvalue = 3\nissue = \"#1\"\n\n\
             [depth.\"c/gone\"]\nvalue = 9\nissue = \"#2\"\n",
        ),
    );
    let mut stale: Vec<(&str, StaleReason)> = report
        .stale_exceptions
        .iter()
        .map(|s| (s.entity.as_str(), s.reason))
        .collect();
    stale.sort_by(|a, b| a.0.cmp(b.0));
    assert_eq!(
        stale,
        vec![
            ("c/a", StaleReason::NowPassing),
            ("c/gone", StaleReason::EntityNotFound)
        ]
    );
}

#[test]
fn graph_shape_rule_skips_without_a_dependency_graph() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("fitness.toml"),
        format!("{HEADER}{}", depth_rule(1, "")),
    )
    .unwrap();
    fs::create_dir_all(dir.path().join(".topology")).unwrap();
    let report = FitnessValidator::load(dir.path(), None)
        .unwrap()
        .validate()
        .unwrap();
    assert_eq!(only(&report).status, RuleStatus::Skip);
}

// ─── max_fan_out ────────────────────────────────────────────────────────────

const STAR: &str = r#"{
    "nodes": ["c/root", "c/hub", "c/l1", "c/l2", "c/l3"],
    "edges": [
        ["c/root", "c/hub"], ["c/root", "c/l1"], ["c/root", "c/l2"], ["c/root", "c/l3"],
        ["c/hub", "c/l1"], ["c/hub", "c/l2"], ["c/hub", "c/l3"],
        ["c/hub", "c/l1"], ["c/hub", "c/hub"]
    ]
}"#;

#[test]
fn fan_out_counts_distinct_targets_without_self_edges() {
    let report = run(
        &fan_rule(2, r#"composition_roots = ["c/root"]"#),
        STAR,
        None,
    );
    let r = only(&report);
    assert_eq!(r.status, RuleStatus::Fail);
    assert_eq!(r.violations.len(), 1);
    let v = &r.violations[0];
    assert_eq!(
        (v.entity.as_str(), v.field.as_str(), v.actual, v.threshold),
        ("c/hub", "fan_out", 3.0, 2.0),
        "parallel c/hub -> c/l1 counts once, c/hub -> c/hub not at all"
    );
}

#[test]
fn fan_out_without_roots_judges_every_node() {
    let report = run(&fan_rule(3, ""), STAR, None);
    let r = only(&report);
    assert_eq!(r.violations.len(), 1);
    assert_eq!(r.violations[0].entity, "c/root");
    assert_eq!(r.violations[0].actual, 4.0);
    assert!(r.readings.is_empty());
}

#[test]
fn fan_out_counts_only_targets_in_to() {
    let report = run(
        &fan_rule(1, r#"to = { path = "c/l*" }"#),
        r#"{"nodes": ["c/a", "c/l1", "ext/x", "ext/y"],
            "edges": [["c/a", "c/l1"], ["c/a", "ext/x"], ["c/a", "ext/y"]]}"#,
        None,
    );
    assert_eq!(only(&report).status, RuleStatus::Pass);
}

#[test]
fn uncapped_root_passes_and_still_emits_its_reading() {
    let report = run(
        &fan_rule(3, r#"composition_roots = ["c/root"]"#),
        STAR,
        None,
    );
    let r = only(&report);
    assert_eq!(r.status, RuleStatus::Pass);
    assert_eq!(r.readings.len(), 1);
    assert_eq!(
        (
            r.readings[0].entity.as_str(),
            r.readings[0].field.as_str(),
            r.readings[0].value
        ),
        ("c/root", "fan_out", 4.0)
    );
}

#[test]
fn root_max_caps_a_root() {
    let report = run(
        &fan_rule(3, "composition_roots = [\"c/root\"]\nroot_max = 3"),
        STAR,
        None,
    );
    let r = only(&report);
    assert_eq!(r.status, RuleStatus::Fail);
    let v = &r.violations[0];
    assert_eq!(
        (v.entity.as_str(), v.field.as_str(), v.actual, v.threshold),
        ("c/root", "fan_out", 4.0, 3.0)
    );
    assert_eq!(r.readings[0].value, 4.0, "reading emitted when capped too");
}

#[test]
fn several_roots_are_allowed() {
    let report = run(
        &fan_rule(1, r#"composition_roots = ["c/app-*"]"#),
        r#"{"nodes": ["c/app-a", "c/app-b", "c/l1", "c/l2"],
            "edges": [["c/app-a", "c/l1"], ["c/app-a", "c/l2"],
                      ["c/app-b", "c/l1"], ["c/app-b", "c/l2"]]}"#,
        None,
    );
    let r = only(&report);
    assert_eq!(r.status, RuleStatus::Pass);
    assert_eq!(r.readings.len(), 2);
}

#[test]
fn depending_on_a_root_is_a_violation_on_the_dependent() {
    let report = run(
        &fan_rule(5, r#"composition_roots = ["c/root"]"#),
        r#"{"nodes": ["c/root", "c/tests", "c/l1"],
            "edges": [["c/root", "c/l1"], ["c/tests", "c/root"], ["c/tests", "c/root"]]}"#,
        None,
    );
    let r = only(&report);
    assert_eq!(r.status, RuleStatus::Fail);
    assert_eq!(r.violations.len(), 1, "parallel edges collapse");
    let v = &r.violations[0];
    assert_eq!(
        (v.entity.as_str(), v.field.as_str()),
        ("c/tests", "depends_on_composition_root")
    );
    assert_eq!(
        v.message.as_deref(),
        Some("depends on composition root c/root")
    );
}

#[test]
fn a_root_depending_on_another_root_is_a_violation() {
    let report = run(
        &fan_rule(5, r#"composition_roots = ["c/r1", "c/r2"]"#),
        r#"{"nodes": ["c/r1", "c/r2"], "edges": [["c/r1", "c/r2"], ["c/r2", "c/r2"]]}"#,
        None,
    );
    let r = only(&report);
    assert_eq!(r.violations.len(), 1, "self-edge is not a root dependency");
    assert_eq!(r.violations[0].entity, "c/r1");
    assert_eq!(r.violations[0].field, "depends_on_composition_root");
}

const ROOT_DEP: &str = r#"{"nodes": ["c/root", "c/bench", "c/l1", "c/l2", "c/l3"],
    "edges": [["c/bench", "c/root"], ["c/bench", "c/l1"], ["c/bench", "c/l2"], ["c/bench", "c/l3"]]}"#;

#[test]
fn root_dependency_is_excused_only_by_targets_naming_that_root() {
    let rule = fan_rule(5, r#"composition_roots = ["c/root"]"#);
    let named = run(
        &rule,
        ROOT_DEP,
        Some("[fan.\"c/bench\"]\ntargets = [\"c/root\"]\nissue = \"#1\"\n"),
    );
    assert_eq!(only(&named).status, RuleStatus::Pass);

    let other = run(
        &rule,
        ROOT_DEP,
        Some("[fan.\"c/bench\"]\ntargets = [\"c/elsewhere\"]\nissue = \"#1\"\n"),
    );
    assert_eq!(only(&other).status, RuleStatus::Fail);

    for budget in ["value = 99\n", ""] {
        let report = run(
            &rule,
            ROOT_DEP,
            Some(&format!("[fan.\"c/bench\"]\n{budget}issue = \"#1\"\n")),
        );
        assert_eq!(
            only(&report).status,
            RuleStatus::Fail,
            "a budget entry ({budget:?}) never excuses a root dependency"
        );
    }
}

#[test]
fn a_targets_only_entry_never_excuses_a_fan_out_ceiling() {
    let rule = fan_rule(2, r#"composition_roots = ["c/root"]"#);
    let report = run(
        &rule,
        ROOT_DEP,
        Some("[fan.\"c/bench\"]\ntargets = [\"c/root\"]\nissue = \"#1\"\n"),
    );
    let r = only(&report);
    assert_eq!(r.status, RuleStatus::Fail);
    let fan = r.violations.iter().find(|v| v.field == "fan_out").unwrap();
    assert!(!fan.excepted);
    let dep = r
        .violations
        .iter()
        .find(|v| v.field == "depends_on_composition_root")
        .unwrap();
    assert!(dep.excepted);

    let both = run(
        &rule,
        ROOT_DEP,
        Some("[fan.\"c/bench\"]\nvalue = 4\ntargets = [\"c/root\"]\nissue = \"#1\"\n"),
    );
    assert_eq!(only(&both).status, RuleStatus::Pass);
    assert_eq!(only(&both).exceptions_used, 2);
}

#[test]
fn fan_out_exception_value_is_a_ratchet_ceiling() {
    let rule = fan_rule(2, r#"composition_roots = ["c/root"]"#);
    let fits = run(
        &rule,
        STAR,
        Some("[fan.\"c/hub\"]\nvalue = 3\nissue = \"#1\"\n"),
    );
    assert_eq!(only(&fits).status, RuleStatus::Pass);
    let over = run(
        &rule,
        STAR,
        Some("[fan.\"c/hub\"]\nvalue = 2\nissue = \"#1\"\n"),
    );
    assert_eq!(only(&over).status, RuleStatus::Fail);
}

#[test]
fn root_glob_matching_no_node_fails_closed() {
    let report = run(
        &fan_rule(3, r#"composition_roots = ["c/renamed-root"]"#),
        STAR,
        Some("[fan.\"INVALID_RULE:fan\"]\nissue = \"#1\"\n"),
    );
    let r = only(&report);
    assert_eq!(r.status, RuleStatus::Fail);
    assert_eq!(r.violations.len(), 1);
    let v = &r.violations[0];
    assert_eq!(v.entity, "INVALID_RULE:fan");
    assert_eq!(v.field, "composition_roots");
    assert!(
        !v.excepted,
        "the exception file cannot excuse a config error"
    );
    assert!(v.message.as_deref().unwrap().contains("matches no node"));
}

#[test]
fn root_glob_matching_a_node_outside_from_fails_closed() {
    let report = run(
        &fan_rule(3, r#"composition_roots = ["*/root"]"#),
        r#"{"nodes": ["c/root", "x/root"], "edges": []}"#,
        None,
    );
    let r = only(&report);
    assert_eq!(r.status, RuleStatus::Fail);
    let msg = r.violations[0].message.as_deref().unwrap();
    assert!(
        msg.contains("outside `from`") && msg.contains("x/root"),
        "{msg}"
    );
}

#[test]
fn root_glob_fails_closed_even_under_warning_severity() {
    let report = run(
        &fan_rule(
            3,
            "composition_roots = [\"c/none\"]\nseverity = \"warning\"",
        ),
        STAR,
        None,
    );
    assert_eq!(only(&report).status, RuleStatus::Fail);
}

// ─── Config validation ──────────────────────────────────────────────────────

#[test]
fn graph_shape_pattern_requires_max() {
    let rules = r#"[[rules.structural]]
id = "depth"
name = "Depth"
pattern = "max_dependency_depth"
from = { path = "c/*" }
"#;
    assert!(load_err(rules).contains("requires `max`"));
}

#[test]
fn graph_shape_pattern_requires_from() {
    let rules = r#"[[rules.structural]]
id = "fan"
name = "Fan"
pattern = "max_fan_out"
max = 3
"#;
    assert!(load_err(rules).contains("requires `from`"));
}

#[test]
fn root_fields_are_rejected_on_depth_pattern() {
    let err = load_err(&depth_rule(3, r#"composition_roots = ["c/a"]"#));
    assert!(err.contains("only valid for `max_fan_out`"), "{err}");
}

#[test]
fn root_max_requires_composition_roots() {
    let err = load_err(&fan_rule(3, "root_max = 9"));
    assert!(err.contains("requires `composition_roots`"), "{err}");
}

#[test]
fn graph_shape_fields_are_rejected_on_other_patterns() {
    let rules = r#"[[rules.structural]]
id = "layer"
name = "Layer"
pattern = "forbidden_import"
from = { path = "c/a" }
to = { path = "c/b" }
max = 3
"#;
    assert!(load_err(rules).contains("only valid for"));
}

#[test]
fn invalid_root_glob_is_rejected_at_load() {
    let err = load_err(&fan_rule(3, r#"composition_roots = ["c/[unclosed"]"#));
    assert!(err.contains("invalid glob"), "{err}");
}

// ─── Report schema ──────────────────────────────────────────────────────────

#[test]
fn report_with_readings_and_messages_matches_schema() {
    let report = run(
        &format!(
            "{}\n{}",
            depth_rule(0, ""),
            fan_rule(0, r#"composition_roots = ["c/a"]"#)
        ),
        CHAIN,
        None,
    );
    let value = serde_json::to_value(&report).unwrap();
    let results = value["results"].as_array().unwrap();
    assert!(results[0]["violations"][0]["message"].is_string());
    assert!(results[1]["readings"].is_array());
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/fitness-report.schema.json")).unwrap();
    let validator = jsonschema::options().build(&schema).unwrap();
    let errors: Vec<String> = validator
        .iter_errors(&value)
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn root_config_error_does_not_stale_existing_exceptions() {
    // A mis-globbed root stops evaluation; the c/hub exception must not be
    // reported stale (it may still be violating once the glob is fixed).
    let report = run(
        &fan_rule(2, r#"composition_roots = ["c/renamed-root"]"#),
        STAR,
        Some("[fan.\"c/hub\"]\nvalue = 3\nissue = \"#1\"\n"),
    );
    assert_eq!(only(&report).status, RuleStatus::Fail);
    assert!(
        report.stale_exceptions.is_empty(),
        "{:?}",
        report.stale_exceptions
    );
}

#[test]
fn explicit_empty_composition_roots_is_rejected_where_not_valid() {
    let err = load_err(&depth_rule(3, "composition_roots = []"));
    assert!(err.contains("only valid for `max_fan_out`"), "{err}");
    let rules = r#"[[rules.structural]]
id = "layer"
name = "Layer"
pattern = "forbidden_import"
from = { path = "c/a" }
to = { path = "c/b" }
composition_roots = []
"#;
    assert!(load_err(rules).contains("only valid for"));
}
