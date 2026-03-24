# ADR-0004: Force-Directed Layout with Saved Positions

**Status:** Accepted  
**Date:** 2025-12-15  
**Deciders:** Initial design session  
**Applies to:** EXP-V1-0001.3D01 (3D substandard)

## Context

Force-directed layout algorithms are non-deterministic:

- Different random seeds → different layouts
- Multiple runs → unstable visual positions
- Hard to compare "before vs after" topology changes

## Decision

**Save computed positions** in the coupling matrix artifact:

```json
{
  "layout": {
    "algorithm": "force-directed",
    "seed": 42,
    "positions": {
      "auth": [1.2, 3.4, 0.5],
      "api": [1.5, 3.2, 0.8]
    }
  }
}
```

## Rationale

- **Reproducibility** — Same artifact → same visualization
- **Stability** — Modules stay in place unless topology changes
- **Diffing** — Can see which modules moved between commits
- **Performance** — Skip layout computation on re-render

## Consequences

- Larger artifact size (minor, ~100 bytes per module)
- Positions become "sticky" (could be stale if algorithm improves)
- Need "recalculate layout" option for refresh

## Implementation Notes

- Positions only recalculated when:
  - Modules added/removed
  - Coupling matrix changes significantly (threshold TBD)
  - User explicitly requests recalculation

## Alternatives Rejected

- **No saved positions**: Unstable visualizations, can't diff
- **Always recalculate**: Jarring visual changes, slow

