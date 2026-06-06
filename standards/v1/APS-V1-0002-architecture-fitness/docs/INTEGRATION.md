# APS-V1-0002 Architecture Fitness - Integration Guide

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

- `.topology/metrics/functions.json` - per-function McCabe cyclomatic, SonarSource cognitive, Halstead metrics, LOC
- `.topology/metrics/modules.json` - per-module aggregates and Martin metrics (Ca, Ce, I, A, D)
- `.topology/metrics/coupling.json` - flat per-module Martin view, optimized for fitness consumption
- `.topology/graphs/coupling-matrix.json` - module-to-module coupling strengths
- `.topology/manifest.toml` - run metadata

All `*.json` artifacts carry `schema_version: "1.0.0"` and validate against the schemas in `APS-V1-0001/schemas/`.

## 3. Configure fitness rules

Create `fitness.toml` at the repo root. Minimal config using the default rules for the six active dimensions (MT01, MD01, ST01, SC01, LG01, AC01). The five default-enabled actives (MT01, MD01, ST01, SC01, LG01) auto-enable; AC01 is opt-in and must be turned on explicitly:

```toml
[config]
topology_dir = ".topology"
severity_default = "error"

# Default-enabled actives auto-enable: MT01, MD01, ST01, SC01, LG01.
# AC01 is active but opt-in (requires an a11y adapter output).
# PF01, AV01 remain incubating; enabling them runs rules in advisory mode.
[dimensions]
AC01 = true

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

- `fitness-report.json` - machine-readable report matching `fitness-report.schema.json`
- Exit code:
  - `0` - system score ≥ `min_score` and no error-severity failures
  - `1` - any error-severity failure, or system score below `min_score`
  - `2` - only warning-severity violations

### Strict-artifact enforcement

The six active dimensions are MT01, MD01, ST01, SC01, LG01, AC01. If a rule on any of them references a source artifact (a topology metrics file or an adapter output) and that artifact is missing, the rule fails with `PROMOTION_REQUIREMENT_UNMET` rather than silently skipping. This is deliberate: active dimensions promise data exists; its absence is a contract violation.

Adopters wiring up adapter-backed dimensions for the first time should expect this. For SC01, LG01, AC01 you MUST either (a) configure an adapter so that the normalized artifact is generated before validation, or (b) explicitly disable the dimension in `[dimensions]` with a `reasons.<CODE>` entry. Silently leaving the dimension enabled without an adapter output is no longer permitted.

Incubating dimensions (PF01, AV01) continue to skip silently on missing artifacts: their thresholds are project-specific and cannot be enforced without a per-project ADR.

**Adoption note:** this is a raised conformance bar compared to APS-V1-0002 v1.0.0, where only MT01 and MD01 were active. Pre-existing `fitness.toml` files that omit `[dimensions]` will now strictly enforce ST01, SC01, LG01 by default. Adopters who do not yet have adapter outputs for SC01 / LG01 should either land the adapter configuration in the same change as the version bump, or disable those dimensions explicitly with a tracked reason.

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

Every exception REQUIRES an `issue` reference - it MUST be tracked work, not just a silenced warning. The `value` acts as a budget: if the metric climbs above 42, the exception is insufficient and the violation re-surfaces. Regenerating exceptions tightens monotonically - `apss run fitness ratchet` will never widen an existing budget.

## 6. CI integration

GitHub Actions example using the **slot-composition pattern**: APSS dimensions are one input slot, and project-native checks (e.g., a harness performance gate) occupy another.

```yaml
jobs:
  quality-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Slot 1: APSS Architecture Fitness
      - name: APSS Fitness
        run: |
          apss run code-topology analyze
          apss run architecture-fitness validate

      # Slot 2: Project-native checks (layered alongside APSS)
      - name: Native Performance Gate
        run: ./scripts/harness-perf-check.sh --threshold 200ms

      - name: Upload fitness report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: fitness-report
          path: fitness-report.json
```

For pull-request trend tracking, cache the previous run's `fitness-report.json` and pass it with `--previous-report path/to/prior.json`. The engine emits `system_fitness.trend` deltas so reviewers can see whether a PR improves or regresses each dimension.

## 7. Progressive rollout

ST01, SC01, LG01, AC01 are now `active` alongside MT01 and MD01. Adopters typically roll out in this order:

1. Land MT01 and MD01 against topology output (steps 2-5 above).
2. Add the SC01 adapter (e.g., `builtin:cargo-audit`) and start enforcing CVSS thresholds.
3. Add LG01 (`builtin:cargo-deny` or equivalent) for license policy.
4. Add ST01 structural rules (forbidden_import / required_import / layer_enforcement) keyed to your domain boundaries.
5. Add AC01 against axe-core or pa11y output for projects with a web frontend.

PF01 and AV01 remain `incubating`. Enabling either causes rules to run in advisory mode:

```toml
[dimensions]
PF01 = true
# PF01 is incubating; configured error severities downgrade to warning.

