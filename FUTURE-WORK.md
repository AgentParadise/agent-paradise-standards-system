# Future Work

Ideas and potential standards captured for future development.

---

## EXP-V1-XXXX: ADR Standard (Architecture Decision Records)

**Status:** Idea  
**Priority:** Medium  
**Captured:** 2025-12-15

### Vision

A "standard of standards" that defines how ADRs should be structured across all APS standards.

### Key Features

- **Progressive disclosure** — Index with summaries, drill into details on demand
- **Searchable** — Simple command to query ADRs across standards
- **Context-efficient** — Designed for agentic consumption (minimal tokens for max understanding)
- **Adoptable** — Other standards implement THIS standard for their ADRs

### Rough Structure

```
<standard>/
└── docs/adrs/
    ├── index.toml           # Machine-readable index with summaries
    ├── 0001-decision.md     # Full ADR content
    └── ...
```

### Open Questions

- Index format: TOML? JSON? Markdown table?
- Search interface: CLI command? Agent skill?
- How to link ADRs across dependent standards?

---

## EXP-V1-0001: Topology Visualization Scalability

**Status:** Idea  
**Priority:** High  
**Captured:** 2025-12-17

### Problem

The 3D force-directed visualization becomes unusable ("hairball") for large codebases (600+ modules). Tested on `agentic-engineering-framework` with 609 modules — completely unnavigable.

### Proposed Solutions

| Feature | Description | Effort |
|---------|-------------|--------|
| **Package-level aggregation** | Show packages as collapsed nodes first, expand on click to reveal modules | Medium |
| **LOD (Level of Detail)** | Hide distant/low-coupling nodes when zoomed out, reveal on zoom | Medium |
| **Edge bundling** | Group parallel edges going to same area into bundles | High |
| **Focus mode (N-hop)** | Click node → show only N-hop neighbors, fade/hide rest | Low |
| **Smart filter defaults** | Auto-detect module count, start with higher coupling threshold for large graphs | Low |
| **Hierarchical layout** | Tree/radial layout by directory structure instead of pure force-directed | Medium |
| **WebGL instancing** | Use instanced rendering for 1000s of nodes (current approach won't scale) | High |

### Quick Wins

1. **Focus mode** — Lowest effort, highest impact for exploration
2. **Smart filter defaults** — Trivial to implement, immediate UX improvement
3. **Package-level view** — `clusters.html` already does this, could make it the default for large codebases

### Open Questions

- At what module count should we switch to package-level by default? (100? 200?)
- Should LOD be automatic or user-controlled?
- Consider using deck.gl or regl for WebGL performance at scale?

---

## Other Ideas

*(Add future ideas here as they arise)*

