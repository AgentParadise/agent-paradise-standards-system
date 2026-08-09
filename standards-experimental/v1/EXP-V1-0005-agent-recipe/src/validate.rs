//! Directory validation for EXP-V1-0005 (Agent Recipe Standard).
//!
//! [`validate_recipe_dir`] is built ON TOP of [`crate::schema::load_recipe_dir`]:
//! loading and validation share one code path (plan revision R1). A failed
//! load (missing marker, malformed manifest/agent, unresolved `default_agent`,
//! I/O error) is surfaced as a single [`apss_core::Diagnostic`] carrying the
//! loader's stable error code. A recipe that loads cleanly is then subjected to
//! the additional structural rules that the typed loader does not enforce on
//! its own - unresolved `subagents` references, empty names, and malformed
//! skill/tool references - each reported with its own per-rule error code.
//!
//! Field-shape rules (unknown fields, non-string keys, unrecognized `harness`
//! or `effort` enum values) are enforced by `#[serde(deny_unknown_fields)]`
//! and the typed enums during load, so they surface here as
//! `RECIPE_MALFORMED_MANIFEST` / `RECIPE_MALFORMED_HARNESS_YAML` on the offending
//! file rather than as separate codes.

use apss_core::{Diagnostic, Diagnostics};
use std::path::Path;

use crate::schema::{
    self, AGENTS_DIR, AgentManifest, HarnessKind, RECIPE_MARKER_FILE, Recipe, RecipeLoadError,
    load_recipe_dir, resolve_inherited,
};

/// Directory-level validation error codes, layered on top of the loader codes
/// in [`crate::schema::error_codes`].
pub mod error_codes {
    /// A `subagents` entry names an agent with no matching `agents/*.yaml`.
    pub const RECIPE_SUBAGENT_UNRESOLVED: &str = "RECIPE_SUBAGENT_UNRESOLVED";
    /// `recipe.yaml`'s `name` is present (serde) but empty.
    pub const RECIPE_EMPTY_RECIPE_NAME: &str = "RECIPE_EMPTY_RECIPE_NAME";
    /// An agent manifest's `name` is present (serde) but empty.
    pub const RECIPE_EMPTY_AGENT_NAME: &str = "RECIPE_EMPTY_AGENT_NAME";
    /// An agent manifest's `model.name` is present (serde) but empty.
    pub const RECIPE_EMPTY_MODEL_NAME: &str = "RECIPE_EMPTY_MODEL_NAME";
    /// A `skills` entry is an empty string.
    pub const RECIPE_INVALID_SKILL_REF: &str = "RECIPE_INVALID_SKILL_REF";
    /// A `tools` entry is an empty string.
    pub const RECIPE_INVALID_TOOL_REF: &str = "RECIPE_INVALID_TOOL_REF";
    /// `system_instructions.content` is present (serde) but empty.
    pub const RECIPE_EMPTY_INSTRUCTIONS_CONTENT: &str = "RECIPE_EMPTY_INSTRUCTIONS_CONTENT";
    /// An agent with no declared `harness` (harness-agnostic) lists a `tools`
    /// entry that is harness-builtin under some harness and does not resolve
    /// as a recipe-provided tool.
    pub const RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL: &str =
        "RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL";
}

/// Harness-builtin tool identifiers for Claude Code, transcribed verbatim
/// from `docs/05-harness-tool-vocabulary.md` (Claude Code table). That
/// document is the normative source; do not add or remove a name here
/// without updating it first.
const CLAUDE_BUILTINS: &[&str] = &[
    "Bash",
    "BashOutput",
    "KillShell",
    "Read",
    "Edit",
    "MultiEdit",
    "Write",
    "Glob",
    "Grep",
    "NotebookEdit",
    "WebFetch",
    "WebSearch",
    "Task",
    "TodoWrite",
    "ExitPlanMode",
    "AskUserQuestion",
];

/// Harness-builtin tool identifiers for Codex CLI, transcribed verbatim from
/// `docs/05-harness-tool-vocabulary.md` (Codex CLI table). That document is
/// the normative source; do not add or remove a name here without updating
/// it first. Codex has no single shell tool name: `shell`, `shell_command`,
/// and the unified-exec pair `exec_command` / `write_stdin` are all
/// shell-family builtins and all four MUST be present.
const CODEX_BUILTINS: &[&str] = &[
    "shell",
    "shell_command",
    "exec_command",
    "write_stdin",
    "apply_patch",
    "update_plan",
    "view_image",
    "web_search",
];

