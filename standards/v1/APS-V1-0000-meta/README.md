# APS Meta-Standard

**ID:** `APS-V1-0000`
**Type:** Standard
**Slug:** `meta`
**Version:** `1.2.0`

## Index

- [standard.toml](standard.toml)
- [Specification](docs/01_spec.md)
- [Examples](examples/)
- [Tests](tests/)
- [Agent Skills](agents/skills/)

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate standard APS-V1-0000
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
