# Requirements

**ID:** `APS-V1-0003.RQ01`
**Type:** Substandard (optional doc type)
**Slug:** `requirements`
**Version:** `0.1.0`
**Parent:** `APS-V1-0003` (Documentation and Context Engineering)

Validates a system's requirements document: what the system must be and
do, with numbered requirements, measures, check status, user stories, a
"later and out of scope" list, and a change log. Agents read it before
building or changing the system and apply the requirements gate.
Optional: a project without a requirements document passes. Implemented
as a feature-module of the parent crate behind the `RQ01` cargo feature
(ADR-0002).

## Index

- [substandard.toml](substandard.toml)
- [Overview](docs/00_overview.md)
- [Specification](docs/01_spec.md)
- [Worked example](../../examples/requirements.md)

## Implementation

The validator lives in the parent crate at
`src/substandards/requirements.rs` behind the `RQ01` feature.

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard APS-V1-0003.RQ01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
