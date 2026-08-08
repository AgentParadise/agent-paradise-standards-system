//! Conformance test: every recipe directory under `examples/valid/` must
//! validate cleanly, and every recipe directory under `examples/invalid/` must
//! emit the error code(s) its `README.md` header documents. This keeps the
//! shipped example directories honest as the directory validator evolves, and
//! they are the canonical fixtures downstream consumers (Plan B / `itmux`)
//! vendor (plan revision R9).

use agent_recipe::{load_recipe_dir, validate_recipe_dir};
use std::fs;
use std::path::{Path, PathBuf};

/// Write a minimal but loadable recipe directory: `recipe.yaml` with
/// `default_agent: main` plus a single `agents/<rel_path>` file with the
/// given contents.
fn write_minimal_recipe(root: &Path, rel_path: &str, agent_yaml: &str) {
    fs::write(
        root.join("recipe.yaml"),
        "name: harness-test\nversion: 0.1.0\ndefault_agent: main\n",
    )
    .expect("write recipe.yaml");
    let agent_path = root.join(rel_path);
    fs::create_dir_all(agent_path.parent().expect("agents dir")).expect("create agents dir");
    fs::write(&agent_path, agent_yaml).expect("write agent yaml");
}

fn examples_dir(sub: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(sub)
}

/// Directories directly under `examples/<sub>/`, sorted for determinism.
fn case_dirs(sub: &str) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = fs::read_dir(examples_dir(sub))
        .expect("examples subdir should exist")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// Parse the `# Expect: CODE1, CODE2` header from an invalid case's README.
fn expected_codes(case_dir: &Path) -> Vec<String> {
    let readme = fs::read_to_string(case_dir.join("README.md"))
        .unwrap_or_else(|_| panic!("invalid case {} must have a README.md", case_dir.display()));
    let line = readme
        .lines()
        .find_map(|line| line.trim().strip_prefix("# Expect:"))
        .unwrap_or_else(|| {
            panic!(
                "invalid case {} README.md must declare `# Expect: <CODE>`",
                case_dir.display()
            )
        });
    line.split(',')
        .map(|code| code.trim().to_string())
        .filter(|code| !code.is_empty())
        .collect()
}

#[test]
fn all_valid_examples_pass_validation() {
    let cases = case_dirs("valid");
    assert!(!cases.is_empty(), "expected at least one valid example");
    for case in cases {
        let diagnostics = validate_recipe_dir(&case);
        let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert!(
            !diagnostics.has_errors(),
            "valid example {} should have no errors, got: {codes:?}",
            case.display()
        );
    }
}

#[test]
fn all_invalid_examples_emit_their_declared_codes() {
    let cases = case_dirs("invalid");
    assert!(!cases.is_empty(), "expected at least one invalid example");
    for case in cases {
        let expected = expected_codes(&case);
        assert!(
            !expected.is_empty(),
            "invalid case {} declared no expected codes",
            case.display()
        );

        let diagnostics = validate_recipe_dir(&case);
        let actual: Vec<String> = diagnostics.iter().map(|d| d.code.clone()).collect();
        assert!(
            diagnostics.has_errors(),
            "invalid example {} should have produced errors, got none",
            case.display()
        );
        for code in expected {
            assert!(
                actual.contains(&code),
                "invalid example {} should emit {code}, got: {actual:?}",
                case.display()
            );
        }
    }
}

#[test]
fn harness_is_optional_and_absent_means_agnostic() {
    let dir = tempfile::tempdir().unwrap();
    write_minimal_recipe(
        dir.path(),
        "agents/main.yaml",
        "name: main\nmodel:\n  name: anthropic/claude-opus-4-8\n  effort: high\n",
    );
    let recipe = load_recipe_dir(dir.path()).expect("must load without harness");
    let agent = recipe.agents.get("main").expect("main agent should load");
    assert_eq!(agent.harness, None);
}

#[test]
fn unrecognized_harness_fails_to_parse() {
    let dir = tempfile::tempdir().unwrap();
    write_minimal_recipe(
        dir.path(),
        "agents/main.yaml",
        "name: main\nharness: nonesuch\nmodel:\n  name: anthropic/claude-opus-4-8\n  effort: high\n",
    );
    let err = load_recipe_dir(dir.path()).unwrap_err();
    assert!(format!("{err}").contains("harness"), "got: {err}");
}
