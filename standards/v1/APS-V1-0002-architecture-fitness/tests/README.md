# Tests — Architecture Fitness Functions

## Test Strategy

Phase 2 will add tests migrated from EXP-V1-0003 plus new test categories.

### Planned Test Categories

#### Unit Tests (`src/lib.rs`)
- Threshold evaluation (carried forward from EXP-V1-0003)
- Dependency rule evaluation
- Structural rule evaluation
- Dimensional scoring
- Composite fitness calculation
- Adapter normalization

#### Integration Tests (`tests/`)
- Multi-dimensional validation against real topology artifacts
- Composite scoring with weighted dimensions
- Adapter integration (VSA, security scanners)
- Report generation with dimensional + composite sections
- Exception ratcheting across dimensions
- Backward compatibility with EXP-V1-0003 fitness.toml files

## Running Tests

```bash
cargo test -p architecture-fitness
```

## Test Fixtures

Example artifacts in `examples/` serve as test fixtures for integration tests.
