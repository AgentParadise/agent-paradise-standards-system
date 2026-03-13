# ADR-0002: Hybrid Coupling Metrics

**Status:** Accepted  
**Date:** 2025-12-15  
**Deciders:** Initial design session

## Context

We need coupling data for two purposes:

1. **Quality analysis** — Actionable insights about module health
2. **3D visualization** — Distance/proximity in spatial layout

## Decision

Compute BOTH:

1. **Martin's metrics** (Ca, Ce, Instability, Abstractness, Distance) per module
2. **Full coupling matrix** (NxN normalized coupling strengths)

## Rationale

- **Martin's metrics** give developers actionable insights:
  - "This module is in the Zone of Pain"
  - "Instability is 0.9 — very fragile"

- **Coupling matrix** feeds visualization:
  - Symmetric matrix → distance metric
  - Directly usable by force-directed layouts
  - Enables 3D spatial positioning

## Consequences

- Slightly more computation (acceptable)
- Two representations to maintain
- Comprehensive coverage of use cases

## Alternatives Rejected

- **Matrix only**: Loses actionable quality insights
- **Martin's only**: Doesn't give pairwise coupling for visualization

