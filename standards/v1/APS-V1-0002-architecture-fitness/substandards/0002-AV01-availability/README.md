# Availability Dimension

**ID:** `APS-V1-0002.AV01`
**Type:** Substandard
**Slug:** `availability`
**Version:** `1.0.0`

Availability / SLO / error-budget governance via adapters. Incubating: thresholds are project-specific and the standard cannot publish a universal R4 citation. Promotion to active requires a per-project ADR setting concrete SLOs.

## Index

- [substandard.toml](substandard.toml)
- [Specification](docs/01_spec.md)
- Engine: parent crate at `standards/v1/APS-V1-0002-architecture-fitness/src/lib.rs`. Incubating dimensions downgrade error severities to warning and skip silently on missing artifacts. Composite excludes them unless `system_fitness.include_incubating = true`.

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard APS-V1-0002.AV01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
