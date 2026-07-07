//! Directory-shape schema and loader for EXP-V1-0004 (Agent Recipe Standard).
//!
//! A **recipe** is a directory, not a single YAML file:
//!
//! ```text
//! <recipe>/
//!   recipe.yaml            # RecipeManifest: name, version, default_agent
//!   agents/<name>.yaml     # AgentManifest (per-agent, unified agents+subagents)
//!   skills/                # optional: bundled skill packages
//!   SYSTEM.md               # optional: shared base instructions
//! ```
//!
//! This module owns the typed contract ([`RecipeManifest`], [`AgentManifest`],
//! [`Recipe`]) and the canonical loader ([`load_recipe_dir`]). It has **no
//! CLI-only dependencies** so that downstream consumers (notably `itmux`,
//! Plan B) can depend on this crate purely as a library, per the design in
//! `docs/design/plans/2026-07-07-planA-agent-recipe-directory-standard.md`
//! (agentic-primitives, revision R2).
//!
//! Validating a directory against every rule in the spec (unresolved
//! `subagents`, malformed skill refs, directory-level diagnostics with error
//! codes, etc.) is `validate_recipe_dir` in `src/validate.rs` (Task 2). Per
//! revision R1 of the plan, loading and validation share one code path:
//! `validate_recipe_dir` is expected to be implemented as
//! `load_recipe_dir(...).map(|_| ()).or(<diagnostics from RecipeLoadError>)`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Root marker file name. Its presence denotes "this directory is a recipe".
pub const RECIPE_MARKER_FILE: &str = "recipe.yaml";

/// Directory (relative to the recipe root) holding one YAML file per agent.
pub const AGENTS_DIR: &str = "agents";

/// Optional directory (relative to the recipe root) holding bundled skills.
pub const SKILLS_DIR: &str = "skills";

/// Optional shared base system instructions file (relative to the recipe root).
pub const SYSTEM_MD_FILE: &str = "SYSTEM.md";

/// Root manifest: `recipe.yaml`. Its presence is the recipe marker.
///
/// ```yaml
/// name: pr-reviewer
/// version: 0.1.0
/// default_agent: main
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeManifest {
    /// Identifier for the recipe.
    pub name: String,
    /// SemVer-ish recipe version.
    pub version: String,
    /// Name of the entry-point agent. Resolves to `agents/<default_agent>.yaml`.
    pub default_agent: String,
}

/// Which harness executes an agent.
///
/// Intentionally a closed enum in v1: future MINOR versions of this standard
/// MAY add variants (e.g. `opencode`, `gemini`) without breaking existing
/// recipes; consumers on an older version of this crate will fail to parse
/// (via serde) rather than silently accepting an unrecognized harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentKind {
    /// Claude Code.
    Claude,
    /// OpenAI Codex CLI.
    Codex,
}

/// Coarse reasoning/thinking effort level.
///
/// Maps to harness-specific concepts such as `thinking_level` (Claude) or
/// reasoning effort (Codex).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

/// Model selection: `model.name` and `model.effort`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelSpec {
    /// Provider-qualified model identifier, e.g. `anthropic/claude-opus-4-8`.
    pub name: String,
    /// Coarse reasoning effort level.
    pub effort: EffortLevel,
}

/// How `system_instructions.content` combines with `SYSTEM.md`.
///
/// See R3 semantics on [`resolved_system`]:
/// - `Append` => final system = `SYSTEM.md` (if present) + `"\n\n"` + `content`.
/// - `Replace` => final system = `content` only (`SYSTEM.md` is ignored).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstructionMode {
    /// Add `content` after the recipe's shared `SYSTEM.md`.
    Append,
    /// Use `content` in place of the recipe's shared `SYSTEM.md`.
    Replace,
}

/// Optional per-agent system instruction override.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemInstructions {
    /// Whether `content` appends to or replaces the recipe's `SYSTEM.md`.
    pub mode: InstructionMode,
    /// The instruction text.
    pub content: String,
}

