//! Conformance corpus: every real Syntropic137 workflow phase that carries
//! agent-shaped content (harness/provider, model, or a tool allowlist) must
//! express as a valid recipe, using the field vocabulary `render_agent_yaml`
//! emits (below).
//!
//! **What this test proves.** Across the 18 real, independently-authored
//! workflows in this corpus, the six fields `render_agent_yaml` emits -
//! `name`, `harness`, `model.name`, `model.max_tokens`, `tools`, and
//! `allow_delegation` - suffice to transcribe every agent-shaped phase
//! without loss of the data those workflows actually carry, and no rule in
//! this standard rejects an ordinary, real workflow expressed that way. That
//! is the whole of what this test establishes.
//!
//! **How the transcription itself is held honest.** The corpus below is
//! hand-encoded, so validating it proves only that the transcriptions are
//! well-formed, not that they faithfully describe their sources. The 14 local
//! sources are therefore vendored under `tests/fixtures/corpus/` and
//! compared field by field (see "Source fidelity" at the end of this file):
//! `local_corpus_phase_inventory_matches_source` fails if a phase is dropped,
//! renamed, or invented, and `local_corpus_phase_fields_match_source` fails if
//! `model`, `max_tokens`, `tools`, or `allow_delegation` is mistranscribed,
//! resolving each through the phase's `prompt_file` frontmatter where the
//! phase body does not state it. Absence is compared as strictly as presence,
//! so deleting a source value fails rather than switching the check off. The 4
//! marketplace cases are read over the network from a separate repository and
//! are NOT verified this way; `corpus_source_coverage_is_declared` asserts
//! that 14/4 split so the unverified remainder stays visible rather than
//! implied.
//!
//! The vendored copies differ from their originals in exactly one respect:
//! em dashes in prose fields (`name`, `description`, `prompt_template`) were
//! replaced per AGENTS.md, which forbids them repo-wide. No field this test
//! compares was touched.
//!
//! **What this test does NOT prove.** `render_agent_yaml` never emits
//! `from`, `mcp`, `subagents`, `skills`, `system_instructions`,
//! `model.effort`, `model.temperature`, or `description`, and
//! `migrate_workflow` never creates a `tools/`, `evals/`, `judges/`,
//! `prompts/`, or `skills/` directory, or a `SYSTEM.md`. Consequently this
//! test proves nothing about whether any composition rule is sound
//! (`from:` inheritance, `tools`/`mcp`/`allow_delegation` narrowing), about
//! MCP policy expressiveness, or about multi-agent delegation
//! (`subagents`) - none of that machinery is exercised here at all. Those
//! properties are covered, where they are covered, by the fixture- and
//! example-driven tests in `tests/conformance_test.rs`, not by this file.
//! Do not read a pass here as evidence that inheritance, narrowing, or MCP
//! policy work; it is evidence only that the plain single-agent field
//! vocabulary is sufficient for this corpus.
//!
//! The corpus and every field mapping applied here is documented in
//! `docs/06-migration-from-syntropic137.md`, which is the authoritative
//! explanation for *why* each phase migrates the way it does (in particular
//! the harness-inference rule applied when a phase carries no explicit
//! `agent.provider`/`agent_id`: see that document's "Harness inference"
//! section, which cites Syntropic137's own source as the justification, an
//! absent `provider` defaults to `claude` in Syntropic137's own domain
//! model), and states this same field-vocabulary-only scope in its own
//! "Scope of the conformance corpus" section.
//!
//! A failure in `every_syntropic137_workflow_expresses_as_a_valid_recipe` is
//! a gap in this standard, not a defect in the workflow: see the migration
//! doc's "Gaps found" section for how each one (if any) was resolved.

use agent_recipe::validate_recipe_dir;
use std::fs;
use std::path::{Path, PathBuf};

/// One phase's agent-shaped content, already migrated field-by-field per the
/// mapping table in `docs/06-migration-from-syntropic137.md`. Fields the
/// mapping assigns to the run spec or workflow layer (`provider`, `agent_id`,
/// `order`, `execution_type`, `input_artifacts`, `output_artifacts`,
/// `timeout_seconds`) are deliberately not represented here: they have no
/// destination in a recipe.
struct AgentPhase {
    /// The phase id from the source workflow.yaml. Used as both the agent
    /// name and the `agents/<id>.yaml` file stem.
    phase_id: &'static str,
    /// `agent.harness`, inferred per the migration doc's harness-inference
    /// rule when the source declares no explicit provider/agent_id of its
    /// own. `None` means the migrated agent is harness-agnostic.
    harness: Option<&'static str>,
    /// `model.name`, taken verbatim from the source (`agent.model` or a
    /// `prompt_file` frontmatter `model:`). Deliberately NOT provider-
    /// qualified even when the standard's own docs recommend it - see Gap 1
    /// in the migration doc.
    model: Option<&'static str>,
    /// `model.max_tokens`, from the source phase's `max_tokens` (or
    /// frontmatter `max-tokens`).
    max_tokens: Option<u32>,
    /// `tools`, from the source phase's `allowed_tools` / `allowed-tools`
    /// (or, for `github-pr.yaml`, its `tools:` key - see the migration doc's
    /// allowed_tools finding for why that key is transcribed here even
    /// though Syntropic137's own schema silently discards it).
    tools: Option<&'static [&'static str]>,
    /// `allow_delegation`, from `agent.allow_delegation`.
    allow_delegation: bool,
}

