# Distribution & Installation

**ID:** `APS-V1-0000.DI01`
**Type:** Substandard
**Slug:** `distribution`
**Version:** `1.0.0`

## Index

- [substandard.toml](substandard.toml)
- [Specification](docs/01_spec.md)

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard APS-V1-0000.DI01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
