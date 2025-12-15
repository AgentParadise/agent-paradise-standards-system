# APS-V1-0000 Tests

This directory contains tests for the meta-standard.

## Test Categories

### Unit Tests (`unit/`)

Test individual validation rules in isolation:

- Directory existence checks
- Metadata schema validation
- ID format validation

### Integration Tests (`integration/`)

Test complete validation workflows:

- Validate the meta-standard's own structure
- Validate example packages in `examples/`
- End-to-end CLI validation

## Running Tests

```bash
# Run all tests for this crate
cargo test -p aps-v1-0000-meta

# Run with verbose output
cargo test -p aps-v1-0000-meta -- --nocapture
```

## Test Requirements

Per the meta-standard spec (§11.2):

- Tests MUST validate the standard's structure and metadata
- Tests MUST validate at least one example under `examples/`
- CI MUST run these tests for official standards

