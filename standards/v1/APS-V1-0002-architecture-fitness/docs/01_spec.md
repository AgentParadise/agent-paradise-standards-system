# APS-V1-0002 — Architecture Fitness Functions

**Version**: 1.0.0
**Status**: Active
**Category**: Technical

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Scope and Authority

### 1.1 Purpose

This standard defines a **comprehensive architectural governance framework** based on the principles of evolutionary architecture (Ford et al., 2017). It provides declarative fitness functions — automated, continuous, objective assertions on architectural properties — organized into composable dimensional substandards.

The core thesis of evolutionary architecture is:

> **Architecture = Requirements + Architectural Characteristics**

- **Requirements** (functional behavior) are tested via unit, functional, and integration tests. Requirements are NOT in scope of this standard.
- **Architectural Characteristics** (the "-ilities" — maintainability, security, scalability, etc.) are tested via **fitness functions**. This IS the scope of this standard.

Fitness functions are to architecture what unit tests are to business logic: automated assertions that run continuously in CI and fail on violations.

### 1.2 Scope

This standard covers:

- **Dimensional governance model** — A framework of composable architectural dimensions, each governed by a substandard
- **Metrics catalog** — Comprehensive definitions of all architectural metrics with mathematical formulas, original authors, industry thresholds, and rationale (see [02_metrics-catalog.md](./02_metrics-catalog.md))
- **Rule format specification** — `fitness.toml` schema for threshold, dependency, and structural rules
- **Exception format specification** — `fitness-exceptions.toml` schema for tracked violations with ratchet semantics
- **Report format specification** — `fitness-report.json` schema for per-dimension and composite results
- **System-Level Fitness Function** — Weighted aggregation of dimensions for holistic architectural health and tradeoff analysis
- **Adapter contract** — Anti-corruption layer interface for normalizing external tool output
- **Validation semantics** — How rules are evaluated, scored, and reported

This standard does NOT cover:

- Topology artifact generation (see APS-V1-0001)
- Specific scanner implementations (scanners are external; adapters bridge them)
- Unit, functional, or integration testing of business requirements
- CI pipeline configuration (informative only)

### 1.3 Relationship to APS-V1-0001

APS-V1-0001 (Code Topology) defines the **measurement layer** — it produces `.topology/metrics/` artifacts containing complexity, coupling, and structural data.

This standard defines the **assertion layer** — it consumes those artifacts (and optionally external tool output via adapters) and evaluates architectural rules against them.

```
APS-V1-0001 (measure)    → .topology/metrics/*.json ─┐
                                                      ├→ APS-V1-0002 (assert)
External tools (scan)     → adapter normalization ────┘       │
                                                              ↓
                                                    fitness-report.json
                                                    (per-dimension + composite)
```

### 1.4 Relationship to Substandards

Each architectural dimension is governed by a substandard. The parent standard defines the governance model, rule format, report format, and system-level fitness function. Substandards define dimension-specific metrics, default rules, and adapter contracts.

Each dimension has two orthogonal classifications:

- **Status** — `active` or `incubating` (see §3.3 Dimension Promotion Requirements and §3.4 Dimension Lifecycle). Only `active` dimensions MAY produce error-severity failures that block CI.
- **Default** — `default-enabled` or `opt-in`. Whether a dimension auto-activates when `[dimensions]` is omitted.

| Substandard | Dimension | Status | Default | Notes |
|-------------|-----------|--------|---------|-------|
| APS-V1-0002.MT01 | Maintainability | active | default-enabled | Function-level McCabe / SonarSource / Halstead metrics from `functions.json` (APS-V1-0001) |
| APS-V1-0002.MD01 | Modularity & Coupling | active | default-enabled | Martin package metrics (Ca, Ce, I, A, D) from `coupling.json` (APS-V1-0001) |
| APS-V1-0002.ST01 | Structural Integrity | incubating | default-enabled | Structural patterns implementable now; CK metrics require class-level analyzer |
| APS-V1-0002.SC01 | Security | incubating | default-enabled | Awaits adapter framework + `builtin:cargo-audit` |
| APS-V1-0002.LG01 | Legality | incubating | default-enabled | Awaits adapter framework + `builtin:cargo-deny` |
| APS-V1-0002.AC01 | Accessibility | incubating | opt-in | Awaits adapter framework |
| APS-V1-0002.PF01 | Performance | incubating | opt-in | Awaits adapter framework |
| APS-V1-0002.AV01 | Availability | incubating | opt-in | Awaits adapter framework |

