# Agent Recipe Standard

**ID:** `EXP-V1-0004`
**Type:** Experiment
**Slug:** `agent-recipe`
**Version:** `0.1.0`

⚠️ **EXPERIMENTAL**: This standard is in incubation and may change significantly before promotion.

A declarative, harness-neutral **directory shape** for an agent recipe: what agent(s) to run
(harness, model, reasoning effort, skills, system instructions), independent of where or how it
executes. Adopted from the `pi.recipes` shape. See [docs/00_overview.md](docs/00_overview.md)
(pending update) for a quick tour and [docs/01_spec.md](docs/01_spec.md) for the normative
specification.

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

## Index

- [experiment.toml](experiment.toml)
- [Overview](docs/00_overview.md)
- [Specification](docs/01_spec.md)
- [Examples](examples/)
- [Tests](tests/)
- [Agent Skills](agents/skills/)

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate experiment EXP-V1-0004
cargo test -p apss-v1-0004-agent-recipe
```

