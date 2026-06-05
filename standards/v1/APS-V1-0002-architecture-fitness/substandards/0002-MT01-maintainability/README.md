# Maintainability Dimension

**ID:** `APS-V1-0002.MT01`
**Type:** Substandard
**Slug:** `maintainability`
**Version:** `1.0.0`

McCabe / SonarSource / Halstead complexity governance over function-level metrics. Active per ADR 0002.

## Index

- [substandard.toml](substandard.toml)
- [Specification](docs/01_spec.md)
- [Tests](tests/)
- Engine: parent crate at `standards/v1/APS-V1-0002-architecture-fitness/src/lib.rs`. This substandard publishes `DEFAULT_RULES_TOML`; the engine consumes it.

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard APS-V1-0002.MT01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