**MT01 and MD01 are `active`** and strictly enforced: missing source artifacts for their rules produce `PROMOTION_REQUIREMENT_UNMET` (§12) rather than silent skips. All other dimensions are `incubating` — their rule severities are downgraded to warning and missing artifacts skip silently. Implementation status per dimension is disclosed in [Appendix D](#appendix-d-current-implementation-status). Promotion to `active` is gated on R1–R5 (§3.3).

### 1.5 Promotion Lineage

This standard is promoted from EXP-V1-0003 (Architecture Fitness Functions, experimental). All EXP-V1-0003 `fitness.toml` files are forward-compatible with this standard — the new sections (`[dimensions]`, `[system_fitness]`) are all optional with backward-compatible defaults.

EXP-V1-0003 remains in place per the meta-standard (experiments are never removed).

### 1.6 Normative References

- Ford, N., Parsons, R., & Kua, P. (2017). *Building Evolutionary Architectures*. O'Reilly Media.
- McCabe, T.J. (1976). "A Complexity Measure." *IEEE Transactions on Software Engineering*, SE-2(4).
- Campbell, G.A. / SonarSource (2017). "Cognitive Complexity: A new way of measuring understandability."
- Halstead, M.H. (1977). *Elements of Software Science*. Elsevier.
- Coleman, D. et al. (1994). "Using Metrics to Evaluate Software System Maintainability." *IEEE Computer*, 27(8).
- Martin, R.C. (1994). "OO Design Quality Metrics: An Analysis of Dependencies." *OOPSLA*. Expanded in: *Agile Software Development* (2003), Chapter 20.
- Chidamber, S.R. & Kemerer, C.F. (1994). "A Metrics Suite for Object Oriented Design." *IEEE TSE*, 20(6).
- Henderson-Sellers, B. (1996). *Object-Oriented Metrics: Measures of Complexity*. Prentice Hall.
- Henry, S. & Kafura, D. (1981). "Software Structure Metrics Based on Information Flow." *IEEE TSE*, SE-7(5).
- Watson, A.H. & McCabe, T.J. (1996). *Structured Testing: A Testing Methodology Using the Cyclomatic Complexity Metric*. NIST SP 500-235.
- ArchUnit (2017). FreezingArchRule ratchet pattern.
- dependency-cruiser. Forbidden/allowed/required rule model.

---

## 2. Core Definitions

### 2.1 Fitness Function

An **architecture fitness function** is an objective integrity assessment of some architecture characteristic (Ford et al., 2017). Per Ford's taxonomy, fitness functions in this standard are:

| Property | Description |
|----------|-------------|
| **Automated** | Executable without human judgment |
| **Continuous** | Runs on every change (CI) |
| **Architectural** | Asserts on system-level properties, not unit behavior |

### 2.2 Dimension

A **dimension** is an independently measurable and governable architectural characteristic. Each dimension maps to one substandard and has its own metrics, rules, and scoring. Dimensions are composable — the system-level fitness function aggregates them.

### 2.3 Rule

A **rule** is a single architectural assertion declared in `fitness.toml`. Rules have:

- **ID** — Unique identifier (e.g., `max-cyclomatic`)
- **Type** — `threshold`, `dependency`, or `structural`
- **Dimension** — Which architectural dimension this rule governs (e.g., `MT01`)
- **Severity** — `error` (blocks CI) or `warning` (advisory)

### 2.4 Exception

An **exception** is a tracked deviation from a rule, declared in `fitness-exceptions.toml`. Exceptions MUST reference a GitHub issue. Exceptions represent technical debt that is acknowledged and planned for resolution.

### 2.5 Ratchet

A **ratchet** is the mechanism by which exception budgets can only shrink over time. If a violation is fixed (metric drops below threshold), the exception becomes **stale** and MUST be removed. New exceptions MUST NOT exceed the current violation count.

### 2.6 Violation

A **violation** is a specific entity (module, file, function, class, slice) that fails a rule's assertion. A violation may be **excepted** (tracked in exceptions file) or **unexcepted** (causes rule failure).

### 2.7 System-Level Fitness Function

The **system-level fitness function** is the holistic aggregation of all per-dimension scores into a single system-wide assessment. It is the primary tool for understanding overall architectural health and for analyzing tradeoffs across dimensions. See section 6.

### 2.8 Adapter

An **adapter** is a component that normalizes external tool output into the fitness function contract. Adapters implement the anti-corruption layer pattern: external tools never change, the adapter translates.

---

## 3. Dimensional Model

### 3.1 Dimension Registry

Every dimension is identified by a 4-character code and governed by a substandard. Each dimension carries a **status** (promotion state — see §3.3 and §3.4) and a **default** (whether it auto-activates):

| Code | Dimension | Characteristics Protected | Data Source | Status | Default |
|------|-----------|--------------------------|-------------|--------|---------|
| MT01 | Maintainability | Readability, testability, complexity | `.topology/metrics/functions.json` | active | default-enabled |
| MD01 | Modularity & Coupling | Separation of concerns, dependency structure | `.topology/metrics/coupling.json` | active | default-enabled |
| ST01 | Structural Integrity | Design patterns, class design, layer enforcement | AST analysis, structural checks | incubating | default-enabled |
| SC01 | Security | Vulnerability freedom, supply chain safety | Security scanner output | incubating | default-enabled |
| LG01 | Legality | License compliance, IP safety | License scanner output | incubating | default-enabled |
| AC01 | Accessibility | WCAG compliance, inclusive design | a11y scanner output | incubating | opt-in |
| PF01 | Performance | Latency, throughput, regression prevention | Load test / benchmark results | incubating | opt-in |
| AV01 | Availability | Resilience, uptime, fault tolerance | Chaos eng / uptime metrics | incubating | opt-in |

### 3.2 Default-Enabled vs Opt-In

The **default** classification controls whether a dimension auto-activates when `[dimensions]` is omitted from `fitness.toml`. It is independent of the dimension's status (§3.4).

- **Default-enabled**: MT01, MD01, ST01, SC01, LG01 — Represent baseline architectural governance that applies to virtually all software projects. Auto-active unless explicitly disabled.
- **Opt-in**: AC01, PF01, AV01 — Require specialized infrastructure (web frontends, load test harnesses, chaos engineering). Inactive unless explicitly enabled.

Disabling a default-enabled dimension MUST include a `reason` field in the configuration to maintain auditability.

An `incubating` dimension MAY be default-enabled: being default-enabled does not imply enforcement. An incubating dimension that is default-enabled will run its rules as advisory (warning-only), per §3.4.

### 3.3 Dimension Promotion Requirements

A dimension MUST NOT be declared `active` in this standard unless **all five** of the following requirements are met. These requirements exist because the central purpose of a fitness function is to provide an **objective, automated, continuous** assertion on an architectural property. A dimension that cannot objectively compute and enforce its metrics offers governance theater, not governance.

| # | Requirement | Description |
|---|-------------|-------------|
| **R1** | **Objective metric definition** | Every rule enforced by the dimension MUST target a metric with a formal, unambiguous definition (mathematical formula or algorithmic specification) documented in [`02_metrics-catalog.md`](./02_metrics-catalog.md). Free-text "good design" criteria are NOT acceptable. |
| **R2** | **Computable algorithm** | A concrete, deterministic, automated producer MUST exist for every metric the dimension enforces. The producer MUST be one of: (a) a native APS standard (e.g., APS-V1-0001 topology), or (b) a registered adapter with an implemented normalizer. Human judgment calls are not acceptable inputs. |
| **R3** | **Artifact schema** | Every JSON artifact the dimension **consumes or produces** MUST be described by a [JSON Schema](https://json-schema.org/) file on disk, published alongside the producing standard. See §3.5 for canonical paths, versioning, and compatibility rules. A prose description in the spec is NOT sufficient — the schema MUST be machine-readable so external tools can validate, transform, or visualize artifacts without reading APSS source code. |
| **R4** | **Recommended default thresholds with citations** | The substandard MUST publish recommended default values for every enforced metric, each with a citation to its source (original author, industry benchmark, or explicit APSS consensus). Defaults without rationale are NOT acceptable. |
| **R5** | **Reference implementation** | The substandard crate MUST contain non-stub Rust code that registers its default rules, validates its config, and verifies that its required artifacts exist. A substandard whose `src/lib.rs` is a `Phase 2` stub MUST NOT be declared `active`. |

A dimension that fails to meet any of R1–R5 MUST be declared `incubating`. Promoting a dimension from `incubating` to `active` requires an ADR documenting how each requirement is satisfied.

### 3.4 Dimension Lifecycle

Each dimension version follows SemVer independently. The status field is normative and affects enforcement behavior:

| Status | Semantics | Enforcement behavior |
|--------|-----------|----------------------|
| `active` | Satisfies all five promotion requirements (§3.3). | Rules MAY be declared at any severity. Error-severity rules MUST cause CI failure on unexcepted violations. Contributes to system-level fitness score per configured weight. |
| `incubating` | Specified but at least one promotion requirement (§3.3) is unmet. | All rules run as advisory. Rule severity is **downgraded** at evaluation time: any `error` configured on an incubating dimension MUST be reported as `warning` in the output with a `downgraded_from_error` flag. Incubating dimensions MUST NOT cause exit code `1`. Contributes to system-level score only if the user explicitly sets `system_fitness.include_incubating = true` (default: `false`). |
| `deprecated` | Scheduled for removal in a future major version. | Rules continue to run but produce a deprecation warning. New projects SHOULD NOT adopt deprecated dimensions. |

Implementers MUST emit the diagnostic code `INCUBATING_DIMENSION_ERROR_DOWNGRADED` (§12) for every rule whose configured severity was downgraded due to incubating status. This makes the soft enforcement visible rather than silent — users know exactly which assertions are advisory and why.

### 3.5 Artifact Contracts

Artifacts are the public contract of an APS standard. A standard is only as useful as the artifacts it produces, and an artifact is only durable as a contract if it has a machine-readable schema. This section specifies how schemas are authored, located, versioned, and enforced.

#### 3.5.1 Scope

Every JSON artifact read or written by a standard or substandard — topology metrics, fitness reports, adapter outputs, exception files — MUST have a corresponding [JSON Schema](https://json-schema.org/) file (Draft 2020-12 or later). This includes:

- **Input artifacts** the standard consumes from upstream producers
- **Output artifacts** the standard emits for downstream consumers
- **Intermediate artifacts** (e.g., normalized adapter output) passed between substandards

A standard MAY reuse a schema published by an upstream standard rather than duplicate it. The reuse MUST be explicit via JSON Schema `$ref`.

#### 3.5.2 Canonical Paths

Schemas MUST live under the owning standard's or substandard's `schemas/` directory:

```
standards/v1/APS-V1-NNNN-<slug>/schemas/<artifact>.schema.json
standards/v1/APS-V1-NNNN-<slug>/substandards/NNNN-XXNN-<slug>/schemas/<artifact>.schema.json
```

Schema files MUST be discoverable by glob (`**/schemas/*.schema.json`). Tool authors can rely on this convention to auto-load every contract in the repository.

Canonical examples (load-bearing for this standard):

| Artifact | Canonical schema path | Owner |
|----------|----------------------|-------|
| `.topology/metrics/modules.json` | `APS-V1-0001-code-topology/schemas/modules.schema.json` | APS-V1-0001 |
| `.topology/metrics/functions.json` | `APS-V1-0001-code-topology/schemas/functions.schema.json` | APS-V1-0001 |
| `.topology/metrics/coupling.json` | `APS-V1-0001-code-topology/schemas/coupling.schema.json` | APS-V1-0001 |
| `fitness.toml` | `APS-V1-0002-architecture-fitness/schemas/fitness-config.schema.json` | APS-V1-0002 |
| `fitness-exceptions.toml` | `APS-V1-0002-architecture-fitness/schemas/fitness-exceptions.schema.json` | APS-V1-0002 |
| `fitness-report.json` | `APS-V1-0002-architecture-fitness/schemas/fitness-report.schema.json` | APS-V1-0002 |
| Adapter-normalized output (Tier 3) | `APS-V1-0002-architecture-fitness/schemas/adapter-output.schema.json` | APS-V1-0002 — deferred |

#### 3.5.3 Schema Requirements

Every schema MUST declare:

- `"$schema"` — explicit JSON Schema dialect
- `"$id"` — stable URI identifying the artifact
- `"title"` and `"description"` — human-readable
- `"type"` and `"properties"` — strongly typed; `"additionalProperties": false` at the root unless the schema explicitly documents forward-compatible extension points
- A required `"schema_version"` property on the artifact itself, matching SemVer (the established APSS convention — already used by APS-V1-0001 topology artifacts)

Schemas SHOULD include examples (`"examples"`) and reference the source spec section.

#### 3.5.4 Versioning and Compatibility

- Schemas follow SemVer independently from the owning standard
- A **minor** or **patch** bump MUST remain backward-compatible (additive only; no field removals, no type narrowing, no enum shrinking)
- A **major** bump MAY break compatibility and MUST ship alongside an ADR describing migration
- Artifacts MUST carry a `"schema_version"` field so validators can select the correct schema
- Consumers MUST validate artifacts against the schema version recorded in the artifact, not against an assumed version

#### 3.5.5 Validation and CI

Producers MUST validate every artifact they write against its schema before emitting it. Failure to validate MUST abort the write and surface the error. This prevents a broken producer from poisoning downstream consumers.

The APSS repository SHOULD include a CI check that, for every `*.schema.json` under `**/schemas/`, parses the file and validates it against JSON Schema Draft 2020-12 meta-schema. Orphaned artifacts (a producer that writes a file with no matching schema) are a build failure.

#### 3.5.6 Why this is normative, not a style preference

Schemas are the mechanism that converts an APS standard from "a Rust crate with opinions" into **a tool-neutral contract that any ecosystem can consume**. A Grafana dashboard, an IDE plugin, a Python reporting script, a GitHub Action, or a different language's adapter can all build against the schemas without coupling to APSS internals or reading Rust source code. This is what makes centralized architectural governance (the stated goal of this standard) practical rather than theoretical.

Without schemas, each consumer reverse-engineers the artifact shape from examples; shapes drift; contracts rot silently. With schemas, the contract is explicit, versioned, and enforceable. This is why R3 (§3.3) is non-negotiable for active dimensions.

---

## 4. Rule Format (`fitness.toml`)

The rule file MUST be named `fitness.toml` and SHOULD be placed at the repository root.

### 4.1 Config Section

```toml
[config]
topology_dir = ".topology"                          # REQUIRED — path to topology artifacts
exceptions = "fitness-exceptions.toml"              # OPTIONAL — default shown
severity_default = "error"                          # OPTIONAL — default: "error"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `topology_dir` | string | MUST | Path to `.topology/` directory relative to repo root |
| `exceptions` | string | MAY | Path to exceptions file (default: `fitness-exceptions.toml`) |
| `severity_default` | string | MAY | Default severity: `"error"` or `"warning"` (default: `"error"`) |

### 4.2 Dimensions Section

```toml
[dimensions]
MT01 = true             # Maintainability (default: true)
MD01 = true             # Modularity (default: true)
ST01 = true             # Structural Integrity (default: true)
SC01 = true             # Security (default: true)
LG01 = true             # Legality (default: true)
AC01 = false            # Accessibility (default: false — opt-in)
PF01 = false            # Performance (default: false — opt-in)
AV01 = false            # Availability (default: false — opt-in)

# Disabling a default-enabled dimension requires a reason
[dimensions.reasons]
# ST01 = "Pure library with no structural constraints beyond coupling"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `<CODE>` | bool | MAY | Enable/disable a dimension. Default-enabled dimensions default to `true`; opt-in dimensions default to `false`. |
| `reasons.<CODE>` | string | MUST (when disabling a default) | Explanation for disabling a default-enabled dimension |

If the `[dimensions]` section is omitted entirely, all default-enabled dimensions are active and opt-in dimensions are inactive.

### 4.3 Threshold Rules

Threshold rules assert that a metric value for each entity does not exceed (or fall below) a given bound.

```toml
[[rules.threshold]]
id = "max-cyclomatic"
name = "Maximum Cyclomatic Complexity"
dimension = "MT01"                                  # Assigns rule to a dimension
source = "metrics/complexity.json"                  # Topology artifact path
field = "cyclomatic_complexity"                     # JSON field to evaluate
max = 10                                            # Upper bound (fail if value > max)
scope = "function"                                  # Entity scope
severity = "error"                                  # Override default severity
exclude = ["**/test_*", "**/tests/**"]              # Glob patterns to exclude
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | MUST | Unique rule identifier |
| `name` | string | MUST | Human-readable rule name |
| `dimension` | string | RECOMMENDED | Dimension code (e.g., `"MT01"`). If omitted, inferred from `source` and `field`. |
| `source` | string | MUST | Path to topology artifact (relative to `topology_dir`) or adapter output |
| `field` | string | MUST | JSON field path to evaluate (supports dot-notation, e.g., `metrics.cognitive`) |
| `max` | float | MUST (one of) | Upper bound — violation if `value > max` |
| `min` | float | MUST (one of) | Lower bound — violation if `value < min` |
| `scope` | string | MUST | Entity granularity: `"function"`, `"file"`, `"module"`, `"class"`, `"slice"`, `"system"` |
| `severity` | string | MAY | `"error"` or `"warning"` (default: config default) |
| `exclude` | array | MAY | Glob patterns for entities to skip |

At least one of `max` or `min` MUST be specified. Both MAY be specified simultaneously.

#### 4.3.1 Field Dot-Notation

The `field` property supports dot-notation for navigating nested JSON structures:

```toml
field = "cyclomatic_complexity"          # flat: entity["cyclomatic_complexity"]
field = "metrics.cognitive"              # nested: entity["metrics"]["cognitive"]
field = "metrics.martin.ce"              # deep: entity["metrics"]["martin"]["ce"]
```

#### 4.3.2 Wrapped Topology Artifacts

Topology artifacts MAY use a wrapper object. The validator auto-detects wrapped formats:

1. **Scope-derived key**: `scope` maps to a plural wrapper key (`"function"` → `"functions"`, `"module"` → `"modules"`, `"slice"` → `"slices"`, `"file"` → `"files"`, `"class"` → `"classes"`). If that key exists and contains an array, it is unwrapped.
2. **Fallback heuristic**: If the scope-derived key is not found, but exactly one key in the object has an array value, that array is unwrapped.
3. **Flat fallback**: If neither condition is met, the object is treated as a flat entity map.

Within unwrapped arrays, entity identifiers are resolved in priority order: `id` > `path` > `name` > `entity`.

### 4.4 Dependency Rules

Dependency rules assert constraints on the import/coupling graph.

```toml
[[rules.dependency]]
id = "no-circular-deps"
name = "No Circular Dependencies"
dimension = "MD01"
type = "forbidden"                                  # "forbidden", "allowed", or "required"
from = { path = "src/**" }                          # Source path matcher
to = { path = "src/**" }                            # Target path matcher
circular = true                                     # Detect cycles
severity = "error"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | MUST | Unique rule identifier |
| `name` | string | MUST | Human-readable rule name |
| `dimension` | string | RECOMMENDED | Dimension code (default: `"MD01"`) |
| `type` | string | MUST | `"forbidden"`, `"allowed"`, or `"required"` |
| `from` | PathMatcher | MUST | Source entity matcher |
| `to` | PathMatcher | MUST | Target entity matcher |
| `circular` | bool | MAY | Detect circular dependencies (default: `false`) |
| `severity` | string | MAY | `"error"` or `"warning"` |

**PathMatcher:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `path` | string | MUST | Glob pattern for matching entity paths |
| `path_not` | string | MAY | Glob pattern for excluding entity paths |

### 4.5 Structural Rules

Structural rules assert ArchUnit-style constraints on code organization and design patterns.

```toml
[[rules.structural]]
id = "layer-enforcement"
name = "Controllers Must Not Import Repositories"
dimension = "ST01"
pattern = "forbidden_import"
from = { path = "**/controllers/**" }
to = { path = "**/repositories/**" }
severity = "error"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | MUST | Unique rule identifier |
| `name` | string | MUST | Human-readable rule name |
| `dimension` | string | RECOMMENDED | Dimension code (default: `"ST01"`) |
| `pattern` | string | MUST | Pattern type from the structural pattern catalog |
| `from` | PathMatcher | MAY | Source entity matcher (pattern-dependent) |
| `to` | PathMatcher | MAY | Target entity matcher (pattern-dependent) |
| `severity` | string | MAY | `"error"` or `"warning"` |

Additional fields are pattern-specific and defined in the ST01 substandard.

---

## 5. Exception Format (`fitness-exceptions.toml`)

### 5.1 Schema

Exceptions are organized by rule ID, then by entity path:

```toml
[max-cyclomatic."src/orchestration/engine.py::execute"]
value = 42
issue = "#138"

[max-cyclomatic."src/setup.py::configure_workspace"]
value = 28
issue = "#185"
```

### 5.2 Exception Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `value` | float | MAY | Current metric value at time of exception (ratchet budget) |
| `targets` | array | MAY | For dependency rules: specific import targets excepted |
| `issue` | string | MUST | GitHub issue reference (e.g., `"#138"`, `"org/repo#42"`) |

The `issue` field is **REQUIRED**. Exceptions without issue references MUST cause a validation error (`MISSING_ISSUE_REF`).

### 5.3 Ratchet Semantics

1. **Budget enforcement**: If `value` is specified and the actual metric value exceeds the exception's `value`, the exception is **insufficient** — the violation is reported as unexcepted.
2. **Stale detection**: If an entity no longer exists in the topology artifacts, or its metric value is now within the rule's threshold, the exception is **stale**. Stale exceptions MUST be reported in the validation output.
3. **Monotonic decrease**: When regenerating exceptions (via `aps run fitness ratchet`), new `value` entries MUST NOT exceed previous values. The ratchet only tightens.

---

## 6. System-Level Fitness Function

The system-level fitness function is the defining feature of architectural governance. It aggregates per-dimension scores into a single holistic assessment, enabling teams to:

1. **Understand overall architectural health** at a glance
2. **Analyze tradeoffs** — see how improving one dimension affects others
3. **Guide evolutionary decisions** — determine the impact of changes across all dimensions simultaneously
4. **Track trends** — monitor whether the architecture is improving or degrading over time

Per Ford et al.: "A system-wide fitness function is a combination of all the individual fitness functions, providing a holistic picture of architectural health."

### 6.1 Configuration

```toml
[system_fitness]
enabled = true                          # OPTIONAL — default: true
min_score = 0.7                         # OPTIONAL — minimum composite score (0.0–1.0)
include_incubating = false              # OPTIONAL — default: false (see §3.4)

[system_fitness.weights]
MT01 = 0.5                              # Maintainability (active)
MD01 = 0.5                              # Modularity (active)
# Incubating dimensions contribute only when include_incubating = true:
# ST01 = 0.0
# SC01 = 0.0
# LG01 = 0.0
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | bool | MAY | Enable system-level scoring (default: `true`) |
| `min_score` | float | MAY | Minimum composite score; below this = system failure (default: `0.7`) |
| `include_incubating` | bool | MAY | Whether incubating dimensions count toward the system score. Default `false` — the composite reflects only enforced governance. When `true`, incubating scores are included but their rule failures still cannot cause exit code `1` (§3.4). |
| `weights.<CODE>` | float | MAY | Weight for a dimension. Weights MUST sum to 1.0 across contributing dimensions (active, plus incubating when `include_incubating = true`). |

If weights are omitted, contributing dimensions receive equal weight. By default, only `active` dimensions contribute: this keeps the composite score an honest measure of what is actually enforced.

### 6.2 Per-Dimension Scoring

Each dimension receives a score in the range [0.0, 1.0] calculated as:

```
dimension_score = 1.0 - (unexcepted_violations / total_entities_evaluated)
```

Where:
- `unexcepted_violations` = number of violations not covered by exceptions
- `total_entities_evaluated` = total entities checked across all rules in that dimension

If a dimension has no rules or no entities, its score is `1.0` (no violations possible).

If a dimension has rules but the source artifact is missing (all rules skipped), the dimension is reported with `runtime_status = "skipped"` and excluded from the system-level calculation.

Incubating dimensions compute a score the same way, but are excluded from the composite unless `system_fitness.include_incubating = true` (§6.1).

### 6.3 System-Level Score Calculation

The system-level score is a weighted average of per-dimension scores:

```
system_score = Σ (weight_i × dimension_score_i) / Σ weight_i
```

Where the sums are over enabled, non-skipped dimensions only. Skipped dimensions are excluded and their weights redistributed proportionally.

### 6.4 Tradeoff Analysis

The system-level fitness report MUST include per-dimension scores alongside the composite, enabling tradeoff visibility:

```
System Fitness: 0.78 (threshold: 0.70) ✓

  MT01 Maintainability:     0.92  ████████████████████░░  (weight: 0.25)
  MD01 Modularity:          0.71  ██████████████░░░░░░░░  (weight: 0.25)
  ST01 Structural:          0.85  █████████████████░░░░░  (weight: 0.20)
  SC01 Security:            0.60  ████████████░░░░░░░░░░  (weight: 0.15)
  LG01 Legality:            0.95  ███████████████████░░░  (weight: 0.15)
```

This makes it immediately visible that security is the weakest dimension and where investment should focus.

### 6.5 Trend Tracking

When a previous report is available, the system-level report SHOULD include deltas:

```
  MT01 Maintainability:     0.92  (+0.03 ↑)
  MD01 Modularity:          0.71  (-0.05 ↓)  ← regression
  SC01 Security:            0.60  ( 0.00 →)
```

Trend data enables teams to detect architectural drift early and correlate changes across dimensions (e.g., "adding this feature improved maintainability but regressed modularity").

### 6.6 System-Level Exit Code

| Condition | Exit Code |
|-----------|-----------|
| System score >= `min_score` and no error-severity rule failures | `0` |
| System score < `min_score` OR any error-severity rule failure | `1` |
| Only warning-severity violations, system score >= `min_score` | `2` |

---

## 7. Report Format (`fitness-report.json`)

### 7.1 Schema

```json
{
  "version": "1.0.0",
  "timestamp": "2026-04-15T10:30:00Z",
  "summary": {
    "total_rules": 12,
    "passed": 10,
    "failed": 1,
    "warned": 1,
    "skipped": 0,
    "total_violations": 8,
    "excepted_violations": 5,
    "stale_exceptions": 1
  },
  "dimensions": {
    "MT01": {
      "name": "Maintainability",
      "runtime_status": "evaluated",
      "promotion_status": "active",
      "enforcement": "enforced",
      "score": 0.92,
      "rules_evaluated": 5,
      "rules_passed": 5,
      "rules_failed": 0,
      "rules_warned": 0,
      "rules_downgraded": 0,
      "total_violations": 3,
      "excepted_violations": 3
    },
    "SC01": {
      "name": "Security",
      "runtime_status": "evaluated",
      "promotion_status": "incubating",
      "enforcement": "advisory",
      "score": 0.60,
      "rules_evaluated": 2,
      "rules_passed": 1,
      "rules_failed": 0,
      "rules_warned": 1,
      "rules_downgraded": 1,
      "total_violations": 2,
      "excepted_violations": 1
    }
  },
  "system_fitness": {
    "score": 0.78,
    "min_score": 0.70,
    "passing": true,
    "weights_used": {
      "MT01": 0.25,
      "MD01": 0.25,
      "ST01": 0.20,
      "SC01": 0.15,
      "LG01": 0.15
    },
    "trend": {
      "previous_score": 0.81,
      "delta": -0.03,
      "direction": "declining",
      "dimension_deltas": {
        "MT01": 0.03,
        "MD01": -0.05,
        "ST01": 0.00,
        "SC01": 0.00,
        "LG01": 0.00
      }
    }
  },
  "results": [],
  "stale_exceptions": []
}
```

### 7.2 Summary Fields

| Field | Type | Description |
|-------|------|-------------|
| `total_rules` | integer | Number of rules evaluated |
| `passed` | integer | Rules with zero unexcepted violations |
| `failed` | integer | Rules with error severity and unexcepted violations |
| `warned` | integer | Rules with warning severity and unexcepted violations |
| `skipped` | integer | Rules that could not be evaluated (missing artifact) |
| `total_violations` | integer | Total violation count (excepted + unexcepted) |
| `excepted_violations` | integer | Violations covered by exceptions |
| `stale_exceptions` | integer | Exceptions that no longer apply |

### 7.3 Dimension Result Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Human-readable dimension name |
| `runtime_status` | string | Whether the dimension ran: `"evaluated"`, `"skipped"` (artifact missing), or `"disabled"` (opted out). |
| `promotion_status` | string | Enforcement posture per §3.4: `"active"`, `"incubating"`, or `"deprecated"`. Source of truth is the substandard manifest cross-checked against Appendix D. |
| `enforcement` | string | `"enforced"` when `promotion_status = "active"`; `"advisory"` when `"incubating"` (all rule severities downgraded to warning per §3.4). |
| `score` | float | Dimension score [0.0, 1.0] |
| `rules_evaluated` | integer | Rules evaluated in this dimension |
| `rules_passed` | integer | Rules passed in this dimension |
| `rules_failed` | integer | Rules failed in this dimension (only possible when `enforcement = "enforced"`) |
| `rules_warned` | integer | Rules warned in this dimension |
| `rules_downgraded` | integer | Rules whose configured `error` severity was downgraded to `warning` due to incubating status. Emits `INCUBATING_DIMENSION_ERROR_DOWNGRADED` per rule. |
| `total_violations` | integer | Total violations in this dimension |
| `excepted_violations` | integer | Excepted violations in this dimension |

### 7.4 System Fitness Fields

| Field | Type | Description |
|-------|------|-------------|
| `score` | float | Weighted composite score [0.0, 1.0] |
| `min_score` | float | Configured minimum threshold |
| `passing` | bool | Whether score >= min_score |
| `weights_used` | object | Actual weights used (after redistribution for skipped dims) |
| `trend` | object | Optional: delta from previous report |

### 7.5 Rule Result Fields

Each entry in `results` follows the same structure as EXP-V1-0003:

```json
{
  "rule_id": "max-cyclomatic",
  "rule_name": "Maximum Cyclomatic Complexity",
  "dimension": "MT01",
  "status": "pass",
  "violations": [
    {
      "entity": "src/orchestration/engine.py::execute",
      "field": "cyclomatic_complexity",
      "actual": 42.0,
      "threshold": 10.0,
      "direction": "max",
      "excepted": true
    }
  ],
  "exceptions_used": 1
}
```

### 7.6 Status Values

| Status | Meaning |
|--------|---------|
| `pass` | All entities within threshold (with or without exceptions) |
| `fail` | At least one unexcepted violation, severity = `error` |
| `warn` | At least one unexcepted violation, severity = `warning` |
| `skip` | Rule could not be evaluated (missing artifact) |

---

## 8. Validation Semantics

### 8.1 Threshold Evaluation

For each `[[rules.threshold]]`:

1. Resolve the topology artifact: `{config.topology_dir}/{rule.source}`
2. If the artifact does not exist, report status `skip` with diagnostic
3. Parse the JSON artifact and extract metric entries per entity
4. For each entity at the specified `scope`:
   a. If entity matches any `exclude` pattern, skip
   b. Extract the `field` value using dot-path navigation
   c. If `max` is set and `value > max`, record a violation
   d. If `min` is set and `value < min`, record a violation
5. For each violation, check exceptions:
   a. Look up `[rule.id."entity_path"]` in exception set
   b. If found and `value` is within budget (or no budget specified), mark as excepted
   c. Otherwise, mark as unexcepted
6. Determine rule status:
   - If no unexcepted violations → `pass`
   - If unexcepted violations and severity = `error` → `fail`
   - If unexcepted violations and severity = `warning` → `warn`

### 8.2 Dependency Evaluation

For each `[[rules.dependency]]`:

1. Load the coupling/dependency graph from topology artifacts
2. Resolve `from` and `to` path matchers against the graph
3. Based on `type`:
   - **`forbidden`**: If any edge exists from `from` to `to`, record a violation. If `circular = true`, detect cycles using Tarjan's SCC algorithm.
   - **`required`**: If no edge exists from `from` to `to`, record a violation.
   - **`allowed`**: If any edge exists from `from` to `to` that is NOT in the allowed set, record a violation.
4. Apply exception and severity logic as in threshold evaluation

### 8.3 Structural Evaluation

Structural rules delegate to the ST01 substandard's pattern catalog. Each pattern defines its own evaluation logic. See APS-V1-0002.ST01 for pattern definitions.

### 8.4 Stale Exception Detection

After evaluating all rules, scan all exceptions:

1. If the referenced entity does not exist in any evaluated artifact → stale (reason: `entity_not_found`)
2. If the entity exists but its metric value is now within the rule's threshold → stale (reason: `now_passing`)

Stale exceptions are reported in `stale_exceptions` but do not affect the overall pass/fail status.

### 8.5 Dimension Scoring

After all rules are evaluated, compute per-dimension scores as defined in section 6.2.

### 8.6 System-Level Scoring

After dimension scoring, compute the system-level fitness score as defined in section 6.3. Compare against `min_score` threshold.

### 8.7 Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All rules pass, system score >= min_score |
| `1` | At least one error-severity failure OR system score < min_score |
| `2` | Only warning-severity violations, system score >= min_score |

---

## 9. Adapter Contract

### 9.1 Purpose

External tools (security scanners, license checkers, accessibility auditors, load test frameworks) produce their own output formats. Adapters normalize this output into the fitness function contract, implementing the anti-corruption layer pattern.

### 9.2 Normalized Metrics Format

Adapters MUST produce output conforming to the standard wrapped artifact format:

```json
{
  "<scope_plural>": [
    {
      "id": "<entity_identifier>",
      "metrics": {
        "<field_name>": <numeric_value>,
        ...
      }
    }
  ]
}
```

This is the same format consumed by threshold rules from topology artifacts. The adapter's job is to translate external tool output into this shape.

### 9.3 Adapter Registration

Adapters are registered in `fitness.toml`:

```toml
[[adapters]]
id = "cargo-audit"
dimension = "SC01"
command = "cargo audit --json"                      # Command to produce raw output
input = "reports/cargo-audit.json"                  # OR: path to pre-existing output
output = "adapters/sc01/vulnerabilities.json"       # Normalized output path
normalizer = "builtin:cargo-audit"                  # Normalizer implementation
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | MUST | Unique adapter identifier |
| `dimension` | string | MUST | Which dimension this adapter serves |
| `command` | string | MAY | Shell command to produce raw output (mutually exclusive with `input`) |
| `input` | string | MAY | Path to pre-existing raw output (mutually exclusive with `command`) |
| `output` | string | MUST | Path where normalized output is written |
| `normalizer` | string | MUST | Normalizer identifier: `"builtin:<name>"` or `"script:<path>"` |

### 9.4 Built-in Normalizers

| Normalizer | Dimension | Tool | Status |
|------------|-----------|------|--------|
| `builtin:topology` | MT01, MD01 | APS-V1-0001 | Native (no adapter needed) |
| `builtin:cargo-audit` | SC01 | cargo-audit | Planned |
| `builtin:npm-audit` | SC01 | npm audit | Planned |
| `builtin:cargo-deny` | LG01 | cargo-deny | Planned |
| `builtin:vsa` | MD01 | VSA tool | Planned |

---

## 10. VSA Anti-Corruption Layer

### 10.1 Purpose

The Vertical Slice Architecture (VSA) tool produces slice-level metrics including Slice Independence Score (SIS), cross-context coupling, and boundary violations. These metrics are valuable for modularity assessment but come from a separate tool with its own data model.

The VSA adapter normalizes VSA output into the fitness function contract without the fitness standard depending on VSA internals.

### 10.2 VSA Normalized Output

```json
{
  "slices": [
    {
      "id": "order-processing",
      "metrics": {
        "sis_score": 0.85,
        "cross_context_coupling": 2,
        "boundary_violations": 0,
        "slice_cohesion": 0.92
      }
    }
  ]
}
```

### 10.3 VSA Adapter Registration

```toml
[[adapters]]
id = "vsa"
dimension = "MD01"
input = ".vsa/analysis.json"
output = "adapters/md01/vsa-slices.json"
normalizer = "builtin:vsa"
```

### 10.4 VSA Fitness Rules

```toml
[[rules.threshold]]
id = "min-sis-score"
name = "Minimum Slice Independence Score"
dimension = "MD01"
source = "adapters/md01/vsa-slices.json"
field = "metrics.sis_score"
min = 0.7
scope = "slice"
severity = "error"

[[rules.threshold]]
id = "max-cross-context-coupling"
name = "Maximum Cross-Context Coupling"
dimension = "MD01"
source = "adapters/md01/vsa-slices.json"
field = "metrics.cross_context_coupling"
max = 3
scope = "slice"
severity = "warning"
```

---

## 11. CLI Interface (Informative)

> This section is informative. The CLI is provided by the `aps` tool.

### 11.1 Commands

```bash
# Validate all rules
aps run fitness validate <path>

# Validate specific dimensions only
aps run fitness validate . --dimensions MT01,MD01

# Write JSON report
aps run fitness validate . --report fitness-report.json

# Auto-generate exceptions from current violations
aps run fitness ratchet <path>

# Show system-level fitness summary
aps run fitness summary <path>

# Show report in specific format
aps run fitness report <path> --format human|json
```

### 11.2 Options

| Option | Description |
|--------|-------------|
| `--config <path>` | Path to fitness.toml (default: `./fitness.toml`) |
| `--report <path>` | Write JSON report to file |
| `--dimensions <list>` | Comma-separated dimension codes to evaluate |
| `--format <fmt>` | Output format: `human` or `json` (default: `human`) |
| `--previous <path>` | Path to previous report for trend analysis |

---

## 12. Error Codes

| Code | Description |
|------|-------------|
| `MISSING_FITNESS_TOML` | No `fitness.toml` found at specified path |
| `INVALID_RULE` | Rule definition is malformed or missing required fields |
| `MISSING_TOPOLOGY_DIR` | Configured `topology_dir` does not exist |
| `MISSING_ISSUE_REF` | Exception is missing required `issue` field |
| `STALE_EXCEPTION` | Exception references entity that no longer violates |
| `THRESHOLD_EXCEEDED` | Metric value exceeds rule threshold |
| `INVALID_DIMENSION` | Unknown dimension code in configuration |
| `DIMENSION_DISABLED_NO_REASON` | Default-enabled dimension disabled without reason |
| `SYSTEM_FITNESS_BELOW_THRESHOLD` | System-level score below configured minimum |
| `INVALID_STRUCTURAL_PATTERN` | Structural rule references unknown pattern |
| `INVALID_WEIGHTS` | System fitness weights do not sum to 1.0 |
| `DEPENDENCY_CYCLE_DETECTED` | Circular dependency found (forbidden) |
| `INCUBATING_DIMENSION_ERROR_DOWNGRADED` | A rule on an `incubating` dimension declared `severity = "error"`; it was downgraded to `warning` per §3.4. Diagnostic includes the dimension code and rule ID so users can locate what is and is not being enforced. |
| `PROMOTION_REQUIREMENT_UNMET` | A dimension is declared `active` in its substandard manifest but does not satisfy one or more of the R1–R5 promotion requirements (§3.3). Reported at config validation time. |

---

## Appendix A: Ford's Fitness Function Taxonomy

This standard implements the following categories from Ford et al. (2017):

| Ford Category | This Standard | Notes |
|---------------|--------------|-------|
| Triggered fitness functions | All rules | Every rule is evaluated on every change (CI) |
| Atomic fitness functions | Individual rules | Each rule asserts on one metric/constraint |
| Holistic fitness function | System-level fitness | Aggregates all dimensions |
| Static fitness functions | MT01, MD01, ST01, LG01 | Evaluated from source code / static analysis |
| Dynamic fitness functions | SC01, PF01, AV01 | Evaluated from runtime / scanner output |
| Temporal fitness functions | Trend tracking | Deltas from previous reports |
| Intentional fitness functions | All configured rules | Explicitly declared in fitness.toml |
| Emergent fitness functions | System-level composite | Emerges from combining dimensions |

### Appendix B: Migration from EXP-V1-0003

Every EXP-V1-0003 `fitness.toml` is valid under APS-V1-0002. The following behaviors apply for backward compatibility:

1. If `[dimensions]` is absent, default-enabled dimensions auto-enable. Each dimension's enforcement behavior is determined by its `promotion_status` (§3.4), not its presence in `[dimensions]`.
2. If `[system_fitness]` is absent, system-level scoring uses equal weights and `min_score = 0.7`.
3. If a rule has no `dimension` field, it is assigned based on heuristics:
   - Rules with `source` matching `metrics/complexity*` or `metrics/functions*` → `MT01`
   - Rules with `source` matching `metrics/coupling*` or `metrics/modules*` → `MD01`
   - Rules with `type` (dependency rules) → `MD01`
   - All other rules → `MT01` (safe default)
4. Report format is extended (new fields added), never reduced.
5. All EXP-V1-0003 error codes are preserved.

### Appendix C: Validation Checklist

- [ ] `fitness.toml` present and parseable
- [ ] All rules have valid IDs and at least one of max/min (threshold) or type (dependency)
- [ ] `topology_dir` exists
- [ ] All default-enabled dimensions either active or disabled with reason
- [ ] All exceptions have issue references
- [ ] System fitness weights sum to 1.0 (if specified)
- [ ] All referenced adapters are available
- [ ] No stale exceptions (advisory)

---

## Appendix D: Current Implementation Status

This appendix is **normative disclosure**. It documents exactly which promotion requirements (§3.3) each dimension currently satisfies, so consumers know what they can and cannot enforce. Each dimension's substandard manifest MUST agree with this table; divergence produces `PROMOTION_REQUIREMENT_UNMET`.

Legend: ✓ met · ✗ unmet · ◐ partial

| Dim | Status | R1 Metric | R2 Algorithm | R3 Schema file | R4 Defaults | R5 Reference impl | Producer / Blocker |
|-----|--------|-----------|--------------|----------------|-------------|-------------------|--------------------|
| MT01 | **active** | ✓ Cyclomatic, cognitive, Halstead | ✓ APS-V1-0001 `metrics/functions.json` | ✓ `functions.schema.json` published | ✓ McCabe 1976, SonarSource 2017, Halstead 1977 | ✓ `architecture-fitness-mt01` crate | — |
| MD01 | **active** | ✓ Ca, Ce, I, A, D | ✓ APS-V1-0001 `metrics/coupling.json` (LANG01-rust writer) | ✓ `coupling.schema.json` published | ✓ Martin 1994, 2003 | ✓ `architecture-fitness-md01` crate | — |
| ST01 | incubating | ◐ Structural patterns ✓ · CK metrics ✗ | ◐ Structural ✓ · CK metrics need class-level AST (no producer) | ✗ no structural-rule or CK schemas published | ◐ Structural ✓ · CK ✗ | ✗ stub crate | CK metrics (DIT, CBO, LCOM) blocked on class-level analyzer. Structural subset can promote independently after schemas + non-stub crate. |
| SC01 | incubating | ✓ CVSS severity, vulnerability count | ✗ Adapter framework declared but not invoked; no normalizers implemented | ✗ `adapter-output.schema.json` not published | ✓ CVSS thresholds cited | ✗ stub crate | Requires (a) adapter runner in engine, (b) `builtin:cargo-audit` normalizer, (c) adapter-output schema |
| LG01 | incubating | ✓ License category (permissive/weak-copyleft/etc.) | ✗ Adapter framework not implemented | ✗ no adapter schema | ✓ category policy defaults | ✗ stub crate | Requires adapter runner + `builtin:cargo-deny` + schema |
| AC01 | incubating | ✓ WCAG level (A/AA/AAA), violation count | ✗ Adapter framework not implemented | ✗ no adapter schema | ✓ WCAG 2.1 AA defaults | ✗ stub crate | Requires adapter runner + axe-core / pa11y normalizer + schema |
| PF01 | incubating | ✓ Latency (p50/p95/p99), throughput | ✗ Adapter framework not implemented | ✗ no adapter schema | ◐ Defaults are project-specific; no universal citation | ✗ stub crate | Requires adapter runner + k6 / Criterion normalizer + schema |
| AV01 | incubating | ✓ SLO, error budget, MTTR | ✗ Adapter framework not implemented | ✗ no adapter schema | ◐ SLO targets are project-specific | ✗ stub crate | Requires adapter runner + monitor-output normalizer + schema |

### D.1 Promotion Roadmap

The ordering below reflects the shortest path to expanding enforceable governance. Every promotion to `active` MUST satisfy all of R1–R5, including a published JSON Schema file (§3.5) for every artifact the dimension consumes or produces.

1. **Tier 1 — Promote MT01 and MD01** (shortest path from incubating → active)
   - In APS-V1-0001: publish `schemas/coupling.schema.json`, `schemas/complexity.schema.json`, `schemas/modules.schema.json`
   - In APS-V1-0001 LANG01-rust: add a flat `coupling.json` writer conforming to the new schema (Martin metrics are already computed)
   - In APS-V1-0002: publish `schemas/fitness-config.schema.json`, `schemas/fitness-exceptions.schema.json`, `schemas/fitness-report.schema.json`
   - Replace MT01 and MD01 stub crates with reference implementations that register default rules and validate artifact presence
   - Wire strict-artifact enforcement in the engine: when an `active` dimension's required artifact is missing, emit `PROMOTION_REQUIREMENT_UNMET` rather than silent `Skip`
   - Ship `INTEGRATION.md` showing end-to-end pipeline (topology → fitness → CI)
   - Flip MT01 and MD01 to `active` in §1.4, §3.1, and Appendix D. Require an ADR documenting R1–R5 satisfaction.

2. **Tier 2 — Promote ST01 structural subset** — structural rules (forbidden_import, required_import, layer_enforcement) independently satisfy R1–R5 once their schema and reference implementation ship. Promote as a scoped subset; CK metrics remain `incubating`.

3. **Tier 3 — Adapter framework** — one-time infrastructure that unblocks SC01, LG01, AC01, PF01, AV01. Requires: `run_adapters()` invocation in the engine, normalizer registry, `ADAPTER_NOT_FOUND` failure path, and `adapter-output.schema.json` published under APS-V1-0002.

4. **Tier 4 — Promote SC01** — after `builtin:cargo-audit` normalizer ships. Highest security value; concrete CVSS thresholds; established tool. First adapter-backed dimension to reach `active`.

5. **Tier 5 — Promote LG01** — after `builtin:cargo-deny`. Piggybacks on the same tool family as SC01.

6. **Tier 6 — AC01 / PF01 / AV01** — project-by-project. Promotion to `active` requires either a per-project ADR (for project-specific defaults) or an industry-wide citation for the normalizer's output.

### D.2 Auditing This Appendix

This appendix MUST be kept in sync with substandard manifests. An ADR is required to change any row. Implementers running `fitness validate` SHOULD cross-check each enabled dimension against this table and emit `PROMOTION_REQUIREMENT_UNMET` on divergence.
