# Agent Skills — 3D Force-Directed Projector

## Available Skills

### `render-3d-coupling`

Generate 3D coupling visualization from topology artifacts.

**Usage:**
```
Please render a 3D coupling visualization for this project.
```

**Prerequisites:**
- `.topology/` directory exists with `graphs/coupling-matrix.json`

**Output:**
- HTML file with interactive 3D viewer

### `analyze-clusters`

Identify tightly coupled module clusters from the visualization.

**Usage:**
```
Analyze the 3D coupling visualization and identify architectural clusters.
```

**Output:**
- List of identified clusters
- Coupling strength within/between clusters
- Recommendations for decoupling

## Skill Implementations

*[Skills to be implemented in future milestones]*

```yaml
# render-3d-coupling.yaml (planned)
name: render-3d-coupling
description: Generate 3D coupling visualization
inputs:
  - topology_dir: Path to .topology/ directory
  - output: Output file path
  - format: Output format (html, json)
outputs:
  - file: Generated visualization file
```