impl AgentPhase {
    const fn minimal(phase_id: &'static str) -> Self {
        AgentPhase {
            phase_id,
            harness: None,
            model: None,
            max_tokens: None,
            tools: None,
            allow_delegation: false,
        }
    }
}

/// One corpus workflow: a Syntropic137 workflow.yaml (or package workflow),
/// reduced to the phases that carry agent-shaped content. `workflow_layer_only`
/// names the phases this workflow declares that carry NO agent-shaped content
/// at all (no `agent:` block, no bare `model`, no `allowed_tools`/`tools`) -
/// pure `order`/`execution_type`/`input_artifacts`/`output_artifacts`/
/// `prompt_template` workflow-layer content. Recording them here (rather than
/// omitting them) is what keeps a workflow with zero agent-shaped phases from
/// silently vanishing from the corpus instead of being counted as
/// "out of scope, and here is why".
struct WorkflowCase {
    /// The workflow's own `id:` field, used as the recipe name and, sanitized,
    /// the temp directory name.
    workflow_id: &'static str,
    /// Where this workflow was read from, for the corpus inventory.
    source: &'static str,
    /// Phases with agent-shaped content, migrated to `AgentPhase`. The first
    /// entry becomes `default_agent`.
    agent_phases: Vec<AgentPhase>,
    /// Phase ids this workflow declares that have no agent-shaped content.
    workflow_layer_only: Vec<&'static str>,
}

