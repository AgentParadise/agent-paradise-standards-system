# Agent Recipe Standard Tests

Tests for the experimental standard.

- Unit tests live alongside the implementation in `src/lib.rs` and cover
  each validation rule / error code in `docs/01_spec.md` section 5.
- `conformance_test.rs` in this directory is an integration test that walks
  every file under `examples/valid/` and `examples/invalid/` and asserts the
  expected pass/fail outcome, so the shipped examples never drift from the
  validator's actual behavior.

## Running Tests

```bash
cargo test -p apss-v1-0004-agent-recipe
```

