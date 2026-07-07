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
//! Field-shape rules (unknown fields, non-string keys, unrecognized `agent`
//! or `effort` enum values) are enforced by `#[serde(deny_unknown_fields)]`
//! and the typed enums during load, so they surface here as
//! `RECIPE_MALFORMED_MANIFEST` / `RECIPE_MALFORMED_AGENT_YAML` on the offending
//! file rather than as separate codes.

use apss_core::{Diagnostic, Diagnostics};
use std::path::Path;

use crate::schema::{
    self, AGENTS_DIR, AgentManifest, RECIPE_MARKER_FILE, Recipe, RecipeLoadError, load_recipe_dir,
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
            .with_hint(format!("add {AGENTS_DIR}/{default_agent}.yaml or point default_agent at an existing agent file")),
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
    // The loader discards each agent's concrete source path (a `.yaml` vs
    // `.yml` extension is not retained), so the diagnostic location is
    // reconstructed as `agents/<stem>.yaml` for a readable pointer.
    let agent_path = root.join(AGENTS_DIR).join(format!("{stem}.yaml"));

    if agent.name.trim().is_empty() {
        diagnostics.push(
            Diagnostic::error(
                error_codes::RECIPE_EMPTY_AGENT_NAME,
                format!("agent '{stem}' has an empty name"),
            )
            .with_path(agent_path.clone()),
        );
    }

    if agent.model.name.trim().is_empty() {
        diagnostics.push(
            Diagnostic::error(
                error_codes::RECIPE_EMPTY_MODEL_NAME,
                format!("agent '{stem}' has an empty model.name"),
            )
            .with_path(agent_path.clone()),
        );
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

    for (index, tool) in agent.tools.iter().enumerate() {
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
                        "agent '{stem}' references subagent '{subagent}' with no matching {AGENTS_DIR}/{subagent}.yaml"
                    ),
                )
                .with_path(agent_path.clone())
                .with_hint(format!(
                    "add {AGENTS_DIR}/{subagent}.yaml or remove '{subagent}' from subagents"
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
                .contains(&schema::error_codes::RECIPE_MALFORMED_AGENT_YAML.to_string())
        );
    }

    #[test]
    fn unresolved_subagent_reports_code() {
        let diagnostics = validate_recipe_dir(&fixtures_dir().join("unresolved-subagent"));
        assert!(codes(&diagnostics).contains(&error_codes::RECIPE_SUBAGENT_UNRESOLVED.to_string()));
        // The recipe otherwise loads cleanly, so this is the only error.
        assert_eq!(diagnostics.error_count(), 1);
    }
}
