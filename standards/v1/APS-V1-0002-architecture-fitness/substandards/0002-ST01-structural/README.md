# Structural Integrity Dimension

**ID:** `APS-V1-0002.ST01`
**Type:** Substandard
**Slug:** `structural`
**Version:** `1.0.0`

Structural-pattern catalog (forbidden_import, required_import, layer_enforcement) evaluated over the topology dependency graph. Active per ADR 0003. CK class-level metrics (DIT, CBO, LCOM) are a scoped follow-on awaiting a class-level analyzer.

## Index

- [substandard.toml](substandard.toml)
- [Specification](docs/01_spec.md)
- Engine: parent crate's `evaluate_structural_rule` in `standards/v1/APS-V1-0002-architecture-fitness/src/lib.rs` maps the pattern catalog onto the dependency-graph evaluator. Integration tests live at `standards/v1/APS-V1-0002-architecture-fitness/tests/structural_eval.rs`.

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard APS-V1-0002.ST01
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