/// The 14 workflows read from the local (stale, per the task-11 brief and
/// `docs/06-migration-from-syntropic137.md`) `Syntropic137/syntropic137`
/// checkout at `workflows/examples/**/*.yaml`, plus the 4 read read-only from
/// `syntropic137/syntropic137-marketplace` via `gh api`. 18 total, not the
/// brief's anticipated 12 + 4 = 16: the local checkout has grown 2 extra
/// package-format examples (`research-package/`, `starter-plugin/`) since the
/// brief was written. See the migration doc's "Corpus inventory" section.
fn corpus_cases() -> Vec<WorkflowCase> {
    vec![
        // ---- Local: Syntropic137/syntropic137/workflows/examples/*.yaml ----
        WorkflowCase {
            workflow_id: "codex-demo-workflow-v1",
            source: "local: workflows/examples/codex-demo.yaml",
            agent_phases: vec![AgentPhase {
                phase_id: "codex-implement",
                harness: Some("codex"),
                max_tokens: Some(4096),
                ..AgentPhase::minimal("codex-implement")
            }],
            workflow_layer_only: vec![],
        },
        WorkflowCase {
            workflow_id: "multi-agent-claude-then-codex-v2",
            source: "local: workflows/examples/multi-agent-write-then-read.yaml",
            agent_phases: vec![
                // agent_id: claude -> harness=claude (provider is
                // claude-interactive, a run-spec placement concern; see the
                // migration doc's harness-inference rule).
                AgentPhase {
                    phase_id: "write",
                    harness: Some("claude"),
                    max_tokens: Some(1024),
                    ..AgentPhase::minimal("write")
                },
                // agent_id: codex under provider: claude-interactive - the
                // pane actually runs the codex CLI, so the migrated harness
                // is codex, not claude. See the migration doc's discussion
                // of this exact case.
                AgentPhase {
                    phase_id: "read",
                    harness: Some("codex"),
                    max_tokens: Some(1024),
                    ..AgentPhase::minimal("read")
                },
            ],
            workflow_layer_only: vec![],
        },
        WorkflowCase {
            workflow_id: "subagent-demo-workflow-v2",
            source: "local: workflows/examples/subagent-demo.yaml",
            agent_phases: vec![],
            workflow_layer_only: vec!["coordinator", "verify"],
        },
        WorkflowCase {
            workflow_id: "github-pr-workflow",
            source: "local: workflows/examples/github-pr.yaml",
            agent_phases: vec![
                // `tools: [bash, computer]` - a dead field in Syntropic137's
                // own schema (PhaseYamlDefinition has no `tools` field and
                // does not set extra="forbid", so it is silently dropped),
                // but the only agent-shaped signal this phase declares.
                // Migrated literally into `tools`, harness-agnostic since no
                // model/provider is present anywhere in the phase.
                AgentPhase {
                    phase_id: "create-pr",
                    max_tokens: Some(8192),
                    tools: Some(&["bash", "computer"]),
                    ..AgentPhase::minimal("create-pr")
                },
                AgentPhase {
                    phase_id: "verify-pr",
                    max_tokens: Some(2048),
                    tools: Some(&["bash"]),
                    ..AgentPhase::minimal("verify-pr")
                },
            ],
            workflow_layer_only: vec![],
        },
        WorkflowCase {
            workflow_id: "codex-delegates-to-claude",
            source: "local: workflows/examples/codex-delegates-to-claude.yaml",
            agent_phases: vec![AgentPhase {
                phase_id: "build-and-delegate",
                harness: Some("codex"),
                max_tokens: Some(4096),
                allow_delegation: true,
                ..AgentPhase::minimal("build-and-delegate")
            }],
            workflow_layer_only: vec![],
        },
        WorkflowCase {
            workflow_id: "implementation-workflow-v1",
            source: "local: workflows/examples/implementation.yaml",
            agent_phases: vec![],
            workflow_layer_only: vec!["research", "innovate", "plan", "execute", "review"],
        },
        WorkflowCase {
            workflow_id: "research-workflow-v2",
            source: "local: workflows/examples/research.yaml",
            agent_phases: vec![],
            workflow_layer_only: vec!["discovery", "deep-dive", "synthesis"],
        },
        WorkflowCase {
            workflow_id: "multi-agent-programmatic",
            source: "local: workflows/examples/multi-agent-programmatic.yaml",
            agent_phases: vec![
                AgentPhase {
                    phase_id: "plan",
                    harness: Some("claude"),
                    max_tokens: Some(2048),
                    ..AgentPhase::minimal("plan")
                },
                AgentPhase {
                    phase_id: "implement",
                    harness: Some("codex"),
                    max_tokens: Some(4096),
                    ..AgentPhase::minimal("implement")
                },
            ],
            workflow_layer_only: vec![],
        },
        WorkflowCase {
            workflow_id: "research-with-prompts-v1",
            source: "local: workflows/examples/research-with-prompts.yaml \
                      (phase 1 via prompts/research-discovery.md frontmatter)",
            agent_phases: vec![
                // frontmatter: model: sonnet, allowed-tools: Read,Glob,Grep,Bash,
                // max-tokens: 4096. harness inferred as claude (sonnet is an
                // Anthropic model name and the frontmatter declares no
                // provider); see the migration doc's harness-inference rule.
                AgentPhase {
                    phase_id: "discovery",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    max_tokens: Some(4096),
                    tools: Some(&["Read", "Glob", "Grep", "Bash"]),
                    allow_delegation: false,
                },
            ],
            // Phase 2 (synthesis) uses an inline prompt_template only - no
            // agent block, no model, no tools anywhere.
            workflow_layer_only: vec!["synthesis"],
        },
        WorkflowCase {
            workflow_id: "multi-agent-markers",
            source: "local: workflows/examples/multi-agent-claude-then-codex-markers.yaml",
            agent_phases: vec![
                AgentPhase {
                    phase_id: "claude_first",
                    harness: Some("claude"),
                    max_tokens: Some(1024),
                    ..AgentPhase::minimal("claude_first")
                },
                AgentPhase {
                    phase_id: "codex_second",
                    harness: Some("codex"),
                    max_tokens: Some(1024),
                    ..AgentPhase::minimal("codex_second")
                },
            ],
            workflow_layer_only: vec![],
        },
        WorkflowCase {
            workflow_id: "reply-ok-interactive",
            source: "local: workflows/examples/reply-ok-interactive.yaml",
            agent_phases: vec![AgentPhase {
                phase_id: "reply",
                harness: Some("claude"),
                // `agent.model: sonnet` - bare, not provider-qualified. See
                // Gap 1 in the migration doc.
                model: Some("sonnet"),
                max_tokens: Some(1024),
                ..AgentPhase::minimal("reply")
            }],
            workflow_layer_only: vec![],
        },
        WorkflowCase {
            workflow_id: "research-package-v1",
            source: "local: workflows/examples/research-package/workflow.yaml \
                      (both phases via phases/*.md frontmatter)",
            agent_phases: vec![
                AgentPhase {
                    phase_id: "discovery",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    max_tokens: Some(4096),
                    tools: Some(&["Read", "Glob", "Grep", "Bash"]),
                    allow_delegation: false,
                },
                AgentPhase {
                    phase_id: "synthesis",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    max_tokens: Some(8192),
                    tools: Some(&["Read", "Glob", "Grep", "Bash", "Write"]),
                    allow_delegation: false,
                },
            ],
            workflow_layer_only: vec![],
        },
        WorkflowCase {
            workflow_id: "starter-research-v1",
            source: "local: workflows/examples/starter-plugin/workflows/research/workflow.yaml \
                      (phases/investigate.md + shared phase-library/summarize.md frontmatter)",
            agent_phases: vec![
                AgentPhase {
                    phase_id: "investigate",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    max_tokens: Some(4096),
                    tools: Some(&["Read", "Glob", "Grep", "Bash", "WebSearch"]),
                    allow_delegation: false,
                },
                // shared://summarize -> phase-library/summarize.md: model +
                // max-tokens only, no allowed-tools key at all.
                AgentPhase {
                    phase_id: "summarize",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    max_tokens: Some(4096),
                    tools: None,
                    allow_delegation: false,
                },
            ],
            workflow_layer_only: vec![],
        },
        WorkflowCase {
            workflow_id: "starter-pr-review-v1",
            source: "local: workflows/examples/starter-plugin/workflows/pr-review/workflow.yaml \
                      (phases/review.md + shared phase-library/summarize.md frontmatter)",
            agent_phases: vec![
                AgentPhase {
                    phase_id: "review",
                    harness: Some("claude"),
                    model: Some("opus"),
                    max_tokens: Some(8192),
                    tools: Some(&["Read", "Glob", "Grep", "Bash"]),
                    allow_delegation: false,
                },
                AgentPhase {
                    phase_id: "summarize",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    max_tokens: Some(4096),
                    tools: None,
                    allow_delegation: false,
                },
            ],
            workflow_layer_only: vec![],
        },
        // ---- Marketplace: syntropic137/syntropic137-marketplace plugins ----
        // Every phase in all 4 marketplace workflows is `prompt_file`-based
        // with an inline `allowed_tools:` list AND a same-shaped frontmatter
        // `model:`/`allowed-tools:` pair in the referenced .md - all 11
        // phase frontmatter files were fetched and read directly (not
        // generalized from a sample); most declare `model: sonnet`, but
        // `sdlc-ci-fix`'s `verify`, `sdlc-pr-review`'s `context`, and
        // `sdlc-release-prep`'s `publish` declare `model: haiku` - the
        // lighter-weight confirm/gate phases in each pipeline. None declares
        // `max_tokens`/`max-tokens` anywhere. harness=claude is inferred the
        // same way as the local frontmatter-driven cases above.
        WorkflowCase {
            workflow_id: "code-review",
            source: "marketplace: plugins/code-review/workflows/review/workflow.yaml",
            agent_phases: vec![
                AgentPhase {
                    phase_id: "analyze",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    tools: Some(&["bash", "git", "read"]),
                    ..AgentPhase::minimal("analyze")
                },
                AgentPhase {
                    phase_id: "report",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    tools: Some(&["bash", "git"]),
                    ..AgentPhase::minimal("report")
                },
            ],
            workflow_layer_only: vec![],
        },
        WorkflowCase {
            workflow_id: "sdlc-ci-fix",
            source: "marketplace: plugins/sdlc-trunk/workflows/ci-fix/workflow.yaml",
            agent_phases: vec![
                AgentPhase {
                    phase_id: "diagnose",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    tools: Some(&["bash", "git", "read"]),
                    ..AgentPhase::minimal("diagnose")
                },
                AgentPhase {
                    phase_id: "fix",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    tools: Some(&["bash", "git", "read", "edit"]),
                    ..AgentPhase::minimal("fix")
                },
                AgentPhase {
                    phase_id: "verify",
                    harness: Some("claude"),
                    model: Some("haiku"),
                    tools: Some(&["bash", "git"]),
                    ..AgentPhase::minimal("verify")
                },
            ],
            workflow_layer_only: vec![],
        },
        WorkflowCase {
            workflow_id: "sdlc-pr-review",
            source: "marketplace: plugins/sdlc-trunk/workflows/pr-review/workflow.yaml",
            agent_phases: vec![
                AgentPhase {
                    phase_id: "context",
                    harness: Some("claude"),
                    model: Some("haiku"),
                    tools: Some(&["bash", "git", "read"]),
                    ..AgentPhase::minimal("context")
                },
                AgentPhase {
                    phase_id: "analyze",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    tools: Some(&["bash", "git", "read"]),
                    ..AgentPhase::minimal("analyze")
                },
                AgentPhase {
                    phase_id: "report",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    tools: Some(&["bash", "git"]),
                    ..AgentPhase::minimal("report")
                },
            ],
            workflow_layer_only: vec![],
        },
        WorkflowCase {
            workflow_id: "sdlc-release-prep",
            source: "marketplace: plugins/sdlc-trunk/workflows/release-prep/workflow.yaml",
            agent_phases: vec![
                AgentPhase {
                    phase_id: "audit",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    tools: Some(&["bash", "git", "read"]),
                    ..AgentPhase::minimal("audit")
                },
                AgentPhase {
                    phase_id: "notes",
                    harness: Some("claude"),
                    model: Some("sonnet"),
                    tools: Some(&["bash", "git"]),
                    ..AgentPhase::minimal("notes")
                },
                AgentPhase {
                    phase_id: "publish",
                    harness: Some("claude"),
                    model: Some("haiku"),
                    tools: Some(&["bash", "git"]),
                    ..AgentPhase::minimal("publish")
                },
            ],
            workflow_layer_only: vec![],
        },
    ]
}

