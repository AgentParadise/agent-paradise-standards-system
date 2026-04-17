# APS-V1-0002 Architecture Fitness — Integration Guide

This guide walks through wiring architectural fitness governance into a Rust project end-to-end: topology measurement → fitness assertion → CI gating. It assumes APS-V1-0001 (Code Topology) and APS-V1-0002 (Architecture Fitness) are both installed.

## Pipeline Overview

```
┌─────────────────────┐    writes     ┌──────────────────────────┐    reads    ┌──────────────────────┐
│  APS-V1-0001        │ ────────────> │ .topology/metrics/*.json │ ──────────> │  APS-V1-0002         │
│  (code-topology)    │               │                          │             │  (fitness engine)    │
│  language adapters  │               │   functions.json         │             │  reads fitness.toml  │
│  analyze source     │               │   modules.json           │             │  + exceptions        │
│                     │               │   coupling.json          │             │                      │
└─────────────────────┘               └──────────────────────────┘             └──────────┬───────────┘
                                                                                           │
                                                                                           │ writes
                                                                                           ▼
                                                                              ┌──────────────────────┐
                                                                              │ fitness-report.json  │
                                                                              │ (per-rule + system)  │
                                                                              └──────────────────────┘
```

**Separation of concerns:** APS-V1-0001 produces data. APS-V1-0002 asserts on it. The artifacts at `.topology/metrics/` are the contract between them, governed by the schemas in `APS-V1-0001/schemas/`.

## 1. Prerequisites

- A Rust project (single crate or workspace) using 2021+ edition.
- `apss` CLI installed. The project's `apss.toml` declares the standards:

  ```toml
  [project]
  name = "my-project"

  [standards.code-topology]
  version = "1.0.0"

  [standards.architecture-fitness]
  version = "1.0.0"
  ```

Run `apss install` to build the project-local CLI into `.apss/bin/`.

## 2. Generate topology artifacts

```bash
apss run code-topology analyze
```

This writes:

- `.topology/metrics/functions.json` — per-function McCabe cyclomatic, SonarSource cognitive, Halstead metrics, LOC
- `.topology/metrics/modules.json` — per-module aggregates and Martin metrics (Ca, Ce, I, A, D)
- `.topology/metrics/coupling.json` — flat per-module Martin view, optimized for fitness consumption
- `.topology/graphs/coupling-matrix.json` — module-to-module coupling strengths
- `.topology/manifest.toml` — run metadata

All `*.json` artifacts carry `schema_version: "1.0.0"` and validate against the schemas in `APS-V1-0001/schemas/`.

## 3. Configure fitness rules

Create `fitness.toml` at the repo root. Minimal config using the default rules for the two active dimensions:

```toml
[config]
topology_dir = ".topology"
severity_default = "error"

# MT01 + MD01 auto-enabled (active, default-enabled)

[system_fitness]
enabled = true
min_score = 0.7

[[rules.threshold]]
id = "mt01-max-cyclomatic"
name = "Maximum Cyclomatic Complexity"
dimension = "MT01"
source = "metrics/functions.json"
field = "metrics.cyclomatic"
max = 10
scope = "function"
severity = "error"
exclude = ["**/tests/**"]

[[rules.threshold]]
id = "mt01-max-cognitive"
name = "Maximum Cognitive Complexity"
dimension = "MT01"
source = "metrics/functions.json"
field = "metrics.cognitive"
max = 15
scope = "function"
severity = "error"
exclude = ["**/tests/**"]

[[rules.threshold]]
id = "md01-max-efferent-coupling"
name = "Maximum Efferent Coupling (Ce)"
dimension = "MD01"
source = "metrics/coupling.json"
field = "efferent_coupling"
max = 20
scope = "module"
severity = "error"

[[rules.threshold]]
id = "md01-max-main-sequence-distance"
name = "Maximum Distance from Main Sequence"
dimension = "MD01"
source = "metrics/coupling.json"
field = "distance_from_main_sequence"
max = 0.7
scope = "module"
severity = "error"
```