/// One agent, loaded from `agents/<name>.yaml`. Unifies default agents and
/// subagents: whether a given `AgentManifest` is the recipe's entry point is
/// determined by `RecipeManifest::default_agent`, not by any field here.
///
/// Field names/enums intentionally match the prior single-YAML EXP-V1-0004
/// schema (`name`, `agent`, `model`, `skills`, `system_instructions`) for
/// pi-compatibility; `tools` and `subagents` are new in the directory shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentManifest {
    /// Agent name. SHOULD match the file stem (`agents/<name>.yaml`).
    pub name: String,
    /// Which harness runs this agent.
    pub agent: AgentKind,
    /// Model selection.
    pub model: ModelSpec,
    /// Harness-agnostic skill references to inject, in listed order.
    ///
    /// Per R3, each entry resolves to a plugin-dir path: (a) `skills/<ref>/`
    /// inside the recipe if that subdirectory exists, else (b) the ref
    /// as-is (an external skill path/name). Resolution order is the order
    /// these entries are declared, and callers (e.g. Plan B's
    /// `claude_plugin_dirs`) MUST preserve it.
    #[serde(default)]
    pub skills: Vec<String>,
    /// Optional system instruction override. See [`resolved_system`] for the
    /// exact merge semantics with the recipe's shared `SYSTEM.md`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_instructions: Option<SystemInstructions>,
    /// Tool references (no code, no execution) - just names/identifiers the
    /// agent is allowed to use. Resolution/enforcement is a consumer concern.
    #[serde(default)]
    pub tools: Vec<String>,
    /// Names of other agents (files in `agents/`, without the `.yaml`
    /// extension) this agent may delegate to as subagents.
    #[serde(default)]
    pub subagents: Vec<String>,
}

/// A fully loaded recipe directory: the manifest, every parsed agent (keyed
/// by file stem), the gathered `skills/` entries, and the optional shared
/// `SYSTEM.md` content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipe {
    /// Parsed `recipe.yaml`.
    pub manifest: RecipeManifest,
    /// Every parsed `agents/*.yaml`, keyed by file stem (the agent name used
    /// by `default_agent` and `subagents` references).
    pub agents: BTreeMap<String, AgentManifest>,
    /// Entries found directly under `skills/`, sorted, if that directory
    /// exists. This is the recipe's bundled skill inventory, not a resolved
    /// per-agent reference list (see [`AgentManifest::skills`] for that).
    pub skills: Vec<PathBuf>,
    /// Contents of `SYSTEM.md`, if present.
    pub system_md: Option<String>,
}

impl Recipe {
    /// The agent resolved from `manifest.default_agent`.
    ///
    /// This is always `Some` for a `Recipe` produced by [`load_recipe_dir`]
    /// (which fails the load otherwise); it is `Option` here because `Recipe`
    /// values may also be constructed directly (e.g. in tests).
    pub fn default_agent(&self) -> Option<&AgentManifest> {
        self.agents.get(&self.manifest.default_agent)
    }
}

/// Machine-readable error codes for [`RecipeLoadError`]. Consumed by
/// `validate_recipe_dir` (Task 2, `src/validate.rs`), which is expected to
/// convert a `RecipeLoadError` into an `apss_core::Diagnostic` carrying the
/// matching code.
pub mod error_codes {
    /// `recipe.yaml` is absent from the candidate directory.
    pub const RECIPE_MISSING_MARKER: &str = "RECIPE_MISSING_MARKER";
    /// `recipe.yaml` exists but failed to parse as a [`super::RecipeManifest`].
    pub const RECIPE_MALFORMED_MANIFEST: &str = "RECIPE_MALFORMED_MANIFEST";
    /// An `agents/*.yaml` file failed to parse as an [`super::AgentManifest`].
    pub const RECIPE_MALFORMED_AGENT_YAML: &str = "RECIPE_MALFORMED_AGENT_YAML";
    /// `default_agent` does not resolve to any parsed entry in `agents/`.
    pub const RECIPE_DEFAULT_AGENT_UNRESOLVED: &str = "RECIPE_DEFAULT_AGENT_UNRESOLVED";
    /// An I/O error occurred while reading the recipe directory.
    pub const RECIPE_IO_ERROR: &str = "RECIPE_IO_ERROR";
}

