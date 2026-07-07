# Rationale and Prior Art

This document tells the "how we got here" story behind EXP-V1-0005: what
existing work inspired it, why the recipe is a *directory* rather than a single
file, what we deliberately kept, and what we deliberately changed. It is
informative - the normative rules live in [01_spec.md](./01_spec.md), and the
precise pi.recipes deltas are catalogued in
[02-pi-compatibility.md](./02-pi-compatibility.md).

## Credits / Prior Art

The recipe-directory concept is **derived from pi.recipes**, the recipe format
from the [**introspection.dev**](https://introspection.dev) project by the
introspection-recipes team. pi.recipes established the core idea this standard
builds on: an agent run can be captured as a reusable, declarative artifact -
agent, model, skills, and instructions - instead of being reconstructed from
command-line flags every time.

- pi.recipes / introspection.dev: https://introspection.dev
- Reference implementation: https://github.com/introspection-recipes/pi-codex

EXP-V1-0005 is our generalization of that idea into a harness-neutral,
language-neutral directory standard. Where our shape overlaps pi.recipes we keep
the same field names and enum values on purpose (see
[What Carries Over](./02-pi-compatibility.md#what-carries-over)); where we
diverge, we say why below and in the compatibility doc.

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

## What We Deliberately Diverged On (and Why)

These are the intentional departures from pi.recipes. They are catalogued with
their schema anchors in [02-pi-compatibility.md](./02-pi-compatibility.md); the
summary and reasoning:

| Divergence | pi.recipes | EXP-V1-0005 | Why |
|-----------|------------|-------------|-----|
| No TypeScript `extensions` | `extensions/` of executable TS code | `tools` are references (names) only; no `extensions/`, no code | Keeps a recipe a pure, safe-to-commit **data** artifact - cloning or diffing never pulls in executable code - and keeps the schema crate free of any runtime/language dependency. |
| Harness is per-agent | Runtime-level | Required `agent: claude \| codex` on each agent manifest | One recipe can mix harnesses (a `claude` default agent delegating to a `codex` subagent). The enum is closed per version but version-extensible (`opencode`, `gemini`, ...). |
| `effort` instead of `thinking_level` | `thinking_level` (Claude-specific) | `model.effort: low \| medium \| high` (harness-neutral) | Three coarse levels map cleanly across harnesses that expose different reasoning granularities; the adapter translates to the native parameter, not the recipe. |
| Agents and subagents unified | Separate concepts | One `agents/` directory; a subagent is just an agent another agent references in `subagents` | No separate schema, directory, or marker for subagents. Role is decided by `default_agent` and other agents' `subagents` lists, nothing structural. |

Each of these is verifiable against the reference types in
[`src/schema.rs`](../src/schema.rs): `tools: Vec<String>` (references, no code),
the `AgentKind` enum (`Claude`, `Codex`) on `AgentManifest.agent`, the
`EffortLevel` enum (`Low`, `Medium`, `High`) on `ModelSpec.effort`, and the
single `agents: BTreeMap<String, AgentManifest>` that unifies agents and
subagents on `Recipe`.

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
- [02-pi-compatibility.md](./02-pi-compatibility.md) - the detailed pi.recipes deltas.
- [03-syntropic137-mapping.md](./03-syntropic137-mapping.md) - how a consumer runs a recipe.
