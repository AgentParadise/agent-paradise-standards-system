# Agent Recipe Standard

**ID:** `EXP-V1-0005`
**Type:** Experiment
**Slug:** `agent-recipe`
**Version:** `0.2.0`

⚠️ **EXPERIMENTAL**: This standard is in incubation and may change significantly before promotion.

A declarative, harness-neutral **directory shape** for an agent recipe: what agent(s) to run
(harness, model, reasoning effort, skills, system instructions), independent of where or how it
executes. Adopted from the `pi.recipes` shape. See [docs/00_overview.md](docs/00_overview.md)
for a quick tour and [docs/01_spec.md](docs/01_spec.md) for the normative specification.

## Credits / Prior Art

The recipe-directory concept is **derived from [pi.recipes](https://introspection.dev)**, the
recipe format from the **introspection.dev** project (reference implementation:
[introspection-recipes/pi-codex](https://github.com/introspection-recipes/pi-codex)). pi.recipes
established the core idea this standard builds on - an agent run captured as a reusable, declarative
artifact (agent, model, skills, instructions) rather than reconstructed from CLI flags each time.
EXP-V1-0005 generalizes that idea into a harness-neutral, language-neutral directory standard, and
is **inspired by pi.recipes, not compatible with it** - semantics changed (enforced `tools`, the
`harness` field) that make a pi recipe fail to load here. See
[docs/04-rationale-and-prior-art.md](docs/04-rationale-and-prior-art.md) for the full derivation
story and the deliberate deltas.

A recipe is a directory (the presence of `recipe.yaml` is the marker):

```text
pr-reviewer/
  recipe.yaml            # name, version, default_agent
  agents/
    main.yaml             # harness, model, skills, system_instructions, tools, subagents
    reviewer.yaml
  skills/
    code-review/
  SYSTEM.md
```

See [examples/valid/pr-reviewer/](examples/valid/pr-reviewer/) for this recipe as a real directory,
and `agent_recipe::load_recipe_dir` (in `src/schema.rs`) for the canonical loader.

## The Triad

- **Shape** - `src/schema.rs`: typed structs + `load_recipe_dir(path)`.
- **Validate** - `src/validate.rs`: `validate_recipe_dir(path) -> Diagnostics`, built on the loader.
- **Generate** - `src/generate.rs` + `templates/recipe/`: `scaffold_recipe(name, dest)`; output always validates.

## CLI

```bash
# Scaffold a new conformant recipe directory
cargo run -p aps-cli --bin apss-dev -- run agent-recipe create my-recipe [--dir <parent>]

# Validate a recipe directory (exit 0 = clean, 1 = errors)
cargo run -p aps-cli --bin apss-dev -- run agent-recipe validate ./my-recipe
```

(slug aliases: `recipe`, `exp-v1-0005`)

## Index

- [experiment.toml](experiment.toml)
- [Overview](docs/00_overview.md)
- [Specification](docs/01_spec.md)
- [Syntropic137 Mapping](docs/03-syntropic137-mapping.md)
- [Rationale and Prior Art](docs/04-rationale-and-prior-art.md)
- [Examples](examples/)
- [Tests](tests/)
- [Agent Skills](agents/skills/)

## Validation

```bash
# Structural (meta-standard) check of this crate's package layout:
cargo run -p aps-cli --bin apss-dev -- v1 validate experiment EXP-V1-0005
cargo test -p apss-v1-0005-agent-recipe
```
