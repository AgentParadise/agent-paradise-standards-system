# Architecture Fitness Functions — Overview

## What is this?

**APS-V1-0002** defines a comprehensive architectural governance framework based on the principles of evolutionary architecture (Ford et al., 2017). It provides **fitness functions** — automated, continuous, objective assertions on architectural properties — organized into composable dimensional substandards.

The core insight from *Building Evolutionary Architectures*:

> **Architecture = Requirements + Architectural Characteristics**

- **Requirements** (functional behavior) are tested by unit, functional, and integration tests.
- **Architectural Characteristics** (the "-ilities") are tested by **fitness functions** — this standard.

Fitness functions are to architecture what unit tests are to business logic.

## Why does it matter?

As codebases grow, architectural constraints drift. Without automated governance:

- Complexity creeps into functions that were once simple
- Coupling tightens between modules that should be independent
- Dependencies accumulate in directions that violate design principles
- Security vulnerabilities enter through unscanned dependencies
- License compliance erodes as new libraries are added

Fitness functions catch this drift automatically, continuously, on every commit.

## The Dimensional Model

Architecture has multiple dimensions. Each dimension is governed by a substandard:

| Substandard | Dimension | What It Protects |
|-------------|-----------|-----------------|
| **MT01** | Maintainability | Readability, testability, complexity |
| **MD01** | Modularity & Coupling | Separation of concerns, dependency structure |
| **ST01** | Structural Integrity | Design patterns, class design, layer enforcement |
| **SC01** | Security | Vulnerability freedom, supply chain safety |
| **LG01** | Legality | License compliance, IP safety |
| **AC01** | Accessibility | WCAG compliance (opt-in) |
| **PF01** | Performance | Latency, throughput regression (opt-in) |
| **AV01** | Availability | Resilience, uptime (opt-in) |

Dimensions are composable — enable what matters for your project, disable what doesn't.

## System-Level Fitness Function

The defining feature of this standard is the **system-level fitness function** — a weighted aggregation of all dimension scores into a single holistic assessment.

```
System Fitness: 0.78 (threshold: 0.70) ✓

  MT01 Maintainability:     0.92  ████████████████████░░  (+0.03 ↑)
  MD01 Modularity:          0.71  ██████████████░░░░░░░░  (-0.05 ↓)
  ST01 Structural:          0.85  █████████████████░░░░░  ( 0.00 →)
  SC01 Security:            0.60  ████████████░░░░░░░░░░  ( 0.00 →)
  LG01 Legality:            0.95  ███████████████████░░░  ( 0.00 →)
```

This enables teams to:
- **See overall health** at a glance
- **Analyze tradeoffs** — how improving one dimension affects others
- **Guide decisions** — determine the impact of changes across all dimensions
- **Track trends** — detect architectural drift early

## Architecture

```
APS-V1-0001 (measure)    → .topology/metrics/*.json ─┐
                                                      ├→ APS-V1-0002 (assert)
External tools (scan)     → adapter normalization ────┘       │
                                                              ├→ Per-dimension scores
                                                              ├→ System-level fitness
                                                              └→ fitness-report.json
```

## Quick Example

**fitness.toml:**
```toml
[config]
topology_dir = ".topology"

[system_fitness]
min_score = 0.7

[[rules.threshold]]
id = "max-cognitive"
name = "Maximum Cognitive Complexity"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cognitive_complexity"
max = 15
scope = "function"

[[rules.threshold]]
id = "max-fan-out"
name = "Maximum Efferent Coupling"
dimension = "MD01"
source = "metrics/coupling.json"
field = "efferent_coupling"
max = 20
scope = "module"
```

**Run:**
```bash
aps run fitness validate .
```

## What's New vs EXP-V1-0003

Promoted from EXP-V1-0003 with significantly expanded scope:

| Feature | EXP-V1-0003 | APS-V1-0002 |
|---------|-------------|-------------|
| Threshold rules | Yes | Yes (carried forward) |
| Exception ratcheting | Yes | Yes (carried forward) |
| Stale detection | Yes | Yes (carried forward) |
| Dimensional model | No | Yes — 8 composable dimensions |
| System-level fitness | No | Yes — weighted composite with tradeoff analysis |
| Metrics catalog | Informal | Normative — 20+ metrics with formulas and citations |
| Dependency rules | Planned | Normative |
| Structural rules | Planned | Normative |
| Adapter contract | No | Yes — anti-corruption layer for external tools |
| VSA integration | No | Yes — via adapter |
| Trend tracking | No | Yes — delta from previous reports |

**Backward compatible**: Every existing EXP-V1-0003 `fitness.toml` works unchanged.

## Getting Started

### 1. Generate topology artifacts
```bash
aps run topology analyze . --output .topology
```

### 2. Create fitness.toml
```toml
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-cyclomatic"
name = "Maximum Cyclomatic Complexity"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 10
scope = "function"
```

### 3. Validate
```bash
aps run fitness validate .
```

### 4. Handle violations
```toml
# fitness-exceptions.toml
[max-cyclomatic."src/engine.py::execute"]
value = 42
issue = "#138"
```

### 5. Add to CI
```yaml
- name: Check architectural fitness
  run: aps run fitness validate .
```

## Status

**Official** — Promoted from EXP-V1-0003.

### Phase 1 (Current)
- Normative specification complete
- Full metrics catalog with 20+ metrics
- Directory structure and substandard scaffolding

### Phase 2 (Next)
- Rust implementation (migrating + extending EXP-V1-0003 core)
- Dimensional scoring engine
- System-level fitness calculation
- Adapter framework
- CLI commands

## Related Standards

- **APS-V1-0001** (Code Topology) — Measurement layer producing `.topology/metrics/`
- **EXP-V1-0003** — Predecessor experiment (remains in place)
- **APS-V1-0000** (Meta-Standard) — Governance rules for this standard

## Learn More

- [Full specification](./01_spec.md) — Normative rules, rule format, report format
- [Metrics catalog](./02_metrics-catalog.md) — All metrics with formulas, authors, thresholds
- [Examples](../examples/) — Sample configurations and reports
- [Agent skills](../agents/skills/) — AI agent integration

---

*Promoted from EXP-V1-0003. Based on Ford, N. et al. (2017), Building Evolutionary Architectures.*
