//! Conformance test: every recipe directory under `examples/valid/` must
//! validate cleanly, and every recipe directory under `examples/invalid/` must
//! emit the error code(s) its `README.md` header documents. This keeps the
//! shipped example directories honest as the directory validator evolves, and
//! they are the canonical fixtures downstream consumers (Plan B / `itmux`)
//! vendor (plan revision R9).

use agent_recipe::{
    AgentManifest, EffortLevel, HarnessPromptMode, InstructionMode, SystemInstructions,
    ToolProtocol, load_recipe_dir, resolve_inherited, validate_recipe_dir,
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

// ─── Final review Fix 1: the agnostic-builtin rule must run against the
// RESOLVED manifest (post-`from:`), not the as-authored one. Before this
// fix, `{harness: claude, tools: [Bash]}` parent + `{from: main, tools:
// [Bash]}` child spuriously fired RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL:
// the child is not harness-agnostic, it resolves to `harness: claude`.
// Both directions are tested: the previously-broken legal case must now
// pass, and a genuinely agnostic agent (no harness anywhere in its `from:`
// chain) must still be rejected. ───────────────────────────────────────────

#[test]
fn from_child_inheriting_harness_may_narrow_a_builtin_toolset() {
    // Parent declares `harness: claude` and both builtins; child narrows to
    // a subset (`Bash` only) and declares no `harness` of its own. The
    // child is not harness-agnostic - it resolves `harness: claude` through
    // `from:` - so referencing `Bash` must not trip the agnostic-builtin
    // rule. Before Fix 1 this recipe was rejected.
    let dir = tempfile::tempdir().unwrap();
    write_minimal_recipe(
        dir.path(),
        "agents/main.yaml",
        "name: main\nharness: claude\nmodel:\n  name: anthropic/claude-opus-4-8\n  effort: high\ntools:\n  - Bash\n  - Read\n",
    );
    fs::write(
        dir.path().join("agents/child.yaml"),
        "name: child\nfrom: main\ntools:\n  - Bash\n",
    )
    .unwrap();

    let recipe = load_recipe_dir(dir.path()).expect("recipe should load");
    let resolved = resolve_inherited(&recipe, "child").expect("child should resolve from main");
    assert_eq!(
        resolved.harness,
        Some(agent_recipe::HarnessKind::Claude),
        "child must resolve harness from its from: parent"
    );

    let diagnostics = validate_recipe_dir(dir.path());
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        !codes.contains(&"RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL"),
        "a from:-child that resolves harness: claude and narrows to a subset \
         of the parent's builtins must not be rejected as agnostic, got: {codes:?}"
    );
    assert!(!diagnostics.has_errors(), "got: {codes:?}");
}

