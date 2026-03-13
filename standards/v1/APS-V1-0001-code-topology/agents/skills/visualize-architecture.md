# Skill: Visualize Architecture

## Overview

Generate visual representations of code architecture from `.topology/` artifacts using built-in or custom projectors.

## When to Use

- When asked to "show the architecture" or "visualize dependencies"
- When explaining code structure to stakeholders
- When identifying tightly coupled clusters visually
- When creating documentation diagrams
- When assessing architectural health at a glance

## Prerequisites

- `.topology/` directory must exist with valid artifacts
- Run `aps run topology analyze` first if artifacts don't exist

## CLI Usage

```bash
# Generate all visualizations (recommended)
aps run topology viz .topology --type all

# Generate specific visualization
aps run topology viz .topology --type codecity
aps run topology viz .topology --type clusters
aps run topology viz .topology --type vsa
aps run topology viz .topology --type 3d

# Custom output location
aps run topology viz .topology --type codecity --output my-city.html
```

## Built-in Visualizations

### 🌐 `3d` — 3D Force-Directed Coupling Graph (Default)

Best for: Understanding coupling relationships and Martin's metrics

| Visual | Meaning |
|--------|---------|
| Node position | Force-directed by coupling strength |
| Node size | Function count |
| Node color | Instability (red=unstable, blue=stable) |
| Edge thickness | Coupling strength |
| Clustered nodes | Tightly coupled modules |

### 🏙️ `codecity` — 3D City Metaphor

Best for: Quick architectural health assessment, identifying complexity hotspots

| Building Property | Metric |
|-------------------|--------|
| **Height** | Cyclomatic Complexity |
| **Width/Depth** | sqrt(Function Count) |
| **Color** | Health Score (green=healthy, red=critical) |
| **District** | Top-level package (slice) |

**Interpretation:**
- Tall red buildings = complexity hotspots needing refactoring
- Short green buildings = healthy, well-structured modules
- Separate districts = good package organization

### 🔧 `clusters` — 2D Package Relationship Graph

Best for: Understanding package-level coupling, finding isolated/over-coupled packages

| Visual | Meaning |
|--------|---------|
| Circle size | Module count in package |
| Circle color | Average health of modules |
| Lines | Inter-package coupling |
| Line thickness | Coupling strength |
| Well-separated circles | Good encapsulation |

### 🍰 `vsa` — Vertical Slice Architecture Matrix

Best for: Validating VSA compliance, identifying missing layers, detecting coupling smells

| Axis | Meaning |
|------|---------|
| Columns | Feature slices (domains) |
| Rows | Architectural layers (handlers, services, models, data, utils) |
| Cell color | Health score of modules in that slice+layer |
| Cell number | Module count |

**Interpretation:**
- Tall columns with all layers = well-structured vertical slices
- Wide utils row = too much shared code (coupling smell)
- Missing layers = incomplete feature slices
- Red cells = problematic areas needing attention

### 📊 `all` — Generate All + Dashboard Index

Generates all four visualizations plus an `index.html` dashboard with:
- Summary statistics (modules, slices, average health)
- Links to all visualizations
- Quick architectural health overview

## Health Score Formula

All visualizations use a standardized health score (0-100%):

| Component | Ideal | Bad | Weight |
|-----------|-------|-----|--------|
| Complexity/function | 3-8 | >15 | 20% |
| Cognitive load/function | <10 | >30 | 20% |
| LOC/function | 10-50 | >100 | 20% |
| Total coupling | 1-20 | 0 or >20 | 20% |
| Module size | 5-30 functions | <2 or >50 | 20% |

**Color Gradient:**

| Health | Color | Label |
|--------|-------|-------|
| ≥80% | 🟢 `#00ff88` | Excellent |
| ≥65% | 🟢 `#44dd77` | Good |
| ≥50% | 🟡 `#88cc55` | OK |
| ≥35% | 🟠 `#ddaa33` | Warning |
| ≥20% | 🔴 `#ff7744` | Poor |
| <20% | 🔴 `#ff3333` | Critical |

## Layer Detection (for VSA)

Layers are automatically detected from path patterns:

| Layer | Keywords in path |
|-------|------------------|
| handlers | handler, controller, api, routes, endpoint, view |
| services | service, usecase, application, interactor |
| models | model, entity, domain, schema, type |
| data | repository, repo, data, store, db |
| utils | util, helper, common, shared, lib |
| other | (default if no match) |

## Example Conversations

### Generate All Visualizations

```
User: Show me the architecture of this codebase.

Agent: I'll generate all visualizations for you.

[Runs: aps run topology viz .topology --type all]

Generated:
- .topology/viz/index.html (dashboard)
- .topology/viz/3d.html (coupling graph)
- .topology/viz/codecity.html (city metaphor)
- .topology/viz/clusters.html (package clusters)
- .topology/viz/vsa.html (slice architecture)

Open the dashboard: open .topology/viz/index.html

Summary: 24 modules, 8 slices, 72% average health (Good).
```

### Identify Problem Areas

```
User: Which modules need the most attention?

Agent: Looking at the CodeCity visualization, I can see several red buildings:

1. **core.auth** (Critical, 18% health)
   - Very tall building (high complexity)
   - 47 functions, avg 15 CC per function
   - Recommendation: Split into auth_basic and auth_oauth

2. **utils.legacy** (Poor, 24% health)  
   - Wide building (many functions: 82)
   - But short (low complexity per function)
   - Recommendation: Extract domain-specific utilities

3. **api.handlers** (Warning, 38% health)
   - Moderate height
   - Yellow color
   - Consider extracting validation logic
```

### Check VSA Compliance

```
User: Are we following vertical slice architecture correctly?

Agent: Looking at the VSA diagram:

✅ Good patterns:
- Features (orders, products, users) each have all layers
- Minimal cross-slice dependencies

⚠️ Issues:
- Large utils row (12 modules) - too much shared code
- missing "data" layer in 3 slices - using services directly?
- "legacy" slice is mostly handlers with no model layer

Recommendations:
1. Move shared utilities into the appropriate slices
2. Add repository pattern for database access
3. Consider deprecating the legacy slice
```

## Output Locations

| Mode | Output Files |
|------|--------------|
| `--type 3d` | `topology-3d.html` (or `--output`) |
| `--type codecity` | `codecity.html` (or `--output`) |
| `--type clusters` | `clusters.html` (or `--output`) |
| `--type vsa` | `vsa.html` (or `--output`) |
| `--type all` | `.topology/viz/*.html` |

## Error Handling

| Error | Action |
|-------|--------|
| No modules.json | Run `aps run topology analyze` first |
| No coupling-matrix.json | Run `aps run topology analyze` first |
| Unknown --type | Show available types |

## Related Skills

- `analyze-topology` — Generate topology artifacts first
- `query-coupling` — Get detailed coupling metrics
