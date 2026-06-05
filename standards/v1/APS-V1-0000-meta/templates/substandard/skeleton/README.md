# {{name}}

**ID:** `{{id}}`
**Type:** Substandard
**Slug:** `{{slug}}`
**Version:** `{{version}}`

## Index

- [substandard.toml](substandard.toml)
- [Specification](docs/01_spec.md)

## Validation

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate substandard {{id}}
```

