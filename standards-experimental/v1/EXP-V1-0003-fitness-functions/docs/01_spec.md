# EXP-V1-0003 — Architecture Fitness Functions

**Version**: 0.1.0
**Status**: Experimental
**Category**: Technical

⚠️ **EXPERIMENTAL**: This standard is in incubation and may change significantly before promotion.

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Scope and Authority

### 1.1 Purpose

This standard defines a **declarative format for architecture fitness functions** — automated assertions on architectural properties that run in CI and fail on violations. Fitness functions are the *assertion layer* on top of APS-V1-0001's *measurement layer*.

The format is designed to be:

1. **Declarative** — Rules expressed as TOML, not imperative test code
2. **Reusable** — Same rule format works across any codebase with topology artifacts
3. **Ratcheting** — Violations can be excepted with mandatory issue references; budgets can only shrink
4. **CI-friendly** — Exit codes and JSON reports for automation

### 1.2 Scope

This standard covers:

- **Rule format specification** — `fitness.toml` schema for threshold, dependency, and structural rules
- **Exception format specification** — `fitness-exceptions.toml` schema for tracked violations
- **Report format specification** — `fitness-report.json` schema for validation output
- **Validation semantics** — How rules are evaluated against topology artifacts
- **Ratchet semantics** — How exceptions degrade over time

This standard does NOT cover:

- Topology artifact generation (see APS-V1-0001)
- Language-specific AST analysis (delegated to substandards)
- CI pipeline configuration (informative only)

### 1.3 Relationship to APS-V1-0001

APS-V1-0001 (Code Topology) defines the **measurement layer** — it produces `.topology/metrics/` artifacts containing complexity, coupling, and structural data.

This standard defines the **assertion layer** — it consumes those artifacts and evaluates architectural rules against them.

```
APS-V1-0001 (measure) → .topology/metrics/*.json
                              ↓
EXP-V1-0003 (assert)  → fitness.toml rules
                              ↓
                         fitness-report.json
```

### 1.4 Normative References

