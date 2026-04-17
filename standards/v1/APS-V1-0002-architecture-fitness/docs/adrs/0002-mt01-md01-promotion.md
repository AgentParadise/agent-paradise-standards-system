# ADR 0002 — Promote MT01 and MD01 from `incubating` to `active`

**Status**: Accepted
**Decision Date**: 2026-04-16

## Problem

APS-V1-0002 v1.0.0 shipped with every dimension declared `incubating` because R3 (machine-readable artifact schemas) and R5 (non-stub reference implementations) were unmet. While incubating, dimensions cannot produce error-severity failures — their output is advisory. This undermines the central promise of the standard: *objective, automated, continuous* architectural governance. If no dimension is ever enforced, the standard is theatre.

## Decision

Promote **MT01 (Maintainability)** and **MD01 (Modularity & Coupling)** from `incubating` to `active`. All other dimensions remain `incubating` pending follow-on work.

## Evidence of R1–R5 Satisfaction

| # | Requirement | MT01 | MD01 |
|---|-------------|------|------|
| R1 | Objective metric | Cyclomatic (McCabe 1976), Cognitive (SonarSource 2017), Halstead (1977) — all formally defined in [`02_metrics-catalog.md`](../02_metrics-catalog.md) | Ca, Ce, I, A, D (Martin 1994, 2003) — formally defined in catalog |
| R2 | Computable algorithm | APS-V1-0001.LANG01-rust computes all three metrics into `.topology/metrics/functions.json` via `write_function_metrics` | APS-V1-0001.LANG01-rust computes Martin metrics into `.topology/metrics/coupling.json` via `write_coupling` |
| R3 | Artifact schema | `APS-V1-0001/schemas/functions.schema.json` (JSON Schema Draft 2020-12) | `APS-V1-0001/schemas/coupling.schema.json` (JSON Schema Draft 2020-12) |
| R4 | Cited defaults | Cyclomatic ≤ 10 (McCabe 1976), Cognitive ≤ 15 (SonarSource default), Halstead Volume ≤ 1000 (Halstead 1977) | Ce ≤ 20 (Martin 2003), 0.1 ≤ I ≤ 0.9 (Martin 1994), D ≤ 0.7 (Martin 1994 — Zone of Pain / Zone of Uselessness) |
| R5 | Reference implementation | `architecture-fitness-mt01` crate — publishes `DEFAULT_RULES_TOML`, 2 unit tests + 4 integration tests exercising the engine against fixture artifacts | `architecture-fitness-md01` crate — publishes `DEFAULT_RULES_TOML`, 2 unit tests + 4 integration tests exercising the engine against fixture artifacts |

## Enforcement Semantics Change

With MT01 and MD01 active, the engine enters **strict-artifact mode** for their rules (per §3.3 R3 and §12 `PROMOTION_REQUIREMENT_UNMET`):

- A rule on an active dimension whose source artifact is missing now produces a failing `RuleResult` rather than `Skip`. The dimension promised this data exists; its absence is a contract violation.
- Incubating dimensions (ST01, SC01, LG01, AC01, PF01, AV01) continue to skip silently on missing artifacts.

Source of truth: `DimensionCode::promotion_status()` in `architecture-fitness/src/lib.rs`. This MUST remain in sync with the Status column of §1.4 and the Status row of Appendix D.

## Alternatives Considered

1. **Leave everything incubating until all 8 dimensions can promote together** — Rejected. MT01 and MD01 both have measurable producers today; delaying their enforcement does not help the dimensions that need adapter frameworks. Honest partial enforcement is more valuable than uniform non-enforcement.

2. **Promote without strict-artifact mode** — Rejected. Without strict mode, an active dimension silently skipping missing artifacts is indistinguishable from a passing dimension in CI output. That is exactly the governance theatre this standard was designed to prevent.

3. **Make strict mode opt-in via a config flag** — Rejected. Opt-in strictness would let projects claim MT01/MD01 governance while quietly disabling it. The point of promotion is commitment.

## Consequences

- Projects using APS-V1-0002 with MT01 or MD01 rules MUST run APS-V1-0001 topology generation before `apss run fitness validate`; otherwise rules fail rather than skip.
- Incubating dimensions remain advisory. The composite system fitness score reflects only enforced dimensions (MT01 + MD01) by default, per §6.1. Users who want incubating scores in the composite set `include_incubating = true`.
- Future dimension promotions follow this same pattern: land producer + schema + reference crate + ADR in a single bounded change.

## References

- APS-V1-0002 spec §1.4 (Substandard classifications), §3.3 (R1–R5), §3.4 (Lifecycle), §3.5 (Artifact Contracts), §12 (Error Codes), Appendix D (Current Implementation Status)
- APS-V1-0001 schemas at `standards/v1/APS-V1-0001-code-topology/schemas/`
- Reference crates at `standards/v1/APS-V1-0002-architecture-fitness/substandards/0002-MT01-maintainability/` and `.../0002-MD01-modularity/`