/// A tool reference is harness-builtin when the named harness provides it
/// natively. See `docs/05-harness-tool-vocabulary.md`, which is the source of
/// truth for these identifiers; do not add a name that is not in that table.
pub fn is_harness_builtin(harness: HarnessKind, tool: &str) -> bool {
    match harness {
        HarnessKind::Claude => CLAUDE_BUILTINS.contains(&tool),
        HarnessKind::Codex => CODEX_BUILTINS.contains(&tool),
    }
}

/// Whether `tool` is harness-builtin under any harness this standard knows
/// about. An agent that omits `harness` has no single vocabulary to check
/// against, so a name that is builtin under any harness signals a harness
/// dependency the agent is not declaring (section 4.3/4.6).
fn is_builtin_under_any_harness(tool: &str) -> bool {
    is_harness_builtin(HarnessKind::Claude, tool) || is_harness_builtin(HarnessKind::Codex, tool)
}

/// Whether `tool` resolves as a recipe-provided tool.
///
/// `tools/` directory resolution does not exist yet (a later task adds it,
/// analogous to `skills/` resolution in section 5). Until then every entry
/// is treated as unresolvable as recipe-provided, so this always returns
/// `false`. When `tools/` resolution lands, this becomes the single place
/// to check `tools/<ref>/` under the recipe root, mirroring how `skills`
/// resolution works today.
fn resolves_as_recipe_provided(_root: &Path, _tool: &str) -> bool {
    false
}

/// Validate a recipe directory, collecting all violations into
/// [`apss_core::Diagnostics`].
///
/// This is the diagnostics-producing counterpart to
/// [`crate::schema::load_recipe_dir`]: it calls the loader and, on success,
/// runs the extra structural checks in [`validate_loaded_recipe`]. An empty
/// [`Diagnostics`] means the directory is a fully conformant recipe.
pub fn validate_recipe_dir(path: &Path) -> Diagnostics {
    let mut diagnostics = Diagnostics::new();
    match load_recipe_dir(path) {
        Ok(recipe) => validate_loaded_recipe(path, &recipe, &mut diagnostics),
        Err(error) => diagnostics.push(diagnostic_from_load_error(&error)),
    }
    diagnostics
}

/// Map a [`RecipeLoadError`] to a diagnostic carrying its stable code, message,
/// and anchored path. This is the single point where a failed load becomes a
/// diagnostic, keeping the loader and validator error codes in lockstep.
fn diagnostic_from_load_error(error: &RecipeLoadError) -> Diagnostic {
    let diagnostic =
        Diagnostic::error(error.code(), error.to_string()).with_path(error.path().to_path_buf());
    match error {
        RecipeLoadError::MissingMarker { .. } => {
            diagnostic.with_hint(format!("add a {RECIPE_MARKER_FILE} manifest at the recipe root"))
        }
        RecipeLoadError::DefaultAgentUnresolved { default_agent, .. } => diagnostic
            .with_hint(format!("add {AGENTS_DIR}/{default_agent}.yaml (or .yml) or point default_agent at an existing agent file")),
        RecipeLoadError::FromCycle { name, .. } => diagnostic.with_hint(format!(
            "break the 'from' cycle: '{name}' is reached twice while resolving inheritance"
        )),
        RecipeLoadError::FromUnresolved { from, .. } => diagnostic.with_hint(format!(
            "add {AGENTS_DIR}/{from}.yaml (or .yml) or point 'from' at an existing agent"
        )),
        RecipeLoadError::FromWidensTools { offending, .. } => diagnostic.with_hint(format!(
            "remove {offending:?} from tools, or add them to the parent's tools"
        )),
        _ => diagnostic,
    }
}

/// Run the structural rules that the typed loader does not enforce on its own.
fn validate_loaded_recipe(root: &Path, recipe: &Recipe, diagnostics: &mut Diagnostics) {
    let manifest_path = root.join(RECIPE_MARKER_FILE);
    if recipe.manifest.name.trim().is_empty() {
        diagnostics.push(
            Diagnostic::error(
                error_codes::RECIPE_EMPTY_RECIPE_NAME,
                "recipe name must be non-empty",
            )
            .with_path(manifest_path),
        );
    }

    for (stem, agent) in &recipe.agents {
        validate_agent(root, stem, agent, recipe, diagnostics);
    }

    // `from:` resolution is checked separately, over the agents that
    // actually declare it: a cycle can only occur through a chain of `from`
    // links, so an agent with no `from` cannot be the site of one, and
    // resolving it would be a no-op. `resolve_inherited` walks the whole
    // chain, so this also exercises chains longer than two.
    for (stem, agent) in &recipe.agents {
        if agent.from.is_some() {
            if let Err(error) = resolve_inherited(recipe, stem) {
                diagnostics.push(diagnostic_from_load_error(&error));
            }
        }
    }
}

