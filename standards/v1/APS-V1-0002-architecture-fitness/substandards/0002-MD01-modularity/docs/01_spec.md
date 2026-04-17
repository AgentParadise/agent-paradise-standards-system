# APS-V1-0002.MD01 — Modularity and Coupling Dimension

**Version**: 1.0.0
**Parent**: APS-V1-0002 (Architecture Fitness Functions)

---

## 1. Scope

This substandard governs **module-level coupling and dependency structure** — the separation of concerns, dependency directionality, and architectural balance that enable independent evolution of system components.

**Data sources**:
- `.topology/metrics/` artifacts produced by APS-V1-0001 (native — no adapter)
- VSA adapter output for slice-level metrics (optional, via adapter)

## 2. Metrics Owned

### 2.1 Martin's Package Metrics (Native)

| Metric | Scope | Formula | Author |
|--------|-------|---------|--------|
| Afferent Coupling (Ca) | Module | Ca = incoming dependency count | Martin (1994) |
| Efferent Coupling (Ce) | Module | Ce = outgoing dependency count | Martin (1994) |
| Instability (I) | Module | I = Ce / (Ca + Ce) | Martin (1994) |
| Abstractness (A) | Module | A = abstract_types / total_types | Martin (1994) |
| Distance from Main Sequence (D) | Module | D = \|A + I - 1\| | Martin (1994) |

### 2.2 System-Level Coupling (Native)

| Metric | Scope | Formula |
|--------|-------|---------|
| Composite Coupling | Module-pair | Weighted blend of import/call/symbol/type/change coupling |
| Coupling Density | System | actual_edges / possible_edges |

### 2.3 Information Flow (Derivable)

| Metric | Scope | Formula | Author |
|--------|-------|---------|--------|
| Fan-in | Function, Module | In-degree in call/dependency graph | Henry & Kafura (1981) |
| Fan-out | Function, Module | Out-degree in call/dependency graph | Henry & Kafura (1981) |

### 2.4 VSA Metrics (Via Adapter)

| Metric | Scope | Description |
|--------|-------|-------------|
| Slice Independence Score (SIS) | Slice | How independent a vertical slice is from others |
| Cross-Context Coupling | Slice | Number of cross-bounded-context dependencies |

See [02_metrics-catalog.md](../../docs/02_metrics-catalog.md) for complete definitions.

## 3. Default Rules

```toml
# --- Coupling Thresholds ---

[[rules.threshold]]
id = "max-efferent-coupling"
name = "Maximum Efferent Coupling (Ce)"
dimension = "MD01"
source = "metrics/coupling.json"
field = "efferent_coupling"
max = 20
scope = "module"
severity = "error"

[[rules.threshold]]
id = "instability-balance"
name = "Instability Balance"
dimension = "MD01"
source = "metrics/coupling.json"
field = "instability"
min = 0.1
max = 0.9
scope = "module"
severity = "warning"
exclude = ["**/interfaces/**", "**/ports/**"]

[[rules.threshold]]
id = "max-main-sequence-distance"
name = "Maximum Distance from Main Sequence"
dimension = "MD01"
source = "metrics/coupling.json"
field = "distance_from_main_sequence"
max = 0.7
scope = "module"
severity = "error"

# --- Dependency Constraints ---

[[rules.dependency]]
id = "no-circular-deps"
name = "No Circular Dependencies"
dimension = "MD01"
type = "forbidden"
from = { path = "src/**" }
to = { path = "src/**" }
circular = true
severity = "error"
```

## 4. Martin's Zone Model

The Distance from Main Sequence (D) captures two architectural pathologies:

```
A (Abstractness)
1.0 ┌─────────────────────────┐
    │ Zone of       ╲         │
    │ Uselessness    ╲        │
    │                 ╲ Main  │
    │                  ╲ Seq. │
    │                   ╲     │
    │          Zone of   ╲    │
    │          Pain       ╲   │
    │                      ╲  │
0.0 └─────────────────────────┘
   0.0          I          1.0
```

- **Zone of Pain** (D > 0.5, low A, low I): Concrete modules that many others depend on. Rigid — any change risks cascading failures. Resolution: add abstractions (interfaces/traits).
- **Zone of Uselessness** (D > 0.5, high A, high I): Abstract modules nobody depends on. Over-engineered. Resolution: remove unused abstractions or consolidate.

Modules on or near the Main Sequence (D < 0.3) are well-balanced.

## 5. VSA Integration

When the VSA adapter is registered, MD01 gains slice-level governance:

```toml
[[adapters]]
id = "vsa"
dimension = "MD01"
input = ".vsa/analysis.json"
output = "adapters/md01/vsa-slices.json"
normalizer = "builtin:vsa"

[[rules.threshold]]
id = "min-sis-score"
name = "Minimum Slice Independence Score"
dimension = "MD01"
source = "adapters/md01/vsa-slices.json"
field = "metrics.sis_score"
min = 0.7
scope = "slice"
severity = "error"
```

The VSA adapter normalizes slice data into the standard wrapped artifact format, allowing threshold rules to assert on slice-level coupling without the fitness standard knowing VSA internals.

## 6. Scoring

Dimension score formula:

```
MD01_score = 1.0 - (unexcepted_violations / total_entities_evaluated)
```

MD01 is typically the most actionable dimension — coupling violations have clear remediation paths (extract interface, reduce dependencies, break cycles).

## 7. Topology Artifact Mapping

| Rule Source | Topology Artifact | Entity ID Format |
|-------------|-------------------|------------------|
| `metrics/coupling.json` | Module-level Martin's metrics | `<module_path>` |
| `metrics/coupling-matrix.json` | Pairwise composite coupling | `<module_a> → <module_b>` |
| Call graph (derivable) | Function-level fan-in/out | `<language>:<module>::<function>` |
| VSA adapter output | Slice-level metrics | `<slice_name>` |
