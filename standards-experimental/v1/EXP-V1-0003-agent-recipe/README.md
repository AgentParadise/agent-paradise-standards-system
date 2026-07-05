# Agent Recipe Standard

**ID:** `EXP-V1-0003`
**Type:** Experiment
**Slug:** `agent-recipe`
**Version:** `0.1.0`

⚠️ **EXPERIMENTAL**: This standard is in incubation and may change significantly before promotion.

A declarative, harness-neutral schema for an **agent recipe**: what agent to run (harness, model,
reasoning effort, skills, system instructions), independent of where or how it executes. Adopted
from the `pi.recipes` shape. See [docs/00_overview.md](docs/00_overview.md) for a quick tour and
[docs/01_spec.md](docs/01_spec.md) for the normative specification.

```yaml
name: pr-reviewer
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
```

## Index

- [experiment.toml](experiment.toml)
- [Overview](docs/00_overview.md)
- [Specification](docs/01_spec.md)
- [Examples](examples/)
- [Tests](tests/)
- [Agent Skills](agents/skills/)

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate experiment EXP-V1-0003
cargo test -p apss-v1-0003-agent-recipe
```

