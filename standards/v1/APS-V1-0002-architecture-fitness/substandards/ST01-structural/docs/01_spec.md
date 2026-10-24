# APS-V1-0002.ST01 - Structural Integrity Dimension

**Version**: 1.1.0
**Parent**: APS-V1-0002 (Architecture Fitness Functions)

---

## 1. Scope

This substandard governs **structural design quality** - ArchUnit-style checks, class-level design metrics, layer enforcement, and pattern detection. It ensures that code organization follows intended architectural patterns.

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

See [02_metrics-catalog.md](../../../docs/02_metrics-catalog.md) for complete definitions.

### 2.2 Structural Pattern Checks (Active)

These checks use existing topology data and/or AST analysis:

| Check | Type | Description |
|-------|------|-------------|
| Layer enforcement | Dependency rule | Controllers must not import repositories directly |
| No circular dependencies | Dependency rule | Shared with MD01 (may be declared in either dimension) |
| Import directionality | Dependency rule | Enforce allowed/forbidden import paths |
| Naming conventions | Structural rule | Enforce naming patterns for architectural elements |
| Dependency depth | Structural rule (`max_dependency_depth`, §3.3) | Ceiling on the longest dependency path from a node |
| Fan-out and composition roots | Structural rule (`max_fan_out`, §3.4) | Ceiling on efferent coupling; only flat, declared composition roots may wire many nodes |

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

### 3.3 Graph Shape: `max_dependency_depth`

Both graph-shape patterns (§3.3, §3.4) read the same input as the patterns above: `graphs/dependency-graph.json` under `topology_dir` (nodes, and directed edges `[from, to]` meaning "from depends on to"). Both use the exception file with a per-entity `value` ceiling, as threshold rules do, so a project can baseline today and ratchet down. A node's edge to itself is not a dependency in either pattern: it adds no depth, no fan-out, and a root's self-edge does not break flatness. The carve-out is local to these patterns; the graph keeps the self-loop, so cycle rules still see it.

```toml
[[rules.structural]]
id = "st01-max-dependency-depth"
name = "Crates stay shallow"
pattern = "max_dependency_depth"
from = { path = "crates/**" }
to = { path = "crates/**" }   # optional, default every node
max = 4
```

- **Path set:** a path starts at a node in `from`; an edge counts if and only if its target is in `to`. A path MAY pass through nodes outside `from`.
- **Dependency depth** of a node is the number of edges on the longest such path starting at it. A node with no counted dependencies has depth 0. Depth = Lakos level - 1 (Lakos, *Large-Scale C++ Software Design*, 1996, levelization, counts a leaf as level 1).
- **Cycles:** each strongly connected component counts as one node (depth over the condensation); every member of a cycle shares its component's depth. Cycles themselves are judged by the existing `circular = true` dependency rule, not here.
- **Violation:** each `from` node whose depth > `max` (`field = "dependency_depth"`, `actual` = depth). The violation's `message` names the longest path (`longest path: a -> b -> {c, d}`, a cycle shown as its members); ties are broken by lexicographic node order, so reports and goldens are stable.

### 3.4 Graph Shape: `max_fan_out` with Composition Roots

```toml
[[rules.structural]]
id = "st01-flat-composition-root"
name = "Only the composition root wires many crates"
pattern = "max_fan_out"
from = { path = "crates/**" }
max = 4
composition_roots = ["crates/dream-ship-engine"]
root_max = 12          # optional; absent = no cap for a root
```

- **Fan-out** is efferent coupling, Ce (Martin, *Agile Software Development*, 2002): the number of distinct nodes in `to` (optional, default every node) a node depends on directly. Self-edges are excluded; parallel edges collapse to one. The field is `fan_out`.
- A node that is not a declared composition root violates when fan-out > `max` (`field = "fan_out"`).
- **Declared composition roots** MAY exceed `max`, up to `root_max` when set (Seemann, *Dependency Injection in .NET*, 2011, Composition Root: the one place that wires every adapter). Several roots are allowed: one per application or entry point.
- **When `root_max` is absent,** each root's fan-out is still emitted as a reading on every run (`RuleResult.readings`, `field = "fan_out"`), so growth stays visible to the ratchet even uncapped. Readings are emitted when `root_max` is set, too.
- **A composition root MUST be flat:** any `from` node with a direct edge to a declared root is a violation (`field = "depends_on_composition_root"`, entity = the dependent, `message` names the root). That includes a root depending on another root. Only roots may depend on many leaves, and nothing depends on a root.
- **This catches test, bench and example crates and second binaries that link the root lib.** The exception file is the only escape, per entity; there is never a pattern-level allowlist. A root-dependency violation is excused only by an exception whose `targets` lists that root; a `value` budget never excuses it, and a `targets`-only entry never excuses a `fan_out` ceiling.
- **Fails closed (config error):** a `composition_roots` glob that matches no node, or that matches a node outside `from`, fails the rule with a single `INVALID_RULE:<rule id>` violation (`field = "composition_roots"`) that no exception can excuse. A renamed root cannot silently lift the cap.

Config load rejects `max`, `composition_roots` or `root_max` on any other pattern, a graph-shape rule without `max` or `from`, `composition_roots` or `root_max` on `max_dependency_depth`, `root_max` without `composition_roots`, and an unparseable glob.

Direction (`layer_enforcement`) already exists. Whether core nodes depend on ports rather than concrete adapters needs a port/adapter tag per node, so it is a later pattern.

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
