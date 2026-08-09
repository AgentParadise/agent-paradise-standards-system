//! Conformance corpus: every real Syntropic137 workflow phase that carries
//! agent-shaped content (harness/provider, model, or a tool allowlist) must
//! express as a valid recipe. This is the acceptance test for the whole
//! EXP-V1-0005 revision (Task 11): before this test exists, the standard has
//! no conformant consumer drawn from real, independently-authored data.
//!
//! The corpus and every field mapping applied here is documented in
//! `docs/06-migration-from-syntropic137.md`, which is the authoritative
//! explanation for *why* each phase migrates the way it does (in particular
//! the harness-inference rule applied when a phase carries no explicit
//! `agent.provider`/`agent_id`: see that document's "Harness inference"
//! section, which cites Syntropic137's own source as the justification, an
//! absent `provider` defaults to `claude` in Syntropic137's own domain
//! model).
//!
//! A failure in `every_syntropic137_workflow_expresses_as_a_valid_recipe` is
//! a gap in this standard, not a defect in the workflow: see the migration
//! doc's "Gaps found" section for how each one (if any) was resolved.

use agent_recipe::validate_recipe_dir;
use std::fs;
use std::path::PathBuf;

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