/// Render one `AgentPhase` as `agents/<phase_id>.yaml` content.
fn render_agent_yaml(phase: &AgentPhase) -> String {
    let mut out = format!("name: {}\n", phase.phase_id);
    if let Some(harness) = phase.harness {
        out.push_str(&format!("harness: {harness}\n"));
    }
    if phase.model.is_some() || phase.max_tokens.is_some() {
        out.push_str("model:\n");
        if let Some(model) = phase.model {
            out.push_str(&format!("  name: {model}\n"));
        }
        if let Some(max_tokens) = phase.max_tokens {
            out.push_str(&format!("  max_tokens: {max_tokens}\n"));
        }
    }
    if let Some(tools) = phase.tools {
        out.push_str("tools:\n");
        for tool in tools {
            out.push_str(&format!("  - {tool}\n"));
        }
    }
    if phase.allow_delegation {
        out.push_str("allow_delegation: true\n");
    }
    out
}

/// Migrate one `WorkflowCase` into a recipe directory under a fresh temp
/// dir, per the field mapping in `docs/06-migration-from-syntropic137.md`.
/// Returns `None` when the workflow has no agent-shaped phases at all: there
/// is nothing to migrate, and building a recipe with a name but no real
/// content would be a vacuous pass, not a conformance check.
fn migrate_workflow(case: &WorkflowCase) -> Option<PathBuf> {
    if case.agent_phases.is_empty() {
        return None;
    }

    let temp = tempfile::tempdir().expect("temp dir");
    // Leak the tempdir so its path survives past this function - the
    // corpus test only needs the directory to live for the duration of one
    // #[test] process, and leaking a handful of tempdirs across one test
    // binary run is a deliberate, bounded trade for not having to thread a
    // guard object through every case.
    let root = temp.keep();

    fs::write(
        root.join("recipe.yaml"),
        format!(
            "name: {}\nversion: 0.1.0\ndefault_agent: {}\n",
            case.workflow_id, case.agent_phases[0].phase_id
        ),
    )
    .expect("write recipe.yaml");

    fs::create_dir_all(root.join("agents")).expect("create agents dir");
    for phase in &case.agent_phases {
        fs::write(
            root.join("agents").join(format!("{}.yaml", phase.phase_id)),
            render_agent_yaml(phase),
        )
        .expect("write agent yaml");
    }

    Some(root)
}

