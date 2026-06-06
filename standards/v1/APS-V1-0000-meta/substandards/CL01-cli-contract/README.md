# CLI Contract

**ID:** `APS-V1-0000.CL01`
**Type:** Substandard
**Slug:** `cli-contract`
**Version:** `1.1.0`

## Index

- [substandard.toml](substandard.toml)
- [Specification](docs/01_spec.md)
- [Examples](examples/)
- [Tests](tests/)
- [Agent Skills](agents/skills/)

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard APS-V1-0000.CL01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
