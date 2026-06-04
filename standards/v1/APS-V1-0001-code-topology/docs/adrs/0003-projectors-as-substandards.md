# ADR-0003: Projectors as Substandards

**Status:** Accepted  
**Date:** 2025-12-15  
**Deciders:** Initial design session

## Context

Multiple visualization approaches exist:

- 3D force-directed graphs
- 2D Graphviz/DOT diagrams
- Mermaid markdown diagrams
- Treemaps, sunbursts, etc.

These have different:
- Rendering technologies (WebGL, SVG, text)
- Layout algorithms
- Configuration options
- Evolution timelines

## Decision

The **main standard defines the artifact format**. Each visualization is a **separate substandard**:

```
EXP-V1-0001 (Main)
├── EXP-V1-0001.3D01 — 3D Force-Directed
├── EXP-V1-0001.GV01 — Graphviz/DOT
└── EXP-V1-0001.MM01 — Mermaid
```

## Rationale

- **Clean separation** — Data format is stable; renderers evolve independently
- **Parallel development** — Different teams can work on different projectors
- **Easy extension** — Add new visualizations without touching core
- **APS-native** — Follows the substandard pattern from the meta-standard

## Consequences

- More packages to maintain
- Substandards must conform to parent artifact format
- Clear interface definition needed (Projector trait)

## Alternatives Rejected

- **All-in-one**: Couples unrelated concerns, hard to extend
- **Separate standards**: Loses the hierarchical relationship

