//! Directory-shape schema and loader for EXP-V1-0005 (Agent Recipe Standard).
//!
//! A **recipe** is a directory, not a single YAML file:
//!
//! ```text
//! <recipe>/
//!   recipe.yaml            # RecipeManifest: name, version, default_agent
//!   agents/<name>.yaml     # AgentManifest (per-agent, unified agents+subagents)
//!   skills/                # optional: bundled skill packages
//!   tools/<ref>/tool.yaml  # optional: recipe-provided tools
//!   evals/<name>/          # optional: eval cases (input.json + expected.md)
//!   judges/<name>.yaml     # optional: judge manifests
//!   prompts/<name>.md      # optional: prompt text referenced by judges
//!   SYSTEM.md               # optional: shared base instructions
//! ```
//!
//! This module owns the typed contract ([`RecipeManifest`], [`AgentManifest`],
//! [`Recipe`]) and the canonical loader ([`load_recipe_dir`]). It has **no
//! CLI-only dependencies** so that downstream consumers (notably `itmux`,
//! Plan B) can depend on this crate purely as a library.
//!
//! Validating a directory against every rule in the spec (unresolved
//! `subagents`, malformed skill refs, directory-level diagnostics with error
//! codes, etc.) is `validate_recipe_dir` in `src/validate.rs`. Loading and
//! validation share one code path: `validate_recipe_dir` calls
//! `load_recipe_dir` and, on a successful load, applies the additional
//! structural rules the typed loader does not enforce on its own (empty
//! `name`/`model.name`/`system_instructions.content`, unresolved `subagents`,
//! malformed skill/tool refs). A failed load is surfaced as a single
//! diagnostic carrying the loader's stable error code.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};
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

/// Optional directory (relative to the recipe root) holding recipe-provided
/// tools: one subdirectory per tool, each with its own `tool.yaml`. See
/// section 5.2.
pub const TOOLS_DIR: &str = "tools";

/// Per-tool manifest file name, resolved as `tools/<ref>/tool.yaml`.
pub const TOOL_MANIFEST_FILE: &str = "tool.yaml";

/// Optional directory (relative to the recipe root) holding eval cases: one
/// subdirectory per case, each with `input.json` and `expected.md`. See
/// section 9.
pub const EVALS_DIR: &str = "evals";

/// Per-eval-case input file name, resolved as `evals/<name>/input.json`.
pub const EVAL_INPUT_FILE: &str = "input.json";

/// Per-eval-case expected-output file name, resolved as
/// `evals/<name>/expected.md`.
pub const EVAL_EXPECTED_FILE: &str = "expected.md";

/// Optional directory (relative to the recipe root) holding judge manifests:
/// one YAML file per judge. See section 9.
pub const JUDGES_DIR: &str = "judges";

/// Optional directory (relative to the recipe root) holding prompt text
/// referenced by judges (or anything else) via `prompts/<file>.md`. See
/// section 9.
pub const PROMPTS_DIR: &str = "prompts";

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
    /// Package-tier MCP server policy: the ceiling every agent's own `mcp`
    /// (section 7) is checked against. Absent, or present with no `servers`
    /// entries, permits no MCP server at all - a restrictive default,
    /// deliberately unlike `tools`' permissive `None` (section 4.6). See
    /// [`McpPolicy`] and [`mcp_policy_widenings`].
    #[serde(default, skip_serializing_if = "McpPolicy::is_empty")]
    pub mcp: McpPolicy,
}

/// Per-server MCP method policy. `include` names the permitted methods;
/// `exclude` removes from that set. An empty `include` permits no methods,
/// mirroring `tools: Some(vec![])` (section 4.6). This standard does not
/// support a wildcard `include` value in this version: every `include`/
/// `exclude` entry is compared as a literal method name, never expanded. See
/// [`McpServerPolicy::effective_methods`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpServerPolicy {
    /// Permitted method names for this server.
    #[serde(default)]
    pub include: Vec<String>,
    /// Method names removed from `include`. Narrows further; never widens.
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl McpServerPolicy {
    /// The effective (permitted) method set for this server policy:
    /// `include` minus `exclude`, both compared as literal strings. This is
    /// the single definition of "effective set" every tier and consumer of
    /// this rule MUST use (section 7).
    pub fn effective_methods(&self) -> BTreeSet<&str> {
        let mut methods: BTreeSet<&str> = self.include.iter().map(String::as_str).collect();
        for excluded in &self.exclude {
            methods.remove(excluded.as_str());
        }
        methods
    }
}

/// MCP server policy for one tier (package or agent): which servers are
/// named, and the [`McpServerPolicy`] (method allowlist) for each. A server
/// this policy does not mention at all is not permitted - there is no
/// implicit "everything else is allowed" fallback. See section 7.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpPolicy {
    /// Per-server method policy, keyed by server id.
    #[serde(default)]
    pub servers: BTreeMap<String, McpServerPolicy>,
}

impl McpPolicy {
    /// Whether this policy names no servers at all (the default, restrictive
    /// state: no MCP access).
    pub fn is_empty(&self) -> bool {
        self.servers.is_empty()
    }
}

/// The servers `candidate` permits that `ceiling` does not permit for it,
/// naming each offending server id. `ceiling` is whatever policy `candidate`
/// must not exceed - the package's `mcp` when checking an agent, or a
/// resolved parent's `mcp` when checking a `from:` child against that one
/// link. Two ways a server can be offending:
///
/// 1. `candidate` names a server `ceiling` does not mention at all. This is
///    always a widening - inventing access `ceiling` never granted - even
///    if `candidate`'s policy for that server is otherwise empty.
/// 2. `candidate` and `ceiling` both name the server, but `candidate`'s
///    [`McpServerPolicy::effective_methods`] is not a subset of `ceiling`'s.
///
/// This is the single shared computation every narrowing check in this
/// crate - the package-tier check and the per-`from:`-link check alike, and
/// any future consumer of this rule - MUST use, rather than re-implementing
/// the include/exclude comparison in more than one place.
pub fn mcp_policy_widenings(ceiling: &McpPolicy, candidate: &McpPolicy) -> Vec<String> {
    let mut offending = Vec::new();
    for (server_id, candidate_policy) in &candidate.servers {
        match ceiling.servers.get(server_id) {
            None => offending.push(server_id.clone()),
            Some(ceiling_policy) => {
                let candidate_methods = candidate_policy.effective_methods();
                let ceiling_methods = ceiling_policy.effective_methods();
                if !candidate_methods.is_subset(&ceiling_methods) {
                    offending.push(server_id.clone());
                }
            }
        }
    }
    offending
}

/// Which harness executes an agent.
///
/// Intentionally a closed enum in v1: future MINOR versions of this standard
/// MAY add variants (e.g. `opencode`, `gemini`) without breaking existing
/// recipes; consumers on an older version of this crate will fail to parse
/// (via serde) rather than silently accepting an unrecognized harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HarnessKind {
    /// Claude Code.
    Claude,
    /// OpenAI Codex CLI.
    Codex,
}

/// How a recipe-provided tool (section 5.2) is invoked.
///
/// Both protocols share the same portability guarantee: neither one links a
/// harness API. `McpStdio` (the default) means the tool is spoken to as an
/// MCP server over stdio, which supplies schema and invocation semantics for
/// free and is already cross-harness. `Subprocess` is the escape hatch for a
/// one-file script where a full MCP server is overkill: argv in, JSON on
/// stdout, a non-zero exit code means failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ToolProtocol {
    /// The tool is an MCP server spoken to over stdio.
    #[default]
    McpStdio,
    /// The tool is a plain subprocess: argv in, JSON on stdout, non-zero
    /// exit means failure.
    Subprocess,
}

/// A recipe-provided tool's manifest: `tools/<ref>/tool.yaml`.
///
/// This standard defines the tool's declaration and invocation contract; it
/// does NOT execute tools itself, so this crate carries no process-spawning
/// dependency. See section 5.2 for the portability rule this manifest
/// exists to support: a recipe-provided tool MUST NOT link a harness API and
/// MUST be invocable as a subprocess by any conforming consumer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolManifest {
    /// Tool name. MUST be non-empty. Conventionally matches the `tools/<ref>`
    /// directory name, though resolution keys off the directory, not this
    /// field.
    pub name: String,
    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The executable to invoke. MUST be non-empty. This standard does NOT
    /// validate that the command exists on disk or is executable: a recipe
    /// is a portable artifact that may be validated on a machine that will
    /// never run it.
    pub command: String,
    /// Fixed leading arguments passed to `command` on every invocation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Invocation protocol. Defaults to `mcp-stdio`.
    #[serde(default)]
    pub protocol: ToolProtocol,
}

/// One eval case discovered from `evals/<name>/`. A case is a directory
/// containing exactly `input.json` (the case input) and `expected.md` (the
/// bar the eval's output is judged against). This standard defines the
/// declaration only: it does not parse `input.json`, does not define a
/// scoring model, and does not define how `expected.md` is compared against
/// an actual run. Those are consumer concerns (section 9).
///
/// `evals/<name>/` missing either file is malformed and MUST be reported
/// (`RECIPE_MALFORMED_EVAL_CASE`), never silently skipped: a quietly
/// incomplete eval case is a bar that has silently shrunk while still
/// reporting green.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalCase {
    /// The eval case name: the `evals/<name>` directory name.
    pub name: String,
    /// Path to `evals/<name>/input.json`.
    pub input_path: PathBuf,
    /// Path to `evals/<name>/expected.md`.
    pub expected_path: PathBuf,
}

