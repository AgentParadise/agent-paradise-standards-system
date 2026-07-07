# Agent Recipe Standard Tests

Tests for the experimental standard.

- Unit tests for the directory-shape schema types and the `load_recipe_dir`
  loader live alongside the implementation in `src/schema.rs`.
- `tests/fixtures/` contains real recipe directories used by those loader
  tests (a valid recipe, a recipe missing `recipe.yaml`, one with a dangling
  `default_agent`, one with a malformed `agents/*.yaml`, and a minimal
  recipe with no optional `skills/`/`SYSTEM.md`).
- The directory-level validator (`validate_recipe_dir`, Task 2 of the
  directory-recipe rework plan) will add its own conformance test walking
  `examples/valid/` and `examples/invalid/`, mirroring this crate's prior
  single-file conformance suite.

## Running Tests

```bash
cargo test -p apss-v1-0004-agent-recipe
```
