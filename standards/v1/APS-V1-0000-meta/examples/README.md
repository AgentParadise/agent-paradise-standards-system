# APS-V1-0000 Meta-Standard Examples

Minimal valid metadata files for each package type. Copy these as starting points.

## Available Examples

| Example | Description |
|---------|-------------|
| [`minimal-standard.toml`](minimal-standard.toml) | Minimal valid `standard.toml` for official standards |
| [`minimal-substandard.toml`](minimal-substandard.toml) | Minimal valid `substandard.toml` with parent reference |
| [`minimal-experiment.toml`](minimal-experiment.toml) | Minimal valid `experiment.toml` for incubating standards |

## Package Structure Reference

### Standard / Experiment (§5.1)

```
APS-V1-XXXX-slug/
├── standard.toml       # or experiment.toml
├── Cargo.toml
├── src/lib.rs
├── docs/01_spec.md
├── examples/           # MUST have content
├── tests/              # MUST have test coverage
└── agents/skills/      # MUST have skills or README
```

### Substandard (§5.2 — reduced requirements)

```
XXXX-YY01-slug/
├── substandard.toml
├── Cargo.toml
├── src/lib.rs          # Inline tests count as coverage
└── docs/01_spec.md     # Agent-readable: what it consumes/produces
```
