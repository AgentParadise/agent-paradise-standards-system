# Legality Dimension

**ID:** `APS-V1-0002.LG01`
**Type:** Substandard
**Slug:** `legality`
**Version:** `1.0.0`

License compliance via adapters (`builtin:cargo-deny` or equivalent license scanner). Category policy (permissive, weak-copyleft, strong-copyleft, proprietary, unknown). Active per ADR 0003.

## Index

- [substandard.toml](substandard.toml)
- [Specification](docs/01_spec.md)
- Engine: parent crate at `standards/v1/APS-V1-0002-architecture-fitness/src/lib.rs`. Active dimensions enter strict-artifact mode: missing adapter output produces `PROMOTION_REQUIREMENT_UNMET` rather than silent skip.

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard APS-V1-0002.LG01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
