# Rationale and Prior Art

This document tells the "how we got here" story behind EXP-V1-0005: what
existing work inspired it, why the recipe is a *directory* rather than a single
file, what we deliberately kept, and what we deliberately changed. It is
informative - the normative rules live in [01_spec.md](./01_spec.md).

## Credits / Prior Art

The recipe-directory concept is **derived from pi.recipes**, the recipe format
from the [**introspection.dev**](https://introspection.dev) project by the
introspection-recipes team (reference implementation:
[introspection-recipes/pi-codex](https://github.com/introspection-recipes/pi-codex)).
pi.recipes established the core idea this standard builds on: an agent run can
be captured as a reusable, declarative artifact - agent, model, skills, and
instructions - instead of being reconstructed from command-line flags every
time.

- pi.recipes / introspection.dev: https://introspection.dev
- Reference implementation: https://github.com/introspection-recipes/pi-codex

EXP-V1-0005 is our generalization of that idea into a harness-neutral,
language-neutral directory standard. Where our shape overlaps pi.recipes we
keep the same field names and enum values on purpose (see
[What Carries Over](#what-carries-over) below); where we diverge, including in
ways that break loading a pi recipe outright, we say why below.

## Inspired By pi.recipes, Not Compatible With It

The credit above is real: pi.recipes contributed the core idea this standard
builds on, a recipe as a directory of agent manifests plus shared assets. What
this standard does NOT offer is loading a pi recipe as-is. Three semantic
changes make that impossible:

- **`tools` became an enforced allowlist.** In pi.recipes (and in an earlier
  draft of this standard) `tools` was a hint: a consumer could grant more than
  a recipe listed. Section 4.6 of [01_spec.md](./01_spec.md) now makes it an
  allowlist a conforming consumer MUST NOT exceed. A pi recipe authored under
  the old, permissive assumption may under-declare `tools` relative to what it
  actually needs, and would silently lose capability if loaded here.
- **`system_instructions.mode` split into two independent axes.** pi.recipes
  has one `mode` (append/replace against the shared system prompt). This
  standard separates that from a second, independent axis,
  `harness_prompt`, which governs composition with the *harness's own*
  built-in system prompt (section 6 of [01_spec.md](./01_spec.md)). The two
  axes do not have a one-to-one mapping back to pi's single `mode`.
- **`harness` has no pi equivalent.** pi.recipes never needed a field naming
  its own runtime, because pi IS the harness - there is nothing to name. This
  standard is deliberately harness-neutral, so it needs a field pi's format
  structurally cannot have.

Because of these three changes, a pi recipe does not load correctly against
this standard. Claiming partial compatibility would promise something the
revised semantics cannot deliver, which is worse than a clean break plus
honest credit. **Migration from a pi recipe is possible under a documented
mapping; it is not a load.** The credit for the idea stands regardless: this
standard generalizes pi.recipes' core insight so it is not tied to a single
harness.

## The Goal, in One Line

A nice, standard, reusable way to define a run - portable across harnesses
(Claude Code, Codex, ...) and consumers (agentic-primitives `itmux run`,
Syntropic137, eval runners) - so a run is a committed artifact you can share,
diff, and re-run, not a pile of shell flags.

## What pi.recipes Gave Us

pi.recipes contributed the foundational insight: an agent run is worth treating
as a **first-class declarative artifact**. Instead of encoding "which agent,
which model, which skills, which instructions" as ad-hoc flags passed to a CLI,
you write it down once, in a portable form, and hand that artifact to whatever
runs it.

Concretely, the ideas we adopted:

- A **declarative manifest** describing the agent, its model, and its reasoning
  effort.
- **Skills** as named, injectable capabilities rather than inlined prompt text.
- A shape that separates *what to run* from *inject/export* runtime concerns, so
  the same definition is reusable across invocations.
- Field names and enum values that we keep identical where they overlap, so
  someone fluent in pi.recipes reads our manifests without a translation table.

## Why a Directory, Not a Single File

pi.recipes centers on a recipe artifact; EXP-V1-0005 makes the artifact a
**directory** whose marker is `recipe.yaml`. The presence of `recipe.yaml`
denotes "this directory is a recipe", the same way a package manifest marks a
package root. We chose a directory over a single YAML file because a real run
carries more than one file's worth of material:

1. **Bundled skills.** A recipe can ship its own skills under `skills/<name>/`,
   so a self-contained recipe travels with the capabilities it depends on
   instead of assuming they already exist on the consumer. An agent's `skills`
   reference resolves to a bundled `skills/<ref>/` first, then falls back to an
   external reference (see [01_spec.md section 5](./01_spec.md)).
2. **A shared SYSTEM prompt.** `SYSTEM.md` at the recipe root holds base
   instructions shared by every agent, merged per-agent via
   `system_instructions` (`append` / `replace`). Keeping this as its own file
   makes a potentially long, multi-paragraph prompt reviewable and diffable on
   its own, rather than buried as an escaped scalar inside one big YAML file.
3. **Multiple agents and subagents.** Each agent is one file under `agents/`.
   Splitting agents across files (instead of a single nested list) keeps each
   agent independently readable and reviewable, and lets a recipe grow to
   several agents without one file becoming unwieldy.

In short: cramming bundled skills, a shared system prompt, and multiple agents
into one YAML file would make it large and hard to review. A directory with a
marker file carries all of it cleanly, and the marker gives consumers a cheap,
unambiguous "is this a recipe?" test.

## What We Changed and Why

This table is prior-art documentation, not a compatibility claim: it exists so
someone who knows pi.recipes can see exactly what differs and why, even though
the two formats no longer interoperate.

| Divergence | pi.recipes | EXP-V1-0005 | Why |
|-----------|------------|-------------|-----|
| No TypeScript `extensions` | `extensions/` of executable TS code | No `extensions/`, no code; `tools` names what an agent may use, enforced as an allowlist (see below) | Keeps a recipe a pure, safe-to-commit **data** artifact - cloning or diffing never pulls in executable code - and keeps the schema crate free of any runtime/language dependency. |
| `tools` enforcement | Not applicable (no `tools` concept; `extensions/` is code, not a permission list) | `tools` is an ALLOWLIST: a conforming consumer MUST NOT grant a tool the list does not permit. Absent means unrestricted; `[]` means none | Makes a recipe's permission surface auditable statically, without reading a single run. This is the change that most directly breaks naive pi compatibility (see "Inspired By, Not Compatible With" above). |
| Harness selection | Runtime-level; pi IS the harness, so its format never named one | OPTIONAL `harness` field per agent manifest, closed enum (`claude`, `codex`) | A field pi structurally cannot have. Absence is meaningful (harness-agnostic, checked against harness-builtin tool names); presence is a declared dependency. One recipe can mix harnesses per agent. Closed per version, version-extensible (`opencode`, `gemini`, ...). |
| `effort` instead of `thinking_level` | `thinking_level` (Claude-specific) | `model.effort: low \| medium \| high` (harness-neutral) | Three coarse levels map cleanly across harnesses that expose different reasoning granularities; the adapter translates to the native parameter, not the recipe. |
| System prompt composition | Single `mode` (append/replace) | Two independent axes: `mode` (composition with `SYSTEM.md`) and `harness_prompt` (append/replace the harness's own built-in prompt) | pi's single `mode` conflates two different prompts being composed. Splitting them lets a recipe say "add to my shared SYSTEM.md" and "replace the harness's default prompt" independently. |
| Agents and subagents unified | Separate concepts | One `agents/` directory; a subagent is just an agent another agent references in `subagents` | No separate schema, directory, or marker for subagents. Role is decided by `default_agent` and other agents' `subagents` lists, nothing structural. |

Each of these is verifiable against the reference types in
[`src/schema.rs`](../src/schema.rs): `tools: Option<Vec<String>>` (an allowlist,
`None` unrestricted vs `Some(vec![])` empty), the `HarnessKind` enum (`Claude`,
`Codex`) on `AgentManifest.harness` (itself `Option<HarnessKind>`), the
`EffortLevel` enum (`Low`, `Medium`, `High`) on `ModelSpec.effort`, the
`HarnessPromptMode` enum on `SystemInstructions.harness_prompt`, and the
single `agents: BTreeMap<String, AgentManifest>` that unifies agents and
subagents on `Recipe`.

## What Carries Over

Field names and enum values are kept identical to pi.recipes' single-agent
manifest shape where they overlap: `name`, `model.name`, `model.effort`,
`skills`, and the `mode` half of `system_instructions` (`append` \|
`replace`). Someone fluent in pi.recipes reads these fields without a
translation table, even though the recipe as a whole does not load as pi's
format. The directory shape adds `harness`, `tools`, `subagents`, `mcp`,
`evals/`, `judges/`, and `prompts/` on top of that carried-over agent shape;
none of these has a pi.recipes equivalent.

## How It Maps onto the APSS Meta-Standard

Beyond adopting pi.recipes' shape, EXP-V1-0005 fits it into the APSS
meta-standard's **shape + validate + generate** triad, so the recipe idea is not
just a document but an executable, testable standard crate:

| Leg | Where | What it is |
|-----|-------|-----------|
| **Shape** | [`src/schema.rs`](../src/schema.rs) | Typed structs (`RecipeManifest`, `AgentManifest`, `Recipe`) + the canonical `load_recipe_dir(path)` loader. |
| **Validate** | [`src/validate.rs`](../src/validate.rs) | `validate_recipe_dir(path) -> Diagnostics`, built on the loader (one code path), with stable per-rule error codes. |
| **Generate** | [`src/generate.rs`](../src/generate.rs) + [`templates/recipe/`](../templates/recipe/) | `scaffold_recipe(name, dest)` / the `create` command; generator output always validates (round-trip test). |

That triad is what turns "a nice reusable way to define a run" into something a
consumer can depend on: `itmux run`, Syntropic137, and eval runners call the one
loader the validator and generator are built on, so there is a single source of
truth for the shape.

## See Also

- [00_overview.md](./00_overview.md) - quick tour of the directory shape.
- [01_spec.md](./01_spec.md) - the normative specification.
- [05-harness-tool-vocabulary.md](./05-harness-tool-vocabulary.md) - the harness-builtin tool names the `tools` allowlist and `harness` field are checked against.
- [03-syntropic137-mapping.md](./03-syntropic137-mapping.md) - how a consumer runs a recipe.