/// Every real Syntropic137 workflow phase that carries agent-shaped content
/// must express as a valid recipe. A failure here is a gap in this standard,
/// not a defect in the workflow: see `docs/06-migration-from-syntropic137.md`
/// for how every gap the corpus actually surfaced was resolved.
#[test]
fn every_syntropic137_workflow_expresses_as_a_valid_recipe() {
    let mut expressed_cleanly = 0usize;
    let mut skipped_workflow_layer_only = 0usize;

    for case in corpus_cases() {
        match migrate_workflow(&case) {
            Some(dir) => {
                let diagnostics = validate_recipe_dir(&dir);
                let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
                assert!(
                    !diagnostics.has_errors(),
                    "{} ({}) produced diagnostics: {codes:?}",
                    case.workflow_id,
                    case.source
                );
                expressed_cleanly += 1;
            }
            None => {
                // No agent-shaped phase anywhere in this workflow: every
                // phase is workflow/run-layer only (order, execution_type,
                // input/output artifacts, prompt_template). Asserting this
                // explicitly, rather than silently skipping, is what keeps
                // this from being a vacuous pass - see the module doc.
                assert!(
                    !case.workflow_layer_only.is_empty(),
                    "{} has no agent_phases but also declares no workflow_layer_only \
                     phases - it should list SOME phases somewhere",
                    case.workflow_id
                );
                skipped_workflow_layer_only += 1;
            }
        }
    }

    assert_eq!(
        expressed_cleanly, 15,
        "expected 15 of 18 corpus workflows to carry agent-shaped content \
         and express cleanly as recipes"
    );
    assert_eq!(
        skipped_workflow_layer_only, 3,
        "expected 3 of 18 corpus workflows (subagent-demo, implementation, \
         research) to be entirely workflow-layer content with no agent-shaped \
         phase at all"
    );
}

/// The corpus inventory itself: 18 workflows (14 local + 4 marketplace), not
/// the task-11 brief's anticipated 12 + 4 = 16. See
/// `docs/06-migration-from-syntropic137.md`'s "Corpus inventory" section for
/// why: the local checkout has grown 2 package-format examples
/// (`research-package/`, `starter-plugin/`, 3 workflow.yaml files) beyond
/// what the brief counted.
#[test]
fn corpus_inventory_matches_documented_count() {
    let cases = corpus_cases();
    assert_eq!(cases.len(), 18, "corpus should have 18 workflows total");

    let local_count = cases
        .iter()
        .filter(|c| c.source.starts_with("local:"))
        .count();
    let marketplace_count = cases
        .iter()
        .filter(|c| c.source.starts_with("marketplace:"))
        .count();
    assert_eq!(local_count, 14, "expected 14 local workflows");
    assert_eq!(marketplace_count, 4, "expected 4 marketplace workflows");
}

