# VIZ01-dashboard: Specification

## 1. Visualization Types

### 1.1 3D Force-Directed Graph

**Purpose:** Visualize module coupling as an interactive 3D network.

**Key Features:**
- Nodes represent modules
- Edges represent coupling relationships
- Force-directed layout clusters tightly coupled modules
- Node size based on function count
- Node color based on health (distance from main sequence)
- Edge thickness based on coupling strength

**Interactions:**
- Drag to rotate camera
- Scroll to zoom
- Hover for tooltip with metrics
- Click to focus connections
- Sidebar for module list with coupling filter

### 1.2 CodeCity

**Purpose:** 3D city metaphor for visualizing codebase structure.

**Key Features:**
- Buildings represent modules
- Building height = cyclomatic complexity (log scale)
- Building width = function count
- Building color = health score
- Districts = feature slices

**Interactions:**
- Drag to rotate camera
- Right-click to pan
- Scroll to zoom
- Hover for module details

### 1.3 Package Clusters

**Purpose:** 2D graph showing package/slice relationships.

**Key Features:**
- Circles represent packages/slices
- Circle size = module count
- Circle color = average health
- Lines = coupling between packages
- Line thickness = coupling strength

**Interactions:**
- Drag to pan
- Scroll to zoom
- Click sidebar to focus package
- Coupling threshold slider

### 1.4 VSA Diagram

**Purpose:** Matrix visualization for Vertical Slice Architecture.

**Key Features:**
- Columns = feature slices
- Rows = architectural layers
- Cells = module count with health color
- Hover for module list

**Layers (default):**
- handlers, services, models, data, adapters, utils, core, examples, tests, other

## 2. Data Format

### 2.1 Input: modules.json

```json
[
  {
    "id": "module::path",
    "name": "path",
    "slice": "feature_name",
    "layer": "services",
    "health": 0.85,
    "health_label": "Good",
    "color": "#44dd77",
    "function_count": 12,
    "total_cyclomatic": 45,
    "total_cognitive": 32,
    "lines_of_code": 350,
    "ca": 3,
    "ce": 5
  }
]
```

### 2.2 Input: coupling-matrix.json

```json
{
  "schema_version": "2.0.0",
  "metric": "composite_coupling",
  "modules": ["mod_a", "mod_b", "mod_c"],
  "matrix": [
    [0.0, 0.5, 0.2],
    [0.3, 0.0, 0.8],
    [0.1, 0.4, 0.0]
  ]
}
```

### 2.3 Output: HTML Files

Each visualization produces a self-contained HTML file with:
- Embedded CSS styles
- Embedded JavaScript (Three.js or Canvas 2D)
- Embedded JSON data
- No external dependencies (CDN scripts only)

## 3. API

### 3.1 Rust Crate API

```rust
use code_topology_viz::{force_3d, codecity, clusters, vsa, index};

// 3D Force-Directed
let html = force_3d::generate(&scene_json, node_count, edge_count);

// CodeCity
let html = codecity::generate(&modules_json, &coupling_json);

// Package Clusters
let html = clusters::generate(&modules_json, &coupling_json);

// VSA Diagram
let html = vsa::generate(&modules_json);

// Dashboard Index
let html = index::generate(module_count, slice_count, avg_health);
```

### 3.2 CLI Integration

```bash
# Generate specific type
aps run topology viz --type 3d

# Generate all with dashboard
aps run topology viz --type all

# Custom output path
aps run topology viz --type codecity --output my-city.html
```

## 4. Health Color Scale

| Health Range | Color | Label |
|--------------|-------|-------|
| ≥ 80% | `#00ff88` | Excellent |
| ≥ 65% | `#44dd77` | Good |
| ≥ 50% | `#88cc55` | OK |
| ≥ 35% | `#ddaa33` | Warning |
| ≥ 20% | `#ff7744` | Poor |
| < 20% | `#ff3333` | Critical |

## 5. Browser Compatibility

Visualizations require a modern browser with:
- ES6 module support
- WebGL (for 3D visualizations)
- CSS Grid and Flexbox
- Tested on: Chrome 90+, Firefox 90+, Safari 14+, Edge 90+
