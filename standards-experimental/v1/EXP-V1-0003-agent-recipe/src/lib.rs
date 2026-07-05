//! Agent Recipe Standard (Experimental)
//!
//! This crate is the reference implementation of **EXP-V1-0003**, a declarative,
//! harness-neutral schema for an "agent recipe": a description of *what agent to
//! run* (harness, model, reasoning effort, skills, system instructions),
//! independent of *where* or *how* it is executed.
//!
//! See `docs/01_spec.md` for the normative specification. This module provides:
//!
//! - Typed Rust structures for the recipe schema ([`Recipe`], [`ModelSpec`],
//!   [`SystemInstructions`], [`AgentKind`], [`EffortLevel`], [`InstructionMode`]).
//! - A validator ([`validate_document`]) that checks a raw YAML document against
//!   every rule in spec section 5 and reports *all* violations at once via
//!   [`apss_core::Diagnostics`], rather than failing fast on the first one.
//! - A convenience loader ([`Recipe::from_yaml_str`]) for the common case of
//!   "parse a known-good recipe".
//!
//! ⚠️ EXPERIMENTAL: This standard is in incubation and may change significantly.

use apss_core::{Diagnostic, Diagnostics};
use serde::{Deserialize, Serialize};

/// Register this package with a composed APSS runner.
///
/// No composed CLI commands are exposed yet: this experiment's surface area
/// is the [`validate_document`] library function, consumed directly by Rust
/// callers (see `agents/skills/README.md`). A CLI subcommand (e.g. `aps
/// agent-recipe validate <file>`) is a natural follow-up once this schema has
/// a real consumer.
pub fn register(registry: &mut dyn apss_core::registry::StandardRegistry) {
    registry.register(
        apss_core::registry::RegisteredStandard {
            id: "EXP-V1-0003".to_string(),
            slug: "agent-recipe".to_string(),
            name: "Agent Recipe Standard".to_string(),
            description: "Harness-neutral agent recipe schema experiment".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            commands: Vec::new(),
        },
        Box::new(NoopCommandHandler),
    );
}

struct NoopCommandHandler;

impl apss_core::registry::CommandHandler for NoopCommandHandler {
    fn execute(&self, _command: &str, _args: &[String], _config: &toml::Value) -> i32 {
        eprintln!("No composed CLI commands are registered for agent-recipe yet.");
        5
    }

    fn commands(&self) -> Vec<apss_core::registry::CommandInfo> {
        Vec::new()
    }
}

/// Machine-readable error codes emitted by [`validate_document`].
///
/// These correspond one-to-one with the rules in `docs/01_spec.md` section 5.
pub mod error_codes {
    /// `name` is missing or empty.
    pub const MISSING_NAME: &str = "AGENT_RECIPE_MISSING_NAME";
    /// `agent` is missing.
    pub const MISSING_AGENT: &str = "AGENT_RECIPE_MISSING_AGENT";
    /// `agent` is present but not a recognized harness value.
    pub const UNKNOWN_AGENT: &str = "AGENT_RECIPE_UNKNOWN_AGENT";
    /// `model` is missing.
    pub const MISSING_MODEL: &str = "AGENT_RECIPE_MISSING_MODEL";
    /// `model.name` is missing or empty.
    pub const MISSING_MODEL_NAME: &str = "AGENT_RECIPE_MISSING_MODEL_NAME";
    /// `model.effort` is missing or not one of `low`/`medium`/`high`.
    pub const INVALID_MODEL_EFFORT: &str = "AGENT_RECIPE_INVALID_MODEL_EFFORT";
    /// A `skills` entry is not a non-empty string.
    pub const INVALID_SKILL_REF: &str = "AGENT_RECIPE_INVALID_SKILL_REF";
    /// `system_instructions.mode` is not one of `append`/`replace`.
    pub const INVALID_INSTRUCTIONS_MODE: &str = "AGENT_RECIPE_INVALID_INSTRUCTIONS_MODE";
    /// `system_instructions.content` is missing or empty.
    pub const EMPTY_INSTRUCTIONS_CONTENT: &str = "AGENT_RECIPE_EMPTY_INSTRUCTIONS_CONTENT";
    /// A field was present that is not part of the schema.
    pub const UNKNOWN_FIELD: &str = "AGENT_RECIPE_UNKNOWN_FIELD";
    /// The document did not parse as YAML at all, or was not a mapping.
    pub const MALFORMED_DOCUMENT: &str = "AGENT_RECIPE_MALFORMED_DOCUMENT";
}

