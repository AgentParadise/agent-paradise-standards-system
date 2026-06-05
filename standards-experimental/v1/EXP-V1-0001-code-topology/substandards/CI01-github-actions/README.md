# GitHub Actions CI Integration

**ID:** `EXP-V1-0001.CI01`
**Type:** Substandard
**Slug:** `github-actions`
**Version:** `0.1.0`

## Index

- [substandard.toml](substandard.toml)
- [Specification](docs/01_spec.md)
- [Examples](examples/)

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard EXP-V1-0001.CI01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