/// A judge's manifest: `judges/<name>.yaml`. Declares the judge only; this
/// standard defines no scoring model, passing threshold, or execution
/// semantics for how a judge is actually run against an eval case - that is
/// a consumer concern (section 9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JudgeManifest {
    /// Judge name. MUST be non-empty.
    pub name: String,
    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The judge's prompt, given inline. At least one of `prompt` /
    /// `prompt_file` MUST be present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Reference to a prompt file, conventionally resolving to
    /// `prompts/<prompt_file>`. Used instead of an inline `prompt`. At least
    /// one of `prompt` / `prompt_file` MUST be present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_file: Option<String>,
    /// Loader-populated provenance: the `judges/*.yaml` file this manifest
    /// was parsed from. Not part of the on-disk schema - `#[serde(skip)]`
    /// keeps it out of both serialized output and accepted input - so a
    /// diagnostic anchored to a specific judge (e.g. an empty `name`, which
    /// has no other identifying field) can still point an author at the
    /// right file. Mirrors [`EvalCase::input_path`]/[`EvalCase::expected_path`].
    #[serde(skip)]
    pub source_path: PathBuf,
}

/// Coarse reasoning/thinking effort level.
///
/// Maps to harness-specific concepts such as `thinking_level` (Claude) or
/// reasoning effort (Codex). Defaults to `Medium` when a `model` block
/// omits `effort`, per section 4.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EffortLevel {
    Low,
    #[default]
    Medium,
    High,
}

/// Model selection: `model.name` and `model.effort`.
///
/// `name` is optional: an agent that inherits via `from:` (section 4.7) may
/// declare a `model` block that overrides `effort` alone, leaving `name` to
/// resolve from the parent. An agent with no `from:` and no inherited `name`
/// anywhere in its chain asserts no opinion about which model to use; see
/// section 4.4.
///
/// `PartialEq` only (not `Eq`): `temperature` is an `Option<f32>`, and `f32`
/// does not implement `Eq` (NaN is not equal to itself), so `ModelSpec` -
/// and, transitively, [`AgentManifest`] and [`Recipe`] - cannot derive `Eq`
/// once this field exists.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelSpec {
    /// Provider-qualified model identifier, e.g. `anthropic/claude-opus-4-8`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Coarse reasoning effort level. Defaults to `Medium` when omitted.
    #[serde(default)]
    pub effort: EffortLevel,
    /// Declared ceiling on output tokens for this model, not a fixed value.
    /// A run (section 4.5) MAY override the declared `model` and MAY narrow
    /// this ceiling further; it MUST NOT raise it, per the same monotonic
    /// narrowing principle sections 4.7 and 7 establish for `tools` and
    /// `mcp`. This standard does NOT validate `max_tokens` against any
    /// model's actual limit - see section 4.4.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Declared sampling temperature. This standard does NOT validate it
    /// against a numeric range: providers disagree on the valid range (some
    /// accept `0..=2`, others `0..=1`), and this standard already declines
    /// to validate that `model.name` names a real model (section 4.4) for
    /// the same reason - range-checking one provider's convention would
    /// make a valid recipe invalid on another provider.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
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

/// Whether the resolved system prompt is appended to the harness's own
/// default prompt or replaces it. Independent of [`InstructionMode`],
/// which governs composition with `SYSTEM.md` only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HarnessPromptMode {
    #[default]
    Append,
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
    /// Whether the resolved system prompt appends to or replaces the
    /// harness's own built-in system prompt. Independent of `mode`, which
    /// governs composition with `SYSTEM.md` only; a harness adapter (not
    /// `resolved_system`) consumes this field.
    #[serde(default)]
    pub harness_prompt: HarnessPromptMode,
}

/// A single `skills` entry: either a bare reference or a pinned object.
///
/// This standard is designed for a recipe's `evals/` and `judges/` (section
/// 9) to be a meaningful, attributable definition of good. That definition
/// is only meaningful if the recipe's inputs are reproducible: a skill
/// resolved as `@latest` means two runs of the same recipe are not
/// necessarily the same agent, so a comparison between them proves nothing.
/// The pinned form lets a recipe author record exactly which skill, and
/// optionally which source, version, and content hash, an agent was built
/// against.
///
/// Both forms are accepted deliberately. Every recipe authored before this
/// field existed uses the bare-string form (e.g. `skills: [code-review]`),
/// and accepting it unchanged keeps this addition additive rather than
/// breaking. The two forms resolve identically: [`SkillRef::name`] returns
/// the same value the resolution logic in section 5.1 consumes regardless
/// of which form was used, so a bare string and a pinned object naming the
/// same `ref` behave the same way.
///
/// `Deserialize` is implemented by hand rather than derived with
/// `#[serde(untagged)]`. An untagged derive tries each variant in turn and,
/// on total failure, discards every per-variant error in favor of a single
/// generic "data did not match any variant of untagged enum SkillRef"
/// message - which never says whether the problem was an unknown field, a
/// missing `ref`, or something else. That is weaker than every other
/// diagnostic in this crate, all of which name their cause. Dispatching by
/// shape (string vs. mapping) before deserializing lets a mapping decode
/// straight into [`PinnedSkillRef`], so `#[serde(deny_unknown_fields)]`'s
/// own field-level error (e.g. "unknown field `versoin`, expected one of
/// ...") surfaces unmodified instead of being swallowed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum SkillRef {
    /// `skills: [research]` - an unpinned, harness-agnostic skill name or
    /// path, resolved exactly as before this field existed (section 5.1).
    Bare(String),
    /// `skills: [{ ref: research, version: 1.2.0, ... }]` - a pinned skill
    /// reference. See [`PinnedSkillRef`] for the field semantics.
    Pinned(PinnedSkillRef),
}

impl<'de> Deserialize<'de> for SkillRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SkillRefVisitor;

        impl<'de> serde::de::Visitor<'de> for SkillRefVisitor {
            type Value = SkillRef;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a skill reference: either a bare string or a pinned object")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SkillRef::Bare(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SkillRef::Bare(value))
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                // Deserializing directly into `PinnedSkillRef` (rather than
                // trying it as one of several untagged candidates) is what
                // preserves serde's own field-level error - an unknown
                // field or missing `ref` is reported by name, not
                // discarded in favor of a generic "no variant matched".
                let pinned =
                    PinnedSkillRef::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                Ok(SkillRef::Pinned(pinned))
            }
        }

        deserializer.deserialize_any(SkillRefVisitor)
    }
}

impl SkillRef {
    /// The value skill resolution (section 5.1) consumes: the bare string
    /// itself, or a pinned object's `ref`. Both forms resolve identically.
    pub fn name(&self) -> &str {
        match self {
            SkillRef::Bare(name) => name,
            SkillRef::Pinned(pinned) => &pinned.r#ref,
        }
    }
}

/// A pinned skill reference: `{ ref, source_url, version, resolved_sha }`.
///
/// `ref` is the only required field - it is what [`SkillRef::name`] returns
/// and what section 5.1 resolution consumes, identically to the bare-string
/// form. `source_url`, `version`, and `resolved_sha` are all OPTIONAL: a
/// recipe author may reasonably pin by `version` before any resolver has
/// produced a `resolved_sha`, so requiring all four up front would make the
/// pinned form unusable until tooling exists to populate it.
///
/// This standard records what a recipe declares. It does NOT resolve a
/// skill reference over the network or compute a `resolved_sha` itself;
/// that is a consumer's concern, not this standard's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinnedSkillRef {
    /// The skill reference. MUST be non-empty. Resolved exactly as the
    /// bare-string form is (section 5.1).
    pub r#ref: String,
    /// Where the skill was fetched from (e.g. a git URL). Informative only;
    /// this standard does not fetch it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    /// The pinned version. MUST NOT be `latest` or `@latest`
    /// (case-insensitively), and MUST NOT be empty or all-whitespace, when
    /// present - see `RECIPE_SKILL_UNPINNED`. Absent is not an error: it
    /// asserts no version opinion, not an unpinned reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The resolved content hash, if a resolver has already computed one.
    /// The strongest reproducibility guarantee, but NOT required in this
    /// version of the standard.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_sha: Option<String>,
}

