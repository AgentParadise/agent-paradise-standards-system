//! code-topology functions.json ↔ functions.schema.json drift guard.

use aps_schema_test::{collect_errors, compile};
use code_topology_rust_adapter::RustAdapter;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

const SCHEMA: &str = include_str!(
    "../../../standards/v1/APS-V1-0001-code-topology/schemas/functions.schema.json"
);

fn analyze_fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        r#"
pub mod thing;

pub fn add(a: i32, b: i32) -> i32 {
    if a > 0 { a + b } else { b }
}
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/thing.rs"),
        r#"
pub trait Thing {
    fn do_it(&self) -> u32 {
        0
    }
}

pub struct Widget { pub n: u32 }

impl Thing for Widget {
    fn do_it(&self) -> u32 { self.n }
}
"#,
    )
    .unwrap();
    let adapter = RustAdapter::new();
    let result = adapter.analyze(root).expect("analyze ok");
    result
        .write_artifacts(&root.join(".topology"))
        .expect("write ok");
    dir
}

fn read_artifact(project: &TempDir, rel: &str) -> Value {
    let raw = fs::read_to_string(project.path().join(".topology").join(rel))
        .unwrap_or_else(|e| panic!("read {rel}: {e}"));
    serde_json::from_str(&raw).expect("artifact is valid JSON")
}

#[test]
fn functions_json_matches_schema() {
    let project = analyze_fixture();
    let v = read_artifact(&project, "metrics/functions.json");
    let validator = compile(SCHEMA);
    let errors = collect_errors(&validator, &v);
    assert!(
        errors.is_empty(),
        "schema errors: {errors:#?}\nfunctions.json: {}",
        serde_json::to_string_pretty(&v).unwrap()
    );
}
