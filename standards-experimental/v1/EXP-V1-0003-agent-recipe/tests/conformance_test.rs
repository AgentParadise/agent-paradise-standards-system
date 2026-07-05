//! Conformance test: every example under `examples/valid/` must validate
//! cleanly, and every example under `examples/invalid/` must produce at
//! least one diagnostic. This keeps the shipped examples honest as the
//! validator evolves.

use std::fs;
use std::path::Path;

fn examples_dir(sub: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(sub)
}

#[test]
fn all_valid_examples_pass_validation() {
    let dir = examples_dir("valid");
    let mut checked = 0;

    for entry in fs::read_dir(&dir).expect("examples/valid must exist") {
        let entry = entry.expect("readable directory entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path).expect("readable example file");
        let diagnostics = agent_recipe::validate_document(&content);
        assert!(
            !diagnostics.has_errors(),
            "expected {} to be valid, got diagnostics: {diagnostics:?}",
            path.display()
        );

        // Every valid example must also deserialize into the typed Recipe.
        agent_recipe::Recipe::from_yaml_str(&content)
            .unwrap_or_else(|err| panic!("expected {} to parse as Recipe: {err}", path.display()));

        checked += 1;
    }

    assert!(checked > 0, "expected at least one valid example to check");
}

#[test]
fn all_invalid_examples_fail_validation() {
    let dir = examples_dir("invalid");
    let mut checked = 0;

    for entry in fs::read_dir(&dir).expect("examples/invalid must exist") {
        let entry = entry.expect("readable directory entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path).expect("readable example file");
        let diagnostics = agent_recipe::validate_document(&content);
        assert!(
            diagnostics.has_errors(),
            "expected {} to be invalid, but validation produced no errors",
            path.display()
        );

        checked += 1;
    }

    assert!(
        checked > 0,
        "expected at least one invalid example to check"
    );
}
