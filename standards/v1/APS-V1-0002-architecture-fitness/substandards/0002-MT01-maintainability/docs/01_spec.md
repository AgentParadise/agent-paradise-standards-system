# APS-V1-0002.MT01 — Maintainability Dimension

**Version**: 1.0.0
**Parent**: APS-V1-0002 (Architecture Fitness Functions)

---

## 1. Scope

This substandard governs **function-level and file-level maintainability** — the ability to understand, modify, and extend code without introducing defects.

**Data source**: `.topology/metrics/` artifacts produced by APS-V1-0001 (Code Topology). No adapter required — all metrics are computed natively by the topology standard.

## 2. Metrics Owned

| Metric | Scope | Formula | Author |
|--------|-------|---------|--------|
| Cyclomatic Complexity (CC) | Function | CC = 1 + decision_points | McCabe (1976) |
| Cognitive Complexity (CogC) | Function | Incremental with nesting penalty | Campbell/SonarSource (2017) |
| Halstead Volume (V) | Function | V = N × log₂(η) | Halstead (1977) |
| Halstead Difficulty (D) | Function | D = (η₁/2) × (N₂/η₂) | Halstead (1977) |
| Halstead Effort (E) | Function | E = D × V | Halstead (1977) |
| Halstead Estimated Bugs (B) | Function | B = V / 3000 | Halstead (1977) |
| Lines of Code (LOC/SLOC) | File, Module | Line counts | Brooks (1975), Boehm (1981) |
| Maintainability Index (MI) | File, Module | MI = 171 - 5.2×ln(V) - 0.23×CC - 16.2×ln(LOC) | Coleman et al. (1994) |

See [02_metrics-catalog.md](../../docs/02_metrics-catalog.md) for complete definitions.

## 3. Default Rules

```toml
[[rules.threshold]]
id = "max-cognitive"
name = "Maximum Cognitive Complexity (function)"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cognitive_complexity"
max = 15
scope = "function"
severity = "error"
exclude = ["**/test_*", "**/tests/**"]

[[rules.threshold]]
id = "max-cyclomatic"
name = "Maximum Cyclomatic Complexity (function)"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 10
scope = "function"
severity = "error"
exclude = ["**/test_*", "**/tests/**"]

[[rules.threshold]]
id = "max-loc"
name = "Maximum Lines of Code per File"
dimension = "MT01"
source = "metrics/file_metrics.json"
field = "lines_of_code"
max = 500
scope = "file"
severity = "warning"

[[rules.threshold]]
id = "max-halstead-volume"
name = "Maximum Halstead Volume (function)"
dimension = "MT01"
source = "metrics/complexity.json"
field = "halstead_volume"
max = 1000
scope = "function"
severity = "warning"

[[rules.threshold]]
id = "min-maintainability-index"
name = "Minimum Maintainability Index"
dimension = "MT01"
source = "metrics/file_metrics.json"
field = "maintainability_index"
min = 20
scope = "file"
severity = "error"
```

## 4. Scoring

Dimension score formula:

```
MT01_score = 1.0 - (unexcepted_violations / total_entities_evaluated)
```

The Maintainability Index provides a secondary composite signal — files with MI < 20 are strongly correlated with difficult-to-maintain code regardless of individual metric values.

## 5. Interactions with Other Dimensions

- **MT01 → MD01**: High coupling (MD01) often causes low maintainability (MT01). Modules with high Ce tend to have functions with high CC because they orchestrate many dependencies.
- **MT01 → ST01**: God classes (ST01: high WMC) manifest as files with high total complexity (MT01: low MI).
- The Maintainability Index incorporates CC and Halstead Volume, creating a cross-metric dependency within MT01.

## 6. Topology Artifact Mapping

| Rule Source | Topology Artifact | Entity ID Format |
|-------------|-------------------|------------------|
| `metrics/complexity.json` | Function-level metrics | `<language>:<module>::<function>` |
| `metrics/file_metrics.json` | File-level aggregated metrics | `<relative_path>` |
| `metrics/modules.json` | Module-level aggregated metrics | `<module_path>` |
