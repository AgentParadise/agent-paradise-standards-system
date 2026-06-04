# Examples — Code Topology and Coupling Analysis

This directory contains sample artifacts demonstrating the Code Topology standard.

## Sample Topology (`sample-topology/`)

A complete `.topology/` artifact set for a hypothetical polyglot project with:
- **5 modules**: `auth`, `crypto`, `api`, `db`, `utils`
- **2 languages**: Rust, TypeScript
- **8 functions** with varying complexity levels

### Directory Structure

```
sample-topology/
├── manifest.toml              # Analysis metadata
├── metrics/
│   ├── functions.json         # Per-function complexity metrics
│   ├── files.json             # Per-file aggregates
│   └── modules.json           # Per-module aggregates + Martin's metrics
├── graphs/
│   ├── call-graph.json        # Function call relationships
│   ├── dependency-graph.json  # Module dependencies
│   └── coupling-matrix.json   # Coupling coefficients + 3D positions
└── snapshots/
    └── 2025-12-15.json        # Historical snapshot
```

### Key Metrics in Sample

| Module | Cyclomatic | Cognitive | Instability | Distance |
|--------|------------|-----------|-------------|----------|
| auth   | 11         | 16        | 0.5         | 0.4      |
| crypto | 6          | 7         | 0.33        | 0.67 ⚠️  |
| api    | 18         | 26        | 1.0         | 0.0 ✅   |
| db     | 5          | 7         | 0.5         | 0.3      |
| utils  | 1          | 0         | 0.0         | 1.0 ⚠️   |

- **api**: Fully unstable (I=1.0) but on main sequence (D=0.0) — good!
- **crypto**: In "Zone of Pain" (concrete + stable) — may be rigid
- **utils**: In "Zone of Pain" — many depend on it, hard to change

### Hotspots

Functions with highest complexity:

1. `typescript:api/handlers::processRequest` — CC=12, Cog=18 ⚠️
2. `rust:auth::validator::validate_token` — CC=8, Cog=12
3. `typescript:api/handlers::validateInput` — CC=6, Cog=8

### Coupling Matrix

```
        auth  crypto  api   db    utils
auth    1.00  0.75    0.60  0.20  0.40
crypto  0.75  1.00    0.25  0.10  0.15
api     0.60  0.25    1.00  0.55  0.35
db      0.20  0.10    0.55  1.00  0.30
utils   0.40  0.15    0.35  0.30  1.00
```

Observations:
- `auth` and `crypto` are tightly coupled (0.75)
- `api` and `db` are moderately coupled (0.55)
- `crypto` is relatively isolated from `db` (0.10)

## Usage

### Load in Rust

```rust
use std::fs;
use serde_json;

// Load coupling matrix
let content = fs::read_to_string("sample-topology/graphs/coupling-matrix.json")?;
let matrix: serde_json::Value = serde_json::from_str(&content)?;
println!("Modules: {:?}", matrix["modules"]);
```

### Render with 3D Projector

```bash
topology project \
  --projector 3d-force \
  --topology examples/sample-topology \
  --format html \
  --output coupling.html
```

### Validate Against Proto Schema

```bash
# Using buf (if available)
buf lint proto/
buf breaking proto/ --against .git#branch=main
```

## Creating Your Own Samples

1. Copy `sample-topology/` as a template
2. Modify metrics to match your test case
3. Ensure coupling matrix is symmetric with diagonal = 1.0
4. Run validation tests to verify format compliance
