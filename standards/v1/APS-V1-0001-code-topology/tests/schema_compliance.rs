//! Round-trip validation: produced artifacts must conform to their schemas.
//!
//! These tests fail if either a writer's output shape drifts or a schema
//! tightens in a way the writer no longer satisfies. Producer and contract
//! stay in lockstep.

use code_topology_rust_adapter::RustAdapter;
use jsonschema::Validator;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

const FUNCTIONS_SCHEMA: &str = include_str!("../schemas/functions.schema.json");
const MODULES_SCHEMA: &str = include_str!("../schemas/modules.schema.json");
const COUPLING_SCHEMA: &str = include_str!("../schemas/coupling.schema.json");

/// Build a tiny Cargo project with one module that has a function and a trait
/// (so abstractness is non-zero and coupling records populate).
fn write_fixture_project() -> TempDir {
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
    // Default-body trait methods give cyclomatic >= 1 and keep Thing abstract
    // so abstractness > 0, exercising the full coupling shape.
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
    dir
}

fn analyze_fixture() -> TempDir {
    let project = write_fixture_project();
    let adapter = RustAdapter::new();
    let result = adapter.analyze(project.path()).expect("analyze ok");
    result
        .write_artifacts(&project.path().join(".topology"))
        .expect("write ok");
    project
}

fn compile(schema_str: &str) -> Validator {
    let schema: Value = serde_json::from_str(schema_str).expect("schema parses");
    jsonschema::options()
        .build(&schema)
        .expect("schema compiles")
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
    let validator = compile(FUNCTIONS_SCHEMA);
    let errors: Vec<String> = validator
        .iter_errors(&v)
        .map(|e| format!("at {}: {}", e.instance_path(), e))
        .collect();
    assert!(
        errors.is_empty(),
        "schema errors: {errors:#?}\nfunctions.json: {}",
        serde_json::to_string_pretty(&v).unwrap()
    );
}

#[test]
fn modules_json_matches_schema() {
    let project = analyze_fixture();
    let v = read_artifact(&project, "metrics/modules.json");
    let validator = compile(MODULES_SCHEMA);
    let errors: Vec<_> = validator.iter_errors(&v).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "schema errors: {errors:#?}");
}

#[test]
fn coupling_json_matches_schema() {
    let project = analyze_fixture();
    let v = read_artifact(&project, "metrics/coupling.json");
    let validator = compile(COUPLING_SCHEMA);
    let errors: Vec<_> = validator.iter_errors(&v).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "schema errors: {errors:#?}");
}
