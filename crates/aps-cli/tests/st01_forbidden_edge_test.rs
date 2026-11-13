//! End to end: code-topology writes the graph APS-V1-0002 ST01 reads, so a
//! forbidden `use` edge FAILS fitness and removing it PASSES. Before the
//! graph contract fix the two standards disagreed on the file name and
//! shape, and every ST01 dependency rule skipped or passed without seeing an
//! edge.
//!
//! Scope: an edge exists for a resolved `use` path. A bare type use with no
//! `use` line makes no edge (code-topology scanner, issue #148).

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn apss_dev() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_apss-dev"))
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(apss_dev())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run apss-dev")
}

const FITNESS: &str = r#"
[config]
topology_dir = ".topology"

[[rules.structural]]
id = "domain-not-infra"
name = "Domain must not import infra"
pattern = "forbidden_import"
from = { path = "**/domain/**" }
to = { path = "**/infra/**" }
severity = "error"
"#;

fn write_crate(root: &Path, domain_model: &str) {
    for dir in ["src/domain", "src/infra"] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }
    let files = [
        (
            "Cargo.toml",
            "[package]\nname = \"fx\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        ),
        ("src/lib.rs", "pub mod domain;\npub mod infra;\n"),
        ("src/domain/mod.rs", "pub mod model;\n"),
        ("src/domain/model.rs", domain_model),
        ("src/infra/mod.rs", "pub mod db;\n"),
        (
            "src/infra/db.rs",
            "pub struct Db;\nimpl Db {\n    pub fn get(&self) -> u32 {\n        1\n    }\n}\n",
        ),
        ("fitness.toml", FITNESS),
    ];
    for (path, body) in files {
        fs::write(root.join(path), body).unwrap();
    }
}

fn analyze_then_validate(root: &Path) -> (Output, String) {
    let analyze = run(root, &["run", "topology", "analyze", "."]);
    assert!(
        analyze.status.success(),
        "analyze failed: {}",
        String::from_utf8_lossy(&analyze.stderr)
    );
    assert!(
        root.join(".topology/graphs/dependency-graph.json").exists(),
        "code-topology must write the graph ST01 reads"
    );
    let validate = run(root, &["run", "architecture-fitness", "validate", "."]);
    let stdout = String::from_utf8_lossy(&validate.stdout).into_owned();
    (validate, stdout)
}

#[test]
fn forbidden_use_edge_fails_and_removing_it_passes() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();

    write_crate(
        root,
        "use crate::infra::db::Db;\n\npub fn order(d: &Db) -> u32 {\n    d.get() + 1\n}\n",
    );
    let (out, stdout) = analyze_then_validate(root);
    assert!(!out.status.success(), "forbidden edge must fail:\n{stdout}");
    assert!(
        stdout.contains("[FAIL] Domain must not import infra"),
        "{stdout}"
    );
    assert!(stdout.contains("src/domain/model.rs"), "{stdout}");

    // The mutation: drop the forbidden import, keep the module.
    fs::write(
        root.join("src/domain/model.rs"),
        "pub fn order(x: u32) -> u32 {\n    x + 1\n}\n",
    )
    .unwrap();
    let (out, stdout) = analyze_then_validate(root);
    assert!(out.status.success(), "clean crate must pass:\n{stdout}");
    assert!(
        stdout.contains("[PASS] Domain must not import infra"),
        "{stdout}"
    );
}

#[test]
fn missing_graph_fails_instead_of_skipping() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path();
    write_crate(root, "pub fn order(x: u32) -> u32 {\n    x + 1\n}\n");
    fs::create_dir_all(root.join(".topology")).unwrap();

    let out = run(root, &["run", "architecture-fitness", "validate", "."]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!out.status.success(), "missing graph must fail:\n{stdout}");
    assert!(
        stdout.contains("PROMOTION_REQUIREMENT_UNMET:graphs/dependency-graph.json"),
        "{stdout}"
    );
}
