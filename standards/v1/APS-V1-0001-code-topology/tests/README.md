# Tests — Code Topology and Coupling Analysis

## Test Categories

### Unit Tests (`src/lib.rs`)

Core type and metric calculation tests:

- `test_creation` — Standard instantiation
- `test_error_codes_defined` — Error codes are non-empty
- `test_halstead_calculation` — Halstead derived metrics
- `test_halstead_zero_handling` — Division by zero protection
- `test_martin_metrics_calculation` — Martin's metrics (I, A, D)
- `test_martin_metrics_zone_of_pain` — Zone detection
- `test_coupling_matrix_*` — Matrix creation, symmetry, validation
- `test_function_metrics_default` — Default metric values
- `test_visibility_serialization` — Enum serialization

### Integration Tests (`tests/artifact_parsing.rs`)

Artifact file parsing and validation:

- `test_parse_functions_json` — Parse functions.json from sample
- `test_parse_modules_json` — Parse modules.json with Martin's metrics
- `test_parse_coupling_matrix_json` — Parse coupling matrix
- `test_coupling_matrix_has_layout` — Verify layout positions
- `test_all_modules_in_coupling_matrix` — Cross-file consistency
- `test_values_in_valid_range` — Coupling values in [0, 1]

## Running Tests

```bash
# Run all tests for code-topology
cargo test -p code-topology

# Run only integration tests
cargo test -p code-topology --test artifact_parsing

# Run with output
cargo test -p code-topology -- --nocapture
```

## Test Fixtures

Sample artifacts are in `examples/sample-topology/`:

```
examples/sample-topology/
├── manifest.toml
├── metrics/
│   ├── functions.json      # 8 functions
│   ├── files.json          # 5 files
│   └── modules.json        # 5 modules
├── graphs/
│   ├── call-graph.json     # 8 nodes, 6 edges
│   ├── dependency-graph.json
│   └── coupling-matrix.json  # 5x5 with layout
└── snapshots/
    └── 2025-12-15.json
```

## Adding Tests

When adding new tests:

1. **Unit tests** — Add to `#[cfg(test)]` module in `src/lib.rs`
2. **Integration tests** — Add to `tests/*.rs`
3. **Sample artifacts** — Add to `examples/sample-topology/`

Ensure all tests pass before committing:

```bash
cargo test -p code-topology -p code-topology-3d
```
