# Agent Recipe Standard Tests

Tests for the experimental standard.

- Unit tests for the directory-shape schema types and the `load_recipe_dir`
  loader live alongside the implementation in `src/schema.rs`; unit tests for
  the directory validator live in `src/validate.rs`.
- `tests/fixtures/` contains real recipe directories used by those unit tests
  (a valid recipe, a recipe missing `recipe.yaml`, one with a dangling
  `default_agent`, one with a dangling `subagents` entry, one with a malformed
  `agents/*.yaml`, and a minimal recipe with no optional `skills/`/`SYSTEM.md`).
- `tests/conformance_test.rs` walks the canonical example directories under
  `examples/valid/` (asserting zero errors) and `examples/invalid/` (asserting
  each case emits the error code declared in its `README.md` `# Expect:`
  header), so the shipped examples never drift from the validator's behavior.

## Running Tests

```bash
cargo test -p apss-v1-0004-agent-recipe
```
