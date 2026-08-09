# Migration from Syntropic137 workflow phases (Task 11 acceptance corpus)

This document is **informative**. It is the field-by-field mapping and
worked corpus behind `tests/corpus_test.rs`: every Syntropic137 workflow
phase this crate could read (18 total, see below) reduced to its
agent-shaped content, migrated to a recipe, and validated. It is the
acceptance evidence for the whole EXP-V1-0005 revision: before
`corpus_test.rs` existed, this standard had no conformant consumer drawn
from real, independently-authored data - only its own hand-built fixtures.

`docs/03-syntropic137-mapping.md` (informative, written earlier in this
revision) describes how a consumer *loads and runs* a recipe directory
(`AgentRunSpec`, the `load_recipe_dir` flow). This document is narrower and
came later: given one Syntropic137 workflow *phase*, which of its fields
belong in a recipe at all, and where.

## Corpus inventory

18 workflows total, not the 12 + 4 = 16 the Task 11 brief anticipated:

- **14 local**, read from `Syntropic137/syntropic137/workflows/examples/**/*.yaml`
  in a checkout verified to be **2 commits behind `origin/main`** at the time
  of reading, via `git status --short --branch` (which reported
  `main...origin/main [behind 2]`) and confirmed with
  `git rev-list --count HEAD..origin/main` (`2`) / `origin/main..HEAD` (`0`,
  so no local commits are ahead). Both commands read the existing local
  `origin/main` tracking ref only; neither fetches, pulls, resets, or cleans
  the checkout, so this figure reflects the last time that ref was updated,
  not necessarily the true HEAD of the remote at the moment of reading - a
  later reader re-running the same two commands can tell whether the corpus
  is still current or has drifted further. The brief warned it would be
  around 7 behind as of 2026-08-08; it had partially caught up since (the
  remote-tracking ref itself may simply be more current now than when the
  brief was written, not necessarily because the checkout was updated in the
  interim). Per the brief's explicit instruction, this checkout was read
  as-is: no `pull`, `fetch`, `reset`, or `clean` was run against it, and
  nothing in it was modified. The 14 files:
  `codex-demo.yaml`, `multi-agent-write-then-read.yaml`, `subagent-demo.yaml`,
  `github-pr.yaml`, `codex-delegates-to-claude.yaml`, `implementation.yaml`,
  `research.yaml`, `multi-agent-programmatic.yaml`, `research-with-prompts.yaml`,
  `multi-agent-claude-then-codex-markers.yaml`, `reply-ok-interactive.yaml`,
  `research-package/workflow.yaml`,
  `starter-plugin/workflows/research/workflow.yaml`,
  `starter-plugin/workflows/pr-review/workflow.yaml`.
  (`starter-plugin/syntropic137.yaml` also matches `**/*.yaml` under that
  directory but is a plugin *manifest*, not a workflow - it has no
  `phases:` - so it is excluded from the 14.)
- **4 marketplace**, read read-only via `gh api` against
  `syntropic137/syntropic137-marketplace` (no local checkout, nothing to go
  stale): `plugins/code-review/workflows/review/workflow.yaml`,
  `plugins/sdlc-trunk/workflows/{ci-fix,pr-review,release-prep}/workflow.yaml`.

**Why 18, not 16 - stated limitation.** The brief's "12" was accurate for an
earlier state of the local checkout. Two of the 14 files above
(`research-package/`, `starter-plugin/`) are **package-format** examples
added by `feat(workflow): add package format and syn workflow install`
(`74ea7776`) and `feat(marketplace): add workflow marketplace` (`2aff6733`),
contributing 3 of the 14 `workflow.yaml` files (`research-package` has one,
`starter-plugin` has two, one per bundled workflow). The brief's count
predates those commits reaching this checkout. This is exactly the kind of
drift the brief asked to be recorded rather than silently resolved by
re-counting to match: the corpus used here is what the checkout actually
contained, not what the brief expected it to contain.

Of the 18, **15 carry agent-shaped content** (at least one phase with an
`agent:` block, a bare `model`, or a tool allowlist) and **3 are entirely
workflow-layer** (`subagent-demo-workflow-v2`, `implementation-workflow-v1`,
`research-workflow-v2` - every phase is `order`/`execution_type`/artifacts/
`prompt_template` only). `corpus_test.rs` asserts both counts explicitly so
neither can silently drift without the test noticing.

## The `allowed_tools` finding

The Task 11 brief asked: `allowed_tools` appears in all 4 marketplace
workflows and in none of the local examples - is this a version split, a
plugin-vs-example difference, or genuine divergence? Resolved by reading
Syntropic137's own schema
(`packages/syn-domain/src/syn_domain/contexts/orchestration/_shared/workflow_definition.py`),
not by guessing from the YAML alone:

