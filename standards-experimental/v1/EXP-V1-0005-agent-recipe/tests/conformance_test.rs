//! Conformance test: every recipe directory under `examples/valid/` must
//! validate cleanly, and every recipe directory under `examples/invalid/` must
//! emit the error code(s) its `README.md` header documents. This keeps the
//! shipped example directories honest as the directory validator evolves, and
//! they are the canonical fixtures downstream consumers (Plan B / `itmux`)
//! vendor (plan revision R9).

use agent_recipe::{
    EffortLevel, HarnessPromptMode, InstructionMode, SystemInstructions, ToolProtocol,
    load_recipe_dir, resolve_inherited, validate_recipe_dir,
};
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
fn agnostic_agent_referencing_builtin_tool_is_rejected() {
    // Belt-and-suspenders alongside the example-driven check above: this
    // pins the exact fixture path and code so a rename of either silently
    // dropping coverage is caught here too.
    let diagnostics =
        validate_recipe_dir(&examples_dir("invalid").join("agnostic-agent-uses-builtin"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL"),
        "got: {codes:?}"
    );
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

#[test]
fn harness_prompt_defaults_to_append_and_is_independent_of_mode() {
    let si: SystemInstructions = serde_yaml::from_str("mode: replace\ncontent: hello\n").unwrap();
    // `mode` governs SYSTEM.md composition only.
    assert_eq!(si.mode, InstructionMode::Replace);
    // The harness's own prompt is untouched unless explicitly replaced.
    assert_eq!(si.harness_prompt, HarnessPromptMode::Append);
}

// ─── from: inheritance (section 4.7) ──────────────────────────────────────
//
// The upstream test brief for this behavior asserted
// `child.model.as_ref().unwrap().name.as_deref()` / `.effort`, which this
// crate's `resolve_inherited` signature satisfies directly since `model` is
// `Option<ModelSpec>` and `ModelSpec::name` is `Option<String>` (see
// `schema::ModelSpec`); the brief's own `report.codes()` helper does not
// exist on `apss_core::Diagnostics`, so the cycle/widens-tools assertions
// below use the same `diagnostics.iter().map(|d| d.code.as_str())` pattern
// already used elsewhere in this file. Both adaptations preserve exactly
// what the brief's tests asserted.

#[test]
fn from_inherits_parent_fields_and_child_overrides_win() {
    let recipe = load_recipe_dir(&examples_dir("valid").join("pr-reviewer")).unwrap();
    let child = resolve_inherited(&recipe, "reviewer").unwrap();
    // model.name inherited from the parent (`main`), effort overridden by
    // the child (`reviewer` declares its own `model.effort: high`).
    assert_eq!(
        child.model.as_ref().and_then(|m| m.name.as_deref()),
        Some("anthropic/claude-opus-4-8")
    );
    assert_eq!(child.model.as_ref().unwrap().effort, EffortLevel::High);
    // harness is also inherited: `reviewer` declares no harness of its own.
    assert_eq!(child.harness, Some(agent_recipe::HarnessKind::Claude));
}

#[test]
fn from_cycle_is_rejected() {
    let diagnostics = validate_recipe_dir(&examples_dir("invalid").join("from-cycle"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(codes.contains(&"RECIPE_FROM_CYCLE"), "got: {codes:?}");
}

#[test]
fn from_may_not_widen_tools() {
    let diagnostics = validate_recipe_dir(&examples_dir("invalid").join("from-widens-tools"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"RECIPE_FROM_WIDENS_TOOLS"),
        "got: {codes:?}"
    );
}

#[test]
fn from_unresolved_is_rejected() {
    let diagnostics = validate_recipe_dir(&examples_dir("invalid").join("from-unresolved"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(codes.contains(&"RECIPE_FROM_UNRESOLVED"), "got: {codes:?}");
}

// ─── mcp policy (section 7) ────────────────────────────────────────────────

#[test]
fn agent_mcp_policy_may_not_widen_package_policy() {
    // Belt-and-suspenders alongside the example-driven check above: this
    // pins the exact fixture path and code so a rename of either silently
    // dropping coverage is caught here too.
    let diagnostics = validate_recipe_dir(&examples_dir("invalid").join("mcp-agent-widens-policy"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"RECIPE_MCP_AGENT_WIDENS_POLICY"),
        "got: {codes:?}"
    );
}

#[test]
fn agent_mcp_policy_may_not_widen_immediate_from_parent() {
    // A three-level `from:` chain where the package's own policy is
    // permissive enough that the child's resolved policy is within the
    // package ceiling; the only violation is the widening at the
    // parent -> child link, isolating the per-link check from the
    // package-tier check (`RECIPE_MCP_AGENT_WIDENS_POLICY`).
    let diagnostics = validate_recipe_dir(&examples_dir("invalid").join("mcp-from-widens-policy"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"RECIPE_MCP_FROM_WIDENS_POLICY"),
        "got: {codes:?}"
    );
    assert!(
        !codes.contains(&"RECIPE_MCP_AGENT_WIDENS_POLICY"),
        "this fixture's package policy is permissive enough that only the \
         from:-link code should fire, got: {codes:?}"
    );
}

#[test]
fn agent_naming_a_server_the_package_never_mentioned_widens() {
    // The subtle case: a naive check that only compares servers present in
    // both policies would miss this, since `reporting` never appears in the
    // package's policy at all.
    let diagnostics =
        validate_recipe_dir(&examples_dir("invalid").join("mcp-agent-unmentioned-server"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"RECIPE_MCP_AGENT_WIDENS_POLICY"),
        "got: {codes:?}"
    );
}

// ─── tools/ directory (section 5.2) ────────────────────────────────────────

#[test]
fn recipe_provided_tool_resolves_under_tools_dir() {
    let recipe = load_recipe_dir(&examples_dir("valid").join("pr-reviewer")).unwrap();
    let resolved = recipe
        .resolve_tool("extract_citations")
        .expect("must resolve");
    assert_eq!(resolved.protocol, ToolProtocol::McpStdio);
}

#[test]
fn unknown_tool_ref_does_not_resolve() {
    let recipe = load_recipe_dir(&examples_dir("valid").join("pr-reviewer")).unwrap();
    assert!(recipe.resolve_tool("no-such-tool").is_none());
}

#[test]
fn agnostic_agent_referencing_builtin_tool_is_still_rejected_without_tools_dir() {
    // The transition this task makes possible must not have broken the
    // rejection it was carved out for: a harness-agnostic agent naming a
    // builtin with no matching `tools/` entry is still an error.
    let diagnostics =
        validate_recipe_dir(&examples_dir("invalid").join("agnostic-agent-uses-builtin"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL"),
        "got: {codes:?}"
    );
}

#[test]
fn agnostic_agent_referencing_recipe_provided_tool_now_passes() {
    // Same shape of name collision as the still-rejected fixture above
    // (`Bash`, a Claude-Code builtin), but this recipe also ships
    // `tools/Bash/tool.yaml`. Recipe-provided wins the ambiguity, so the
    // harness-agnostic agent referencing it is not an error.
    let diagnostics =
        validate_recipe_dir(&examples_dir("valid").join("agnostic-agent-recipe-provided-tool"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        !diagnostics.has_errors(),
        "expected recipe-provided precedence to clear the builtin-name ambiguity, got: {codes:?}"
    );
    assert!(
        !codes.contains(&"RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL"),
        "got: {codes:?}"
    );
}

#[test]
fn tool_manifest_with_empty_name_and_command_is_rejected() {
    let diagnostics = validate_recipe_dir(&examples_dir("invalid").join("invalid-tool-manifest"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"RECIPE_INVALID_TOOL_MANIFEST"),
        "got: {codes:?}"
    );
}