#[test]
fn from_child_still_agnostic_after_resolution_is_still_rejected_for_builtin() {
    // Same shape (`from:` child narrowing to `Bash`), but neither the
    // parent nor the child declares a `harness` anywhere in the chain. The
    // child's resolved `harness` is still `None`, so it genuinely is
    // harness-agnostic and referencing a name that is builtin under some
    // harness must still be rejected. This is the case Fix 1 must not
    // silently break while fixing the false positive above.
    let dir = tempfile::tempdir().unwrap();
    write_minimal_recipe(
        dir.path(),
        "agents/main.yaml",
        "name: main\nmodel:\n  name: anthropic/claude-opus-4-8\n  effort: high\n",
    );
    fs::write(
        dir.path().join("agents/child.yaml"),
        "name: child\nfrom: main\ntools:\n  - Bash\n",
    )
    .unwrap();

    let recipe = load_recipe_dir(dir.path()).expect("recipe should load");
    let resolved = resolve_inherited(&recipe, "child").expect("child should resolve from main");
    assert_eq!(
        resolved.harness, None,
        "neither parent nor child declares a harness, so the resolved chain stays agnostic"
    );

    let diagnostics = validate_recipe_dir(dir.path());
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL"),
        "a genuinely harness-agnostic from:-child naming a builtin tool must \
         still be rejected, got: {codes:?}"
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

// ─── evals/, judges/, and prompts/ directories (section 9) ────────────────
//
// The definition of good (evals/ + judges/) travels with the recipe;
// evaluation results do not (section 4.5). These tests cover discovery of
// the three directories and the one normative failure mode each of
// evals/ and judges/ has in this minimal-shape version of the standard.

#[test]
fn evals_and_judges_load_from_their_directories() {
    let recipe = load_recipe_dir(&examples_dir("valid").join("pr-reviewer")).unwrap();
    assert!(!recipe.evals.is_empty(), "evals/ must be discovered");
    assert!(!recipe.judges.is_empty(), "judges/ must be discovered");
}

#[test]
fn eval_case_carries_its_input_and_expected_paths() {
    let recipe = load_recipe_dir(&examples_dir("valid").join("pr-reviewer")).unwrap();
    let case = recipe
        .evals
        .iter()
        .find(|c| c.name == "flags-sql-injection")
        .expect("fixture eval case should be discovered");
    assert!(case.input_path.ends_with("input.json"));
    assert!(case.expected_path.ends_with("expected.md"));
}

#[test]
fn judges_may_use_either_inline_prompt_or_prompt_file() {
    let recipe = load_recipe_dir(&examples_dir("valid").join("pr-reviewer")).unwrap();
    let inline = recipe
        .judges
        .iter()
        .find(|j| j.name == "correctness")
        .expect("inline-prompt judge should be discovered");
    assert!(inline.prompt.is_some());
    assert!(inline.prompt_file.is_none());

    let by_file = recipe
        .judges
        .iter()
        .find(|j| j.name == "security")
        .expect("prompt_file judge should be discovered");
    assert!(by_file.prompt.is_none());
    assert_eq!(by_file.prompt_file.as_deref(), Some("security-bar.md"));
}

#[test]
fn prompts_directory_is_discovered() {
    let recipe = load_recipe_dir(&examples_dir("valid").join("pr-reviewer")).unwrap();
    assert!(
        recipe
            .prompts
            .iter()
            .any(|p| p.ends_with("security-bar.md")),
        "prompts/ must be discovered, got: {:?}",
        recipe.prompts
    );
}

#[test]
fn eval_case_missing_expected_md_is_rejected() {
    let diagnostics = validate_recipe_dir(&examples_dir("invalid").join("malformed-eval-case"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"RECIPE_MALFORMED_EVAL_CASE"),
        "got: {codes:?}"
    );
}

#[test]
fn judge_with_neither_prompt_nor_prompt_file_is_rejected() {
    let diagnostics = validate_recipe_dir(&examples_dir("invalid").join("invalid-judge-manifest"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"RECIPE_INVALID_JUDGE_MANIFEST"),
        "got: {codes:?}"
    );
}

#[test]
fn skill_ref_accepts_bare_string_or_pinned_object() {
    use agent_recipe::SkillRef;

    let bare: SkillRef = serde_yaml::from_str("research").unwrap();
    assert_eq!(bare.name(), "research");

    let pinned: SkillRef = serde_yaml::from_str(
        "ref: research\nsource_url: https://example.com/s.git\nversion: 1.2.0\nresolved_sha: abc123\n",
    )
    .unwrap();
    assert_eq!(pinned.name(), "research");
}

#[test]
fn latest_version_is_rejected() {
    // Belt-and-suspenders alongside the example-driven check above: this
    // pins the exact fixture path and code so a rename of either silently
    // dropping coverage is caught here too.
    let diagnostics = validate_recipe_dir(&examples_dir("invalid").join("skill-pinned-latest"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(codes.contains(&"RECIPE_SKILL_UNPINNED"), "got: {codes:?}");
}

#[test]
fn pinned_skill_with_specific_version_is_accepted() {
    // Accepting both forms is the point (additive, not breaking): a pinned
    // object with a real version must validate cleanly, same as a bare
    // string.
    let dir = tempfile::tempdir().unwrap();
    write_minimal_recipe(
        dir.path(),
        "agents/main.yaml",
        "name: main\nharness: claude\nmodel:\n  name: anthropic/claude-opus-4-8\n  effort: high\nskills:\n  - research\n  - ref: security\n    version: 1.2.0\n    resolved_sha: abc123\n",
    );
    let diagnostics = validate_recipe_dir(dir.path());
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        !diagnostics.has_errors(),
        "pinned skill with a real version should validate cleanly, got: {codes:?}"
    );
}

#[test]
fn existing_bare_string_skill_fixtures_still_load_unchanged() {
    // Task 9's brief: adding SkillRef must not break any recipe that uses
    // the pre-existing bare-string `skills` form. pr-reviewer's main agent
    // declares `skills: [code-review]`.
    let recipe = load_recipe_dir(&examples_dir("valid").join("pr-reviewer"))
        .expect("pr-reviewer must still load unchanged");
    let agent = recipe.agents.get("main").expect("main agent should load");
    assert_eq!(agent.skills.len(), 1);
    assert_eq!(agent.skills[0].name(), "code-review");

    let diagnostics = validate_recipe_dir(&examples_dir("valid").join("pr-reviewer"));
    assert!(
        !diagnostics.has_errors(),
        "pr-reviewer must still validate cleanly"
    );
}

// ─── Task 10: additive scalars (description, allow_delegation, max_tokens,
// temperature) ───────────────────────────────────────────────────────────

#[test]
fn additive_scalars_parse_and_default_sanely() {
    let m: AgentManifest = serde_yaml::from_str(
        "name: a\ndescription: does a thing\nmodel:\n  max_tokens: 16000\n  temperature: 0.2\n",
    )
    .unwrap();
    assert_eq!(m.description.as_deref(), Some("does a thing"));
    assert!(!m.allow_delegation, "delegation is off unless asked for");
    let model = m.model.unwrap();
    assert_eq!(model.max_tokens, Some(16000));
    assert_eq!(model.temperature, Some(0.2));
}

#[test]
fn additive_scalars_are_absent_by_default() {
    // Every one of the four new fields must be optional/defaulted so a
    // recipe authored before this task continues to parse unchanged.
    let m: AgentManifest = serde_yaml::from_str("name: a\n").unwrap();
    assert_eq!(m.description, None);
    assert!(!m.allow_delegation);
    assert_eq!(m.model, None);
}

#[test]
fn every_existing_example_recipe_still_loads_and_validates_unchanged() {
    // None of examples/valid/* or examples/invalid/* use the four new
    // fields, and adding them must not change any existing recipe's parse
    // or validation outcome. This sweeps every shipped fixture rather than
    // asserting against one, since Task 10 is purely additive.
    for dir in case_dirs("valid") {
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        load_recipe_dir(&dir)
            .unwrap_or_else(|e| panic!("valid example {name} must still load: {e}"));
        let diagnostics = validate_recipe_dir(&dir);
        assert!(
            !diagnostics.has_errors(),
            "valid example {name} must still validate cleanly, got: {:?}",
            diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>()
        );
    }
    for dir in case_dirs("invalid") {
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        let expected = expected_codes(&dir);
        let diagnostics = validate_recipe_dir(&dir);
        let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
        for code in &expected {
            assert!(
                codes.contains(&code.as_str()),
                "invalid example {name} must still emit {code}, got: {codes:?}"
            );
        }
    }
}

#[test]
fn allow_delegation_and_subagents_are_orthogonal() {
    // The brief is explicit that these must not be conflated: an agent may
    // have either, both, or neither. This exercises "both" and "neither" in
    // one recipe to make the independence observable, not just asserted in
    // prose.
    let dir = tempfile::tempdir().unwrap();
    write_minimal_recipe(
        dir.path(),
        "agents/main.yaml",
        "name: main\nallow_delegation: true\nsubagents:\n  - helper\n",
    );
    fs::write(dir.path().join("agents/helper.yaml"), "name: helper\n").unwrap();

    let recipe = load_recipe_dir(dir.path()).expect("recipe should load");
    let main = recipe.agents.get("main").unwrap();
    assert!(main.allow_delegation);
    assert_eq!(main.subagents, vec!["helper".to_string()]);

    let helper = recipe.agents.get("helper").unwrap();
    assert!(
        !helper.allow_delegation,
        "helper declares neither delegation nor subagents"
    );
    assert!(helper.subagents.is_empty());

    let diagnostics = validate_recipe_dir(dir.path());
    assert!(
        !diagnostics.has_errors(),
        "allow_delegation must not affect subagents resolution: {:?}",
        diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn additive_scalars_participate_in_from_merge() {
    // Section 4.7: a child that omits a field inherits the resolved
    // parent's value; a child that declares its own wins.
    let dir = tempfile::tempdir().unwrap();
    write_minimal_recipe(
        dir.path(),
        "agents/main.yaml",
        "name: main\ndescription: parent description\nmodel:\n  max_tokens: 8000\n  temperature: 0.5\n",
    );
    fs::write(
        dir.path().join("agents/child.yaml"),
        "name: child\nfrom: main\nmodel:\n  max_tokens: 4000\n",
    )
    .unwrap();

    let recipe = load_recipe_dir(dir.path()).expect("recipe should load");
    let resolved = resolve_inherited(&recipe, "child").expect("child should resolve from main");

    // description: omitted by the child, inherited from the parent.
    assert_eq!(resolved.description.as_deref(), Some("parent description"));

    // model.max_tokens: declared by the child, so the child's value wins.
    // model.temperature: omitted by the child, inherited from the parent.
    let model = resolved.model.expect("model should be present");
    assert_eq!(model.max_tokens, Some(4000));
    assert_eq!(model.temperature, Some(0.5));
}

// ─── Fix round 1, Finding 1: allow_delegation must narrow via `from:`, like
// `tools` and `mcp`. Both directions are tested: a widening child (parent
// false, child true) MUST be rejected, and a tightening child (parent true,
// child false) MUST remain legal. ─────────────────────────────────────────

#[test]
fn from_may_not_widen_allow_delegation() {
    // Belt-and-suspenders alongside the example-driven check in the sweep
    // test above: this pins the exact fixture path and code so a rename of
    // either silently dropping coverage is caught here too.
    let diagnostics = validate_recipe_dir(&examples_dir("invalid").join("from-widens-delegation"));
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"RECIPE_FROM_WIDENS_DELEGATION"),
        "got: {codes:?}"
    );
}

#[test]
fn from_may_tighten_allow_delegation() {
    // The other direction of the same rule: a parent that permits
    // delegation and a child that declares its own `allow_delegation:
    // false` is narrowing, not widening, and MUST be accepted. A fix that
    // rejected this case too (e.g. by comparing for inequality rather than
    // "child true, parent false") would be worse than the bug it fixed.
    let dir = tempfile::tempdir().unwrap();
    write_minimal_recipe(
        dir.path(),
        "agents/main.yaml",
        "name: main\nallow_delegation: true\n",
    );
    fs::write(
        dir.path().join("agents/child.yaml"),
        "name: child\nfrom: main\nallow_delegation: false\n",
    )
    .unwrap();

    let recipe = load_recipe_dir(dir.path()).expect("recipe should load");
    let resolved =
        resolve_inherited(&recipe, "child").expect("tightening allow_delegation must resolve");
    assert!(!resolved.allow_delegation);

    let diagnostics = validate_recipe_dir(dir.path());
    assert!(
        !diagnostics.has_errors(),
        "tightening allow_delegation from true to false must validate cleanly, got: {:?}",
        diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn from_may_inherit_or_repeat_allow_delegation_without_widening() {
    // Equal-on-both-sides cases (the child omits the field entirely, or
    // repeats the same value as the parent) must not be flagged as
    // widenings - only a genuine false-to-true transition is rejected.
    let dir = tempfile::tempdir().unwrap();
    write_minimal_recipe(
        dir.path(),
        "agents/main.yaml",
        "name: main\nallow_delegation: true\n",
    );
    fs::write(
        dir.path().join("agents/omits.yaml"),
        "name: omits\nfrom: main\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("agents/repeats.yaml"),
        "name: repeats\nfrom: main\nallow_delegation: true\n",
    )
    .unwrap();

    let recipe = load_recipe_dir(dir.path()).expect("recipe should load");
    for child_name in ["omits", "repeats"] {
        resolve_inherited(&recipe, child_name)
            .unwrap_or_else(|e| panic!("{child_name} should resolve without widening: {e}"));
    }

    let diagnostics = validate_recipe_dir(dir.path());
    assert!(
        !diagnostics.has_errors(),
        "got: {:?}",
        diagnostics.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}