/// Validate a single agent manifest against the rules that require the loaded
/// [`Recipe`] as context (subagent resolution) or that serde cannot express
/// (non-empty strings).
fn validate_agent(
    root: &Path,
    stem: &str,
    agent: &AgentManifest,
    recipe: &Recipe,
    diagnostics: &mut Diagnostics,
) {
    // Anchor diagnostics to the file the agent was actually loaded from, so a
    // `.yml` manifest points at `agents/<stem>.yml` rather than a non-existent
    // `agents/<stem>.yaml`. Fall back to the reconstructed `.yaml` path only if
    // the source was not retained (should not happen for a loaded recipe).
    let agent_path = recipe
        .agent_sources
        .get(stem)
        .cloned()
        .unwrap_or_else(|| root.join(AGENTS_DIR).join(format!("{stem}.yaml")));

    if agent.name.trim().is_empty() {
        diagnostics.push(
            Diagnostic::error(
                error_codes::RECIPE_EMPTY_AGENT_NAME,
                format!("agent '{stem}' has an empty name"),
            )
            .with_path(agent_path.clone()),
        );
    }

    // `model` and `model.name` are both optional: absent means "no opinion",
    // which is not an error (section 4.4), and a `from:`-child may leave
    // `name` unset to inherit it. A `model.name` that IS present but empty
    // (or all-whitespace) is the only case this rejects.
    if let Some(model) = &agent.model {
        if let Some(name) = &model.name {
            if name.trim().is_empty() {
                diagnostics.push(
                    Diagnostic::error(
                        error_codes::RECIPE_EMPTY_MODEL_NAME,
                        format!("agent '{stem}' has an empty model.name"),
                    )
                    .with_path(agent_path.clone()),
                );
            }
        }
    }

    for (index, skill) in agent.skills.iter().enumerate() {
        if skill.trim().is_empty() {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::RECIPE_INVALID_SKILL_REF,
                    format!("agent '{stem}' skills[{index}] must be a non-empty reference"),
                )
                .with_path(agent_path.clone()),
            );
        }
    }

    let tools: &[String] = agent.tools.as_deref().unwrap_or(&[]);

    for (index, tool) in tools.iter().enumerate() {
        if tool.trim().is_empty() {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::RECIPE_INVALID_TOOL_REF,
                    format!("agent '{stem}' tools[{index}] must be a non-empty reference"),
                )
                .with_path(agent_path.clone()),
            );
        }
    }

    // An agent that omits `harness` is claiming harness-agnosticism: it must
    // run under any conforming harness, so it must not reference a name that
    // is builtin under any harness unless that name also resolves as a
    // recipe-provided tool (see `resolves_as_recipe_provided`).
    if agent.harness.is_none() {
        for (index, tool) in tools.iter().enumerate() {
            if is_builtin_under_any_harness(tool) && !resolves_as_recipe_provided(root, tool) {
                diagnostics.push(
                    Diagnostic::error(
                        error_codes::RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL,
                        format!(
                            "agent '{stem}' declares no harness but tools[{index}] ('{tool}') is a harness-builtin tool name"
                        ),
                    )
                    .with_path(agent_path.clone())
                    .with_hint(format!(
                        "declare a harness for '{stem}', or remove '{tool}' from tools, or provide it under tools/ once bundled tool resolution exists"
                    )),
                );
            }
        }
    }

    if let Some(instructions) = &agent.system_instructions {
        if instructions.content.trim().is_empty() {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::RECIPE_EMPTY_INSTRUCTIONS_CONTENT,
                    format!("agent '{stem}' has empty system_instructions.content"),
                )
                .with_path(agent_path.clone()),
            );
        }
    }

    for subagent in &agent.subagents {
        if !recipe.agents.contains_key(subagent) {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::RECIPE_SUBAGENT_UNRESOLVED,
                    format!(
                        "agent '{stem}' references subagent '{subagent}' with no matching {AGENTS_DIR}/{subagent}.yaml (or .yml)"
                    ),
                )
                .with_path(agent_path.clone())
                .with_hint(format!(
                    "add {AGENTS_DIR}/{subagent}.yaml (or .yml) or remove '{subagent}' from subagents"
                )),
            );
        }
    }
}

