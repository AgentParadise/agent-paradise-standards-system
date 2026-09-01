# Architecture Decision Records

**ID:** `APS-V1-0003.AD01`
**Type:** Substandard
**Slug:** `adr`
**Version:** `1.0.0`
**Parent:** `APS-V1-0003` (Documentation and Context Engineering)

Enforces ADR naming, frontmatter, required keywords, context files, and
backlinking/dead-reference accuracy. Implemented as a feature-module of the
parent crate behind the `AD01` cargo feature (ADR-0002).

## Index

- [substandard.toml](substandard.toml)
- [Overview](docs/00_overview.md)
- [Specification](docs/01_spec.md)
- [Templates](templates/)

## Implementation

The validator and scaffold logic live in the parent crate at
`src/substandards/adr.rs` behind the `AD01` feature.

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard APS-V1-0003.AD01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
