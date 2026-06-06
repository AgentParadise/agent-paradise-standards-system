# {{name}}

**ID:** `{{id}}`
**Type:** Experiment
**Slug:** `{{slug}}`
**Version:** `{{version}}`

## Index

- [experiment.toml](experiment.toml)
- [Specification](docs/01_spec.md)
- [Examples](examples/)
- [Tests](tests/)
- [Agent Skills](agents/skills/)

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate experiment {{id}}
```

