# ADR 0001 — Promotion from EXP-V1-0003 with Expanded Scope

**Status**: Accepted
**Decision Date**: 2026-04-15

## Problem

EXP-V1-0003 (Architecture Fitness Functions) proved the declarative fitness function model with threshold evaluation, exception ratcheting, and stale detection. It is dogfooded in syntropic137. However, its scope is limited to threshold assertions on code topology metrics — a narrow slice of what architectural governance requires.

The concepts from Ford et al. *Building Evolutionary Architectures* (2017) describe a much richer model: multiple architectural dimensions, system-wide fitness aggregation for tradeoff analysis, and fitness functions that span security, legality, performance, and more.

## Decision

Promote EXP-V1-0003 to APS-V1-0002 with the following expansions:

1. **Framework + composable substandards** — The core standard defines the governance model (dimensional scoring, rule format, report format, adapter contract). Each architectural dimension is a substandard that can be enabled or disabled.

2. **Eight dimensional substandards** — MT01 (Maintainability), MD01 (Modularity), ST01 (Structural Integrity), SC01 (Security), LG01 (Legality), AC01 (Accessibility), PF01 (Performance), AV01 (Availability).

3. **Normative metrics catalog** — 20+ metrics with mathematical formulas, original authors, industry thresholds, and rationale. Sourced from McCabe, Halstead, Martin, Chidamber & Kemerer, Henderson-Sellers, and others.

4. **System-level fitness function** — Weighted aggregation of per-dimension scores into a single composite, enabling tradeoff analysis across architectural characteristics.

5. **Adapter contract** — Anti-corruption layer for integrating external tools (security scanners, license checkers, etc.) without coupling the fitness standard to their internals.

6. **VSA anti-corruption layer** — Specific adapter for the Vertical Slice Architecture tool's slice-level metrics.

## Rationale

- EXP-V1-0003's core model (declarative TOML rules, exception ratcheting, JSON reports) is sound and proven. It should be preserved and extended, not replaced.
- The dimensional model enables opt-in/opt-out governance that scales from small libraries to large distributed systems.
- The system-level fitness function is the key insight from Ford — without it, individual metrics exist in isolation and tradeoffs are invisible.
- The adapter contract future-proofs the standard against new tools and scanning technologies.

## Alternatives Considered

1. **Promote EXP-V1-0003 as-is (threshold rules only)** — Rejected because it would require a major version bump later to add dimensions, which is more disruptive than doing it at promotion time.

2. **Keep as experimental, iterate further** — Rejected because the core model is proven and the expanded scope is well-defined. Waiting longer adds no new information.

3. **Separate standards per dimension** — Rejected because dimensions need a shared governance model (composite scoring, report format, adapter contract). Separate standards would fragment the user experience.

## Consequences

- EXP-V1-0003 remains in place per the meta-standard (experiments are never removed).
- All existing `fitness.toml` files are forward-compatible — new sections are optional with backward-compatible defaults.
- Phase 1 (documentation) establishes the standard; Phase 2 (Rust implementation) delivers the tooling.
- CK metrics (DIT, CBO, RFC, WMC, LCOM) are included in the catalog but marked as planned — they require class-level analysis not yet in the topology standard.
