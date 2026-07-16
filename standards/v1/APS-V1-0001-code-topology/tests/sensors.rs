//! Integration tests for the `sensors` CLI command's report projection.
//!
//! Exercises the sensors module against the committed `examples/sample-topology`
//! fixture so we catch drift between the modules.json schema and the sensors
//! layer. The sensors report must be deterministic: same fixture, same output.

use code_topology::ModulesFile;
use code_topology::cli::sensors::{SENSORS_SCHEMA_VERSION, SortBy, build_report, load_report};
use std::path::PathBuf;

fn sample_dir() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/sample-topology"
    ))
}

fn load_sample_modules() -> ModulesFile {
    let content = std::fs::read_to_string(sample_dir().join("metrics/modules.json"))
        .expect("read sample modules.json");
    serde_json::from_str(&content).expect("parse sample modules.json")
}

#[test]
fn load_report_reads_sample_fixture() {
    let report = load_report(&sample_dir(), SortBy::Health, Some(3)).expect("load sensors report");
    assert_eq!(report.schema_version, SENSORS_SCHEMA_VERSION);
    assert_eq!(report.source_schema_version, "0.1.0");
    assert_eq!(report.ranking, "health");
    assert_eq!(report.total_modules, 5);
    assert_eq!(report.modules.len(), 3);
    // Ranks are 1-based and contiguous.
    for (i, entry) in report.modules.iter().enumerate() {
        assert_eq!(entry.rank, i + 1);
    }
}

#[test]
fn health_ranking_is_monotone_worst_first() {
    // The sample fixture's `api` module has the highest avg_cyclomatic (9.0)
    // and the highest avg_cognitive (13.0), so it should sit above the median
    // in the "worst health first" ranking. We do not assert an exact rank
    // because health is a composite and small fixture changes can reorder
    // near-tied modules; instead we assert the monotone property and that
    // `api` outranks the healthiest module (`utils`).
    let modules = load_sample_modules();
    let report = build_report(&modules, SortBy::Health, None);
    for pair in report.modules.windows(2) {
        assert!(pair[0].health <= pair[1].health);
    }
    let api_rank = report
        .modules
        .iter()
        .position(|m| m.id == "api")
        .expect("api present");
    let utils_rank = report
        .modules
        .iter()
        .position(|m| m.id == "utils")
        .expect("utils present");
    assert!(
        api_rank < utils_rank,
        "api (rank {api_rank}) should outrank utils (rank {utils_rank})",
    );
}

#[test]
fn complexity_ranking_puts_api_first() {
    let modules = load_sample_modules();
    let report = build_report(&modules, SortBy::Complexity, Some(2));
    assert_eq!(report.modules.len(), 2);
    assert_eq!(report.modules[0].id, "api");
    assert!(report.modules[0].avg_cyclomatic >= report.modules[1].avg_cyclomatic);
}

#[test]
fn coupling_ranking_orders_by_total_coupling() {
    let modules = load_sample_modules();
    let report = build_report(&modules, SortBy::Coupling, None);
    for pair in report.modules.windows(2) {
        assert!(pair[0].total_coupling >= pair[1].total_coupling);
    }
}

#[test]
fn sensors_report_is_deterministic() {
    let modules = load_sample_modules();
    let a = build_report(&modules, SortBy::Risk, Some(5));
    let b = build_report(&modules, SortBy::Risk, Some(5));
    assert_eq!(a, b, "same input must yield the same sensors document");
}

#[test]
fn sensors_report_roundtrips_through_json() {
    let modules = load_sample_modules();
    let report = build_report(&modules, SortBy::Risk, None);
    let json = serde_json::to_string_pretty(&report).expect("serialize");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse back");
    assert_eq!(parsed["schema_version"], SENSORS_SCHEMA_VERSION);
    assert_eq!(parsed["ranking"], "risk");
    let modules = parsed["modules"].as_array().expect("modules array");
    assert_eq!(modules.len(), 5);
    for (i, m) in modules.iter().enumerate() {
        assert_eq!(m["rank"], (i + 1) as u64);
        assert!(m["reasons"].is_array());
    }
}
