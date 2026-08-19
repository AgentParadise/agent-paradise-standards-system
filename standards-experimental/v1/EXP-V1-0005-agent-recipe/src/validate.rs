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
use std::collections::HashSet;
use std::path::Path;

use crate::schema::{
    self, AGENTS_DIR, AgentManifest, HarnessKind, McpPolicy, RECIPE_MARKER_FILE, Recipe,
    RecipeLoadError, SkillRef, load_recipe_dir, mcp_policy_widenings, resolve_inherited,
};

/// Directory-level validation error codes, layered on top of the loader codes
/// in [`crate::schema::error_codes`].
pub mod error_codes {
    /// A `subagents` entry names an agent with no matching `agents/*.yaml`.
    pub const RECIPE_SUBAGENT_UNRESOLVED: &str = "RECIPE_SUBAGENT_UNRESOLVED";
    /// A named subagent's resolved `tools` is not a subset of the delegating
    /// agent's resolved `tools` (section 4.4a).
    pub const RECIPE_SUBAGENT_WIDENS_TOOLS: &str = "RECIPE_SUBAGENT_WIDENS_TOOLS";
    /// A named subagent's resolved `mcp` is not a subset of the delegating
    /// agent's resolved `mcp` (section 4.4a).
    pub const RECIPE_SUBAGENT_WIDENS_MCP: &str = "RECIPE_SUBAGENT_WIDENS_MCP";
    /// `recipe.yaml`'s `name` is present (serde) but empty.
    pub const RECIPE_EMPTY_RECIPE_NAME: &str = "RECIPE_EMPTY_RECIPE_NAME";
    /// An agent manifest's `name` is present (serde) but empty.
    pub const RECIPE_EMPTY_AGENT_NAME: &str = "RECIPE_EMPTY_AGENT_NAME";
    /// An agent manifest's `model.name` is present (serde) but empty.
    pub const RECIPE_EMPTY_MODEL_NAME: &str = "RECIPE_EMPTY_MODEL_NAME";
    /// A `skills` entry is an empty string (bare form) or has an empty
    /// `ref` (pinned form).
    pub const RECIPE_INVALID_SKILL_REF: &str = "RECIPE_INVALID_SKILL_REF";
    /// A pinned `skills` entry's `version` names `latest`/`@latest`
    /// (case-insensitively) or is empty/whitespace. A recipe's `evals/`
    /// and `judges/` (section 9) are only a meaningful, attributable
    /// definition of good if the recipe's inputs are reproducible; an
    /// unpinned skill breaks that guarantee.
    pub const RECIPE_SKILL_UNPINNED: &str = "RECIPE_SKILL_UNPINNED";
    /// A `tools` entry is an empty string.
    pub const RECIPE_INVALID_TOOL_REF: &str = "RECIPE_INVALID_TOOL_REF";
    /// `system_instructions.content` is present (serde) but empty.
    pub const RECIPE_EMPTY_INSTRUCTIONS_CONTENT: &str = "RECIPE_EMPTY_INSTRUCTIONS_CONTENT";
    /// An agent with no declared `harness` (harness-agnostic) lists a `tools`
    /// entry that is harness-builtin under some harness and does not resolve
    /// as a recipe-provided tool.
    pub const RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL: &str =
        "RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL";
    /// An agent's (fully `from:`-resolved) `mcp` policy is not a subset of
    /// the package's `mcp` policy: it names a server the package does not
    /// mention, or permits a method for a shared server the package does
    /// not. See `schema::mcp_policy_widenings` and section 7.
    pub const RECIPE_MCP_AGENT_WIDENS_POLICY: &str = "RECIPE_MCP_AGENT_WIDENS_POLICY";
    /// A `tools/<ref>/tool.yaml` parsed successfully (well-formed YAML) but
    /// has an empty `name` or an empty `command`. This standard does NOT
    /// validate that `command` exists on disk or is executable (section
    /// 5.2): a recipe is a portable artifact that may be validated on a
    /// machine that will never run it.
    pub const RECIPE_INVALID_TOOL_MANIFEST: &str = "RECIPE_INVALID_TOOL_MANIFEST";
    /// A `judges/*.yaml` parsed successfully (well-formed YAML) but has an
    /// empty `name`, or declares neither `prompt` nor `prompt_file`. This
    /// standard does NOT validate that a `prompt_file` reference resolves
    /// under `prompts/` (section 9): the declaration is what this standard
    /// governs, not the runtime that reads it.
    pub const RECIPE_INVALID_JUDGE_MANIFEST: &str = "RECIPE_INVALID_JUDGE_MANIFEST";
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

/// Whether `tool` resolves as a recipe-provided tool: `tools/<tool>/tool.yaml`
/// was found and parsed at load time (section 5.2).
///
/// This is the precedence rule for a name that is ambiguous between a
/// harness builtin and a `tools/` entry: recipe-provided wins, because the
/// recipe ships the implementation and therefore knows what the name means.
fn resolves_as_recipe_provided(recipe: &Recipe, tool: &str) -> bool {
    recipe.tools.contains_key(tool)
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
        RecipeLoadError::FromWidensMcp { offending, .. } => diagnostic.with_hint(format!(
            "narrow mcp.servers {offending:?} to a subset of the parent's mcp.servers, or grant the missing server(s)/methods in the parent's mcp"
        )),
        RecipeLoadError::FromWidensDelegation { .. } => diagnostic.with_hint(
            "set allow_delegation: false on this agent, or set allow_delegation: true on the parent"
                .to_string(),
        ),
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

    // Every gathered `tools/<ref>/tool.yaml` MUST have a non-empty `name`
    // and `command`, regardless of whether any agent currently references
    // it (mirrors `RECIPE_EMPTY_AGENT_NAME` validating every agent, not
    // only the default one). This standard does NOT check that `command`
    // exists on disk or is executable: see
    // `error_codes::RECIPE_INVALID_TOOL_MANIFEST`.
    for (tool_ref, tool) in &recipe.tools {
        let tool_path = root
            .join(schema::TOOLS_DIR)
            .join(tool_ref)
            .join(schema::TOOL_MANIFEST_FILE);
        if tool.name.trim().is_empty() {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::RECIPE_INVALID_TOOL_MANIFEST,
                    format!("tool '{tool_ref}' has an empty name"),
                )
                .with_path(tool_path.clone()),
            );
        }
        if tool.command.trim().is_empty() {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::RECIPE_INVALID_TOOL_MANIFEST,
                    format!("tool '{tool_ref}' has an empty command"),
                )
                .with_path(tool_path),
            );
        }
    }

    // Every gathered `judges/*.yaml` MUST have a non-empty `name` and at
    // least one of `prompt`/`prompt_file` (section 9). `Recipe::judges` is a
    // flat `Vec`, so each `JudgeManifest` carries its own `source_path`
    // (loader-populated provenance, not part of the on-disk schema) to
    // anchor these diagnostics at the offending file - the empty-name case
    // has no other identifying field, so without this an author with
    // several judges would have no way to know which file to open.
    for judge in &recipe.judges {
        if judge.name.trim().is_empty() {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::RECIPE_INVALID_JUDGE_MANIFEST,
                    "judge has an empty name",
                )
                .with_path(judge.source_path.clone()),
            );
        }
        if judge.prompt.is_none() && judge.prompt_file.is_none() {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::RECIPE_INVALID_JUDGE_MANIFEST,
                    format!(
                        "judge '{}' declares neither prompt nor prompt_file",
                        judge.name
                    ),
                )
                .with_path(judge.source_path.clone())
                .with_hint(format!(
                    "add prompt or prompt_file to judge '{}'",
                    judge.name
                )),
            );
        }
    }

    // `from:` resolution is checked separately, over the agents that
    // actually declare it: a cycle can only occur through a chain of `from`
    // links, so an agent with no `from` cannot be the site of one, and
    // resolving it would be a no-op. `resolve_inherited` walks the whole
    // chain, so this also exercises chains longer than two.
    //
    // The package-vs-agent `mcp` check (`RECIPE_MCP_AGENT_WIDENS_POLICY`,
    // section 7) is folded into this same loop: for an agent that declares
    // `from:`, it MUST run against the fully resolved `mcp` (post-`from:`
    // merge), never the as-authored value, so a widening cannot be
    // laundered through an intermediate parent. For an agent with no
    // `from:`, the as-authored value already IS the fully resolved value.
    for (stem, agent) in &recipe.agents {
        let agent_path = agent_source_path(root, recipe, stem);
        if agent.from.is_some() {
            match resolve_inherited(recipe, stem) {
                Ok(resolved) => {
                    check_mcp_policy_widening(
                        &recipe.manifest.mcp,
                        resolved.mcp.as_ref(),
                        stem,
                        &agent_path,
                        diagnostics,
                    );
                }
                Err(error) => diagnostics.push(diagnostic_from_load_error(&error)),
            }
        } else {
            check_mcp_policy_widening(
                &recipe.manifest.mcp,
                agent.mcp.as_ref(),
                stem,
                &agent_path,
                diagnostics,
            );
        }
    }
}