// Re-export the loader error codes under this module for callers that want a
// single import site for every recipe validation code.
pub use schema::error_codes as load_error_codes;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn codes(diagnostics: &Diagnostics) -> Vec<String> {
        diagnostics.iter().map(|d| d.code.clone()).collect()
    }

    #[test]
    fn valid_recipe_has_no_diagnostics() {
        let diagnostics = validate_recipe_dir(&fixtures_dir().join("valid-recipe"));
        assert!(
            !diagnostics.has_errors(),
            "expected no errors, got: {:?}",
            codes(&diagnostics)
        );
    }

    #[test]
    fn minimal_recipe_has_no_diagnostics() {
        let diagnostics = validate_recipe_dir(&fixtures_dir().join("minimal-recipe"));
        assert!(!diagnostics.has_errors(), "got: {:?}", codes(&diagnostics));
    }

    #[test]
    fn missing_marker_reports_code() {
        let diagnostics = validate_recipe_dir(&fixtures_dir().join("missing-marker"));
        assert!(
            codes(&diagnostics).contains(&schema::error_codes::RECIPE_MISSING_MARKER.to_string())
        );
    }

    #[test]
    fn unresolved_default_agent_reports_code() {
        let diagnostics = validate_recipe_dir(&fixtures_dir().join("unresolved-default-agent"));
        assert!(
            codes(&diagnostics)
                .contains(&schema::error_codes::RECIPE_DEFAULT_AGENT_UNRESOLVED.to_string())
        );
    }

    #[test]
    fn malformed_agent_reports_code() {
        let diagnostics = validate_recipe_dir(&fixtures_dir().join("malformed-agent"));
        assert!(
            codes(&diagnostics)
                .contains(&schema::error_codes::RECIPE_MALFORMED_HARNESS_YAML.to_string())
        );
    }

    #[test]
    fn diagnostic_anchors_to_real_yml_source() {
        // A validator-only diagnostic for an agent loaded from `main.yml` must
        // point at `main.yml`, not a non-existent `main.yaml`.
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();
        std::fs::write(
            root.join("recipe.yaml"),
            "name: r\nversion: 0.1.0\ndefault_agent: main\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("agents")).unwrap();
        std::fs::write(
            root.join("agents").join("main.yml"),
            // Empty model.name triggers a validator-only diagnostic.
            "name: main\nharness: claude\nmodel:\n  name: ''\n  effort: low\n",
        )
        .unwrap();

        let diagnostics = validate_recipe_dir(root);
        assert!(codes(&diagnostics).contains(&error_codes::RECIPE_EMPTY_MODEL_NAME.to_string()));
        let anchored = diagnostics
            .iter()
            .find(|d| d.code == error_codes::RECIPE_EMPTY_MODEL_NAME)
            .and_then(|d| d.location.path.as_ref())
            .expect("diagnostic should carry a path");
        assert!(
            anchored.ends_with("agents/main.yml"),
            "diagnostic should anchor to the real .yml source, got {anchored:?}"
        );
    }

    #[test]
    fn unresolved_subagent_reports_code() {
        let diagnostics = validate_recipe_dir(&fixtures_dir().join("unresolved-subagent"));
        assert!(codes(&diagnostics).contains(&error_codes::RECIPE_SUBAGENT_UNRESOLVED.to_string()));
        // The recipe otherwise loads cleanly, so this is the only error.
        assert_eq!(diagnostics.error_count(), 1);
    }

    #[test]
    fn unresolved_subagent_hint_mentions_both_extensions() {
        // The fix hint must not hardcode `.yaml`: a `.yml` agent file also
        // resolves a subagent reference, so the hint should mention both.
        let diagnostics = validate_recipe_dir(&fixtures_dir().join("unresolved-subagent"));
        let hint = diagnostics
            .iter()
            .find(|d| d.code == error_codes::RECIPE_SUBAGENT_UNRESOLVED)
            .and_then(|d| d.fix_hint.as_deref())
            .expect("subagent-unresolved diagnostic should carry a fix hint");
        assert!(
            hint.contains(".yml"),
            "hint should mention the .yml extension, got: {hint:?}"
        );
    }
}