/// Failure modes of [`load_recipe_dir`].
///
/// Every variant carries the path it failed on so callers (and `validate_recipe_dir`
/// in Task 2) can attach a precise [`apss_core::Diagnostic`] location.
#[derive(Debug, thiserror::Error)]
pub enum RecipeLoadError {
    /// `recipe.yaml` is missing from the candidate directory.
    #[error("missing recipe marker: {path} does not exist")]
    MissingMarker {
        /// Expected path to `recipe.yaml`.
        path: PathBuf,
    },
    /// `recipe.yaml` exists but is not a valid [`RecipeManifest`].
    #[error("malformed recipe manifest at {path}: {source}")]
    MalformedManifest {
        /// Path to the offending `recipe.yaml`.
        path: PathBuf,
        /// Underlying parse error.
        #[source]
        source: serde_yaml::Error,
    },
    /// An `agents/*.yaml` file is not a valid [`AgentManifest`].
    #[error("malformed agent manifest at {path}: {source}")]
    MalformedAgent {
        /// Path to the offending agent YAML file.
        path: PathBuf,
        /// Underlying parse error.
        #[source]
        source: serde_yaml::Error,
    },
    /// `default_agent` does not name any file actually present in `agents/`.
    #[error(
        "default_agent '{default_agent}' does not resolve to {agents_dir}/{default_agent}.yaml"
    )]
    DefaultAgentUnresolved {
        /// The unresolved `default_agent` value from `recipe.yaml`.
        default_agent: String,
        /// The `agents/` directory that was searched.
        agents_dir: PathBuf,
    },
    /// Reading a file or directory failed.
    #[error("io error at {path}: {source}")]
    Io {
        /// Path being read when the error occurred.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

impl RecipeLoadError {
    /// The stable machine-readable code for this error, matching
    /// `error_codes`.
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingMarker { .. } => error_codes::RECIPE_MISSING_MARKER,
            Self::MalformedManifest { .. } => error_codes::RECIPE_MALFORMED_MANIFEST,
            Self::MalformedAgent { .. } => error_codes::RECIPE_MALFORMED_AGENT_YAML,
            Self::DefaultAgentUnresolved { .. } => error_codes::RECIPE_DEFAULT_AGENT_UNRESOLVED,
            Self::Io { .. } => error_codes::RECIPE_IO_ERROR,
        }
    }

    /// The filesystem path this error is anchored to, for attaching a precise
    /// diagnostic location. For [`RecipeLoadError::DefaultAgentUnresolved`]
    /// this is the searched `agents/` directory.
    pub fn path(&self) -> &Path {
        match self {
            Self::MissingMarker { path } => path,
            Self::MalformedManifest { path, .. } => path,
            Self::MalformedAgent { path, .. } => path,
            Self::DefaultAgentUnresolved { agents_dir, .. } => agents_dir,
            Self::Io { path, .. } => path,
        }
    }
}

