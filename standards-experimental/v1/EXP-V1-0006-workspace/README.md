# Agentic Workspace

**ID:** `EXP-V1-0006`  
**Type:** Experiment  
**Slug:** `workspace`  
**Version:** `0.1.0`

This experiment defines a provider-neutral launch manifest and observable
workspace lifecycle for agent execution. It is implemented first by the
Agentic Workspace Local and Docker adapters.

## Artifacts

- [Normative specification](docs/01_spec.md)
- [Launch manifest JSON Schema](schemas/workspace-launch.schema.json)
- [Minimal manifest](examples/minimal/workspace-launch.json)
- Rust types and semantic validation in `src/lib.rs`

## Validation

```bash
cargo test -p apss-v1-0006-workspace
cargo run -p aps-cli --bin apss-dev -- v1 validate experiment EXP-V1-0006
```

The experiment is not eligible for promotion until Local, Docker, and one
remote implementation pass the same conformance suite.