/// The `allowed_tools` discrepancy the task-11 brief asked to resolve before
/// treating either corpus as authoritative: every marketplace phase declares
/// tools (inline `allowed_tools:` plus frontmatter `allowed-tools:`); no
/// *inline* local workflow.yaml phase does, but several local phases carry
/// the same underlying field via `prompt_file` frontmatter instead. See the
/// migration doc for the full resolution (a style difference between
/// demo/bring-up examples and production plugins, not a schema version
/// split).
#[test]
fn allowed_tools_appears_in_every_marketplace_phase_and_some_local_phases() {
    let cases = corpus_cases();
    for case in cases
        .iter()
        .filter(|c| c.source.starts_with("marketplace:"))
    {
        assert!(
            case.agent_phases.iter().all(|p| p.tools.is_some()),
            "{}: every marketplace phase should declare tools",
            case.workflow_id
        );
    }
    let local_phases_with_tools: usize = cases
        .iter()
        .filter(|c| c.source.starts_with("local:"))
        .flat_map(|c| c.agent_phases.iter())
        .filter(|p| p.tools.is_some())
        .count();
    assert!(
        local_phases_with_tools > 0,
        "at least one local phase should declare tools (via frontmatter or \
         the dead `tools:` key in github-pr.yaml)"
    );
}

/// Proves the harness-inference step in `migrate_workflow`'s corpus cases is
/// load-bearing, not decorative window dressing on an otherwise-vacuous test.
///
/// Syntropic137's own `PhaseFrontmatterSchema` (the schema for `prompt_file`
/// frontmatter) has NO `harness` or `provider` field at all - only `model`,
/// `allowed_tools`/`allowed-tools`, `max_tokens`, `timeout_seconds`,
/// `execution_type`, `description`, `argument_hint`. A phase like
/// `research-with-prompts-v1`'s `discovery` (frontmatter: `model: sonnet`,
/// `allowed-tools: Read,Glob,Grep,Bash`) therefore never declares a harness
/// anywhere in the source data. Literally transcribing that phase with no
/// harness (the reading a naive field-by-field copy would produce) DOES trip
/// `RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL`: `Read`/`Glob`/`Grep`/`Bash` are
/// Claude Code builtins, and the standard correctly rejects a harness-agnostic
/// agent that names them.
///
/// This is the standard's rule doing exactly its documented job against real
/// data - not a gap in this standard. It IS a genuine mismatch with
/// Syntropic137's own data model: Syntropic137 defaults an absent `provider`
/// to `claude` at the domain layer (see
/// `packages/syn-domain/.../workflow_definition.py`, `to_domain()`: "When
/// absent, leave provider/agent_id as None so the domain default ('claude')
/// applies"). A faithful migration must make that implicit default explicit
/// (`harness: claude`) rather than leaving it absent - which is exactly what
/// `corpus_cases()`'s harness-inference rule does for every frontmatter-only
/// phase above. See the migration doc's "Harness inference" section and Gap
/// 3 for the full resolution.
#[test]
fn literal_migration_without_harness_inference_trips_the_agnostic_builtin_rule() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path();

    // research-with-prompts-v1 / discovery, transcribed literally: no
    // `harness:` at all (the frontmatter never declares one), `model.name:
    // sonnet` (bare, as declared), `tools:` taken verbatim from
    // `allowed-tools: Read,Glob,Grep,Bash`.
    fs::write(
        root.join("recipe.yaml"),
        "name: research-with-prompts-v1\nversion: 0.1.0\ndefault_agent: discovery\n",
    )
    .expect("write recipe.yaml");
    fs::create_dir_all(root.join("agents")).expect("create agents dir");
    fs::write(
        root.join("agents").join("discovery.yaml"),
        "name: discovery\n\
         model:\n  name: sonnet\n  max_tokens: 4096\n\
         tools:\n  - Read\n  - Glob\n  - Grep\n  - Bash\n",
    )
    .expect("write agent yaml");

    let diagnostics = validate_recipe_dir(root);
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        codes.contains(&"RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL"),
        "a literal (no-harness-inferred) migration of a real Syntropic137 \
         frontmatter phase using Claude Code tool names should trip \
         RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL; got {codes:?}"
    );
}

// --- Source fidelity -------------------------------------------------------
//
// Everything above proves the *transcriptions* validate. That is not the same
// as proving they faithfully describe the workflows they claim to describe: a
// phase could be dropped, or a `max_tokens` mistyped, and every assertion
// above would still pass. These tests close that gap for the 14 local cases by
// vendoring their sources under `tests/fixtures/corpus/` and comparing the
// hand-encoded corpus against them field by field.
//
// The 4 marketplace cases are read from a separate repository over the network
// and cannot be vendored here; they remain hand-encoded and unverified, which
// `corpus_source_coverage_is_declared` states explicitly rather than leaving
// implied.

