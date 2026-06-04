# APS Experimental Standards (V1)

This directory contains **experimental standards** for the APS V1 ecosystem.

## What are Experiments?

Experimental standards are incubating packages used for:

- Rapid iteration on new ideas
- Community feedback before commitment
- Proving out concepts before official support

## Key Rules

| Rule | Description |
|------|-------------|
| **Not Official** | Experiments are never enforced on downstream repositories |
| **Same Validation** | Experiments MUST pass the same validation as official standards |
| **Same Structure** | Experiments follow the same package structure as officials |
| **Promotion Path** | Experiments can be promoted to official after review |

## Package Structure

Experiments follow the same structure as official standards:

```
EXP-V1-XXXX-<slug>/
├── experiment.toml      # Metadata (not standard.toml)
├── Cargo.toml           # Rust crate manifest
├── src/
│   └── lib.rs           # Standard trait implementation
├── docs/
│   └── 01_spec.md       # Normative specification
├── examples/
├── tests/
├── agents/
│   └── skills/
└── templates/           # Optional
```

## Metadata: `experiment.toml`

```toml
schema = "aps.experiment/v1"

[experiment]
id = "EXP-V1-0001"
name = "My Experiment"
slug = "my-experiment"
version = "0.1.0"
category = "technical"

[aps]
aps_major = "v1"

[ownership]
maintainers = ["AgentParadise"]
```

## Creating an Experiment

```bash
# Scaffold a new experiment
aps v1 create experiment my-experiment

# Validate your experiment
aps v1 validate experiment EXP-V1-0001
```

## Promotion Workflow

After peer review and security audit:

```bash
# Promote experiment to official standard
aps v1 promote EXP-V1-0001 --to APS-V1-0005
```

This will:

1. Copy the experiment to `standards/v1/APS-V1-0005-<slug>/`
2. Transform `experiment.toml` → `standard.toml`
3. Update the original experiment with promotion metadata

### Post-Promotion

The original experiment remains with added metadata:

```toml
# Added to experiment.toml after promotion
[promotion]
promoted_to = "APS-V1-0005"
promoted_at = "2025-12-15"
notes = "Promoted after security review"
```

## Guidelines for Experiments

1. **Version at 0.x.x** — Experiments typically start at `0.1.0`
2. **Document clearly** — Explain what you're testing and why
3. **Gather feedback** — Use issues/discussions for community input
4. **Iterate quickly** — Don't be afraid to make breaking changes (it's experimental!)
5. **Aim for promotion** — The goal is to prove the concept and promote

## What Happens After Promotion?

- The experiment stays in `standards-experimental/v1/` for historical reference
- The official standard in `standards/v1/` becomes the active version
- Downstream consumers should migrate to the official standard