[system_fitness]
include_incubating = true   # OPTIONAL: include PF01's score in the composite.
```

Once a project lands an ADR setting concrete SLOs for PF01 or AV01, the dimension can promote to active for that project. Universal promotion in the standard requires industry-wide threshold citations.

## 8. Slot-composition pattern (APSS as one input)

Per spec §3.4.2, APSS-V1-0002 is **one input** to a project's quality gates, not the whole of them. Adopters are encouraged to layer additional non-APSS checks alongside the standard. The lifecycle in §3.4 does not constrain those checks: a project MAY fail the build on a project-native gate that this standard does not address, and APSS conformance is unaffected by their presence or outcome.

Treat each source as its own slot of an aggregator. The example below uses two slots: APSS architecture fitness as Slot A, and a project-native harness performance gate as Slot B. The aggregator records each slot's exit status separately so per-dimension APSS diagnostics survive the rollup.

```yaml
# .github/workflows/quality-gates.yml
jobs:
  quality-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Slot A: APSS architecture fitness (this standard).
      # Six active dimensions (MT01, MD01, ST01, SC01, LG01, AC01) run in
      # strict-artifact mode. PF01 / AV01 remain incubating per ADR 0003.
      - name: Slot A - APSS Fitness
        id: apss
        run: |
          apss run code-topology analyze
          apss run architecture-fitness validate --report fitness-report.json

      # Slot B: project-native performance gate (NOT part of APS-V1-0002).
      # The standard does not constrain this script; it MAY fail the build
      # independently and its outcome does not affect APSS conformance.
      - name: Slot B - Harness Performance Gate
        id: perf
        run: ./scripts/harness-perf-check.sh --p95-budget-ms 250

      # Aggregator: surface each slot's outcome and the APSS fitness report
      # together so the per-dimension diagnostics from Slot A remain visible.
      - name: Upload quality artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: quality-gates
          path: |
            fitness-report.json
            perf-result.json
```

### Locally promoting an incubating dimension (§3.4.1)

A project MAY locally promote `PF01` or `AV01` to build-breaking without changing the dimension's status in this standard. Per §3.4.1 the promotion requires an in-repo ADR that names the dimension, identifies the unmet R1-R5 requirement, and supplies the concrete thresholds. Once that ADR exists, the configuration that wires it into CI looks like:

```toml
# fitness.toml (project-local promotion of PF01)
[dimensions]
PF01 = true                          # enable the rules

[system_fitness]
include_incubating = true            # fold PF01 into the composite
min_score = 0.85                     # project-specific threshold per the ADR

[system_fitness.weights]
MT01 = 0.20
MD01 = 0.20
ST01 = 0.15
SC01 = 0.15
LG01 = 0.10
AC01 = 0.10
PF01 = 0.10                          # weight justified in the ADR

[[rules.threshold]]
id = "pf01-p95-latency"
name = "Maximum p95 Latency"
dimension = "PF01"
source = "adapters/pf01/p95.json"
field = "metrics.p95_ms"
max = 250                            # SLO from the project ADR
scope = "system"
severity = "error"
```

The engine still treats `PF01` as `incubating` in `fitness-report.json`'s `promotion_status` field (the standard-level status is unchanged) and continues to emit `INCUBATING_DIMENSION_ERROR_DOWNGRADED` for the rule. CI failure comes from the composite falling below `min_score`, or from a separately-configured `severity = "error"` outcome that the project's aggregator treats as blocking - the project decides, not the standard.

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `MISSING_TOPOLOGY_DIR` | No `.topology/` directory | Run `apss run code-topology analyze` first |
| `PROMOTION_REQUIREMENT_UNMET` on MT01/MD01 rules | `functions.json` or `coupling.json` absent | Regenerate topology; check that LANG01-rust ran successfully |
| `MISSING_ISSUE_REF` | Exception without `issue = "#..."` | Add an issue reference; exceptions without tracked work are rejected |
| `DIMENSION_DISABLED_NO_REASON` | Default-enabled dimension disabled without reason | Add `[dimensions.reasons]` entry explaining why |
| `INVALID_WEIGHTS` | `[system_fitness.weights]` does not sum to 1.0 | Fix weights or remove the section to fall back to equal weights |

## Canonical references

- [Spec §3.3](./01_spec.md) for R1-R5 promotion requirements
- [Spec §3.4.1](./01_spec.md) for project-local promotion of incubating dimensions
- [Spec §3.4.2](./01_spec.md) for composition with non-APSS checks
- [Spec §3.5](./01_spec.md) for Artifact Contracts
- [ADR 0002](./adrs/0002-mt01-md01-promotion.md) for why MT01 and MD01 are active
- [ADR 0003](./adrs/0003-six-dimension-promotion.md) for why ST01, SC01, LG01, AC01 promoted alongside them
- [APS-V1-0001 schemas](../../APS-V1-0001-code-topology/schemas/) for upstream artifact contracts
- [`fitness-config.schema.json`](../schemas/fitness-config.schema.json) for the config contract
- [`fitness-exceptions.schema.json`](../schemas/fitness-exceptions.schema.json) for the exceptions contract
- [`fitness-report.schema.json`](../schemas/fitness-report.schema.json) for the report contract
