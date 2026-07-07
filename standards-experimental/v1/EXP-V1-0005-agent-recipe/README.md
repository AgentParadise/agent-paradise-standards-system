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
- [pi.recipes Compatibility](docs/02-pi-compatibility.md)
- [Syntropic137 Mapping](docs/03-syntropic137-mapping.md)
- [Examples](examples/)
- [Tests](tests/)
- [Agent Skills](agents/skills/)

## Validation

```bash
# Structural (meta-standard) check of this crate's package layout:
cargo run -p aps-cli --bin apss-dev -- v1 validate experiment EXP-V1-0005
cargo test -p apss-v1-0005-agent-recipe
```