/// One agent, loaded from `agents/<name>.yaml`. Unifies default agents and
/// subagents: whether a given `AgentManifest` is the recipe's entry point is
/// determined by `RecipeManifest::default_agent`, not by any field here.
///
/// Field names/enums intentionally match the prior single-YAML EXP-V1-0005
/// schema (`name`, `harness`, `model`, `skills`, `system_instructions`),
/// which itself kept pi.recipes-inspired naming; `tools` and `subagents` are
/// new in the directory shape. See `docs/04-rationale-and-prior-art.md` for
/// why this standard is inspired by pi.recipes but not compatible with it.
///
/// `PartialEq` only (not `Eq`): `model` carries a `ModelSpec`, which cannot
/// derive `Eq` (see [`ModelSpec`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentManifest {
    /// Agent name. SHOULD match the file stem (`agents/<name>.yaml`).
    pub name: String,
    /// Human-readable description of this agent. Informative only; this
    /// standard does not interpret its contents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Which harness this agent REQUIRES. Absent means harness-agnostic:
    /// the agent must run correctly under any conforming harness, which
    /// section 4.3 enforces by forbidding harness-builtin tool references.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<HarnessKind>,
    /// Model selection. Optional: an agent may omit `model` entirely and
    /// inherit it wholesale via `from:` (section 4.7), or omit only `name`
    /// within a declared `model` block to inherit that one field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelSpec>,
    /// Harness-agnostic skill references to inject, in listed order. Each
    /// entry is a [`SkillRef`]: either a bare string or a pinned object.
    ///
    /// Per R3, each entry resolves to a plugin-dir path via
    /// [`SkillRef::name`]: (a) `skills/<ref>/` inside the recipe if that
    /// subdirectory exists, else (b) the ref as-is (an external skill
    /// path/name). Resolution order is the order these entries are
    /// declared, and callers (e.g. Plan B's `claude_plugin_dirs`) MUST
    /// preserve it.
    #[serde(default)]
    pub skills: Vec<SkillRef>,
    /// Optional system instruction override. See [`resolved_system`] for the
    /// exact merge semantics with the recipe's shared `SYSTEM.md`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_instructions: Option<SystemInstructions>,
    /// Tool references this agent is permitted to use.
    ///
    /// `None` (absent) places no restriction. `Some(vec![])` permits no
    /// tools at all. The two are deliberately distinct: a recipe that omits
    /// the field is not asserting that its agent may do nothing. See
    /// `docs/01_spec.md` section 4.6 for the enforcement rule and
    /// `validate::is_harness_builtin` for the harness-agnostic check that
    /// consumes these entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    /// Agent-tier MCP server policy, checked against the package's `mcp`
    /// (section 7). `None` (absent) declares no restriction of its own: the
    /// agent's effective policy is exactly the package's, which is always a
    /// subset of itself and so never flagged as a widening. `Some(policy)`
    /// narrows to the named servers/methods; a server that policy names but
    /// the package does not, or a method set wider than the package permits
    /// for a shared server, is `RECIPE_MCP_AGENT_WIDENS_POLICY`. See
    /// [`mcp_policy_widenings`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp: Option<McpPolicy>,
    /// Names of other agents (files in `agents/`, without the `.yaml`
    /// extension) this agent may delegate to as subagents.
    ///
    /// This is a different concept from [`AgentManifest::allow_delegation`]
    /// and the two MUST NOT be conflated: `subagents` names siblings WITHIN
    /// this recipe (validated to resolve, `RECIPE_SUBAGENT_UNRESOLVED`);
    /// `allow_delegation` is permission to hand work to the OTHER harness as
    /// a peer, naming no sibling at all. An agent may have either, both, or
    /// neither.
    #[serde(default)]
    pub subagents: Vec<String>,
    /// Whether this agent may delegate work to a peer harness outside this
    /// recipe. Defaults to `false`.
    ///
    /// Distinct from `subagents`: `subagents` names sibling agents WITHIN
    /// this recipe and is validated to resolve; `allow_delegation` grants no
    /// name at all, only the bare permission to hand off to another
    /// harness. The two are orthogonal - an agent may declare either, both,
    /// or neither. Defaulting to `false` is deliberate: a capability that
    /// lets an agent reach outside the recipe boundary SHOULD be opt-in, not
    /// assumed. A plain `bool` (not `Option<bool>`) is used because there is
    /// no meaningful difference between "absent" and "false" here - both
    /// mean the agent may not delegate outside the recipe, and an
    /// `Option<bool>` would only invite a consumer to invent semantics for a
    /// third state this standard does not need.
    ///
    /// It is nonetheless a permission field, narrowing-only via `from:`
    /// exactly like `tools` and `mcp`: a child MAY tighten it from `true`
    /// to `false`, but MUST NOT widen it from `false` to `true`, rejected
    /// as `RECIPE_FROM_WIDENS_DELEGATION`. See [`resolve_inherited`].
    #[serde(default)]
    pub allow_delegation: bool,
    /// Name of a sibling agent this agent inherits from. Resolution is a
    /// field-wise merge: fields the child declares win, fields it omits are
    /// taken from the parent. Permission fields may only narrow. See
    /// [`resolve_inherited`] and section 4.7.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
}

/// A fully loaded recipe directory: the manifest, every parsed agent (keyed
/// by file stem), the gathered `skills/` entries, and the optional shared
/// `SYSTEM.md` content.
///
/// `PartialEq` only (not `Eq`): `agents` carries [`AgentManifest`] values,
/// which cannot derive `Eq` (see [`ModelSpec`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
    /// Parsed `recipe.yaml`.
    pub manifest: RecipeManifest,
    /// Every parsed `agents/*.yaml`, keyed by file stem (the agent name used
    /// by `default_agent` and `subagents` references).
    pub agents: BTreeMap<String, AgentManifest>,
    /// The concrete source file each agent was loaded from, keyed by the same
    /// file stem as [`Recipe::agents`]. Retains the real extension (`.yaml` vs
    /// `.yml`) so diagnostics can point at the file that actually exists.
    pub agent_sources: BTreeMap<String, PathBuf>,
    /// Skill package subdirectories found directly under `skills/`, sorted, if
    /// that directory exists. Only directories are listed (loose files such as
    /// the generator's `.gitkeep` are ignored), so a freshly scaffolded recipe
    /// with an empty `skills/` reports an empty inventory. This is the recipe's
    /// bundled skill inventory, not a resolved per-agent reference list (see
    /// [`AgentManifest::skills`] for that).
    pub skills: Vec<PathBuf>,
    /// Recipe-provided tools found under `tools/`, keyed by the `tools/<ref>`
    /// directory name - the same string an agent's `tools` entry uses to
    /// reference it. Empty if `tools/` does not exist. See section 5.2 and
    /// [`Recipe::resolve_tool`].
    pub tools: BTreeMap<String, ToolManifest>,
    /// Eval cases found under `evals/`, sorted by case name. Empty if
    /// `evals/` does not exist. See section 9.
    pub evals: Vec<EvalCase>,
    /// Judge manifests found under `judges/`, sorted by source file path.
    /// Empty if `judges/` does not exist. See section 9.
    pub judges: Vec<JudgeManifest>,
    /// `prompts/*.md` files found under `prompts/`, sorted. Empty if
    /// `prompts/` does not exist. See section 9.
    pub prompts: Vec<PathBuf>,
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

    /// Resolve a `tools` entry against this recipe's `tools/` directory
    /// (section 5.2). Returns `None` when `name` does not match a
    /// `tools/<name>/tool.yaml` gathered at load time - this is not an
    /// error; the ref may be a harness-builtin name instead.
    pub fn resolve_tool(&self, name: &str) -> Option<ToolManifest> {
        self.tools.get(name).cloned()
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
    pub const RECIPE_MALFORMED_HARNESS_YAML: &str = "RECIPE_MALFORMED_HARNESS_YAML";
    /// Two agent files share the same stem (e.g. `main.yaml` and `main.yml`).
    pub const RECIPE_DUPLICATE_AGENT: &str = "RECIPE_DUPLICATE_AGENT";
    /// `default_agent` does not resolve to any parsed entry in `agents/`.
    pub const RECIPE_DEFAULT_AGENT_UNRESOLVED: &str = "RECIPE_DEFAULT_AGENT_UNRESOLVED";
    /// An I/O error occurred while reading the recipe directory.
    pub const RECIPE_IO_ERROR: &str = "RECIPE_IO_ERROR";
    /// A `from:` chain revisits an agent it has already visited (including an
    /// agent naming itself).
    pub const RECIPE_FROM_CYCLE: &str = "RECIPE_FROM_CYCLE";
    /// A `from:` (or the initially requested agent name) does not resolve to
    /// any parsed entry in `agents/`.
    pub const RECIPE_FROM_UNRESOLVED: &str = "RECIPE_FROM_UNRESOLVED";
    /// A child agent's `tools` is not a subset of its resolved parent's
    /// `tools`, which would widen permission instead of narrowing it.
    pub const RECIPE_FROM_WIDENS_TOOLS: &str = "RECIPE_FROM_WIDENS_TOOLS";
    /// A child agent's `mcp` is not a subset of its resolved parent's `mcp`,
    /// which would widen permission instead of narrowing it. Distinct from
    /// `RECIPE_MCP_AGENT_WIDENS_POLICY` (validate::error_codes), which
    /// checks the fully resolved agent against the package tier: this code
    /// is the per-`from:`-link check, mirroring `RECIPE_FROM_WIDENS_TOOLS`.
    pub const RECIPE_MCP_FROM_WIDENS_POLICY: &str = "RECIPE_MCP_FROM_WIDENS_POLICY";
    /// A child agent declares `allow_delegation: true` while its resolved
    /// parent declares `allow_delegation: false`, which would widen
    /// permission instead of narrowing it, mirroring
    /// `RECIPE_FROM_WIDENS_TOOLS` and `RECIPE_MCP_FROM_WIDENS_POLICY`.
    pub const RECIPE_FROM_WIDENS_DELEGATION: &str = "RECIPE_FROM_WIDENS_DELEGATION";
    /// A `tools/*/tool.yaml` file failed to parse as a [`super::ToolManifest`].
    pub const RECIPE_MALFORMED_TOOL_MANIFEST: &str = "RECIPE_MALFORMED_TOOL_MANIFEST";
    /// An `evals/<name>/` directory is missing `input.json` or
    /// `expected.md`.
    pub const RECIPE_MALFORMED_EVAL_CASE: &str = "RECIPE_MALFORMED_EVAL_CASE";
    /// A `judges/*.yaml` file failed to parse as a [`super::JudgeManifest`].
    pub const RECIPE_MALFORMED_JUDGE_MANIFEST: &str = "RECIPE_MALFORMED_JUDGE_MANIFEST";
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
    /// Two agent files resolve to the same stem (e.g. `main.yaml` and
    /// `main.yml`). Both cannot occupy the same name in the recipe, so this is
    /// rejected rather than silently keeping one.
    #[error("duplicate agent '{stem}': {first} and {second} share the same name")]
    DuplicateAgent {
        /// The colliding agent stem.
        stem: String,
        /// The first file seen with this stem.
        first: PathBuf,
        /// The second file seen with this stem.
        second: PathBuf,
    },
    /// `default_agent` does not name any file actually present in `agents/`.
    #[error(
        "default_agent '{default_agent}' does not resolve to {agents_dir}/{default_agent}.yaml (or .yml)"
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
    /// A `from:` chain revisited an agent already seen while resolving it
    /// (including an agent whose `from:` names itself).
    #[error("cycle detected resolving 'from' chain at agent '{name}'")]
    FromCycle {
        /// The agent name that was revisited.
        name: String,
        /// Best-effort anchor: the source file of the agent where the cycle
        /// was detected, if known.
        path: PathBuf,
    },
    /// A `from:` value (or the initially requested agent name) does not name
    /// any parsed entry in `agents/`.
    #[error("'from: {from}' does not resolve to {AGENTS_DIR}/{from}.yaml (or .yml)")]
    FromUnresolved {
        /// The unresolved name.
        from: String,
        /// Best-effort anchor: the source file of the agent that referenced
        /// `from`, if known.
        path: PathBuf,
    },
    /// A child agent's `tools` is not a subset of its resolved parent's
    /// `tools`.
    #[error(
        "agent '{agent}' widens tools via 'from': {offending:?} not permitted by the resolved parent"
    )]
    FromWidensTools {
        /// The child agent's name.
        agent: String,
        /// The `tools` entries the child grants that its resolved parent
        /// does not permit.
        offending: Vec<String>,
        /// Source file of the offending child agent, if known.
        path: PathBuf,
    },
    /// A child agent's `mcp` is not a subset of its resolved parent's `mcp`,
    /// checked at this one `from:` link (section 7.3).
    #[error(
        "agent '{agent}' widens mcp via 'from': server(s) {offending:?} not permitted by the resolved parent"
    )]
    FromWidensMcp {
        /// The child agent's name.
        agent: String,
        /// The server ids the child grants (or grants wider methods for)
        /// that its resolved parent does not permit.
        offending: Vec<String>,
        /// Source file of the offending child agent, if known.
        path: PathBuf,
    },
    /// A child agent declares `allow_delegation: true` while its resolved
    /// parent declares `allow_delegation: false`. `allow_delegation` is a
    /// permission (the ability to reach outside the recipe to a peer
    /// harness), not a capability declaration, so it is narrowing-only via
    /// `from:` exactly like `tools` and `mcp` (section 4.4a, section 4.7).
    #[error(
        "agent '{agent}' widens allow_delegation via 'from': resolved parent declares allow_delegation: false"
    )]
    FromWidensDelegation {
        /// The child agent's name.
        agent: String,
        /// Source file of the offending child agent, if known.
        path: PathBuf,
    },
    /// A `tools/*/tool.yaml` file is not a valid [`ToolManifest`].
    #[error("malformed tool manifest at {path}: {source}")]
    MalformedTool {
        /// Path to the offending `tool.yaml`.
        path: PathBuf,
        /// Underlying parse error.
        #[source]
        source: serde_yaml::Error,
    },
    /// An `evals/<name>/` directory is missing `input.json` or
    /// `expected.md`.
    #[error("malformed eval case '{name}': missing {missing} at {path}")]
    MalformedEvalCase {
        /// The eval case (directory) name.
        name: String,
        /// Which required file is missing: `input.json` or `expected.md`.
        missing: &'static str,
        /// Path to the offending `evals/<name>/` directory.
        path: PathBuf,
    },
    /// A `judges/*.yaml` file is not a valid [`JudgeManifest`].
    #[error("malformed judge manifest at {path}: {source}")]
    MalformedJudge {
        /// Path to the offending judge YAML file.
        path: PathBuf,
        /// Underlying parse error.
        #[source]
        source: serde_yaml::Error,
    },
}