The canonical rule sets are published by the reference substandard crates as `DEFAULT_RULES_TOML` constants:

- `architecture-fitness-mt01::DEFAULT_RULES_TOML`
- `architecture-fitness-md01::DEFAULT_RULES_TOML`

## 4. Validate

```bash
apss run architecture-fitness validate
```

Outputs:

- `fitness-report.json` — machine-readable report matching `fitness-report.schema.json`
- Exit code:
  - `0` — system score ≥ `min_score` and no error-severity failures
  - `1` — any error-severity failure, or system score below `min_score`
  - `2` — only warning-severity violations

### Strict-artifact enforcement

MT01 and MD01 are **active**. If their source artifacts are missing (e.g., you forgot step 2), the rules fail with `PROMOTION_REQUIREMENT_UNMET` rather than silently skipping. This is deliberate — active dimensions promise data exists; its absence is a contract violation.

Incubating dimensions (ST01, SC01, LG01, AC01, PF01, AV01) continue to skip silently on missing artifacts — they are advisory.

## 5. Record exceptions (ratchet pattern)

When you first wire up fitness on a pre-existing codebase, you will have violations. Rather than fixing everything at once, ratchet:

```toml
# fitness-exceptions.toml
[mt01-max-cyclomatic."rust:orchestration::engine::execute"]
value = 42
issue = "#138"

[mt01-max-cyclomatic."rust:setup::configure_workspace"]
value = 28
issue = "#185"
```

Every exception REQUIRES an `issue` reference — it MUST be tracked work, not just a silenced warning. The `value` acts as a budget: if the metric climbs above 42, the exception is insufficient and the violation re-surfaces. Regenerating exceptions tightens monotonically — `apss run fitness ratchet` will never widen an existing budget.

## 6. CI integration

GitHub Actions example (both commands in one job):

```yaml
- name: Analyze code topology
  run: apss run code-topology analyze

- name: Validate architecture fitness
  run: apss run architecture-fitness validate

- name: Upload fitness report
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: fitness-report
    path: fitness-report.json
```

For pull-request trend tracking, cache the previous run's `fitness-report.json` and pass it with `--previous=path/to/prior.json`. The engine emits `system_fitness.trend` deltas so reviewers can see whether a PR improves or regresses each dimension.

## 7. Progressive rollout

Start incubating dimensions opt-in as their substandards mature. For example, enable ST01 structural patterns without blocking CI:

```toml
[dimensions]
ST01 = true
# ST01 is still incubating — rules run advisory; errors downgrade to warnings.
```

Once an incubating dimension lands its schemas + reference crate + ADR, it can promote to active in a follow-up release.

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `MISSING_TOPOLOGY_DIR` | No `.topology/` directory | Run `apss run code-topology analyze` first |
| `PROMOTION_REQUIREMENT_UNMET` on MT01/MD01 rules | `functions.json` or `coupling.json` absent | Regenerate topology; check that LANG01-rust ran successfully |
| `MISSING_ISSUE_REF` | Exception without `issue = "#..."` | Add an issue reference; exceptions without tracked work are rejected |
| `DIMENSION_DISABLED_NO_REASON` | Default-enabled dimension disabled without reason | Add `[dimensions.reasons]` entry explaining why |
| `INVALID_WEIGHTS` | `[system_fitness.weights]` does not sum to 1.0 | Fix weights or remove the section to fall back to equal weights |

## Canonical references

- [Spec §3.3](./01_spec.md) — R1–R5 promotion requirements
- [Spec §3.5](./01_spec.md) — Artifact Contracts
- [ADR 0002](./adrs/0002-mt01-md01-promotion.md) — Why MT01 and MD01 are active
- [APS-V1-0001 schemas](../../APS-V1-0001-code-topology/schemas/) — Artifact contracts
- [`fitness-config.schema.json`](../schemas/fitness-config.schema.json) — Config contract
- [`fitness-exceptions.schema.json`](../schemas/fitness-exceptions.schema.json) — Exceptions contract
- [`fitness-report.schema.json`](../schemas/fitness-report.schema.json) — Report contract
