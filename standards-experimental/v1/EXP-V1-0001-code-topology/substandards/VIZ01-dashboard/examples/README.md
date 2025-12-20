# VIZ01-dashboard Examples

## Generating Visualizations

```bash
# First, analyze the codebase
aps run topology analyze .

# Generate all visualizations with dashboard
aps run topology viz --type all

# Open the dashboard
open .topology/viz/index.html
```

## Individual Visualizations

```bash
# 3D Force-Directed coupling graph
aps run topology viz --type 3d
open topology-3d.html

# CodeCity 3D city metaphor
aps run topology viz --type codecity
open codecity.html

# 2D Package Clusters
aps run topology viz --type clusters
open clusters.html

# VSA Matrix diagram
aps run topology viz --type vsa
open vsa.html
```

## Programmatic Usage

```rust
use code_topology_viz::{codecity, clusters, vsa};

fn main() {
    let modules = r#"[{"id": "main", "slice": "core", ...}]"#;
    let coupling = r#"{"modules": ["main"], "matrix": [[0.0]]}"#;
    
    let html = codecity::generate(modules, coupling);
    std::fs::write("codecity.html", html).unwrap();
}
```