/// Which harness executes the recipe.
///
/// This is intentionally a closed enum in v1. Per spec section 3.3, future
/// MINOR versions of this standard MAY add variants (e.g. `opencode`,
/// `gemini`) without breaking existing recipes; consumers on an older version
/// of this crate will report `AGENT_RECIPE_UNKNOWN_AGENT` for values they do
/// not yet recognize rather than silently accepting them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentKind {
    /// Claude Code.
    Claude,
    /// OpenAI Codex CLI.
    Codex,
}

impl AgentKind {
    /// Parse a raw string into a known [`AgentKind`], if recognized.
    pub fn from_str_opt(value: &str) -> Option<Self> {
        match value {
            "claude" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            _ => None,
        }
    }
}

/// Coarse reasoning/thinking effort level.
///
/// Maps to harness-specific concepts such as `thinking_level` (Claude) or
/// reasoning effort (Codex). See spec section 3.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

impl EffortLevel {
    /// Parse a raw string into a known [`EffortLevel`], if recognized.
    pub fn from_str_opt(value: &str) -> Option<Self> {
        match value {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }
}

/// How `system_instructions.content` combines with the harness's default
/// system prompt. See spec section 3.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstructionMode {
    /// Add `content` after the harness's default system prompt.
    Append,
    /// Use `content` in place of the harness's default system prompt.
    Replace,
}

impl InstructionMode {
    /// Parse a raw string into a known [`InstructionMode`], if recognized.
    pub fn from_str_opt(value: &str) -> Option<Self> {
        match value {
            "append" => Some(Self::Append),
            "replace" => Some(Self::Replace),
            _ => None,
        }
    }
}

/// Model selection: `model.name` and `model.effort` (spec section 3.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelSpec {
    /// Provider-qualified model identifier, e.g. `anthropic/claude-opus-4-8`.
    pub name: String,
    /// Coarse reasoning effort level.
    pub effort: EffortLevel,
}

/// Optional system instruction override (spec section 3.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemInstructions {
    /// Whether `content` appends to or replaces the harness default.
    pub mode: InstructionMode,
    /// The instruction text.
    pub content: String,
}

/// A validated agent recipe document (spec section 3).
///
/// This type intentionally mirrors the YAML schema exactly: it is the
/// consumer-facing contract, not an internal representation. See
/// `docs/01_spec.md` for field-by-field semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    /// Identifier for the recipe.
    pub name: String,
    /// Which harness runs this recipe.
    pub agent: AgentKind,
    /// Model selection.
    pub model: ModelSpec,
    /// Harness-agnostic skill references to inject. Defaults to empty.
    #[serde(default)]
    pub skills: Vec<String>,
    /// Optional system instruction override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_instructions: Option<SystemInstructions>,
}

/// Error produced while loading a recipe from YAML text.
#[derive(Debug, thiserror::Error)]
pub enum RecipeError {
    /// The document was not well-formed YAML matching the recipe schema.
    #[error("failed to parse recipe YAML: {0}")]
    Parse(#[from] serde_yaml::Error),
}

impl Recipe {
    /// Deserialize a recipe directly from YAML text.
    ///
    /// This is the fast, typed path for a document already known to be
    /// valid (for example, after [`validate_document`] reported no errors).
    /// It does not perform the full rule-by-rule validation in section 5 of
    /// the spec, and will simply fail on the first structural problem via
    /// [`RecipeError::Parse`]. Callers that need a complete list of
    /// violations (e.g. a linter or CI check) should use
    /// [`validate_document`] instead.
    pub fn from_yaml_str(yaml: &str) -> Result<Self, RecipeError> {
        Ok(serde_yaml::from_str(yaml)?)
    }