/// The subset of the Syntropic137 workflow schema this corpus actually claims
/// to transcribe. Unknown fields are ignored on purpose: this is a fidelity
/// check against the fields the corpus carries, not a validator for their
/// schema.
#[derive(serde::Deserialize)]
struct SourceWorkflow {
    id: String,
    #[serde(default)]
    phases: Vec<SourcePhase>,
}

#[derive(serde::Deserialize)]
struct SourcePhase {
    id: String,
    #[serde(default)]
    max_tokens: Option<u32>,
    #[serde(default)]
    agent: Option<SourceAgent>,
    #[serde(default)]
    prompt_file: Option<String>,
    /// Syntropic137 spells the tool allowlist three ways across the corpus.
    #[serde(default, alias = "allowed-tools", alias = "tools")]
    allowed_tools: Option<SourceTools>,
}

#[derive(serde::Deserialize)]
struct SourceAgent {
    #[serde(default)]
    allow_delegation: Option<bool>,
    #[serde(default)]
    model: Option<String>,
}

/// `allowed_tools` appears both as a YAML list and as a comma-separated
/// string (the prompt-frontmatter spelling).
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum SourceTools {
    List(Vec<String>),
    Csv(String),
}

impl SourceTools {
    fn into_vec(self) -> Vec<String> {
        match self {
            SourceTools::List(list) => list,
            SourceTools::Csv(csv) => csv
                .split(',')
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(str::to_string)
                .collect(),
        }
    }
}

/// The `model:` / `max-tokens:` / `allowed-tools:` a phase inherits from its
/// `prompt_file`'s YAML frontmatter.
#[derive(Default, serde::Deserialize)]
struct Frontmatter {
    #[serde(default)]
    model: Option<String>,
    #[serde(default, rename = "max-tokens")]
    max_tokens: Option<u32>,
    #[serde(default, rename = "allowed-tools")]
    allowed_tools: Option<SourceTools>,
}

/// Resolve a phase's `prompt_file` to a vendored path. `shared://<name>`
/// refers to `<plugin root>/phase-library/<name>.md`; anything else is
/// relative to the workflow file's own directory.
fn resolve_prompt_file(workflow_path: &Path, prompt_file: &str) -> Option<PathBuf> {
    let dir = workflow_path.parent()?;
    if let Some(name) = prompt_file.strip_prefix("shared://") {
        // Walk up to the plugin root, the nearest ancestor holding a
        // phase-library/ directory.
        let mut candidate = dir;
        loop {
            let shared = candidate.join("phase-library").join(format!("{name}.md"));
            if shared.is_file() {
                return Some(shared);
            }
            candidate = candidate.parent()?;
            if !candidate.starts_with(fixtures_root()) {
                return None;
            }
        }
    }
    let direct = dir.join(prompt_file);
    direct.is_file().then_some(direct)
}

/// Parse the leading `---` YAML frontmatter block of a prompt Markdown file.
fn parse_frontmatter(path: &Path) -> Frontmatter {
    let text = fs::read_to_string(path).expect("read vendored prompt file");
    let Some(rest) = text.strip_prefix("---\n") else {
        return Frontmatter::default();
    };
    let Some(end) = rest.find("\n---") else {
        return Frontmatter::default();
    };
    serde_yaml::from_str(&rest[..end]).expect("prompt frontmatter should parse")
}

/// The effective model / max_tokens / tools for a source phase: the phase's
/// own inline values, falling back to its `prompt_file` frontmatter.
fn effective_source_fields(
    workflow_path: &Path,
    phase: &SourcePhase,
) -> (Option<String>, Option<u32>, Option<Vec<String>>) {
    let frontmatter = phase
        .prompt_file
        .as_deref()
        .and_then(|prompt_file| resolve_prompt_file(workflow_path, prompt_file))
        .map(|path| parse_frontmatter(&path))
        .unwrap_or_default();

    let model = phase
        .agent
        .as_ref()
        .and_then(|agent| agent.model.clone())
        .or(frontmatter.model);
    let max_tokens = phase.max_tokens.or(frontmatter.max_tokens);
    let tools = match (&phase.allowed_tools, frontmatter.allowed_tools) {
        (Some(SourceTools::List(list)), _) => Some(list.clone()),
        (Some(SourceTools::Csv(csv)), _) => Some(SourceTools::Csv(csv.clone()).into_vec()),
        (None, Some(tools)) => Some(tools.into_vec()),
        (None, None) => None,
    };
    (model, max_tokens, tools)
}

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("corpus")
}

/// The vendored fixture path for a `local:` source label, or `None` for a
/// marketplace case.
fn vendored_source(source: &str) -> Option<PathBuf> {
    let rel = source.strip_prefix("local: ")?;
    // Several labels append a parenthetical note after the path (for example
    // "(phase 1 via prompts/research-discovery.md frontmatter)"). The path is
    // everything before it.
    let rel = rel.split(" (").next().unwrap_or(rel).trim();
    Some(fixtures_root().join(rel))
}

