# Code Topology and Coupling Analysis — Overview

## What is this?

**APS-V1-0001** defines a standard format for capturing and visualizing code architecture across polyglot codebases. It enables:

- **Complexity tracking** — Cyclomatic, Cognitive, Halstead metrics per function/file/module
- **Coupling analysis** — Martin's metrics (Ca, Ce, Instability, Abstractness)
- **Architecture visualization** — Call graphs, dependency graphs, 3D coupling models

## Why does it matter?

As codebases scale (especially with thousands of AI agents contributing), maintaining architecture quality becomes critical. This standard provides:

1. **Committed artifacts** — Version-controlled topology snapshots
2. **Language-agnostic format** — Same structure for Rust, TypeScript, Python, C++
3. **AI-consumable data** — Structured JSON for agent reasoning
4. **Visual projectors** — 2D graphs, 3D models, diagrams

## Quick Example

After running analysis, you get:

```
.topology/
├── manifest.toml              # Metadata
├── metrics/
│   ├── functions.json         # Per-function complexity
│   ├── files.json             # Per-file aggregates
│   └── modules.json           # Martin's coupling metrics
├── graphs/
│   ├── call-graph.json        # Who calls whom
│   ├── dependency-graph.json  # Module dependencies
│   └── coupling-matrix.json   # For 3D visualization!
└── snapshots/
    └── 2025-12-15.json        # Historical snapshot
```

## Key Insight: The Coupling Matrix

The `coupling-matrix.json` is the secret sauce for 3D visualization:

```json
{
  "modules": ["auth", "api", "db", "utils"],
  "matrix": [
    [1.00, 0.75, 0.20, 0.40],
    [0.75, 1.00, 0.65, 0.30],
    [0.20, 0.65, 1.00, 0.15],
    [0.40, 0.30, 0.15, 1.00]
  ]
}
```

This directly maps to 3D positions:
- **Tightly coupled** modules → close together
- **Loosely coupled** modules → far apart

## Architecture

```
APS-V1-0001 (This Standard)
├── Artifact Format ← You are here
├── Metrics Definitions
└── Projector Interface

Substandards:
├── APS-V1-0001.3D01 — 3D Force-Directed Visualization
├── APS-V1-0001.VIZ01 — Dashboard Visualization
├── APS-V1-0001.VIZ01-mermaid — Mermaid Diagrams
├── APS-V1-0001.LANG01 — Rust Language Adapter
└── APS-V1-0001.CI01 — GitHub Actions Integration
```

## Getting Started

1. Read the [full specification](./01_spec.md)
2. Check out [examples](../examples/)
3. Implement an analyzer for your language
4. Build a projector visualization

## Status

✅ **Official** — Promoted from EXP-V1-0001. Supported languages: Rust, Python, TypeScript, TSX.

