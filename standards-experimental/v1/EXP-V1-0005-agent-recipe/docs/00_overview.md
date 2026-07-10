# Agent Recipe Standard - Overview

## What is this?

**EXP-V1-0005** defines a declarative, harness-neutral shape for an **agent recipe**: a *directory* describing *what agent(s) to run* (which harness, which model, which skills, which system instructions) without any knowledge of *where* or *how* execution happens.

A recipe is a directory, not a single file:

```text
pr-reviewer/
  recipe.yaml            # marker + metadata: name, version, default_agent
  agents/
    main.yaml            # one agent: harness, model, skills, system_instructions, tools, subagents
    reviewer.yaml        # another agent (a subagent referenced by main)
  skills/                # optional: bundled skill packages
    code-review/
  SYSTEM.md              # optional: shared base instructions
```

The presence of `recipe.yaml` is the marker that says "this directory is a recipe".

## Credits / Prior Art

This standard is **derived from [pi.recipes](https://introspection.dev)**, the recipe format from the **introspection.dev** project (reference implementation: [introspection-recipes/pi-codex](https://github.com/introspection-recipes/pi-codex)). pi.recipes contributed the core idea: capture an agent run as a reusable, declarative artifact (agent, model, skills, instructions) instead of reconstructing it from CLI flags each time. EXP-V1-0005 generalizes that into a harness-neutral, language-neutral *directory* standard. See [04-rationale-and-prior-art.md](./04-rationale-and-prior-art.md) for the full "how we got here" story and [02-pi-compatibility.md](./02-pi-compatibility.md) for the deliberate deltas.

## Why does it matter?

As agent orchestration spreads across multiple harnesses (Claude Code, Codex, and others to come), tooling needs a stable, harness-neutral way to say "run this agent, configured this way" that does not hard-code any one harness's flags or SDK shape. This standard provides:

1. **A directory contract** - the same recipe directory works whether the consumer targets Claude Code or Codex, and a single recipe can mix harnesses per agent.
2. **Forward compatibility** - the `agent` enum is designed to grow (`opencode`, `gemini`, ...) without breaking existing recipes.
3. **Separation of concerns** - a recipe never contains task input, credentials, or infrastructure details, so it is safe to commit and diff.
4. **One loader, one validator** - downstream consumers depend on the same `load_recipe_dir` the validator is built on, so there is a single source of truth for the shape.

## The Triad: shape + validate + generate

This standard is a versioned Rust crate that implements the full meta-standard triad:

| Leg | Where | What |
|-----|-------|------|
| **Shape** | `src/schema.rs` | Typed structs (`RecipeManifest`, `AgentManifest`, `Recipe`) + the canonical loader `load_recipe_dir(path) -> Result<Recipe, RecipeLoadError>`. |
| **Validate** | `src/validate.rs` | `validate_recipe_dir(path) -> Diagnostics`, built on top of the loader (loading and validation share one code path), with stable per-rule error codes. |
| **Generate** | `src/generate.rs` + `templates/recipe/` | `scaffold_recipe(name, dest)` writes a conformant recipe directory. Generator output always validates (round-trip test). |

## CLI Surface

The recipe crate exposes its validator and generator through the composed development CLI:

```bash
# Scaffold a new conformant recipe directory
apss-dev run agent-recipe create my-recipe [--dir <parent>]

# Validate a recipe directory (exit 0 = clean, 1 = errors)
apss-dev run agent-recipe validate ./my-recipe
```

(aliases for the slug: `recipe`, `exp-v1-0005`). The separate `apss-dev v1 validate experiment EXP-V1-0005` command is a structural meta-standard check of the crate's own package layout and does not take a recipe-directory path.

## The Two Manifests

`recipe.yaml` (the root marker):

```yaml
name: pr-reviewer
version: 0.1.0
default_agent: main
```

`agents/<name>.yaml` (one per agent):

```yaml
name: main
agent: claude
model:
  name: anthropic/claude-opus-4-8
  effort: high
skills:
  - code-review
system_instructions:
  mode: append
  content: |
    Focus exclusively on correctness and security issues.
tools:
  - shell
subagents:
  - reviewer
```

| Field | Purpose |
|-------|---------|
| `recipe.yaml: name` | Identifier for the recipe |
| `recipe.yaml: default_agent` | The entry-point agent; must resolve to `agents/<name>.yaml` |
| `agents/*.yaml: agent` | Which harness runs this agent (`claude` \| `codex` in v1) - per agent |
| `agents/*.yaml: model.name` | Provider-qualified model id |
| `agents/*.yaml: model.effort` | Coarse reasoning effort: `low` \| `medium` \| `high` |
| `agents/*.yaml: skills` | Harness-agnostic skill references, resolved in listed order |
| `agents/*.yaml: system_instructions` | Optional append/replace system prompt text, merged with `SYSTEM.md` |
| `agents/*.yaml: tools` | Tool references (names only, no execution defined here) |
| `agents/*.yaml: subagents` | Names of other agents this agent may delegate to (validated-only in v1) |

## Where a Recipe Fits

A recipe is the core of a larger execution request (informative, not part of this standard):

```text
RunSpec = recipe (a directory path) + task + input_artifacts + credentials + observability + limits
```

A workspace or executor (living in another repository, for example Plan B's `itmux run`) consumes a `RunSpec`, calls `load_recipe_dir` on the recipe path, runs the `default_agent`, and produces a `RunResult`. This standard defines only `recipe`. See [docs/03-syntropic137-mapping.md](./03-syntropic137-mapping.md) for how a recipe directory maps onto an `AgentRunSpec`.

## Status

**Experimental** - This standard is in incubation. Feedback welcome!

### What's Working

- Directory shape defined (`docs/01_spec.md`)
- Rust reference types + `load_recipe_dir` loader with serde (de)serialization
- `validate_recipe_dir` producing structured `Diagnostics` with stable error codes
- Generator (`create`) with a generate -> validate round-trip guarantee
- Example recipe directories (valid and invalid) for conformance testing

### Next Steps

1. Gather feedback from consumers in other repositories (e.g. `itmux run`, Plan B)
2. Promote subagents from validated-only to executed (multi-agent orchestration)
3. Add JSON Schema artifact generation for editor tooling
4. Define the full `RunSpec` envelope as a follow-on standard once a real consumer exists
5. Iterate toward promotion

## Learn More

- Read the [full specification](./01_spec.md)
- See the [pi.recipes compatibility notes](./02-pi-compatibility.md)
- See the [Syntropic137 mapping](./03-syntropic137-mapping.md)
- Read the [rationale and prior art](./04-rationale-and-prior-art.md)
- Check out [examples](../examples/)
- See [agent skills](../agents/skills/)

---

*This is an experimental standard. It may change significantly before promotion to official status.*