- Ford, N. et al. (2017). *Building Evolutionary Architectures* — Fitness function taxonomy
- SonarSource Cognitive Complexity — Metric definitions
- [ArchUnit](https://www.archunit.org/) — FreezingArchRule ratchet pattern
- [dependency-cruiser](https://github.com/sverweij/dependency-cruiser) — Forbidden/allowed/required rule model
- [pytest-archon](https://github.com/jwbargsten/pytest-archon) — Python import rule assertions

---

## 2. Core Definitions

### 2.1 Fitness Function (Ford)

An **architecture fitness function** is an objective integrity assessment of some architecture characteristic. Per Ford's taxonomy:

| Property | Description |
|----------|-------------|
| **Automated** | Executable without human judgment |
| **Continuous** | Runs on every change (CI) |
| **Architectural** | Asserts on system-level properties, not unit behavior |

### 2.2 Rule

A **rule** is a single architectural assertion declared in `fitness.toml`. Rules have:

- **ID** — Unique identifier (e.g., `max-cyclomatic`)
- **Type** — `threshold`, `dependency`, or `structural`
- **Severity** — `error` (blocks CI) or `warning` (advisory)

### 2.3 Exception

An **exception** is a tracked deviation from a rule, declared in `fitness-exceptions.toml`. Exceptions MUST reference a GitHub issue. Exceptions represent technical debt that is acknowledged and planned for resolution.

### 2.4 Ratchet

A **ratchet** is the mechanism by which exception budgets can only shrink over time. If a violation is fixed (metric drops below threshold), the exception becomes **stale** and MUST be removed. New exceptions MUST NOT exceed the current violation count.

### 2.5 Violation

A **violation** is a specific entity (module, file, function) that fails a rule's assertion. A violation may be **excepted** (tracked in exceptions file) or **unexcepted** (causes rule failure).

---

## 3. Rule Format (`fitness.toml`)

The rule file MUST be named `fitness.toml` and SHOULD be placed at the repository root.

### 3.1 Config Section

```toml
[config]
topology_dir = ".topology"                          # REQUIRED — path to topology artifacts
exceptions = "fitness-exceptions.toml"              # OPTIONAL — path to exceptions file (default: "fitness-exceptions.toml")
severity_default = "error"                          # OPTIONAL — default severity for rules (default: "error")
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `topology_dir` | string | MUST | Path to `.topology/` directory relative to repo root |
| `exceptions` | string | MAY | Path to exceptions file (default: `fitness-exceptions.toml`) |
| `severity_default` | string | MAY | Default severity: `"error"` or `"warning"` (default: `"error"`) |

### 3.2 Threshold Rules

Threshold rules assert that a metric value for each entity does not exceed (or fall below) a given bound.

```toml
[[rules.threshold]]
id = "max-cyclomatic"
name = "Maximum Cyclomatic Complexity"
source = "metrics/complexity.json"                  # Topology artifact path
field = "cyclomatic_complexity"                     # JSON field to evaluate
max = 15                                            # Upper bound (fail if value > max)
scope = "function"                                  # Entity scope: "module", "file", "function"
severity = "error"                                  # Override default severity
exclude = ["**/test_*", "**/tests/**"]              # Glob patterns to exclude
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | MUST | Unique rule identifier |
| `name` | string | MUST | Human-readable rule name |
| `source` | string | MUST | Path to topology artifact (relative to `topology_dir`) |
| `field` | string | MUST | JSON field path to evaluate (supports dot-notation, e.g., `metrics.cognitive`) |
| `max` | float | MUST (one of max/min) | Upper bound — violation if `value > max` |
| `min` | float | MUST (one of max/min) | Lower bound — violation if `value < min` |
| `scope` | string | MUST | Entity granularity: `"module"`, `"file"`, `"function"` |
| `severity` | string | MAY | `"error"` or `"warning"` (default: config default) |
| `exclude` | array | MAY | Glob patterns for entities to skip |

At least one of `max` or `min` MUST be specified. Both MAY be specified simultaneously.

#### 3.2.1 Field Dot-Notation

The `field` property supports dot-notation for navigating nested JSON structures. Each dot-separated segment traverses one level of object nesting:

```toml
field = "cyclomatic_complexity"          # flat: entity["cyclomatic_complexity"]
field = "metrics.cognitive"              # nested: entity["metrics"]["cognitive"]
field = "metrics.martin.ce"              # deep: entity["metrics"]["martin"]["ce"]
```

Single-segment fields (no dots) work identically to previous versions.

#### 3.2.2 Wrapped Topology Artifacts

Topology artifacts MAY use a wrapper object instead of a flat object or bare array. The validator auto-detects wrapped formats:

```json
{
  "functions": [
    { "id": "python:module::func_name", "metrics": { "cognitive": 12 } }
  ]
}
```

Detection rules:
1. **Scope-derived key**: The `scope` field maps to a plural wrapper key (`"function"` → `"functions"`, `"module"` → `"modules"`, `"slice"` → `"slices"`, `"file"` → `"files"`). If that key exists and contains an array, it is unwrapped.
2. **Fallback heuristic**: If the scope-derived key is not found, but exactly one key in the object has an array value, that array is unwrapped.
3. **Flat fallback**: If neither condition is met, the object is treated as a flat entity map (backward compatible).

Within unwrapped arrays, entity identifiers are resolved in priority order: `id` > `path` > `name` > `entity`.

### 3.3 Dependency Rules (v0.2.0)

> ⚠️ **Planned for v0.2.0** — format documented here for feedback; not evaluated in v0.1.0.

Dependency rules assert constraints on the import/coupling graph.

```toml
[[rules.dependency]]
id = "no-circular-deps"
name = "No Circular Dependencies"
type = "forbidden"                                  # "forbidden", "allowed", or "required"
from = { path = "src/**" }                          # Source path matcher
to = { path = "src/**" }                            # Target path matcher
circular = true                                     # Detect cycles (forbidden type only)
severity = "error"
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | MUST | Unique rule identifier |
| `name` | string | MUST | Human-readable rule name |
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

### 3.4 Structural Rules (v0.3.0)

> ⚠️ **Planned for v0.3.0** — AST-level pattern assertions, delegated to language-specific substandards.

---

## 4. Exception Format (`fitness-exceptions.toml`)

The exception file tracks known violations that are acknowledged but not yet resolved.

### 4.1 Schema

Exceptions are organized by rule ID, then by entity path:

```toml
[max-cyclomatic."src/orchestration/engine.py::execute"]
value = 42
issue = "#138"

[max-cyclomatic."src/setup.py::configure_workspace"]
value = 28
issue = "#185"

[max-loc."src/setup.py"]
value = 2284
issue = "#185"
```

### 4.2 Exception Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `value` | float | MAY | Current metric value at time of exception. Used for ratchet: if actual value exceeds this, the exception is insufficient. |
| `targets` | array | MAY | For dependency rules: specific import targets excepted |
| `issue` | string | MUST | GitHub issue reference (e.g., `"#138"`, `"org/repo#42"`) |

The `issue` field is **REQUIRED**. Exceptions without issue references MUST cause a validation error (`MISSING_ISSUE_REF`).

### 4.3 Ratchet Semantics

1. **Budget enforcement**: If `value` is specified and the actual metric value exceeds the exception's `value`, the exception is **insufficient** — the violation is reported as unexcepted.
2. **Stale detection**: If an entity no longer exists in the topology artifacts, or its metric value is now within the rule's threshold, the exception is **stale**. Stale exceptions MUST be reported in the validation output.
3. **Monotonic decrease**: When regenerating exceptions (via `aps run fitness ratchet`), new `value` entries MUST NOT exceed previous values. The ratchet only tightens.

---

## 5. Report Format (`fitness-report.json`)

The validation report is a JSON document containing all rule results.

### 5.1 Schema

```json
{
  "version": "0.1.0",
  "timestamp": "2026-03-12T10:30:00Z",
  "summary": {
    "total_rules": 5,
    "passed": 3,
    "failed": 1,
    "warned": 1,
    "total_violations": 4,
    "excepted_violations": 2,
    "stale_exceptions": 1
  },
  "results": [
    {
      "rule_id": "max-cyclomatic",
      "rule_name": "Maximum Cyclomatic Complexity",
      "status": "fail",
      "violations": [
        {
          "entity": "src/orchestration/engine.py::execute",
          "field": "cyclomatic_complexity",
          "actual": 42.0,
          "threshold": 15.0,
          "direction": "max",
          "excepted": true
        },
        {
          "entity": "src/api/routes.py::handle_request",
          "field": "cyclomatic_complexity",
          "actual": 22.0,
          "threshold": 15.0,
          "direction": "max",
          "excepted": false
        }
      ],
      "exceptions_used": 1
    }
  ],
  "stale_exceptions": [
    {
      "rule_id": "max-loc",
      "entity": "src/old_module.py",
      "reason": "entity_not_found"
    }
  ]
}
```

### 5.2 Summary Fields

| Field | Type | Description |
|-------|------|-------------|
| `total_rules` | integer | Number of rules evaluated |
| `passed` | integer | Rules with zero unexcepted violations |
| `failed` | integer | Rules with error severity and unexcepted violations |
| `warned` | integer | Rules with warning severity and unexcepted violations |
| `total_violations` | integer | Total violation count (excepted + unexcepted) |
| `excepted_violations` | integer | Violations covered by exceptions |
| `stale_exceptions` | integer | Exceptions that no longer apply |

### 5.3 Status Values

| Status | Meaning |
|--------|---------|
| `pass` | All entities within threshold (with or without exceptions) |
| `fail` | At least one unexcepted violation, severity = `error` |
| `warn` | At least one unexcepted violation, severity = `warning` |
| `skip` | Rule could not be evaluated (missing artifact) |

---

## 6. Validation Semantics

### 6.1 Threshold Evaluation

For each `[[rules.threshold]]`:

1. Resolve the topology artifact: `{config.topology_dir}/{rule.source}`
2. If the artifact does not exist, report status `skip` with diagnostic
3. Parse the JSON artifact and extract metric entries per entity
4. For each entity at the specified `scope`:
   a. If entity matches any `exclude` pattern, skip
   b. Extract the `field` value
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

### 6.2 Stale Exception Detection

After evaluating all rules, scan all exceptions:

1. If the referenced entity does not exist in any evaluated artifact → stale (reason: `entity_not_found`)
2. If the entity exists but its metric value is now within the rule's threshold → stale (reason: `now_passing`)

Stale exceptions are reported in `stale_exceptions` but do not affect the overall pass/fail status in v0.1.0.

### 6.3 Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All rules pass (no unexcepted error-severity violations) |
| `1` | At least one rule failed (unexcepted error-severity violation) |

Warning-severity violations do not affect the exit code.

---

## 7. CLI Interface (Informative)

> This section is informative. The CLI is provided by the `aps` tool.

### 7.1 Commands

```bash
# Validate rules against topology artifacts
aps run fitness validate <path>
aps run fitness validate .                   # Use fitness.toml in current directory
aps run fitness validate . --config custom-fitness.toml

# Generate exceptions from current violations (v0.2.0)
aps run fitness ratchet <path>

# Generate report only (no exit code enforcement)
aps run fitness report <path>
```

### 7.2 Options

| Option | Description |
|--------|-------------|
| `--config <path>` | Path to fitness.toml (default: `./fitness.toml`) |
| `--format <fmt>` | Output format: `human`, `json` (default: `human`) |
| `--report <path>` | Write JSON report to file |

---

## 8. Error Codes

| Code | Description |
|------|-------------|
| `MISSING_FITNESS_TOML` | No `fitness.toml` found at specified path |
| `INVALID_RULE` | Rule definition is malformed or missing required fields |
| `MISSING_TOPOLOGY_DIR` | Configured `topology_dir` does not exist |
| `MISSING_ISSUE_REF` | Exception is missing required `issue` field |
| `STALE_EXCEPTION` | Exception references entity that no longer violates |
| `THRESHOLD_EXCEEDED` | Metric value exceeds rule threshold |

---

## 9. Promotion Criteria

This experiment can be promoted to an official standard when:

- [ ] Peer review complete
- [ ] At least two codebases using `fitness.toml` format
- [ ] Threshold evaluation passes integration tests against real topology artifacts
- [ ] Dependency rule format finalized (v0.2.0)
- [ ] All open questions resolved
