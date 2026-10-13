# Agentic Workspace

**ID:** `EXP-V1-0006`  
**Type:** Experiment  
**Slug:** `workspace`  
**Version:** `0.2.0`

This experiment defines a provider-neutral launch manifest and observable
workspace lifecycle for agent execution. Since 0.2.0 it also defines the
operation contract at the provider port: operations, status, observation
pages, provider descriptors, finalization, results, and immutable references.
It is implemented first by the
Agentic Workspace Local and Docker adapters.

## Artifacts

- [Normative specification](docs/01_spec.md)
- [Launch manifest JSON Schema](schemas/workspace-launch.schema.json)
- [Operation contract JSON Schemas](schemas/) (spec Section 5.1)
- [Minimal manifest](examples/minimal/workspace-launch.json)
- [Success, failure, and behaviour fixtures](fixtures/)
- Rust types and semantic validation in `src/lib.rs`

## Validation

```bash
cargo test -p apss-v1-0006-workspace
cargo run -p aps-cli --bin apss-dev -- v1 validate experiment EXP-V1-0006
```

The experiment is not eligible for promotion until Local, Docker, and one
remote implementation pass the same conformance suite.