/// Best-effort source path for `stem`, for anchoring a diagnostic. Falls
/// back to the reconstructed `.yaml` path when the source was not retained
/// (should not happen for a `Recipe` produced by `load_recipe_dir`).
fn agent_source_path(root: &Path, recipe: &Recipe, stem: &str) -> std::path::PathBuf {
    recipe
        .agent_sources
        .get(stem)
        .cloned()
        .unwrap_or_else(|| root.join(AGENTS_DIR).join(format!("{stem}.yaml")))
}

/// Check `agent_mcp` (an agent's fully resolved `mcp` policy, if any)
/// against `package` (the recipe's package-tier `mcp` policy), reporting
/// `RECIPE_MCP_AGENT_WIDENS_POLICY` for every server the agent widens.
///
/// `agent_mcp: None` means the agent declares no `mcp` restriction of its
/// own; its effective policy is exactly the package's, which trivially
/// cannot widen it, so this is a no-op in that case.
fn check_mcp_policy_widening(
    package: &McpPolicy,
    agent_mcp: Option<&McpPolicy>,
    stem: &str,
    agent_path: &Path,
    diagnostics: &mut Diagnostics,
) {
    let Some(agent_policy) = agent_mcp else {
        return;
    };
    let offending = mcp_policy_widenings(package, agent_policy);
    if !offending.is_empty() {
        diagnostics.push(
            Diagnostic::error(
                error_codes::RECIPE_MCP_AGENT_WIDENS_POLICY,
                format!(
                    "agent '{stem}' mcp policy widens the package mcp policy for server(s) {offending:?}"
                ),
            )
            .with_path(agent_path.to_path_buf())
            .with_hint(format!(
                "narrow agent '{stem}' mcp.servers {offending:?} to a subset of the package's mcp.servers, or grant the missing server(s)/methods in the package's mcp policy"
            )),
        );
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
        if skill.name().trim().is_empty() {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::RECIPE_INVALID_SKILL_REF,
                    format!("agent '{stem}' skills[{index}] must be a non-empty reference"),
                )
                .with_path(agent_path.clone()),
            );
        }

        // A pinned entry's `version`, when present, MUST identify a
        // specific release. `resolved_sha` is not required here (a recipe
        // author may pin by version before a resolver exists), but a
        // version of `latest`/`@latest` (case-insensitively) or empty
        // makes reproducibility impossible, defeating the point of pinning.
        if let SkillRef::Pinned(pinned) = skill {
            if let Some(version) = &pinned.version {
                let normalized = version.trim().to_ascii_lowercase();
                if normalized.is_empty() || normalized == "latest" || normalized == "@latest" {
                    diagnostics.push(
                        Diagnostic::error(
                            error_codes::RECIPE_SKILL_UNPINNED,
                            format!(
                                "agent '{stem}' skills[{index}] ('{}') must pin a specific version, not '{version}'",
                                pinned.r#ref
                            ),
                        )
                        .with_path(agent_path.clone()),
                    );
                }
            }
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

    // An agent that resolves (post-`from:`) to no harness is claiming
    // harness-agnosticism: it must run under any conforming harness, so it
    // must not reference a name that is builtin under any harness unless
    // that name also resolves as a recipe-provided tool (see
    // `resolves_as_recipe_provided`).
    //
    // This check runs against the RESOLVED manifest (post-`from:` merge),
    // not the as-authored one - consistent with the `mcp` package check
    // (`RECIPE_MCP_AGENT_WIDENS_POLICY`, section 7.3). An as-authored
    // `agent.harness.is_none()` is not proof of agnosticism: a `from:`
    // child that narrows a builtin toolset it received from a parent (e.g.
    // `{from: main, tools: [Bash]}` where `main` declares `harness: claude`)
    // resolves to `harness: claude` and is therefore not agnostic at all.
    // Checking the authored form would make narrowing a builtin toolset
    // impossible without redeclaring `harness` on every child, since
    // narrowing requires naming a subset of the parent's builtins and the
    // as-authored check would forbid naming them. A genuinely agnostic
    // agent (no harness anywhere in its `from:` chain) is still rejected,
    // because `resolved.harness` stays `None` in that case.
    //
    // Resolution failure (cycle / unresolved `from`) is already reported by
    // the `from:` resolution loop in `validate_loaded_recipe`, so this
    // simply skips the check rather than double-reporting.
    if let Ok(resolved) = resolve_inherited(recipe, stem) {
        if resolved.harness.is_none() {
            let resolved_tools: &[String] = resolved.tools.as_deref().unwrap_or(&[]);
            for (index, tool) in resolved_tools.iter().enumerate() {
                if is_builtin_under_any_harness(tool) && !resolves_as_recipe_provided(recipe, tool)
                {
                    diagnostics.push(
                        Diagnostic::error(
                            error_codes::RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL,
                            format!(
                                "agent '{stem}' declares no harness but (resolved) tools[{index}] ('{tool}') is a harness-builtin tool name"
                            ),
                        )
                        .with_path(agent_path.clone())
                        .with_hint(format!(
                            "declare a harness for '{stem}', or remove '{tool}' from tools, or ship it at tools/{tool}/tool.yaml"
                        )),
                    );
                }
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

    // Delegation must not be a permission escape hatch. Sections 4.6 and 4.7
    // claim `tools` is an enforced allowlist and that permission narrows
    // monotonically at every tier; without this check a delegator with
    // `tools: []` could name a sibling declaring `tools: [Bash, Write]` and
    // validate clean, so neither claim would hold. Both sides are compared
    // *resolved*, because a subagent can acquire permission through its own
    // `from:` chain, and an as-authored comparison would miss exactly that.
    //
    // An absent `tools` on the delegator means "unrestricted" (section 4.6),
    // so it bounds nothing and the check is skipped; an absent `tools` on the
    // subagent is likewise unrestricted, which a bounded delegator MUST NOT
    // confer.
    if let Ok(resolved_delegator) = resolve_inherited(recipe, stem) {
        for subagent in &agent.subagents {
            if !recipe.agents.contains_key(subagent) {
                continue;
            }
            let Ok(resolved_subagent) = resolve_inherited(recipe, subagent) else {
                continue;
            };
            if let Some(ceiling) = resolved_delegator.tools.as_deref() {
                let permitted: HashSet<&str> = ceiling.iter().map(String::as_str).collect();
                match resolved_subagent.tools.as_deref() {
                    None => diagnostics.push(
                        Diagnostic::error(
                            error_codes::RECIPE_SUBAGENT_WIDENS_TOOLS,
                            format!(
                                "agent '{stem}' restricts its own tools but delegates to subagent '{subagent}', which declares no tools restriction at all"
                            ),
                        )
                        .with_path(agent_path.clone())
                        .with_hint(format!(
                            "give '{subagent}' an explicit tools allowlist within '{stem}'s own"
                        )),
                    ),
                    Some(subagent_tools) => {
                        let offending: Vec<&str> = subagent_tools
                            .iter()
                            .map(String::as_str)
                            .filter(|tool| !permitted.contains(tool))
                            .collect();
                        if !offending.is_empty() {
                            diagnostics.push(
                                Diagnostic::error(
                                    error_codes::RECIPE_SUBAGENT_WIDENS_TOOLS,
                                    format!(
                                        "agent '{stem}' delegates to subagent '{subagent}', which grants tools {offending:?} that '{stem}' does not permit"
                                    ),
                                )
                                .with_path(agent_path.clone())
                                .with_hint(format!(
                                    "remove {offending:?} from '{subagent}', or grant them to '{stem}'"
                                )),
                            );
                        }
                    }
                }
            }
            let delegator_mcp = resolved_delegator.mcp.clone().unwrap_or_default();
            let subagent_mcp = resolved_subagent.mcp.clone().unwrap_or_default();
            let offending = mcp_policy_widenings(&delegator_mcp, &subagent_mcp);
            if !offending.is_empty() {
                diagnostics.push(
                    Diagnostic::error(
                        error_codes::RECIPE_SUBAGENT_WIDENS_MCP,
                        format!(
                            "agent '{stem}' delegates to subagent '{subagent}', whose mcp policy names server(s) {offending:?} that '{stem}' does not permit"
                        ),
                    )
                    .with_path(agent_path.clone())
                    .with_hint(format!(
                        "narrow '{subagent}'s mcp policy to within '{stem}'s own"
                    )),
                );
            }
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
    fn empty_judge_name_diagnostic_anchors_to_the_offending_judge_file() {
        // Two judges: one valid, one with an empty `name`. The empty-name
        // case has no other identifying field, so the diagnostic's path is
        // the only way an author can tell which of the two files is broken.
        // A test that only checked "an error fired" would also have passed
        // against the pre-fix bug (which anchored every judge diagnostic to
        // the `judges/` directory itself), so this asserts on the path.
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();
        std::fs::write(
            root.join("recipe.yaml"),
            "name: r\nversion: 0.1.0\ndefault_agent: main\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("agents")).unwrap();
        std::fs::write(
            root.join("agents").join("main.yaml"),
            "name: main\nmodel:\n  name: m\n  effort: low\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("judges")).unwrap();
        std::fs::write(
            root.join("judges").join("valid.yaml"),
            "name: valid\nprompt: judge this\n",
        )
        .unwrap();
        std::fs::write(
            root.join("judges").join("broken.yaml"),
            "name: ''\nprompt: judge this\n",
        )
        .unwrap();

        let diagnostics = validate_recipe_dir(root);
        let empty_name_diagnostics: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.code == error_codes::RECIPE_INVALID_JUDGE_MANIFEST)
            .filter(|d| d.message.contains("empty name"))
            .collect();
        assert_eq!(
            empty_name_diagnostics.len(),
            1,
            "expected exactly one empty-name diagnostic, got: {diagnostics:?}"
        );
        let anchored = empty_name_diagnostics[0]
            .location
            .path
            .as_ref()
            .expect("diagnostic should carry a path");
        assert!(
            anchored.ends_with("judges/broken.yaml"),
            "diagnostic should anchor to the specific offending judge file (broken.yaml), \
             not the judges/ directory or the unrelated valid.yaml, got {anchored:?}"
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