    /// Serialize this recipe back to a YAML string.
    pub fn to_yaml_string(&self) -> Result<String, RecipeError> {
        Ok(serde_yaml::to_string(self)?)
    }
}

/// The full set of top-level and nested field names recognized by the schema.
mod known_fields {
    pub const TOP_LEVEL: &[&str] = &["name", "agent", "model", "skills", "system_instructions"];
    pub const MODEL: &[&str] = &["name", "effort"];
    pub const SYSTEM_INSTRUCTIONS: &[&str] = &["mode", "content"];
}

/// Validate a raw recipe YAML document against every rule in spec section 5,
/// collecting *all* violations rather than stopping at the first one.
///
/// Returns an empty [`Diagnostics`] if and only if the document is fully
/// compliant with the standard.
pub fn validate_document(yaml: &str) -> Diagnostics {
    let mut diagnostics = Diagnostics::new();

    let value: serde_yaml::Value = match serde_yaml::from_str(yaml) {
        Ok(value) => value,
        Err(err) => {
            diagnostics.push(Diagnostic::error(
                error_codes::MALFORMED_DOCUMENT,
                format!("recipe document is not valid YAML: {err}"),
            ));
            return diagnostics;
        }
    };

    let Some(root) = value.as_mapping() else {
        diagnostics.push(
            Diagnostic::error(
                error_codes::MALFORMED_DOCUMENT,
                "recipe document must be a YAML mapping at the root",
            )
            .with_hint("wrap the recipe fields in a single top-level mapping"),
        );
        return diagnostics;
    };

    check_unknown_keys(root, known_fields::TOP_LEVEL, "", &mut diagnostics);

    // name
    match root.get("name").and_then(|v| v.as_str()) {
        Some(name) if !name.trim().is_empty() => {}
        _ => diagnostics.push(Diagnostic::error(
            error_codes::MISSING_NAME,
            "`name` must be present and non-empty",
        )),
    }

    // agent
    match root.get("agent") {
        None => diagnostics.push(Diagnostic::error(
            error_codes::MISSING_AGENT,
            "`agent` is required",
        )),
        Some(value) => match value.as_str() {
            Some(raw) if AgentKind::from_str_opt(raw).is_some() => {}
            Some(raw) => diagnostics.push(
                Diagnostic::error(
                    error_codes::UNKNOWN_AGENT,
                    format!("`agent: {raw}` is not a recognized harness"),
                )
                .with_hint("use one of: claude, codex"),
            ),
            None => diagnostics.push(Diagnostic::error(
                error_codes::UNKNOWN_AGENT,
                "`agent` must be a string",
            )),
        },
    }

    // model
    match root.get("model").and_then(|v| v.as_mapping()) {
        None => {
            if root.get("model").is_some() {
                diagnostics.push(Diagnostic::error(
                    error_codes::MISSING_MODEL,
                    "`model` must be a mapping",
                ));
            } else {
                diagnostics.push(Diagnostic::error(
                    error_codes::MISSING_MODEL,
                    "`model` is required",
                ));
            }
        }
        Some(model) => {
            check_unknown_keys(model, known_fields::MODEL, "model.", &mut diagnostics);

            match model.get("name").and_then(|v| v.as_str()) {
                Some(name) if !name.trim().is_empty() => {}
                _ => diagnostics.push(Diagnostic::error(
                    error_codes::MISSING_MODEL_NAME,
                    "`model.name` must be present and non-empty",
                )),
            }

            match model.get("effort").and_then(|v| v.as_str()) {
                Some(raw) if EffortLevel::from_str_opt(raw).is_some() => {}
                Some(raw) => diagnostics.push(
                    Diagnostic::error(
                        error_codes::INVALID_MODEL_EFFORT,
                        format!("`model.effort: {raw}` is not a recognized effort level"),
                    )
                    .with_hint("use one of: low, medium, high"),
                ),
                None => diagnostics.push(Diagnostic::error(
                    error_codes::INVALID_MODEL_EFFORT,
                    "`model.effort` must be one of: low, medium, high",
                )),
            }
        }
    }

    // skills
    if let Some(skills) = root.get("skills") {
        match skills.as_sequence() {
            Some(items) => {
                for (index, item) in items.iter().enumerate() {
                    let valid = item.as_str().is_some_and(|s| !s.trim().is_empty());
                    if !valid {
                        diagnostics.push(Diagnostic::error(
                            error_codes::INVALID_SKILL_REF,
                            format!("`skills[{index}]` must be a non-empty string"),
                        ));
                    }
                }
            }
            None => diagnostics.push(Diagnostic::error(
                error_codes::INVALID_SKILL_REF,
                "`skills` must be an array of strings",
            )),
        }
    }

    // system_instructions
    if let Some(instructions) = root.get("system_instructions") {
        match instructions.as_mapping() {
            Some(instructions) => {
                check_unknown_keys(
                    instructions,
                    known_fields::SYSTEM_INSTRUCTIONS,
                    "system_instructions.",
                    &mut diagnostics,
                );

                match instructions.get("mode").and_then(|v| v.as_str()) {
                    Some(raw) if InstructionMode::from_str_opt(raw).is_some() => {}
                    Some(raw) => diagnostics.push(
                        Diagnostic::error(
                            error_codes::INVALID_INSTRUCTIONS_MODE,
                            format!("`system_instructions.mode: {raw}` is not recognized"),
                        )
                        .with_hint("use one of: append, replace"),
                    ),
                    None => diagnostics.push(Diagnostic::error(
                        error_codes::INVALID_INSTRUCTIONS_MODE,
                        "`system_instructions.mode` must be one of: append, replace",
                    )),
                }

                match instructions.get("content").and_then(|v| v.as_str()) {
                    Some(content) if !content.trim().is_empty() => {}
                    _ => diagnostics.push(Diagnostic::error(
                        error_codes::EMPTY_INSTRUCTIONS_CONTENT,
                        "`system_instructions.content` must be present and non-empty",
                    )),
                }
            }
            None => diagnostics.push(Diagnostic::error(
                error_codes::MALFORMED_DOCUMENT,
                "`system_instructions` must be a mapping",
            )),
        }
    }

    diagnostics
}

/// Report `AGENT_RECIPE_UNKNOWN_FIELD` for any mapping key not in `allowed`.
fn check_unknown_keys(
    mapping: &serde_yaml::Mapping,
    allowed: &[&str],
    prefix: &str,
    diagnostics: &mut Diagnostics,
) {
    for key in mapping.keys() {
        let Some(key) = key.as_str() else { continue };
        if !allowed.contains(&key) {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::UNKNOWN_FIELD,
                    format!("`{prefix}{key}` is not a recognized field"),
                )
                .with_hint(format!("recognized fields: {}", allowed.join(", "))),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_FULL: &str = include_str!("../examples/valid/full.yaml");
    const VALID_MINIMAL: &str = include_str!("../examples/valid/minimal.yaml");

    #[test]
    fn valid_full_recipe_has_no_diagnostics() {
        let diagnostics = validate_document(VALID_FULL);
        assert!(
            !diagnostics.has_errors(),
            "expected no errors, got: {diagnostics:?}"
        );

        let recipe = Recipe::from_yaml_str(VALID_FULL).expect("should parse");
        assert_eq!(recipe.name, "pr-reviewer");
        assert_eq!(recipe.agent, AgentKind::Claude);
        assert_eq!(recipe.model.effort, EffortLevel::High);
        assert_eq!(recipe.skills, vec!["code-review", "security-review"]);
        assert!(recipe.system_instructions.is_some());
    }

    #[test]
    fn valid_minimal_recipe_has_no_diagnostics() {
        let diagnostics = validate_document(VALID_MINIMAL);
        assert!(
            !diagnostics.has_errors(),
            "expected no errors, got: {diagnostics:?}"
        );

        let recipe = Recipe::from_yaml_str(VALID_MINIMAL).expect("should parse");
        assert_eq!(recipe.agent, AgentKind::Codex);
        assert!(recipe.skills.is_empty());
        assert!(recipe.system_instructions.is_none());
    }

    #[test]
    fn round_trips_through_yaml() {
        let recipe = Recipe::from_yaml_str(VALID_FULL).expect("should parse");
        let serialized = recipe.to_yaml_string().expect("should serialize");
        let reparsed = Recipe::from_yaml_str(&serialized).expect("should reparse");
        assert_eq!(recipe, reparsed);
    }

    #[test]
    fn missing_required_fields_are_all_reported() {
        let yaml = "name: \"\"\n";
        let diagnostics = validate_document(yaml);

        let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&error_codes::MISSING_NAME));
        assert!(codes.contains(&error_codes::MISSING_AGENT));
        assert!(codes.contains(&error_codes::MISSING_MODEL));
    }