/// Every `local:` corpus case must name a source that is actually vendored.
/// Without this, a typo'd or deleted source path would silently downgrade the
/// fidelity checks below into no-ops.
#[test]
fn every_local_corpus_source_is_vendored() {
    let mut checked = 0usize;
    for case in corpus_cases() {
        if let Some(path) = vendored_source(case.source) {
            assert!(
                path.is_file(),
                "{} names source {} but {} is not vendored under tests/fixtures/corpus/",
                case.workflow_id,
                case.source,
                path.display()
            );
            checked += 1;
        }
    }
    assert_eq!(
        checked, 14,
        "expected all 14 local corpus sources to be vendored"
    );
}

/// The corpus's `workflow_id` and its phase inventory must match the vendored
/// source. This is the check that makes a dropped or mistranscribed phase
/// visible: the union of `agent_phases` and `workflow_layer_only` must equal
/// the source's phase ids exactly.
#[test]
fn local_corpus_phase_inventory_matches_source() {
    for case in corpus_cases() {
        let Some(path) = vendored_source(case.source) else {
            continue;
        };
        let text = fs::read_to_string(&path).expect("read vendored source");
        let source: SourceWorkflow =
            serde_yaml::from_str(&text).expect("vendored source should parse");

        assert_eq!(
            source.id,
            case.workflow_id,
            "corpus workflow_id disagrees with the id in {}",
            path.display()
        );

        let mut transcribed: Vec<&str> = case
            .agent_phases
            .iter()
            .map(|phase| phase.phase_id)
            .chain(case.workflow_layer_only.iter().copied())
            .collect();
        transcribed.sort_unstable();

        let mut actual: Vec<&str> = source.phases.iter().map(|p| p.id.as_str()).collect();
        actual.sort_unstable();

        assert_eq!(
            transcribed,
            actual,
            "{} transcribes a different phase set than {}",
            case.workflow_id,
            path.display()
        );
    }
}

/// Per-phase field fidelity across every field the corpus claims to carry:
/// `model`, `max_tokens`, `tools`, and `allow_delegation`. Each is compared
/// against the phase's inline value, falling back to its `prompt_file`
/// frontmatter, so a mistranscription in either place fails.
///
/// Absence is compared as strictly as presence. An earlier version asserted
/// `max_tokens` only when the source declared it, which meant deleting the
/// source value silently turned the comparison off instead of failing.
#[test]
fn local_corpus_phase_fields_match_source() {
    for case in corpus_cases() {
        let Some(path) = vendored_source(case.source) else {
            continue;
        };
        let text = fs::read_to_string(&path).expect("read vendored source");
        let source: SourceWorkflow =
            serde_yaml::from_str(&text).expect("vendored source should parse");

        for phase in &case.agent_phases {
            let Some(source_phase) = source.phases.iter().find(|p| p.id == phase.phase_id) else {
                panic!(
                    "{} transcribes phase '{}' that {} does not declare",
                    case.workflow_id,
                    phase.phase_id,
                    path.display()
                );
            };
            let (model, max_tokens, tools) = effective_source_fields(&path, source_phase);

            assert_eq!(
                phase.max_tokens,
                max_tokens,
                "{} phase '{}': max_tokens disagrees with {}",
                case.workflow_id,
                phase.phase_id,
                path.display()
            );
            assert_eq!(
                phase.model.map(str::to_string),
                model,
                "{} phase '{}': model disagrees with {}",
                case.workflow_id,
                phase.phase_id,
                path.display()
            );
            let transcribed_tools: Option<Vec<String>> = phase
                .tools
                .map(|tools| tools.iter().map(|t| (*t).to_string()).collect());
            assert_eq!(
                transcribed_tools,
                tools,
                "{} phase '{}': tools disagree with {}",
                case.workflow_id,
                phase.phase_id,
                path.display()
            );
            let source_delegation = source_phase
                .agent
                .as_ref()
                .and_then(|a| a.allow_delegation)
                .unwrap_or(false);
            assert_eq!(
                phase.allow_delegation,
                source_delegation,
                "{} phase '{}': allow_delegation disagrees with {}",
                case.workflow_id,
                phase.phase_id,
                path.display()
            );
        }
    }
}

/// State the corpus's verification coverage as an assertion rather than a
/// prose claim: 14 of 18 cases are checked against a vendored source, and the
/// 4 marketplace cases are not.
#[test]
fn corpus_source_coverage_is_declared() {
    let cases = corpus_cases();
    let verified = cases
        .iter()
        .filter(|c| vendored_source(c.source).is_some())
        .count();
    let unverified = cases.len() - verified;
    assert_eq!(
        verified, 14,
        "14 local cases are checked against a vendored source"
    );
    assert_eq!(
        unverified, 4,
        "4 marketplace cases are hand-encoded and NOT verified against a source"
    );
}
