# Code Topology and Coupling Analysis

**ID:** `APS-V1-0001`
**Type:** Standard
**Slug:** `code-topology`
**Version:** `0.1.0`

## Index

- [standard.toml](standard.toml)
- [Specification](docs/01_spec.md)
- [Examples](examples/)
- [Tests](tests/)
- [Agent Skills](agents/skills/)

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate standard APS-V1-0001
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
