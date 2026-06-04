# Examples — 3D Force-Directed Projector

## Sample Outputs

### `sample-output/`

Contains example renderings:

- `scene.json` — WebGL scene description
- `coupling.html` — Self-contained HTML viewer (when available)

## Usage Examples

### Basic HTML Generation

```bash
# From a project with .topology/ artifacts
topology project --projector 3d-force --format html --output viz.html
open viz.html
```

### Custom Configuration

```bash
# Larger nodes, filter weak edges
topology project \
  --projector 3d-force \
  --format html \
  --config '{"nodeScale": 2.0, "minEdgeStrength": 0.3}' \
  --output viz.html
```

### JSON Scene for Custom Rendering

```bash
# Export scene data for custom Three.js integration
topology project --projector 3d-force --format json --output scene.json
```

## Programmatic Usage

```rust
use code_topology_3d::{ForceDirectedProjector, ForceDirectedConfig};
use code_topology::{Projector, OutputFormat};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ForceDirectedConfig {
        node_scale: 1.5,
        iterations: 500,
        ..Default::default()
    };
    
    let projector = ForceDirectedProjector::with_config(config);
    let topology = projector.load(Path::new(".topology"))?;
    let html = projector.render(&topology, OutputFormat::Html, None)?;
    
    std::fs::write("coupling.html", html)?;
    Ok(())
}
```