1. **`allowed_tools` is a real, current field** on `PhaseYamlDefinition`
   (line 264: `allowed_tools: list[str] = Field(default_factory=list)`) and
   an equivalent aliased field `allowed-tools` on `PhaseFrontmatterSchema`
   (line 362: `allowed_tools: str | list[str] = Field(..., alias="allowed-tools")`,
   accepting either a YAML list or a comma-separated string). Both are parsed
   by the same current schema; there is no version split between the local
   checkout and the marketplace.
2. **It is genuinely absent, inline, from most local examples** - the 11
   flat `workflows/examples/*.yaml` demo/bring-up files (`codex-demo`,
   `subagent-demo`, `github-pr`, `implementation`, `research`, etc.) never
   populate `allowed_tools` inline. These are internal smoke-test /
   bring-up workflows (their own comments say so - "used during the Phase C
   bring-up of feat/interactive-tmux-workspaces", "Use this to test the
   SUBAGENT_STARTED/SUBAGENT_STOPPED event flow"); an unrestricted tool
   surface is a reasonable default for them.
3. **It is present, indirectly, in several local examples too** - once a
   phase uses `prompt_file`, its `allowed-tools` lives in the referenced
   `.md`'s frontmatter, not inline in `workflow.yaml`. `research-with-prompts.yaml`'s
   `discovery` phase, both `research-package/workflow.yaml` phases, and all
   4 `starter-plugin/` phases carry `allowed-tools` this way. So the real
   split is not "local never has it" but "local demo workflows that use
   `prompt_file` carry it in frontmatter; the flat non-`prompt_file` demo
   workflows have none at all."
4. **All 4 marketplace plugins declare `allowed_tools` inline in
   `workflow.yaml` *and* in their phases' frontmatter too** - all 11
   marketplace phase `.md` files were fetched and read directly (not
   sampled), confirming the inline copy and the frontmatter copy agree for
   every phase (e.g. `bash, git, read` both places for
   `code-review`'s `analyze`). The frontmatter also shows `model` varies by
   phase within a workflow: most phases declare `model: sonnet`, but
   `sdlc-ci-fix`'s `verify`, `sdlc-pr-review`'s `context`, and
   `sdlc-release-prep`'s `publish` declare `model: haiku` instead - the
   lighter-weight confirm/gate step in each pipeline runs a cheaper model.
   No marketplace phase declares `max_tokens`/`max-tokens` anywhere. These
   are production plugins (PR review, CI self-healing, release prep) rather
   than bring-up demos, and unlike the local flat examples, restricting the
   tool surface is a real security concern for them (they run against real
   repos with real credentials), which plausibly explains why their authors
   declared it in both places belt-and-suspenders rather than relying on
   frontmatter alone.
5. **`github-pr.yaml`'s `tools:` key is a red herring, not evidence of a
   third field name.** It sets `tools: [bash, computer]` on both its
   phases. `PhaseYamlDefinition` has no `tools` field, and its
   `model_config = ConfigDict(frozen=True)` does **not** set
   `extra="forbid"` (pydantic v2's unset-`extra` default is `"ignore"`,
   unlike the sibling `AgentYamlDefinition`/`PhaseFrontmatterSchema`, both
   of which explicitly set `extra="forbid"`). So `github-pr.yaml`'s `tools:`
   key is **silently dropped by Syntropic137's own parser** - it has never
   done anything at runtime. It was migrated into the corpus test anyway
   (as `tools: [bash, computer]` / `tools: [bash]`) because it is the only
   agent-shaped signal that phase declares, and because a recipe is a
   portable artifact that should tolerate content a particular consumer's
   parser happens to ignore.

**Verdict:** not a version split, not evidence of genuine divergence -
`allowed_tools`/`allowed-tools` is one field, spelled two ways for two YAML
contexts (workflow-level snake_case vs. frontmatter's Claude-Code-command-style
hyphenated alias), used inconsistently across examples by document *purpose*
(internal bring-up demo vs. production plugin) rather than by *version* or
*location* (local vs. marketplace). Both corpora are equally authoritative
schema-wise; they simply sample different points on the same
demo-to-production spectrum.

## Field-by-field mapping

| Syntropic137 phase field | Destination | Notes |
|---|---|---|
| `agent.provider` (`claude` / `claude-interactive` / `codex`) | **recipe**: `agent.harness` | See "Harness inference" below - the brief's placement/identity split needs a refinement here. |
| `agent.agent_id` (`claude` / `codex` / `gemini`) | **recipe**: `agent.harness` (when it disagrees with `provider`), else **run spec** | See "Harness inference" below. |
| `agent.model` / phase-level `model` / frontmatter `model` | **recipe**: `model.name` | Bare (`sonnet`, `opus`), never provider-qualified, in 100% of the corpus. See Gap 1. |
| `allowed_tools` / `allowed-tools` | **recipe**: `agent.tools` | See the allowed_tools finding above. |
| `tools` (github-pr.yaml only) | **recipe**: `agent.tools` | Dead field in Syntropic137's own schema (see above); still migrated as the only available signal. |
| `agent.allow_delegation` | **recipe**: `agent.allow_delegation` | Direct mapping; only `codex-delegates-to-claude` uses it. |
| `max_tokens` / frontmatter `max-tokens` | **recipe**: `model.max_tokens` | Direct mapping, carried through with its **real per-phase value** for every corpus phase that declares one: `codex-implement` 4096, `write`/`read` 1024 each, `create-pr` 8192, `verify-pr` 2048, `build-and-delegate` 4096, `plan` 2048, `implement` 4096, `claude_first`/`codex_second` 1024 each, `reply` 1024, plus the local `prompt_file`-frontmatter phases (`discovery`/`synthesis`/`investigate`/`review`/`summarize`, 4096-8192). No marketplace phase declares `max_tokens` anywhere - genuinely absent source data (see the allowed_tools finding above), not dropped by this migration. |
| `prompt_template` / `prompt_file` | **out of scope** (run spec `task`, not recipe `prompts/`) | See "Prompt content" below - this document deliberately departs from the brief's draft mapping here. |
| `timeout_seconds` | **workflow layer** | Per-invocation ceiling; not agent identity. |
| `order`, `execution_type` | **workflow layer** | Sequencing, not agent identity. |
| `input_artifacts`, `output_artifacts` | **workflow layer** | Phase-to-phase data flow, not agent identity. |
| `argument_hint` | **run spec** | Describes the expected `task` shape for one invocation, not what the agent is. |
| `id`, `name`, `description` (workflow-level) | **workflow layer** | Workflow identity, not agent identity (though `id`/`name` at the *agent* level do map directly to `AgentManifest.name`). |
| `claude_plugins`, `skills` (phase-level, issue #772/#726) | **out of scope for this migration** | No instance in the 18-workflow corpus; would map to recipe `skills:` if it appeared. Noted for a future corpus revision, not asserted here. |

### Prompt content: a deliberate departure from the brief's draft mapping

The Task 11 brief's Step 2 draft said `prompt_file` and `prompt_template`
map to the recipe's `prompts/` directory. Having now read `docs/01_spec.md`
section 9, that destination is wrong for what these fields actually are:
`prompts/` is documented as "prompt text referenced by judges (or anything
else)" - but a judge's prompt is evaluation criteria, an evals-and-judges
concern that outlives any one invocation. A Syntropic137 phase's
`prompt_template`/`prompt_file` is the **task content for one invocation**
- closer to the run spec's `task` field (`docs/03-syntropic137-mapping.md`)
than to anything in the recipe. Writing it into `prompts/` in this crate's
schema would be accepted (the directory has no content contract), but
nothing in the recipe format would ever read it back - it would be inert
decoration, which is worse than leaving it out. This migration therefore
does **not** carry phase prompt text into the corpus test's synthesized
recipes at all; it is recorded here as out of scope (run spec `task`), not
silently dropped without explanation.

### Harness inference

The brief states: "`provider: claude-interactive` and `agent_id` are
execution-path concerns, not agent identity. A headless run and a tmux pane
can run the same harness." That is correct for the *placement* axis
(`claude` headless vs. `claude-interactive` in a tmux pane - same harness,
different run-spec mechanics) but the corpus shows it is not the whole
picture for `agent_id`:

- `multi-agent-write-then-read.yaml`'s `write` phase:
  `provider: claude-interactive, agent_id: claude` - same harness as
  placement suggests. `agent_id` here is redundant with an implied default.
- Its `read` phase: `provider: claude-interactive, agent_id: codex` - the
  pane `agent_id` selects genuinely runs the **Codex** CLI, not Claude. If
  `agent_id` were purely a run-spec placement detail, this phase's *recipe*
  harness would be undetermined; in fact it is exactly as determined as
  `codex-demo.yaml`'s `provider: codex` phase, just spelled differently
  because the launch mechanism (headless vs. an interactive-tmux pane) 
  differs.

Resolution used throughout this corpus, and encoded in `corpus_test.rs`'s
`AgentPhase.harness`: `agent_id`, when present, determines the recipe's
`harness` (`claude`/`codex`/`gemini` -> `Claude`/`Codex`/*unrepresentable*,
see Gap 2); `provider`, when present and `agent_id` is not, determines it
(`codex` -> `Codex`; `claude` or `claude-interactive` -> `Claude`, the
`-interactive` suffix being exactly the placement nuance the brief
correctly calls a run-spec concern). This is a refinement of the brief's
statement, not a rejection of it: *placement* (headless vs. tmux, which
pane) is a run-spec concern exactly as the brief says; *which CLI actually
executes* is not, and `agent_id` sometimes carries that second signal.

For phases with **no** `agent:` block at all (every `prompt_file`-driven
phase in both corpora - `PhaseFrontmatterSchema` has no `harness`/`provider`
field whatsoever), there is no explicit signal to read. This migration
infers `harness: claude` for those phases, justified by Syntropic137's own
domain-layer default: `workflow_definition.py`'s `to_domain()` comment reads
"When absent, leave provider/agent_id as None so the domain default
('claude') applies." A faithful migration makes that implicit default
explicit rather than leaving the recipe's `harness` empty. **This inference
step is load-bearing, not cosmetic** - `corpus_test.rs`'s
`literal_migration_without_harness_inference_trips_the_agnostic_builtin_rule`
test proves that skipping it and migrating one of these phases literally
(no `harness:`, `tools: [Read, Glob, Grep, Bash]` verbatim) trips
`RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL` for real. See Gap 3.

## Gaps found

Every gap below was resolved *without* changing any validation rule in
`src/validate.rs` or `src/schema.rs`. None required a code change to this
crate; all are either observations for a future revision or migration-side
judgment calls, explicitly justified above.

1. **`model.name` accepts unqualified bare names, though its own doc
   comment says it shouldn't.** `ModelSpec::name`'s doc comment reads
   "Provider-qualified model identifier, e.g. `anthropic/claude-opus-4-8`",
   but `validate.rs` only checks it is non-empty (`RECIPE_EMPTY_MODEL_NAME`)
   - there is no format check. **100% of the corpus's declared models**
     (`sonnet`, `opus`) are bare, unqualified names; none would pass a
     provider-qualification check if one existed. **Verdict: documented gap,
     non-blocking.** Nothing in the corpus fails today because nothing
     enforces the doc comment's convention. Recorded for a future revision
     to decide: either relax the doc comment to describe bare names as
     legal (matching every real workflow observed), or add an opt-in
     provider-qualification check knowing it would reject the entire
     current corpus.
2. **`HarnessKind` has no `Gemini` variant, though Syntropic137's own
   `agent_id` type already permits `Literal["claude", "codex", "gemini"]`.**
   No workflow in this 18-file corpus actually declares `agent_id: gemini`,
   so this is **not exercised** by `corpus_test.rs` and no test was
   fabricated to force it (fabricating a corpus entry to manufacture a
   failure would misrepresent what the real data says). **Verdict:
   forward-looking observation, not a corpus-driven gap.** Flagged here so a
   future corpus refresh (or a real `agent_id: gemini` workflow) has a
   documented trail rather than a surprise.
3. **A literal, no-inference migration of any `prompt_file`-frontmatter
   phase using Claude Code tool names genuinely fails
   `RECIPE_AGNOSTIC_AGENT_USES_BUILTIN_TOOL`.** Demonstrated for real by
   `literal_migration_without_harness_inference_trips_the_agnostic_builtin_rule`.
   **Verdict: not a standard gap - the rule is doing exactly its documented
   job.** Syntropic137's frontmatter schema has no harness/provider field at
   all, so a real Syntropic137 phase never states which harness its
   Claude-specific tool names require; the recipe standard correctly refuses
   to accept that ambiguity silently. This is a genuine mismatch between the
   two systems' data models, resolved (not weakened) by the harness-inference
   step described above: a correct consumer-side migrator must make
   Syntropic137's implicit claude-by-default explicit. **Out of scope for
   this standard** - the fix belongs in migration/adapter code, not in
   `validate.rs`.
4. **No genuine rule-vs-workflow conflict was found that required a
   standard change or that could not be resolved by a legitimate layer
   assignment.** After applying the harness-inference and prompt-content
   resolutions above, all 15 agent-shaped corpus workflows express as valid
   recipes with zero validation errors (`every_syntropic137_workflow_expresses_as_a_valid_recipe`).
   Nothing here is reported as BLOCKED.

## Summary table

| | Count |
|---|---|
| Corpus workflows total | 18 (14 local + 4 marketplace) |
| Workflows with agent-shaped content, expressed cleanly as recipes | 15 / 15 |
| Workflows entirely workflow-layer (no agent-shaped phase) | 3 (`subagent-demo-workflow-v2`, `implementation-workflow-v1`, `research-workflow-v2`) |
| Standard changes required | 0 |
| Gaps recorded | 3 (1 non-blocking doc/validation gap, 1 forward-looking enum gap, 1 resolved cross-system mismatch) |
| BLOCKED findings | 0 |
