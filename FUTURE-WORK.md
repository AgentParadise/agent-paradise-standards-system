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

## Other Ideas

*(Add future ideas here as they arise)*