/// Load and fully parse a recipe directory.
///
/// Reads `recipe.yaml` (error [`RecipeLoadError::MissingMarker`] if absent),
/// parses every `agents/*.yaml`, resolves `default_agent` to a parsed
/// [`AgentManifest`] (error [`RecipeLoadError::DefaultAgentUnresolved`] if it
/// does not name a file under `agents/`), reads the optional `SYSTEM.md`, and
/// gathers the optional `skills/` directory's entries.
///
/// This is the canonical loader: `itmux` (Plan B) depends on this function
/// directly rather than re-implementing the shape (plan revision R2).
pub fn load_recipe_dir(path: &Path) -> Result<Recipe, RecipeLoadError> {
    let manifest_path = path.join(RECIPE_MARKER_FILE);
    if !manifest_path.exists() {
        return Err(RecipeLoadError::MissingMarker {
            path: manifest_path,
        });
    }

    let manifest_content = read_to_string(&manifest_path)?;
    let manifest: RecipeManifest = serde_yaml::from_str(&manifest_content).map_err(|source| {
        RecipeLoadError::MalformedManifest {
            path: manifest_path.clone(),
            source,
        }
    })?;

    let agents_dir = path.join(AGENTS_DIR);
    let mut agents = BTreeMap::new();
    if agents_dir.is_dir() {
        for entry_path in list_yaml_files(&agents_dir)? {
            let content = read_to_string(&entry_path)?;
            let agent: AgentManifest = serde_yaml::from_str(&content).map_err(|source| {
                RecipeLoadError::MalformedAgent {
                    path: entry_path.clone(),
                    source,
                }
            })?;
            let stem = entry_path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            agents.insert(stem, agent);
        }
    }

    if !agents.contains_key(&manifest.default_agent) {
        return Err(RecipeLoadError::DefaultAgentUnresolved {
            default_agent: manifest.default_agent,
            agents_dir,
        });
    }

    let system_md_path = path.join(SYSTEM_MD_FILE);
    let system_md = if system_md_path.exists() {
        Some(read_to_string(&system_md_path)?)
    } else {
        None
    };

    let skills_dir = path.join(SKILLS_DIR);
    let skills = if skills_dir.is_dir() {
        list_dir_entries(&skills_dir)?
    } else {
        Vec::new()
    };

    Ok(Recipe {
        manifest,
        agents,
        skills,
        system_md,
    })
}

/// Compute the final resolved system prompt for an agent, per R3:
///
/// - `system_instructions.mode: append` => `SYSTEM.md` (if present) + `"\n\n"`
///   + `content` (or just `content` if there is no `SYSTEM.md`).
/// - `system_instructions.mode: replace` => `content` only; `SYSTEM.md` is
///   ignored even if present.
/// - No `system_instructions` at all => `SYSTEM.md` verbatim, or `None` if
///   there is no `SYSTEM.md` either.
pub fn resolved_system(agent: &AgentManifest, system_md: Option<&str>) -> Option<String> {
    match &agent.system_instructions {
        Some(instructions) => match instructions.mode {
            InstructionMode::Append => match system_md {
                Some(base) if !base.is_empty() => {
                    Some(format!("{base}\n\n{}", instructions.content))
                }
                _ => Some(instructions.content.clone()),
            },
            InstructionMode::Replace => Some(instructions.content.clone()),
        },
        None => system_md.map(str::to_string),
    }
}

fn read_to_string(path: &Path) -> Result<String, RecipeLoadError> {
    fs::read_to_string(path).map_err(|source| RecipeLoadError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// List `*.yaml`/`*.yml` files directly under `dir`, sorted by path for
/// deterministic loading order.
fn list_yaml_files(dir: &Path) -> Result<Vec<PathBuf>, RecipeLoadError> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|source| RecipeLoadError::Io {
            path: dir.to_path_buf(),
            source,
        })?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext == "yaml" || ext == "yml")
        })
        .collect();
    entries.sort();
    Ok(entries)
}

