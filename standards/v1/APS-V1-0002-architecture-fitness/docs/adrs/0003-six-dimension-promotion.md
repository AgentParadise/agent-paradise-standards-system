# ADR 0003: Promote ST01, SC01, LG01, AC01 to `active` (six active dimensions)

**Status**: Accepted
**Decision Date**: 2026-06-04

## Problem

APS-V1-0002 v1.0.0 promoted MT01 and MD01 to `active` (ADR 0002) but left ST01, SC01, LG01, AC01, PF01, AV01 as `incubating` pending various blockers. In the interim:

1. The engine codifies all eight dimensions in `DimensionCode::ALL`, evaluates them through the same threshold / dependency / structural code paths, validates dimension codes at config load, and emits `runtime_status` / `promotion_status` / `enforcement` per dimension in every report.
2. Six of the eight dimensions carry universally citable default thresholds per Appendix D's R4 column: MT01 (McCabe / SonarSource / Halstead), MD01 (Martin), ST01 (ArchUnit-style structural rules), SC01 (CVSS), LG01 (license category policy), AC01 (WCAG 2.1 AA).
3. PF01 (latency / throughput) and AV01 (availability / error budget) cannot satisfy R4 because their thresholds are project-specific. The standard cannot publish a universal latency or SLO default.
4. Leaving ST01 / SC01 / LG01 / AC01 as `incubating` (advisory-only) when both the engine and the cited defaults are in place produces governance theatre: rules run, findings appear, but everything is downgraded to warning. Users cannot distinguish "no findings" from "findings suppressed because incubating".

## Decision

Promote ST01, SC01, LG01, AC01 from `incubating` to `active` in the same change set that ships the engine path. Combined with MT01 and MD01 (already active), the standard now has six active dimensions and two incubating dimensions (PF01, AV01).

## Evidence of R1-R5 Satisfaction

| # | Requirement | ST01 | SC01 | LG01 | AC01 |
|---|-------------|------|------|------|------|
| R1 | Objective metric | Structural patterns (forbidden / required / layer) | CVSS severity, vulnerability count | License category (permissive / weak-copyleft / strong-copyleft / proprietary / unknown) | WCAG level (A / AA / AAA), violation count |
| R2 | Computable algorithm | Engine `evaluate_structural_rule` plus the dependency-graph evaluator on `.topology/graphs/dependency-graph.json` | Adapter contract (§9) plus `builtin:cargo-audit` reference normalizer | Adapter contract plus `builtin:cargo-deny` (or equivalent) reference normalizer | Adapter contract plus axe-core or pa11y reference normalizer |
| R3 | Artifact schema | `fitness-config.schema.json` `StructuralRule` def; structural rules consume the topology dependency graph (`APS-V1-0001 dependency-graph.schema.json`) | Adapter output validates as a wrapped threshold artifact; rule results validate against `fitness-report.schema.json` | Same as SC01 | Same as SC01 |
| R4 | Cited defaults | ArchUnit and dependency-cruiser conventions (§1.6 Normative References) | CVSS v3.1 thresholds (industry baseline) | Category policy: permissive allow, weak-copyleft warn, strong-copyleft deny | WCAG 2.1 AA defaults |
| R5 | Reference implementation | Engine path is the reference; structural rules evaluate through the shared engine | Engine threshold path consumes adapter-normalized artifacts; reference adapter ships as part of this PR's INTEGRATION guide | Same as SC01 | Same as SC01 |

CK class-level metrics (DIT, CBO, LCOM) remain a scoped follow-on; they require a class-level analyzer that does not yet exist. The ST01 promotion is limited to structural-pattern rules.

## Why PF01 and AV01 Stay Incubating

R4 cannot be satisfied:

- Performance thresholds (p50, p95, p99 latency; throughput) are workload-specific. The same service can be "fast" in one product and "slow" in another. Universal numeric defaults would be misleading.
- Availability targets (uptime, error budget, MTTR) are SLO decisions tied to product tier, customer contracts, and operational maturity. No industry citation exists for universal numeric values.

Promotion of PF01 / AV01 in this standard would require either:

1. An industry-wide citation for default values (none exists), or
2. A change to R4 that allows "project-specific by ADR" as a substitute for universal defaults (rejected: this would erode the requirement that fitness functions be objective).

Projects MAY promote PF01 / AV01 locally via their own ADR setting concrete SLOs. The standard does not block this.

## Enforcement Semantics Change

With six active dimensions, the engine enters strict-artifact mode for any rule whose dimension is active. Adopters who previously relied on the incubating-advisory fallback for SC01 / LG01 / AC01 must now either (a) configure the adapter so the artifact exists at validate time, or (b) explicitly disable the dimension in `[dimensions]` with a `reasons.<CODE>` entry.

The composite system fitness score now reflects up to six dimensions. With `include_incubating = false` (the default), PF01 and AV01 are excluded from the composite even when enabled, ensuring the score reflects only enforced governance.

## Alternatives Considered

1. **Promote only ST01.** Rejected. Once the engine path supports adapter-backed dimensions and the cited defaults exist, holding SC01 / LG01 / AC01 back gains nothing and continues to ship advisory findings that look identical to "no findings".
2. **Promote all eight, including PF01 and AV01.** Rejected. R4 cannot be satisfied for PF01 / AV01 in the standard. Promoting them with placeholder thresholds would amount to governance theatre, the failure mode this standard exists to prevent.
3. **Add a new lifecycle status between `incubating` and `active`.** Rejected. The lifecycle was deliberately simple in v1.0.0. Adding a third status doubles the documentation burden and weakens the binary "enforced vs advisory" signal.

## Consequences

- The `[dimensions]` defaults now mean five default-enabled active dimensions plus AC01 active-opt-in plus PF01 / AV01 incubating-opt-in. Projects with no `[dimensions]` section will enforce MT01, MD01, ST01, SC01, LG01 strictly.
- The example fitness report, INTEGRATION guide, and validate-fitness skill all reflect six active dimensions. PF01 appears in spec examples as the canonical illustration of an incubating dimension's appearance in a report.
- Future per-project promotion of PF01 / AV01 is a local ADR; this standard does not gate it.

## References

- APS-V1-0002 spec §1.4 (Substandard classifications), §3.1 (Dimension Registry), §3.3 (R1-R5), §3.4 (Lifecycle), §3.5 (Artifact Contracts), §6 (System Fitness), §7 (Report Format), §12 (Error Codes), Appendix D (Current Implementation Status, Promotion Roadmap)
- ADR 0002 for the MT01 / MD01 promotion that established the strict-artifact precedent
