//! Conformance test: every example under `examples/valid/` must validate
//! cleanly, and every example under `examples/invalid/` must produce exactly
//! the error code(s) its header comment documents. This keeps the shipped
//! examples honest as the validator evolves.

use std::fs;
use std::path::Path;

fn examples_dir(sub: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(sub)
}

/// Parse the `# Expect: CODE1, CODE2` header line from an invalid example.
///
/// Every invalid example MUST declare the error code(s) it is expected to
/// trigger, so the test can assert the validator emits precisely those.
fn expected_codes(content: &str) -> Vec<String> {
    let line = content
        .lines()
        .find_map(|line| line.trim_start_matches('#').trim().strip_prefix("Expect:"))
        .expect("invalid example must contain an `# Expect:` header line");

    line.split(',')
        .map(|code| code.trim().to_string())
        .filter(|code| !code.is_empty())
        .collect()
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
fn all_invalid_examples_emit_their_declared_codes() {
    let dir = examples_dir("invalid");
    let mut checked = 0;

    for entry in fs::read_dir(&dir).expect("examples/invalid must exist") {
        let entry = entry.expect("readable directory entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path).expect("readable example file");
        let expected = expected_codes(&content);
        assert!(
            !expected.is_empty(),
            "{} declares no expected error codes",
            path.display()
        );

        let diagnostics = agent_recipe::validate_document(&content);
        let actual: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();

        for code in &expected {
            assert!(
                actual.contains(&code.as_str()),
                "expected {} to emit `{code}`, got: {actual:?}",
                path.display()
            );
        }

        checked += 1;
    }

    assert!(
        checked > 0,
        "expected at least one invalid example to check"
    );
}
