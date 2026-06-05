# Accessibility Dimension

**ID:** `APS-V1-0002.AC01`
**Type:** Substandard
**Slug:** `accessibility`
**Version:** `1.0.0`

WCAG 2.1 AA compliance via adapters (axe-core / pa11y reference normalizers). Active per ADR 0003, opt-in by default because most backends and CLIs do not emit a11y artifacts.

## Index

- [substandard.toml](substandard.toml)
- [Specification](docs/01_spec.md)
- Engine: parent crate at `standards/v1/APS-V1-0002-architecture-fitness/src/lib.rs`. Active dimensions enter strict-artifact mode: when the dimension is enabled and the adapter output is missing the rule fails with `PROMOTION_REQUIREMENT_UNMET`.

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard APS-V1-0002.AC01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
