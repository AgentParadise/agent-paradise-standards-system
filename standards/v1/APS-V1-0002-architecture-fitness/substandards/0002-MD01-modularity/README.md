# Modularity and Coupling Dimension

**ID:** `APS-V1-0002.MD01`
**Type:** Substandard
**Slug:** `modularity`
**Version:** `1.0.0`

Martin Ca / Ce / I / A / D package coupling governance over module-level metrics. Active per ADR 0002.

## Index

- [substandard.toml](substandard.toml)
- [Specification](docs/01_spec.md)
- [Tests](tests/)
- Engine: parent crate at `standards/v1/APS-V1-0002-architecture-fitness/src/lib.rs`. This substandard publishes `DEFAULT_RULES_TOML`; the engine consumes it.

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard APS-V1-0002.MD01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
