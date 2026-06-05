# {{name}}

**ID:** `{{id}}`
**Type:** Standard
**Slug:** `{{slug}}`
**Version:** `{{version}}`

## Index

- [standard.toml](standard.toml)
- [Specification](docs/01_spec.md)
- [Examples](examples/)
- [Tests](tests/)
- [Agent Skills](agents/skills/)

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate standard {{id}}
```