impl RecipeLoadError {
    /// The stable machine-readable code for this error, matching
    /// `error_codes`.
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingMarker { .. } => error_codes::RECIPE_MISSING_MARKER,
            Self::MalformedManifest { .. } => error_codes::RECIPE_MALFORMED_MANIFEST,
            Self::MalformedAgent { .. } => error_codes::RECIPE_MALFORMED_HARNESS_YAML,
            Self::DuplicateAgent { .. } => error_codes::RECIPE_DUPLICATE_AGENT,
            Self::DefaultAgentUnresolved { .. } => error_codes::RECIPE_DEFAULT_AGENT_UNRESOLVED,
            Self::Io { .. } => error_codes::RECIPE_IO_ERROR,
            Self::FromCycle { .. } => error_codes::RECIPE_FROM_CYCLE,
            Self::FromUnresolved { .. } => error_codes::RECIPE_FROM_UNRESOLVED,
            Self::FromWidensTools { .. } => error_codes::RECIPE_FROM_WIDENS_TOOLS,
            Self::FromWidensMcp { .. } => error_codes::RECIPE_MCP_FROM_WIDENS_POLICY,
            Self::FromWidensDelegation { .. } => error_codes::RECIPE_FROM_WIDENS_DELEGATION,
            Self::MalformedTool { .. } => error_codes::RECIPE_MALFORMED_TOOL_MANIFEST,
            Self::MalformedEvalCase { .. } => error_codes::RECIPE_MALFORMED_EVAL_CASE,
            Self::MalformedJudge { .. } => error_codes::RECIPE_MALFORMED_JUDGE_MANIFEST,
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
            Self::DuplicateAgent { second, .. } => second,
            Self::DefaultAgentUnresolved { agents_dir, .. } => agents_dir,
            Self::Io { path, .. } => path,
            Self::FromCycle { path, .. } => path,
            Self::FromUnresolved { path, .. } => path,
            Self::FromWidensTools { path, .. } => path,
            Self::FromWidensMcp { path, .. } => path,
            Self::FromWidensDelegation { path, .. } => path,
            Self::MalformedTool { path, .. } => path,
            Self::MalformedEvalCase { path, .. } => path,
            Self::MalformedJudge { path, .. } => path,
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
    if !path_exists(&manifest_path)? {
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
    // Track the source file per stem so a `main.yaml` / `main.yml` collision is
    // reported instead of the second file silently overwriting the first.
    let mut stem_sources: BTreeMap<String, PathBuf> = BTreeMap::new();
    if dir_exists(&agents_dir)? {
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
            if let Some(first) = stem_sources.get(&stem) {
                return Err(RecipeLoadError::DuplicateAgent {
                    stem,
                    first: first.clone(),
                    second: entry_path,
                });
            }
            stem_sources.insert(stem.clone(), entry_path);
            agents.insert(stem, agent);
        }
    }
    let agent_sources = stem_sources;

    if !agents.contains_key(&manifest.default_agent) {
        return Err(RecipeLoadError::DefaultAgentUnresolved {
            default_agent: manifest.default_agent,
            agents_dir,
        });
    }

    let system_md_path = path.join(SYSTEM_MD_FILE);
    let system_md = if path_exists(&system_md_path)? {
        Some(read_to_string(&system_md_path)?)
    } else {
        None
    };

    let skills_dir = path.join(SKILLS_DIR);
    let skills = if dir_exists(&skills_dir)? {
        list_skill_dirs(&skills_dir)?
    } else {
        Vec::new()
    };

    let tools_dir = path.join(TOOLS_DIR);
    let tools = if dir_exists(&tools_dir)? {
        list_tool_manifests(&tools_dir)?
    } else {
        BTreeMap::new()
    };

    let evals_dir = path.join(EVALS_DIR);
    let evals = if dir_exists(&evals_dir)? {
        list_eval_cases(&evals_dir)?
    } else {
        Vec::new()
    };

    let judges_dir = path.join(JUDGES_DIR);
    let judges = if dir_exists(&judges_dir)? {
        list_judge_manifests(&judges_dir)?
    } else {
        Vec::new()
    };

    let prompts_dir = path.join(PROMPTS_DIR);
    let prompts = if dir_exists(&prompts_dir)? {
        list_prompt_files(&prompts_dir)?
    } else {
        Vec::new()
    };

