# EXP-V1-0001.3D01 — 3D Force-Directed Coupling Visualization (Specification)

**Version**: 0.1.0  
**Status**: Experimental  
**Parent**: EXP-V1-0001 (Code Topology and Coupling Analysis)

---

## 1. Scope

This substandard specifies the **3D Force-Directed Projector** for visualizing code coupling from topology artifacts.

### 1.1 Required Artifacts

This projector REQUIRES:
- `graphs/coupling-matrix.json` — Module coupling coefficients
- `metrics/modules.json` — Module-level metrics (RECOMMENDED)

### 1.2 Output Formats

| Format | MIME Type | Description |
|--------|-----------|-------------|
| `webgl` | `application/json` | Three.js-compatible scene description |
| `json` | `application/json` | Same as WebGL (alias) |
| `html` | `text/html` | Self-contained HTML with embedded viewer |
| `gltf` | `model/gltf+json` | GLTF 3D model (future) |

---

## 2. Force-Directed Algorithm

### 2.1 Overview

The layout algorithm simulates a physical system where:
- **Nodes** (modules) repel each other
- **Edges** (coupling) act as springs pulling coupled modules together
- **Simulation** iterates until equilibrium or max iterations

### 2.2 Force Calculations

#### Repulsion Force (between all node pairs)

```
F_repulsion = repulsion_constant / distance²
direction = normalize(node_a.position - node_b.position)
```

#### Attraction Force (along edges)

```
F_attraction = attraction_constant × coupling_strength × distance
direction = normalize(node_b.position - node_a.position)
```

#### Position Update

```
velocity += sum(forces) × delta_time
velocity *= damping  # 0.9 typical
position += velocity × delta_time
```

### 2.3 Determinism

To ensure reproducible layouts:

1. **Seed** — Random number generator is seeded (default: 42)
2. **Save positions** — Final positions are written to `coupling-matrix.json`
3. **Load positions** — If positions exist, use them as initial state

---

## 3. Visual Encoding

### 3.1 Node Properties

| Property | Data Source | Default |
|----------|-------------|---------|
| **Position** | Force simulation | Calculated |
| **Size** | `sqrt(total_cyclomatic) × nodeScale` | 1.0 |
| **Color** | Instability (0=blue, 1=red) | `#8040ff` |
| **Label** | Module ID | — |

### 3.2 Edge Properties

| Property | Data Source | Default |
|----------|-------------|---------|
| **Width** | `coupling_strength × 2.0` | 1.0 |
| **Color** | Coupling strength (dim→bright) | `#808080` |
| **Opacity** | `coupling_strength` | 1.0 |

### 3.3 Color Schemes

| Scheme | Description |
|--------|-------------|
| `instability` | Red (unstable) → Blue (stable) |
| `complexity` | Red (high) → Green (low) |
| `language` | Colors by primary language |
| `custom` | User-provided colors |

---

## 4. Configuration Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ForceDirectedConfig",
  "type": "object",
  "properties": {
    "nodeScale": {
      "type": "number",
      "default": 1.0,
      "description": "Scale factor for node sizes"
    },
    "minEdgeStrength": {
      "type": "number",
      "default": 0.1,
      "minimum": 0,
      "maximum": 1,
      "description": "Minimum coupling strength to render edge"
    },
    "iterations": {
      "type": "integer",
      "default": 300,
      "description": "Force simulation iterations"
    },
    "repulsion": {
      "type": "number",
      "default": 100.0,
      "description": "Repulsion constant between nodes"
    },
    "attraction": {
      "type": "number",
      "default": 0.5,
      "description": "Attraction constant along edges"
    },
    "seed": {
      "type": "integer",
      "default": 42,
      "description": "Random seed for initial positions"
    },
    "colorScheme": {
      "type": "string",
      "enum": ["instability", "complexity", "language", "custom"],
      "default": "instability"
    }
  }
}
```

---

## 5. WebGL Scene Format

### 5.1 Scene Structure

```json
{
  "format": "topology-webgl/v1",
  "camera": {
    "position": [0, 5, 10],
    "target": [0, 0, 0],
    "up": [0, 1, 0]
  },
  "nodes": [
    {
      "id": "auth",
      "label": "auth",
      "position": [1.2, 3.4, 0.5],
      "size": 1.5,
      "color": "#ff6b6b",
      "metrics": {
        "cyclomatic": 56,
        "cognitive": 72,
        "instability": 0.625,
        "function_count": 18
      }
    }
  ],
  "edges": [
    {
      "from": "auth",
      "to": "crypto",
      "strength": 0.85,
      "color": "#4ecdc4",
      "width": 1.7
    }
  ]
}
```

### 5.2 HTML Viewer Requirements

The self-contained HTML output MUST:

1. Include all dependencies via CDN (Three.js)
2. Support mouse/touch interaction (orbit controls)
3. Display node count and edge count
4. Render without server (file:// protocol)

---

## 6. Error Codes

| Code | Description |
|------|-------------|
| `TOPOLOGY_NOT_FOUND` | `.topology/` directory does not exist |
| `REQUIRED_FILE_MISSING` | `graphs/coupling-matrix.json` missing |
| `UNSUPPORTED_FORMAT` | Requested format not in supported list |
| `RENDER_FAILED` | Failed to generate output |

---

## 7. Future Enhancements

- [ ] GLTF export for external 3D viewers
- [ ] Animated layout evolution
- [ ] VR/AR support (WebXR)
- [ ] Clickable nodes with detail panels
- [ ] Time-travel through historical snapshots
- [ ] Cluster detection and highlighting

