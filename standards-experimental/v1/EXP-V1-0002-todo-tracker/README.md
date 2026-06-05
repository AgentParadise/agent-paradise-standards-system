# TODO/FIXME Tracker and Issue Validator

**ID:** `EXP-V1-0002`
**Type:** Experiment
**Slug:** `todo-tracker`
**Version:** `0.1.0`

## Index

- [experiment.toml](experiment.toml)
- [Specification](docs/01_spec.md)
- [Examples](examples/)
- [Tests](tests/)
- [Agent Skills](agents/skills/)

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate experiment EXP-V1-0002
```

Run the full repository validation with:

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```
