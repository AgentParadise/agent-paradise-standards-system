# Architecture Fitness Functions — Overview

## What is this?

**EXP-V1-0003** defines a declarative format for **architecture fitness functions** — automated assertions on architectural properties that run in CI and fail on violations. It is the *assertion layer* on top of APS-V1-0001's *measurement layer*.

Instead of writing bespoke Python/Rust tests for every metric threshold, you declare rules in a `fitness.toml` file and let the validator evaluate them against topology artifacts.

## Why does it matter?

As codebases grow, architectural constraints drift. Fitness functions catch this automatically:

1. **Declarative** — Rules expressed as TOML, not imperative test code
2. **Reusable** — Same format works across any codebase with topology artifacts
3. **Ratcheting** — Violations can be excepted with mandatory issue references; budgets can only shrink
4. **CI-friendly** — Exit codes and JSON reports for automation

## Quick Example

**fitness.toml:**
```toml
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-cognitive"
name = "Maximum Cognitive Complexity"
source = "metrics/functions.json"
field = "metrics.cognitive"
max = 25
scope = "function"
```

**Run:**
```bash
aps run fitness validate .
```

**Output:**
```
Fitness Validation Report
========================
  [PASS] Maximum Cognitive Complexity (max-cognitive)

Summary: 1 passed, 0 failed, 0 warned, 0 violations, 0 stale exceptions
```

If a function exceeds the threshold, the validator reports the violation:
```
  [FAIL] Maximum Cognitive Complexity (max-cognitive)
         python:engine::execute = 42 (threshold: 25 Max)
```

## Key Features

### 1. Threshold Rules

Assert that a metric value per entity stays within bounds:

```toml
[[rules.threshold]]
id = "max-cyclomatic"
name = "Maximum Cyclomatic Complexity"
source = "metrics/functions.json"
field = "metrics.cyclomatic"         # Dot-path navigation supported
max = 15
scope = "function"
severity = "error"
exclude = ["**/test_*"]
```

### 2. Exception Ratcheting

Track known violations with mandatory issue references. Budgets can only shrink over time:

```toml
# fitness-exceptions.toml
[max-cyclomatic."src/engine.py::execute"]
value = 42
issue = "#138"
```

- If the violation is fixed (metric drops below threshold), the exception becomes **stale**
- If the metric exceeds the budget, the exception is **insufficient** — CI fails
- Stale exceptions are reported so they can be cleaned up

### 3. Wrapped Topology Artifacts

Auto-detects wrapper objects commonly produced by APS-V1-0001:

```json
{
  "functions": [
    { "id": "python:module::func_name", "metrics": { "cognitive": 12 } }
  ]
}
```

Entity identifiers resolve in priority order: `id` > `path` > `name` > `entity`.

### 4. Dot-Path Field Navigation

Navigate nested JSON structures with dot-notation:

```toml
field = "metrics.martin.ce"    # entity["metrics"]["martin"]["ce"]
```

## Architecture

```
APS-V1-0001 (measure) → .topology/metrics/*.json
                              ↓
EXP-V1-0003 (assert)  → fitness.toml rules
                              ↓
                         fitness-report.json
```

The relationship is strictly one-way: this standard **consumes** topology artifacts but never modifies them.

## Getting Started

### 1. Generate Topology Artifacts

```bash
aps run topology analyze . --output .topology
```

### 2. Create fitness.toml

```toml
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-loc"
name = "Maximum Lines of Code"
source = "metrics/modules.json"
field = "metrics.lines_of_code"
max = 500
scope = "module"
```

### 3. Validate

```bash
aps run fitness validate .
```

### 4. Handle Violations

For existing violations that can't be fixed immediately:

```toml
# fitness-exceptions.toml
[max-loc."packages.my-module.src.big_file"]
value = 800
issue = "#42"
```

### 5. Add to CI

```yaml
- name: Check fitness thresholds
  run: aps run fitness validate .
```

## Use Cases

### For Developers
- **Prevent drift** — New code that exceeds thresholds fails CI immediately
- **Track debt** — Exceptions create visibility into known violations
- **Simplify rules** — Replace bespoke test scripts with declarative TOML

### For AI Agents
- **Read fitness-report.json** — Understand which metrics are violated
- **Add exceptions** — Acknowledge violations with issue references
- **Monitor ratchet** — Track exceptions shrinking over time

### For Teams
- **Code reviews** — Violations flagged before merge
- **Onboarding** — Rules are self-documenting in fitness.toml
- **Architecture governance** — Thresholds enforce team agreements

## Status

**Experimental** — This standard is in incubation. Feedback welcome!

### What's Working
- ✅ Threshold rule evaluation
- ✅ Exception ratcheting with stale detection
- ✅ Wrapped artifact format support
- ✅ Dot-path field navigation
- ✅ CLI: `aps run fitness validate`
- ✅ Dogfooded in syntropic137

### Planned
- ⏳ Dependency rules (v0.2.0) — forbidden/allowed/required import constraints
- ⏳ Structural rules (v0.3.0) — AST-level pattern assertions
- ⏳ `aps run fitness ratchet` — auto-generate exceptions from current violations

## Related Standards

- **APS-V1-0001 (Code Topology)** — Produces the `.topology/metrics/` artifacts consumed by fitness rules
- **EXP-V1-0002 (TODO Tracker)** — Complementary: tracks TODO comments, fitness tracks metric thresholds

## Learn More

- Read the [full specification](./01_spec.md)
- Check out [examples](../examples/)
- See [agent skills](../agents/skills/)

---

*This is an experimental standard. It may change significantly before promotion to official status.*
