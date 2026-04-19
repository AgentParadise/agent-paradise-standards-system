//! code-topology coupling.json ↔ coupling.schema.json drift guard.

use aps_schema_test::{collect_errors, compile};
use code_topology_rust_adapter::RustAdapter;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

const SCHEMA: &str =
    include_str!("../../../standards/v1/APS-V1-0001-code-topology/schemas/coupling.schema.json");

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
        r#"pub mod thing;
pub fn add(a: i32, b: i32) -> i32 { a + b }
"#,
    )
    .unwrap();
    fs::write(
        root.join("src/thing.rs"),
        r#"pub trait Thing { fn go(&self) -> u32 { 0 } }
pub struct W { pub n: u32 }
impl Thing for W { fn go(&self) -> u32 { self.n } }
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

#[test]
fn coupling_json_matches_schema() {
    let project = analyze_fixture();
    let raw = fs::read_to_string(project.path().join(".topology/metrics/coupling.json")).unwrap();
    let v: Value = serde_json::from_str(&raw).unwrap();
    let validator = compile(SCHEMA);
    let errors = collect_errors(&validator, &v);
    assert!(errors.is_empty(), "schema errors: {errors:#?}");
}
