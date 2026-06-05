# Substandard Structure

**ID:** `APS-V1-0000.SS01`
**Type:** Substandard
**Slug:** `substandard-structure`
**Version:** `1.0.0`

## Index

- [substandard.toml](substandard.toml)
- [Specification](docs/01_spec.md)
- [Examples](examples/)
- [Tests](tests/)
- [Agent Skills](agents/skills/)

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard APS-V1-0000.SS01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
