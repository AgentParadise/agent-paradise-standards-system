# EXP-V1-0001.3D01 — 3D Force-Directed Coupling Visualization

## Overview

This projector renders the coupling matrix from code topology artifacts as an **interactive 3D visualization** where tightly coupled modules cluster together in 3D space.

## Key Features

- **Force-directed layout** — Uses physics simulation where coupling strength acts as spring force
- **Deterministic positions** — Saves calculated positions for reproducible renders
- **Multiple output formats** — WebGL scenes, GLTF models, self-contained HTML viewers
- **Metric-driven styling** — Node size reflects complexity, color reflects instability

## Visual Mapping

| Data | Visual |
|------|--------|
| Module | Sphere node |
| Coupling strength | Edge thickness + spring force |
| Cyclomatic complexity | Node size |
| Instability | Node color (red=unstable, blue=stable) |

## Quick Start

```bash
# Generate 3D visualization
topology project --projector 3d-force --format html --output coupling.html

# Open in browser
open coupling.html
```

## Example Output

The visualization shows:
- **Tightly coupled clusters** — Modules with high coupling are pulled together
- **Isolated modules** — Low-coupling modules float away from clusters
- **Architectural zones** — Natural groupings emerge from the force simulation

## Status

⚠️ **EXPERIMENTAL** — This substandard is in active development.

