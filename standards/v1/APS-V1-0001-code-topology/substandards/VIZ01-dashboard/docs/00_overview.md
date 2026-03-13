# VIZ01-dashboard: Topology Visualization Dashboard

## Overview

This substandard provides interactive HTML visualizations for code topology data. Each visualization offers a different perspective on the codebase structure, coupling relationships, and health metrics.

## Visualization Types

| Type | Description | Best For |
|------|-------------|----------|
| **3D Force-Directed** | Interactive 3D graph with coupling-based clustering | Exploring module relationships |
| **CodeCity** | 3D city metaphor with buildings representing modules | Visualizing complexity distribution |
| **Package Clusters** | 2D force-directed graph of package relationships | Understanding cross-package coupling |
| **VSA Diagram** | Matrix of feature slices vs. architectural layers | Analyzing vertical slice architecture |

## Usage

### Generate All Visualizations

```bash
aps run topology viz --type all
open .topology/viz/index.html
```

### Generate Specific Visualization

```bash
aps run topology viz --type 3d
aps run topology viz --type codecity
aps run topology viz --type clusters
aps run topology viz --type vsa
```

## Features

- **Self-contained HTML** — No server required, open directly in browser
- **Interactive controls** — Pan, zoom, rotate, filter
- **Coupling threshold filter** — Focus on strong relationships
- **Health indicators** — Color-coded by module health score
- **Tooltips** — Detailed metrics on hover
- **Sidebar navigation** — Quick access to modules by coupling strength

## Data Requirements

All visualizations require topology artifacts to be generated first:

```bash
aps run topology analyze .
```

Required files:
- `.topology/metrics/modules.json` — Module metadata and metrics
- `.topology/graphs/coupling-matrix.json` — Coupling relationships