    Ok(Recipe {
        manifest,
        agents,
        agent_sources,
        skills,
        tools,
        evals,
        judges,
        prompts,
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

/// Best-effort source path for `name`, for anchoring `from:`-resolution
/// diagnostics. Falls back to the reconstructed `.yaml` path when the source
/// was not retained (should not happen for a `Recipe` produced by
/// [`load_recipe_dir`]).
fn agent_path_hint(recipe: &Recipe, name: &str) -> PathBuf {
    recipe
        .agent_sources
        .get(name)
        .cloned()
        .unwrap_or_else(|| PathBuf::from(AGENTS_DIR).join(format!("{name}.yaml")))
}

/// Resolve `name` against `recipe`, walking its `from:` chain (parent first,
/// nearest declaration wins) and merging field-wise per section 4.7:
///
/// - `harness`, `description`, `model` (and `model.name`, `model.max_tokens`,
///   `model.temperature` within it): child value wins when present, parent's
///   is inherited when the child omits it.
/// - `allow_delegation`: narrowing only, like `tools`/`mcp`. The child's own
///   declared value (or serde's `false` default, if omitted) always stands
///   as the resolved value - there is no `Option<bool>` "unset" state to
///   inherit through - but a child MUST NOT set it `true` when the resolved
///   parent's is `false`, else [`RecipeLoadError::FromWidensDelegation`]. A
///   child tightening `true` -> `false` is always legal.
/// - `tools`: child MUST be a subset of the resolved parent when both are
///   `Some`, else [`RecipeLoadError::FromWidensTools`]. A child that omits
///   `tools` entirely inherits the parent's value (whatever it is), which is
///   what keeps a restrictive parent from being silently widened back to
///   unrestricted by an omission.
/// - `subagents`: the child's own value always stands, so `subagents: []`
///   deliberately clears an inherited list.
/// - `system_instructions.content`: with `mode: append`, the child's content
///   appends to the parent's *resolved* content (post-inheritance, not the
///   parent's own literal YAML). `mode: replace` discards the parent's
///   instructions entirely. This is independent of `SYSTEM.md` composition,
///   which [`resolved_system`] handles separately.
///
/// A `from:` chain longer than two is legal; a cycle of any length,
/// including an agent naming itself, is rejected as
/// [`RecipeLoadError::FromCycle`]. A `from:` (or the initially requested
/// `name`) that does not resolve to a parsed `agents/*.yaml` entry is
/// rejected as [`RecipeLoadError::FromUnresolved`].
pub fn resolve_inherited(recipe: &Recipe, name: &str) -> Result<AgentManifest, RecipeLoadError> {
    let mut visited = HashSet::new();
    resolve_inherited_inner(recipe, name, &mut visited)
}

fn resolve_inherited_inner(
    recipe: &Recipe,
    name: &str,
    visited: &mut HashSet<String>,
) -> Result<AgentManifest, RecipeLoadError> {
    if !visited.insert(name.to_string()) {
        return Err(RecipeLoadError::FromCycle {
            name: name.to_string(),
            path: agent_path_hint(recipe, name),
        });
    }

    let child =
        recipe
            .agents
            .get(name)
            .cloned()
            .ok_or_else(|| RecipeLoadError::FromUnresolved {
                from: name.to_string(),
                path: agent_path_hint(recipe, name),
            })?;

    match &child.from {
        None => Ok(child),
        Some(parent_name) => {
            let parent = resolve_inherited_inner(recipe, parent_name, visited)?;
            merge_inherited(recipe, &parent, child)
        }
    }
}

/// Merge a resolved `parent` into an as-authored `child`, per the field
/// rules documented on [`resolve_inherited`]. `child.name` and `child.from`
/// are never overwritten: they are the child's own identity, not inherited
/// state.
fn merge_inherited(
    recipe: &Recipe,
    parent: &AgentManifest,
    child: AgentManifest,
) -> Result<AgentManifest, RecipeLoadError> {
    let harness = child.harness.or(parent.harness);

    // `description` is a plain scalar, merged exactly like `harness`: the
    // child's value wins when present, the parent's is inherited when the
    // child omits it.
    let description = child.description.or_else(|| parent.description.clone());

    let model = match (parent.model.clone(), child.model) {
        (parent_model, None) => parent_model,
        (None, Some(child_model)) => Some(child_model),
        (Some(parent_model), Some(child_model)) => Some(ModelSpec {
            name: child_model.name.or(parent_model.name),
            effort: child_model.effort,
            // `max_tokens` and `temperature` are declared-intent scalars,
            // merged exactly like `model.name`: the child's value wins when
            // present, the parent's is inherited when the child omits it.
            // Neither is a permission field, so there is no narrowing check
            // here - the narrowing this crate enforces for `max_tokens` is
            // the run-tier ceiling described on `ModelSpec::max_tokens`,
            // which is normative prose for consumers, not a rule this
            // loader (which never sees a run) can check.
            max_tokens: child_model.max_tokens.or(parent_model.max_tokens),
            temperature: child_model.temperature.or(parent_model.temperature),
        }),
    };

    let tools = match (parent.tools.clone(), child.tools) {
        (parent_tools, None) => parent_tools,
        (None, Some(child_tools)) => Some(child_tools),
        (Some(parent_tools), Some(child_tools)) => {
            let allowed: HashSet<&str> = parent_tools.iter().map(String::as_str).collect();
            let offending: Vec<String> = child_tools
                .iter()
                .filter(|t| !allowed.contains(t.as_str()))
                .cloned()
                .collect();
            if !offending.is_empty() {
                return Err(RecipeLoadError::FromWidensTools {
                    agent: child.name.clone(),
                    offending,
                    path: agent_path_hint(recipe, &child.name),
                });
            }
            Some(child_tools)
        }
    };

    let system_instructions = match (
        parent.system_instructions.clone(),
        child.system_instructions,
    ) {
        (parent_si, None) => parent_si,
        (None, Some(child_si)) => Some(child_si),
        (Some(parent_si), Some(child_si)) => match child_si.mode {
            InstructionMode::Append => Some(SystemInstructions {
                mode: child_si.mode,
                content: format!("{}\n\n{}", parent_si.content, child_si.content),
                harness_prompt: child_si.harness_prompt,
            }),
            InstructionMode::Replace => Some(child_si),
        },
    };

    // `mcp`: narrowing only, exactly like `tools` (section 7.3). When both
    // the child's and the resolved parent's `mcp` are present, the child's
    // policy MUST NOT widen the parent's at this one `from:` link, else
    // `RECIPE_MCP_FROM_WIDENS_POLICY`. When the child omits `mcp`, it
    // inherits the parent's resolved value unchanged. When the parent has no
    // `mcp` of its own (nothing in its chain declared one - deferred to the
    // package tier) and the child declares one, that is always a narrowing
    // and needs no check here.
    //
    // This per-link check does not by itself prevent every widening: it
    // catches only the immediate parent-to-child step. The package-tier
    // ceiling is enforced separately, once, in `validate::validate_recipe_dir`,
    // against this fully resolved value - so a widening cannot be laundered
    // past the package no matter how many `from:` links it passes through,
    // while a widening relative to an intermediate parent is now also
    // caught at the link where it actually happens.
    let mcp = match (parent.mcp.clone(), child.mcp) {
        (parent_mcp, None) => parent_mcp,
        (None, Some(child_mcp)) => Some(child_mcp),
        (Some(parent_mcp), Some(child_mcp)) => {
            let offending = mcp_policy_widenings(&parent_mcp, &child_mcp);
            if !offending.is_empty() {
                return Err(RecipeLoadError::FromWidensMcp {
                    agent: child.name.clone(),
                    offending,
                    path: agent_path_hint(recipe, &child.name),
                });
            }
            Some(child_mcp)
        }
    };

    // `allow_delegation`: narrowing only, exactly like `tools` and `mcp`
    // (section 4.4a, section 4.7). `allow_delegation` is a permission - the
    // ability to reach outside the recipe to a peer harness - not a mere
    // capability declaration, so it is subject to the same monotonic
    // narrowing rule despite being a plain `bool` rather than a collection.
    // A child MAY tighten (`parent: true` -> `child: false`); a child MUST
    // NOT widen (`parent: false` -> `child: true`), rejected as
    // `RECIPE_FROM_WIDENS_DELEGATION`. There is no `Option<bool>` "absent"
    // state to inherit through - the field is always concretely `true` or
    // `false` after parsing - so unlike `tools`/`mcp` this check does not
    // need an `Option` match on both sides; it compares the child's own
    // declared value directly against the resolved parent's.
    if child.allow_delegation && !parent.allow_delegation {
        return Err(RecipeLoadError::FromWidensDelegation {
            agent: child.name.clone(),
            path: agent_path_hint(recipe, &child.name),
        });
    }

    Ok(AgentManifest {
        name: child.name,
        description,
        harness,
        model,
        skills: child.skills,
        system_instructions,
        tools,
        mcp,
        subagents: child.subagents,
        allow_delegation: child.allow_delegation,
        from: child.from,
    })
}

fn read_to_string(path: &Path) -> Result<String, RecipeLoadError> {
    fs::read_to_string(path).map_err(|source| RecipeLoadError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Check whether `path` exists, surfacing a real I/O failure (e.g. permission
/// denied) as [`RecipeLoadError::Io`] instead of treating it as "absent".
///
/// Unlike [`Path::exists`], which collapses every I/O error to `false`, this
/// uses [`Path::try_exists`] so an unreadable recipe fails with `RECIPE_IO_ERROR`
/// per the spec rather than masquerading as a missing marker / missing file.
fn path_exists(path: &Path) -> Result<bool, RecipeLoadError> {
    path.try_exists().map_err(|source| RecipeLoadError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Whether `path` exists and is a directory. `Ok(false)` only for a definite
/// "not found"; any other I/O failure surfaces as [`RecipeLoadError::Io`].
fn dir_exists(path: &Path) -> Result<bool, RecipeLoadError> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(RecipeLoadError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// List `*.yaml`/`*.yml` files directly under `dir`, sorted by path for
/// deterministic loading order. A per-entry read failure is propagated as
/// [`RecipeLoadError::Io`] rather than silently dropped.
fn list_yaml_files(dir: &Path) -> Result<Vec<PathBuf>, RecipeLoadError> {
    let mut entries = Vec::new();
    for entry in read_dir(dir)? {
        let entry = entry.map_err(|source| RecipeLoadError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let is_yaml = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "yaml" || ext == "yml");
        let file_type = entry.file_type().map_err(|source| RecipeLoadError::Io {
            path: path.clone(),
            source,
        })?;
        if file_type.is_file() && is_yaml {
            entries.push(path);
        }
    }
    entries.sort();
    Ok(entries)
}

/// List the skill-package subdirectories directly under `dir`, sorted by path.
///
/// Only directories count as skill packages; loose files (such as the
/// generator's `.gitkeep`) are ignored. A per-entry read failure is propagated
/// as [`RecipeLoadError::Io`] rather than silently dropped.
fn list_skill_dirs(dir: &Path) -> Result<Vec<PathBuf>, RecipeLoadError> {
    let mut entries = Vec::new();
    for entry in read_dir(dir)? {
        let entry = entry.map_err(|source| RecipeLoadError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| RecipeLoadError::Io {
            path: entry.path(),
            source,
        })?;
        if file_type.is_dir() {
            entries.push(entry.path());
        }
    }
    entries.sort();
    Ok(entries)
}

/// Parse every `tools/<ref>/tool.yaml` directly under `dir`, keyed by the
/// `<ref>` directory name. A subdirectory with no `tool.yaml` is ignored
/// (not every directory under `tools/` need be a tool package); one that
/// has one but fails to parse is [`RecipeLoadError::MalformedTool`].
fn list_tool_manifests(dir: &Path) -> Result<BTreeMap<String, ToolManifest>, RecipeLoadError> {
    let mut manifests = BTreeMap::new();
    for entry in read_dir(dir)? {
        let entry = entry.map_err(|source| RecipeLoadError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| RecipeLoadError::Io {
            path: entry.path(),
            source,
        })?;
        if !file_type.is_dir() {
            continue;
        }
        let tool_ref = entry
            .file_name()
            .to_str()
            .map(str::to_string)
            .unwrap_or_default();
        let manifest_path = entry.path().join(TOOL_MANIFEST_FILE);
        if !path_exists(&manifest_path)? {
            continue;
        }
        let content = read_to_string(&manifest_path)?;
        let manifest: ToolManifest =
            serde_yaml::from_str(&content).map_err(|source| RecipeLoadError::MalformedTool {
                path: manifest_path.clone(),
                source,
            })?;
        manifests.insert(tool_ref, manifest);
    }
    Ok(manifests)
}

/// List the eval cases directly under `dir` (`evals/`), sorted by case name.
///
/// Every subdirectory under `evals/` is treated as an eval case, unlike
/// `tools/` (where a subdirectory with no `tool.yaml` is simply ignored):
/// section 9's "MUST be reported, not silently skipped" ruling means a case
/// directory missing `input.json` or `expected.md` fails the whole load as
/// [`RecipeLoadError::MalformedEvalCase`] rather than being dropped from the
/// gathered set.
fn list_eval_cases(dir: &Path) -> Result<Vec<EvalCase>, RecipeLoadError> {
    let mut case_dirs = Vec::new();
    for entry in read_dir(dir)? {
        let entry = entry.map_err(|source| RecipeLoadError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| RecipeLoadError::Io {
            path: entry.path(),
            source,
        })?;
        if !file_type.is_dir() {
            continue;
        }
        let name = entry
            .file_name()
            .to_str()
            .map(str::to_string)
            .unwrap_or_default();
        case_dirs.push((name, entry.path()));
    }
    case_dirs.sort();

    let mut cases = Vec::with_capacity(case_dirs.len());
    for (name, case_dir) in case_dirs {
        let input_path = case_dir.join(EVAL_INPUT_FILE);
        let expected_path = case_dir.join(EVAL_EXPECTED_FILE);
        if !path_exists(&input_path)? {
            return Err(RecipeLoadError::MalformedEvalCase {
                name,
                missing: EVAL_INPUT_FILE,
                path: case_dir,
            });
        }
        if !path_exists(&expected_path)? {
            return Err(RecipeLoadError::MalformedEvalCase {
                name,
                missing: EVAL_EXPECTED_FILE,
                path: case_dir,
            });
        }
        cases.push(EvalCase {
            name,
            input_path,
            expected_path,
        });
    }
    Ok(cases)
}

/// Parse every `judges/*.yaml` directly under `dir`, sorted by source file
/// path for deterministic order (mirroring [`list_yaml_files`], which this
/// reuses).
fn list_judge_manifests(dir: &Path) -> Result<Vec<JudgeManifest>, RecipeLoadError> {
    let mut manifests = Vec::new();
    for entry_path in list_yaml_files(dir)? {
        let content = read_to_string(&entry_path)?;
        let mut judge: JudgeManifest =
            serde_yaml::from_str(&content).map_err(|source| RecipeLoadError::MalformedJudge {
                path: entry_path.clone(),
                source,
            })?;
        judge.source_path = entry_path;
        manifests.push(judge);
    }
    Ok(manifests)
}

/// List `*.md` files directly under `dir` (`prompts/`), sorted by path for
/// deterministic order.
fn list_prompt_files(dir: &Path) -> Result<Vec<PathBuf>, RecipeLoadError> {
    let mut entries = Vec::new();
    for entry in read_dir(dir)? {
        let entry = entry.map_err(|source| RecipeLoadError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let is_md = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "md");
        let file_type = entry.file_type().map_err(|source| RecipeLoadError::Io {
            path: path.clone(),
            source,
        })?;
        if file_type.is_file() && is_md {
            entries.push(path);
        }
    }
    entries.sort();
    Ok(entries)
}

/// Open `dir` for reading, mapping the failure to [`RecipeLoadError::Io`].
fn read_dir(dir: &Path) -> Result<fs::ReadDir, RecipeLoadError> {
    fs::read_dir(dir).map_err(|source| RecipeLoadError::Io {
        path: dir.to_path_buf(),
        source,
    })
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
harness: claude
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
        assert_eq!(agent.harness, Some(HarnessKind::Claude));
        assert_eq!(
            agent.model.as_ref().and_then(|m| m.name.as_deref()),
            Some("anthropic/claude-opus-4-8")
        );
        assert_eq!(agent.model.as_ref().unwrap().effort, EffortLevel::High);
        assert_eq!(
            agent.skills,
            vec![SkillRef::Bare("code-review".to_string())]
        );
        assert_eq!(agent.skills[0].name(), "code-review");
        assert_eq!(agent.tools, Some(vec!["shell".to_string()]));
        assert_eq!(agent.subagents, vec!["reviewer".to_string()]);
        assert!(agent.system_instructions.is_some());
    }

    #[test]
    fn skill_ref_accepts_bare_string_or_pinned_object() {
        let bare: SkillRef = serde_yaml::from_str("research").expect("bare string should parse");
        assert_eq!(bare.name(), "research");
        assert_eq!(bare, SkillRef::Bare("research".to_string()));

        let pinned: SkillRef = serde_yaml::from_str(
            "ref: research\nsource_url: https://example.com/s.git\nversion: 1.2.0\nresolved_sha: abc123\n",
        )
        .expect("pinned object should parse");
        assert_eq!(pinned.name(), "research");
        match &pinned {
            SkillRef::Pinned(p) => {
                assert_eq!(p.r#ref, "research");
                assert_eq!(p.source_url.as_deref(), Some("https://example.com/s.git"));
                assert_eq!(p.version.as_deref(), Some("1.2.0"));
                assert_eq!(p.resolved_sha.as_deref(), Some("abc123"));
            }
            SkillRef::Bare(_) => panic!("expected a pinned SkillRef"),
        }
    }

    #[test]
    fn pinned_skill_ref_only_requires_ref() {
        let pinned: SkillRef =
            serde_yaml::from_str("ref: research\n").expect("ref-only object should parse");
        assert_eq!(pinned.name(), "research");
    }

    #[test]
    fn pinned_skill_ref_typo_names_the_unknown_field_not_a_generic_variant_failure() {
        // A manual Deserialize dispatches a mapping straight into
        // PinnedSkillRef, so a typo like `versoin:` surfaces serde's own
        // field-level error rather than the untagged-enum fallback's
        // generic "data did not match any variant" message.
        let result: Result<SkillRef, _> = serde_yaml::from_str("ref: research\nversoin: 1.2.0\n");
        let err = result.expect_err("typo'd field must fail to parse");
        let message = err.to_string();
        assert!(
            message.contains("versoin"),
            "error should name the unknown field 'versoin', got: {message}"
        );
        assert!(
            !message.contains("did not match any variant"),
            "error should not fall back to the generic untagged-enum message, got: {message}"
        );
    }

    #[test]
    fn pinned_skill_ref_rejects_unknown_fields() {
        let result: Result<SkillRef, _> = serde_yaml::from_str("ref: research\nbogus: nope\n");
        assert!(
            result.is_err(),
            "unknown field on pinned object must fail to parse"
        );
    }

    #[test]
    fn agent_manifest_defaults_optional_collections() {
        let yaml =
            "name: minimal\nharness: codex\nmodel:\n  name: openai/gpt-5-codex\n  effort: low\n";
        let agent: AgentManifest = serde_yaml::from_str(yaml).expect("should parse");
        assert!(agent.skills.is_empty());
        assert!(agent.tools.is_none());
        assert!(agent.subagents.is_empty());
        assert!(agent.system_instructions.is_none());
    }

    #[test]
    fn absent_tools_means_unrestricted_empty_means_none() {
        // `tools` absent and `tools: []` are deliberately distinct states:
        // absent places no restriction, empty permits no tools at all.
        let absent: AgentManifest =
            serde_yaml::from_str("name: a\nmodel:\n  name: m\n  effort: low\n").unwrap();
        assert_eq!(absent.tools, None, "absent must not collapse to empty");

        let empty: AgentManifest =
            serde_yaml::from_str("name: a\nmodel:\n  name: m\n  effort: low\ntools: []\n").unwrap();
        assert_eq!(empty.tools, Some(vec![]), "empty means no tools permitted");
    }

    #[test]
    fn agent_manifest_defaults_harness_to_none() {
        let yaml = "name: minimal\nmodel:\n  name: openai/gpt-5-codex\n  effort: low\n";
        let agent: AgentManifest = serde_yaml::from_str(yaml).expect("should parse");
        assert_eq!(agent.harness, None);
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
        let yaml =
            "name: x\nharness: claude\nmodel:\n  name: foo\n  effort: low\nnotarealfield: 1\n";
        let result: Result<AgentManifest, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err(), "expected unknown field to be rejected");
    }

    #[test]
    fn judge_manifest_parses_with_inline_prompt() {
        let yaml = "name: correctness\nprompt: Judge correctness.\n";
        let judge: JudgeManifest = serde_yaml::from_str(yaml).expect("should parse");
        assert_eq!(judge.name, "correctness");
        assert_eq!(judge.prompt.as_deref(), Some("Judge correctness."));
        assert!(judge.prompt_file.is_none());
    }

    #[test]
    fn judge_manifest_parses_with_prompt_file() {
        let yaml = "name: security\nprompt_file: security-bar.md\n";
        let judge: JudgeManifest = serde_yaml::from_str(yaml).expect("should parse");
        assert!(judge.prompt.is_none());
        assert_eq!(judge.prompt_file.as_deref(), Some("security-bar.md"));
    }

    #[test]
    fn judge_manifest_rejects_unknown_field() {
        let yaml = "name: x\nprompt: y\nnotarealfield: 1\n";
        let result: Result<JudgeManifest, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err(), "expected unknown field to be rejected");
    }

    #[test]
    fn agent_manifest_rejects_non_string_key() {
        let yaml = "name: x\nharness: claude\nmodel:\n  name: foo\n  effort: low\n1: stray\n";
        let result: Result<AgentManifest, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err(), "expected non-string key to be rejected");
    }

    #[test]
    fn agent_manifest_rejects_unrecognized_harness() {
        let yaml = "name: x\nharness: gemini\nmodel:\n  name: foo\n  effort: low\n";
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
        assert_eq!(error.code(), error_codes::RECIPE_MALFORMED_HARNESS_YAML);
    }

    #[test]
    fn load_recipe_dir_without_optional_dirs_still_loads() {
        let recipe = load_recipe_dir(&fixtures_dir().join("minimal-recipe"))
            .expect("minimal fixture (no skills/, no SYSTEM.md) should load");
        assert!(recipe.system_md.is_none());
        assert!(recipe.skills.is_empty());
    }

    #[test]
    fn load_recipe_dir_resolves_yml_default_agent() {
        // A `default_agent` MUST resolve to a `.yml` agent file, not only
        // `.yaml`, and the retained source path keeps the real extension.
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();
        fs::write(
            root.join(RECIPE_MARKER_FILE),
            "name: yml-recipe\nversion: 0.1.0\ndefault_agent: main\n",
        )
        .unwrap();
        fs::create_dir_all(root.join(AGENTS_DIR)).unwrap();
        fs::write(
            root.join(AGENTS_DIR).join("main.yml"),
            "name: main\nharness: claude\nmodel:\n  name: anthropic/claude-opus-4-8\n  effort: high\n",
        )
        .unwrap();

        let recipe = load_recipe_dir(root).expect("recipe with a .yml default agent should load");
        assert!(recipe.default_agent().is_some());
        assert_eq!(
            recipe.agent_sources.get("main").map(|p| p.as_path()),
            Some(root.join(AGENTS_DIR).join("main.yml").as_path())
        );
    }

    #[test]
    fn load_recipe_dir_ignores_loose_files_in_skills() {
        // Only skill-package directories count; a `.gitkeep` file is not a skill.
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();
        fs::write(
            root.join(RECIPE_MARKER_FILE),
            "name: r\nversion: 0.1.0\ndefault_agent: main\n",
        )
        .unwrap();
        fs::create_dir_all(root.join(AGENTS_DIR)).unwrap();
        fs::write(
            root.join(AGENTS_DIR).join("main.yaml"),
            "name: main\nharness: claude\nmodel:\n  name: m\n  effort: low\n",
        )
        .unwrap();
        fs::create_dir_all(root.join(SKILLS_DIR)).unwrap();
        fs::write(root.join(SKILLS_DIR).join(".gitkeep"), "").unwrap();
        fs::create_dir_all(root.join(SKILLS_DIR).join("code-review")).unwrap();

        let recipe = load_recipe_dir(root).expect("should load");
        assert_eq!(recipe.skills.len(), 1, "only the directory should count");
        assert!(recipe.skills[0].ends_with("code-review"));
    }

    #[test]
    fn load_recipe_dir_rejects_duplicate_agent_stem() {
        // agents/main.yaml and agents/main.yml both resolve to stem `main`.
        let error = load_recipe_dir(&fixtures_dir().join("duplicate-agent"))
            .expect_err("colliding agent stems should fail to load");
        assert!(matches!(error, RecipeLoadError::DuplicateAgent { .. }));
        assert_eq!(error.code(), error_codes::RECIPE_DUPLICATE_AGENT);
    }

    // ─── resolved_system ────────────────────────────────────────────────────

    fn agent_with_instructions(mode: InstructionMode, content: &str) -> AgentManifest {
        AgentManifest {
            name: "main".to_string(),
            description: None,
            harness: Some(HarnessKind::Claude),
            model: Some(ModelSpec {
                name: Some("anthropic/claude-opus-4-8".to_string()),
                effort: EffortLevel::High,
                max_tokens: None,
                temperature: None,
            }),
            skills: Vec::new(),
            system_instructions: Some(SystemInstructions {
                mode,
                content: content.to_string(),
                harness_prompt: HarnessPromptMode::default(),
            }),
            tools: None,
            mcp: None,
            subagents: Vec::new(),
            allow_delegation: false,
            from: None,
        }
    }

    fn agent_without_instructions() -> AgentManifest {
        AgentManifest {
            name: "main".to_string(),
            description: None,
            harness: Some(HarnessKind::Claude),
            model: Some(ModelSpec {
                name: Some("anthropic/claude-opus-4-8".to_string()),
                effort: EffortLevel::High,
                max_tokens: None,
                temperature: None,
            }),
            skills: Vec::new(),
            system_instructions: None,
            tools: None,
            mcp: None,
            subagents: Vec::new(),
            allow_delegation: false,
            from: None,
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

    // ─── resolve_inherited: system_instructions merge ─────────────────────
    //
    // These are deliberately kept separate from `resolved_system`, which
    // composes with `SYSTEM.md` (a different axis, section 6). Everything
    // here is about the agent-tier `from:` merge alone (section 4.7).

    fn agent_with_from(name: &str, from: Option<&str>) -> AgentManifest {
        AgentManifest {
            name: name.to_string(),
            description: None,
            harness: Some(HarnessKind::Claude),
            model: None,
            skills: Vec::new(),
            system_instructions: None,
            tools: None,
            mcp: None,
            subagents: Vec::new(),
            allow_delegation: false,
            from: from.map(str::to_string),
        }
    }

    fn recipe_of(agents: Vec<AgentManifest>) -> Recipe {
        let mut map = BTreeMap::new();
        for agent in agents {
            map.insert(agent.name.clone(), agent);
        }
        Recipe {
            manifest: RecipeManifest {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                default_agent: "child".to_string(),
                mcp: McpPolicy::default(),
            },
            agents: map,
            agent_sources: BTreeMap::new(),
            skills: Vec::new(),
            tools: BTreeMap::new(),
            evals: Vec::new(),
            judges: Vec::new(),
            prompts: Vec::new(),
            system_md: None,
        }
    }

    #[test]
    fn from_inherited_system_instructions_append_composes_resolved_parent_across_chain() {
        // grandparent -> parent -> child, each `mode: append`. If a
        // regression swapped the parent's *resolved* content for its raw,
        // un-inherited content, `parent`'s own text ("P") would appear
        // without the grandparent's ("G") ever showing up in `child`'s
        // resolved content, and this exact-match assertion would catch it.
        let mut grandparent = agent_with_from("grandparent", None);
        grandparent.system_instructions = Some(SystemInstructions {
            mode: InstructionMode::Append,
            content: "G".to_string(),
            harness_prompt: HarnessPromptMode::default(),
        });

        let mut parent = agent_with_from("parent", Some("grandparent"));
        parent.system_instructions = Some(SystemInstructions {
            mode: InstructionMode::Append,
            content: "P".to_string(),
            harness_prompt: HarnessPromptMode::default(),
        });

        let mut child = agent_with_from("child", Some("parent"));
        child.system_instructions = Some(SystemInstructions {
            mode: InstructionMode::Append,
            content: "C".to_string(),
            harness_prompt: HarnessPromptMode::default(),
        });

        let recipe = recipe_of(vec![grandparent, parent, child]);
        let resolved = resolve_inherited(&recipe, "child").expect("chain should resolve");
        let content = resolved
            .system_instructions
            .expect("system_instructions should be present")
            .content;

        assert_eq!(content, "G\n\nP\n\nC");
    }

    #[test]
    fn from_inherited_system_instructions_replace_discards_parent_content() {
        let mut parent = agent_with_from("parent", None);
        parent.system_instructions = Some(SystemInstructions {
            mode: InstructionMode::Append,
            content: "PARENT".to_string(),
            harness_prompt: HarnessPromptMode::default(),
        });

        let mut child = agent_with_from("child", Some("parent"));
        child.system_instructions = Some(SystemInstructions {
            mode: InstructionMode::Replace,
            content: "CHILD".to_string(),
            harness_prompt: HarnessPromptMode::default(),
        });

        let recipe = recipe_of(vec![parent, child]);
        let resolved = resolve_inherited(&recipe, "child").expect("chain should resolve");
        let content = resolved
            .system_instructions
            .expect("system_instructions should be present")
            .content;

        // `mode: replace` must discard the parent's content outright, not
        // just place the child's content first. If both modes produced the
        // same composed string, this assertion (and the append test above
        // producing a strictly longer, prefixed string) would diverge.
        assert_eq!(content, "CHILD");
        assert!(!content.contains("PARENT"));
    }

    // ─── mcp policy (section 7) ─────────────────────────────────────────────

    fn server_policy(include: &[&str], exclude: &[&str]) -> McpServerPolicy {
        McpServerPolicy {
            include: include.iter().map(|s| s.to_string()).collect(),
            exclude: exclude.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn policy_of(servers: &[(&str, McpServerPolicy)]) -> McpPolicy {
        McpPolicy {
            servers: servers
                .iter()
                .map(|(id, policy)| (id.to_string(), policy.clone()))
                .collect(),
        }
    }

    #[test]
    fn empty_include_permits_no_methods() {
        let policy = server_policy(&[], &["run_query"]);
        assert!(policy.effective_methods().is_empty());
    }

    #[test]
    fn exclude_removes_from_include() {
        let policy = server_policy(&["run_query", "drop_table"], &["drop_table"]);
        let methods = policy.effective_methods();
        assert!(methods.contains("run_query"));
        assert!(!methods.contains("drop_table"));
    }

    #[test]
    fn agent_naming_server_absent_from_package_is_a_widening() {
        // Subtle case 1: the naive "compare only servers present in both"
        // check would miss this, since `reporting` never appears in
        // `package` at all.
        let package = policy_of(&[("warehouse", server_policy(&["run_query"], &[]))]);
        let agent = policy_of(&[("reporting", server_policy(&["list_reports"], &[]))]);
        assert_eq!(mcp_policy_widenings(&package, &agent), vec!["reporting"]);
    }

    #[test]
    fn agent_widening_methods_on_a_shared_server_is_flagged() {
        let package = policy_of(&[("warehouse", server_policy(&["run_query"], &[]))]);
        let agent = policy_of(&[(
            "warehouse",
            server_policy(&["run_query", "drop_table"], &[]),
        )]);
        assert_eq!(mcp_policy_widenings(&package, &agent), vec!["warehouse"]);
    }

    #[test]
    fn agent_narrowing_methods_on_a_shared_server_is_legal() {
        let package = policy_of(&[(
            "warehouse",
            server_policy(&["run_query", "drop_table"], &[]),
        )]);
        let agent = policy_of(&[("warehouse", server_policy(&["run_query"], &[]))]);
        assert!(mcp_policy_widenings(&package, &agent).is_empty());
    }

    #[test]
    fn agent_removing_a_package_exclusion_is_a_widening() {
        // Subtle case 3: the package excludes `drop_table` even though it is
        // in `include`; an agent that repeats the same `include` but drops
        // the `exclude` restores access the package explicitly withheld.
        let package = policy_of(&[(
            "warehouse",
            server_policy(&["run_query", "drop_table"], &["drop_table"]),
        )]);
        let agent = policy_of(&[(
            "warehouse",
            server_policy(&["run_query", "drop_table"], &[]),
        )]);
        assert_eq!(mcp_policy_widenings(&package, &agent), vec!["warehouse"]);
    }

    #[test]
    fn agent_adding_its_own_exclusion_is_always_legal() {
        let package = policy_of(&[(
            "warehouse",
            server_policy(&["run_query", "drop_table"], &[]),
        )]);
        let agent = policy_of(&[(
            "warehouse",
            server_policy(&["run_query", "drop_table"], &["drop_table"]),
        )]);
        assert!(mcp_policy_widenings(&package, &agent).is_empty());
    }

    #[test]
    fn merge_inherited_lets_child_mcp_narrow_parent() {
        let mut parent = agent_with_from("parent", None);
        parent.mcp = Some(policy_of(&[(
            "warehouse",
            server_policy(&["run_query", "drop_table"], &[]),
        )]));

        let mut child = agent_with_from("child", Some("parent"));
        child.mcp = Some(policy_of(&[(
            "warehouse",
            server_policy(&["run_query"], &[]),
        )]));

        let recipe = recipe_of(vec![parent, child]);
        let resolved = resolve_inherited(&recipe, "child").expect("narrowing should resolve");
        let resolved_mcp = resolved.mcp.expect("child declared its own mcp");
        let methods = resolved_mcp.servers["warehouse"].effective_methods();
        assert!(methods.contains("run_query"));
        assert!(!methods.contains("drop_table"));
    }

    #[test]
    fn merge_inherited_lets_child_inherit_parent_mcp_when_omitted() {
        let mut parent = agent_with_from("parent", None);
        parent.mcp = Some(policy_of(&[(
            "warehouse",
            server_policy(&["run_query"], &[]),
        )]));

        let child = agent_with_from("child", Some("parent"));

        let recipe = recipe_of(vec![parent, child]);
        let resolved = resolve_inherited(&recipe, "child").expect("chain should resolve");
        let resolved_mcp = resolved.mcp.expect("child should inherit parent's mcp");
        assert!(resolved_mcp.servers.contains_key("warehouse"));
    }

    #[test]
    fn merge_inherited_lets_child_declare_mcp_when_parent_has_none() {
        // The parent's own `mcp` is unset (deferred to the package tier, or
        // to a further ancestor); a child declaring its own is always a
        // narrowing relative to "unset", so no per-link check applies here.
        let parent = agent_with_from("parent", None);
        let mut child = agent_with_from("child", Some("parent"));
        child.mcp = Some(policy_of(&[(
            "warehouse",
            server_policy(&["run_query"], &[]),
        )]));

        let recipe = recipe_of(vec![parent, child]);
        let resolved = resolve_inherited(&recipe, "child").expect("chain should resolve");
        assert!(
            resolved
                .mcp
                .expect("child's own mcp should stand")
                .servers
                .contains_key("warehouse")
        );
    }

    #[test]
    fn merge_inherited_rejects_child_mcp_that_widens_the_immediate_parent() {
        // section 7.3: the per-`from:`-link check. `child` names a server
        // `parent` never mentions, which is a widening at this link
        // regardless of what the package tier would separately permit.
        let mut parent = agent_with_from("parent", None);
        parent.mcp = Some(policy_of(&[(
            "warehouse",
            server_policy(&["run_query"], &[]),
        )]));

        let mut child = agent_with_from("child", Some("parent"));
        child.mcp = Some(policy_of(&[(
            "reporting",
            server_policy(&["list_reports"], &[]),
        )]));

        let recipe = recipe_of(vec![parent, child]);
        let error = resolve_inherited(&recipe, "child").expect_err("should reject the widening");
        assert!(matches!(error, RecipeLoadError::FromWidensMcp { .. }));
        assert_eq!(error.code(), error_codes::RECIPE_MCP_FROM_WIDENS_POLICY);
    }

    #[test]
    fn merge_inherited_rejects_widening_at_a_non_adjacent_link_in_a_three_level_chain() {
        // grandparent -> parent narrows correctly; parent -> child widens
        // back open. The failure must be attributed to the child (the link
        // where it actually happens), not silently accepted because the
        // grandparent's original value happens to cover it.
        let mut grandparent = agent_with_from("grandparent", None);
        grandparent.mcp = Some(policy_of(&[(
            "warehouse",
            server_policy(&["run_query", "drop_table"], &[]),
        )]));

        let mut parent = agent_with_from("parent", Some("grandparent"));
        parent.mcp = Some(policy_of(&[(
            "warehouse",
            server_policy(&["run_query"], &[]),
        )]));

        let mut child = agent_with_from("child", Some("parent"));
        child.mcp = Some(policy_of(&[(
            "warehouse",
            server_policy(&["run_query", "drop_table"], &[]),
        )]));

        let recipe = recipe_of(vec![grandparent, parent, child]);
        let error = resolve_inherited(&recipe, "child").expect_err("should reject the widening");
        match error {
            RecipeLoadError::FromWidensMcp { agent, .. } => assert_eq!(agent, "child"),
            other => panic!("expected FromWidensMcp, got {other:?}"),
        }
    }
}
