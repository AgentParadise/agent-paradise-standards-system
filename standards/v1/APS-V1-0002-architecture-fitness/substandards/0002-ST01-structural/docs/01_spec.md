# APS-V1-0002.ST01 — Structural Integrity Dimension

**Version**: 1.0.0
**Parent**: APS-V1-0002 (Architecture Fitness Functions)

---

## 1. Scope

This substandard governs **structural design quality** — ArchUnit-style checks, class-level design metrics, layer enforcement, and pattern detection. It ensures that code organization follows intended architectural patterns.

**Data sources**:
- AST analysis for structural pattern checks
- `.topology/` artifacts for dependency-based structural rules
- Class-level analysis (planned) for CK object-oriented metrics

## 2. Metrics Owned

### 2.1 CK Object-Oriented Metrics (Planned)

These metrics require class-level analysis not yet available in the topology standard. They are included in the specification for completeness and will become enforceable when tooling is available.

| Metric | Scope | Formula | Author | Status |
|--------|-------|---------|--------|--------|
| Depth of Inheritance Tree (DIT) | Class | Max path to root | Chidamber & Kemerer (1994) | Planned |
| Coupling Between Objects (CBO) | Class | Classes coupled to this class | Chidamber & Kemerer (1994) | Planned |
| Response For a Class (RFC) | Class | Methods + called methods | Chidamber & Kemerer (1994) | Planned |
| Weighted Methods per Class (WMC) | Class | Σ CC per method | Chidamber & Kemerer (1994) | Planned |
| Lack of Cohesion in Methods (LCOM) | Class | Henderson-Sellers variant | Henderson-Sellers (1996) | Planned |

See [02_metrics-catalog.md](../../docs/02_metrics-catalog.md) for complete definitions.

### 2.2 Structural Pattern Checks (Active)

These checks use existing topology data and/or AST analysis:

| Check | Type | Description |
|-------|------|-------------|
| Layer enforcement | Dependency rule | Controllers must not import repositories directly |
| No circular dependencies | Dependency rule | Shared with MD01 (may be declared in either dimension) |
| Import directionality | Dependency rule | Enforce allowed/forbidden import paths |
| Naming conventions | Structural rule | Enforce naming patterns for architectural elements |

## 3. Structural Rule Patterns

### 3.1 Layer Enforcement

```toml
[[rules.structural]]
id = "layer-no-skip"
name = "Controllers Must Not Import Repositories"
dimension = "ST01"
pattern = "forbidden_import"
from = { path = "**/controllers/**" }
to = { path = "**/repositories/**" }
severity = "error"

[[rules.structural]]
id = "layer-direction"
name = "Domain Must Not Import Infrastructure"
dimension = "ST01"
pattern = "forbidden_import"
from = { path = "**/domain/**" }
to = { path = "**/infrastructure/**" }
severity = "error"
```

### 3.2 Naming Conventions

```toml
[[rules.structural]]
id = "handler-naming"
name = "Command Handlers Must End With Handler"
dimension = "ST01"
pattern = "naming_convention"
scope = "class"
path = "**/handlers/**"
match = "*Handler"
severity = "warning"
```

## 4. Default Rules

ST01 default rules focus on dependency-based structural checks that can be evaluated with existing topology data:

```toml
[[rules.dependency]]
id = "no-circular-deps"
name = "No Circular Dependencies"
dimension = "ST01"
type = "forbidden"
from = { path = "src/**" }
to = { path = "src/**" }
circular = true
severity = "error"
```

Additional structural rules are project-specific and SHOULD be configured per-project based on the intended architecture (layered, hexagonal, vertical slice, etc.).

## 5. Scoring

Dimension score formula:

```
ST01_score = 1.0 - (unexcepted_violations / total_entities_evaluated)
```

When CK metrics become available, they will contribute to the entity count. Until then, ST01 scoring is based on dependency and structural pattern rule violations only.

## 6. Future: CK Metrics Integration

When class-level analysis is added to APS-V1-0001 or implemented as a dedicated analyzer, ST01 will gain these rules:

```toml
[[rules.threshold]]
id = "max-dit"
name = "Maximum Depth of Inheritance Tree"
dimension = "ST01"
source = "metrics/classes.json"
field = "depth_of_inheritance"
max = 4
scope = "class"
severity = "warning"

[[rules.threshold]]
id = "max-wmc"
name = "Maximum Weighted Methods per Class"
dimension = "ST01"
source = "metrics/classes.json"
field = "weighted_methods"
max = 50
scope = "class"
severity = "warning"

[[rules.threshold]]
id = "max-lcom"
name = "Maximum Lack of Cohesion"
dimension = "ST01"
source = "metrics/classes.json"
field = "lcom_hs"
max = 0.8
scope = "class"
severity = "warning"
```