/// List every entry directly under `dir`, sorted by path.
fn list_dir_entries(dir: &Path) -> Result<Vec<PathBuf>, RecipeLoadError> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|source| RecipeLoadError::Io {
            path: dir.to_path_buf(),
            source,
        })?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    // ─── struct parsing ────────────────────────────────────────────────────

    #[test]
    fn parses_valid_recipe_manifest() {
        let yaml = "name: pr-reviewer\nversion: 0.1.0\ndefault_agent: main\n";
        let manifest: RecipeManifest = serde_yaml::from_str(yaml).expect("should parse");
        assert_eq!(manifest.name, "pr-reviewer");
        assert_eq!(manifest.version, "0.1.0");
        assert_eq!(manifest.default_agent, "main");
    }

    #[test]
    fn parses_valid_agent_manifest() {
        let yaml = "\
name: main
agent: claude
model:
  name: anthropic/claude-opus-4-8
  effort: high
skills:
  - code-review
system_instructions:
  mode: append
  content: Focus on correctness.
tools:
  - shell
subagents:
  - reviewer
";
        let agent: AgentManifest = serde_yaml::from_str(yaml).expect("should parse");
        assert_eq!(agent.name, "main");
        assert_eq!(agent.agent, AgentKind::Claude);
        assert_eq!(agent.model.effort, EffortLevel::High);
        assert_eq!(agent.skills, vec!["code-review".to_string()]);
        assert_eq!(agent.tools, vec!["shell".to_string()]);
        assert_eq!(agent.subagents, vec!["reviewer".to_string()]);
        assert!(agent.system_instructions.is_some());
    }

    #[test]
    fn agent_manifest_defaults_optional_collections() {
        let yaml =
            "name: minimal\nagent: codex\nmodel:\n  name: openai/gpt-5-codex\n  effort: low\n";
        let agent: AgentManifest = serde_yaml::from_str(yaml).expect("should parse");
        assert!(agent.skills.is_empty());
        assert!(agent.tools.is_empty());
        assert!(agent.subagents.is_empty());
        assert!(agent.system_instructions.is_none());
    }

    // ─── unknown field / non-string key rejection ─────────────────────────

    #[test]
    fn recipe_manifest_rejects_unknown_field() {
        let yaml = "name: x\nversion: 0.1.0\ndefault_agent: main\nextra: 1\n";
        let result: Result<RecipeManifest, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err(), "expected unknown field to be rejected");
    }

    #[test]
    fn agent_manifest_rejects_unknown_field() {
        let yaml = "name: x\nagent: claude\nmodel:\n  name: foo\n  effort: low\nnotarealfield: 1\n";
        let result: Result<AgentManifest, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err(), "expected unknown field to be rejected");
    }

    #[test]
    fn agent_manifest_rejects_non_string_key() {
        let yaml = "name: x\nagent: claude\nmodel:\n  name: foo\n  effort: low\n1: stray\n";
        let result: Result<AgentManifest, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err(), "expected non-string key to be rejected");
    }

    #[test]
    fn agent_manifest_rejects_unrecognized_harness() {
        let yaml = "name: x\nagent: gemini\nmodel:\n  name: foo\n  effort: low\n";
        let result: Result<AgentManifest, _> = serde_yaml::from_str(yaml);
        assert!(
            result.is_err(),
            "expected unrecognized agent kind to be rejected"
        );
    }

    #[test]
    fn model_spec_rejects_invalid_effort() {
        let yaml = "name: foo\neffort: extreme\n";
        let result: Result<ModelSpec, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err(), "expected invalid effort to be rejected");
    }

    // ─── load_recipe_dir ───────────────────────────────────────────────────

    #[test]
    fn load_recipe_dir_loads_valid_fixture() {
        let recipe = load_recipe_dir(&fixtures_dir().join("valid-recipe"))
            .expect("valid fixture should load");

        assert_eq!(recipe.manifest.name, "pr-reviewer");
        assert_eq!(recipe.manifest.default_agent, "main");
        assert_eq!(recipe.agents.len(), 2);

        let default_agent = recipe.default_agent().expect("default agent resolves");
        assert_eq!(default_agent.name, "main");
        assert_eq!(default_agent.subagents, vec!["reviewer".to_string()]);

        assert!(recipe.agents.contains_key("reviewer"));
        assert_eq!(
            recipe.system_md.as_deref(),
            Some("Shared base instructions.\n")
        );
        assert_eq!(recipe.skills.len(), 1);
    }

    #[test]
    fn load_recipe_dir_missing_marker() {
        let error = load_recipe_dir(&fixtures_dir().join("missing-marker"))
            .expect_err("missing recipe.yaml should fail to load");
        assert!(matches!(error, RecipeLoadError::MissingMarker { .. }));
        assert_eq!(error.code(), error_codes::RECIPE_MISSING_MARKER);
    }

    #[test]
    fn load_recipe_dir_unresolved_default_agent() {
        let error = load_recipe_dir(&fixtures_dir().join("unresolved-default-agent"))
            .expect_err("dangling default_agent should fail to load");
        assert!(matches!(
            error,
            RecipeLoadError::DefaultAgentUnresolved { .. }
        ));
        assert_eq!(error.code(), error_codes::RECIPE_DEFAULT_AGENT_UNRESOLVED);
    }

    #[test]
    fn load_recipe_dir_malformed_agent_yaml() {
        let error = load_recipe_dir(&fixtures_dir().join("malformed-agent"))
            .expect_err("malformed agent yaml should fail to load");
        assert!(matches!(error, RecipeLoadError::MalformedAgent { .. }));
        assert_eq!(error.code(), error_codes::RECIPE_MALFORMED_AGENT_YAML);
    }

    #[test]
    fn load_recipe_dir_without_optional_dirs_still_loads() {
        let recipe = load_recipe_dir(&fixtures_dir().join("minimal-recipe"))
            .expect("minimal fixture (no skills/, no SYSTEM.md) should load");
        assert!(recipe.system_md.is_none());
        assert!(recipe.skills.is_empty());
    }

    // ─── resolved_system ────────────────────────────────────────────────────

    fn agent_with_instructions(mode: InstructionMode, content: &str) -> AgentManifest {
        AgentManifest {
            name: "main".to_string(),
            agent: AgentKind::Claude,
            model: ModelSpec {
                name: "anthropic/claude-opus-4-8".to_string(),
                effort: EffortLevel::High,
            },
            skills: Vec::new(),
            system_instructions: Some(SystemInstructions {
                mode,
                content: content.to_string(),
            }),
            tools: Vec::new(),
            subagents: Vec::new(),
        }
    }

    fn agent_without_instructions() -> AgentManifest {
        AgentManifest {
            name: "main".to_string(),
            agent: AgentKind::Claude,
            model: ModelSpec {
                name: "anthropic/claude-opus-4-8".to_string(),
                effort: EffortLevel::High,
            },
            skills: Vec::new(),
            system_instructions: None,
            tools: Vec::new(),
            subagents: Vec::new(),
        }
    }

    #[test]
    fn resolved_system_append_combines_base_and_content() {
        let agent = agent_with_instructions(InstructionMode::Append, "Focus on security.");
        let result = resolved_system(&agent, Some("Base instructions."));
        assert_eq!(
            result.as_deref(),
            Some("Base instructions.\n\nFocus on security.")
        );
    }

    #[test]
    fn resolved_system_append_without_base_uses_content_only() {
        let agent = agent_with_instructions(InstructionMode::Append, "Focus on security.");
        let result = resolved_system(&agent, None);
        assert_eq!(result.as_deref(), Some("Focus on security."));
    }

    #[test]
    fn resolved_system_replace_ignores_base() {
        let agent = agent_with_instructions(InstructionMode::Replace, "Only this.");
        let result = resolved_system(&agent, Some("Base instructions."));
        assert_eq!(result.as_deref(), Some("Only this."));
    }

    #[test]
    fn resolved_system_none_falls_back_to_base() {
        let agent = agent_without_instructions();
        let result = resolved_system(&agent, Some("Base instructions."));
        assert_eq!(result.as_deref(), Some("Base instructions."));
    }

    #[test]
    fn resolved_system_none_and_no_base_is_none() {
        let agent = agent_without_instructions();
        let result = resolved_system(&agent, None);
        assert_eq!(result, None);
    }
}
