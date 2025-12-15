# ADR-0001: Layered Artifact Structure

**Status:** Accepted  
**Date:** 2025-12-15  
**Deciders:** Initial design session

## Context

We need a committable, language-agnostic format for storing code topology data. Options considered:

1. **Single unified file** — One large JSON with everything
2. **Layered directory structure** — Separate files by concern
3. **Event-sourced log** — Append-only changes

## Decision

Use a **layered directory structure**:

```
.topology/
├── manifest.toml
├── metrics/
├── graphs/
└── snapshots/
```

## Rationale

- **Modular updates** — Only changed files need commits
- **Clear separation** — Metrics vs graphs vs history
- **Git-friendly** — Easy to diff individual aspects
- **Scalable** — Can compress large files independently

## Consequences

- Multiple files to manage (acceptable complexity)
- Need clear schema versioning per file
- Projectors must handle multi-file loading

## Alternatives Rejected

- **Single file**: Gets huge, hard to diff, all-or-nothing updates
- **Event log**: Requires replay logic, complex for projectors