    #[test]
    fn unknown_agent_is_rejected() {
        let yaml = "name: x\nagent: gemini\nmodel:\n  name: foo\n  effort: low\n";
        let diagnostics = validate_document(yaml);
        let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&error_codes::UNKNOWN_AGENT));
    }

    #[test]
    fn invalid_effort_is_rejected() {
        let yaml = "name: x\nagent: claude\nmodel:\n  name: foo\n  effort: extreme\n";
        let diagnostics = validate_document(yaml);
        let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&error_codes::INVALID_MODEL_EFFORT));
    }

    #[test]
    fn unknown_top_level_field_is_rejected() {
        let yaml = "name: x\nagent: claude\nmodel:\n  name: foo\n  effort: low\nnotarealfield: 1\n";
        let diagnostics = validate_document(yaml);
        let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&error_codes::UNKNOWN_FIELD));
    }

    #[test]
    fn empty_skill_ref_is_rejected() {
        let yaml =
            "name: x\nagent: claude\nmodel:\n  name: foo\n  effort: low\nskills:\n  - \"\"\n";
        let diagnostics = validate_document(yaml);
        let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&error_codes::INVALID_SKILL_REF));
    }

    #[test]
    fn invalid_instructions_mode_is_rejected() {
        let yaml = "name: x\nagent: claude\nmodel:\n  name: foo\n  effort: low\nsystem_instructions:\n  mode: overwrite\n  content: hi\n";
        let diagnostics = validate_document(yaml);
        let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&error_codes::INVALID_INSTRUCTIONS_MODE));
    }

    #[test]
    fn empty_instructions_content_is_rejected() {
        let yaml = "name: x\nagent: claude\nmodel:\n  name: foo\n  effort: low\nsystem_instructions:\n  mode: append\n  content: \"\"\n";
        let diagnostics = validate_document(yaml);
        let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&error_codes::EMPTY_INSTRUCTIONS_CONTENT));
    }

    #[test]
    fn malformed_yaml_is_reported() {
        let yaml = "name: [unterminated\n";
        let diagnostics = validate_document(yaml);
        let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&error_codes::MALFORMED_DOCUMENT));
    }

    #[test]
    fn non_mapping_document_is_reported() {
        let yaml = "- 1\n- 2\n";
        let diagnostics = validate_document(yaml);
        let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert!(codes.contains(&error_codes::MALFORMED_DOCUMENT));
    }
}
