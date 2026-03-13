# Tests — 3D Force-Directed Projector

## Test Categories

### Unit Tests (`src/lib.rs`)

- `test_projector_creation` — Projector instantiation
- `test_supported_formats` — Format enumeration
- `test_config_schema` — JSON Schema generation
- `test_default_config` — Default configuration values
- `test_instability_color` — Color mapping correctness

### Integration Tests (Planned)

- [ ] Load sample coupling matrix and render scene
- [ ] Verify HTML output contains Three.js
- [ ] Test position determinism (same input → same output)
- [ ] Test edge filtering by minEdgeStrength

### Visual Regression Tests (Future)

- [ ] Screenshot comparison against reference images
- [ ] Node positioning stability across versions

## Running Tests

```bash
cd substandards/3D01-force-directed
cargo test
```

## Test Fixtures

Test fixtures should be placed in `tests/fixtures/`:

```
tests/
├── fixtures/
│   ├── coupling-matrix.json    # Sample coupling data
│   └── expected-scene.json     # Expected output
└── integration_test.rs
```

