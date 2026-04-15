---
name: "Documentation and Context Engineering"
description: "Enforces documentation consistency: ADR naming/structure, README indexes, and AI context files"
---

# EXP-V1-0004 — Documentation and Context Engineering

A consistency and context engineering standard that ensures projects maintain structured, searchable documentation with auto-generated indexes optimized for both human developers and AI agents.

## Problem

- Projects lack a single source of truth for architectural decisions, leading to redundant work and conflicting implementations.
- Developers and AI agents struggle to understand the "why" behind existing code, especially as teams and systems evolve.
- Documentation drifts out of sync with actual file contents, making indexes unreliable.

## Solution

Three enforcement domains:

1. **ADR Enforcement (DOC01)** — Standardized Architecture Decision Records with enforced naming (`ADR_XXX_<adr-name>.md`), required front matter (`name`, `description`), and configurable required ADRs.

2. **README/Index Enforcement (DOC02)** — Every directory under `docs/` has a `README.md` with an auto-generated `## Index` section built from front matter of `.md` files. `CLAUDE.md` and `AGENTS.md` serve as lightweight pointers for AI context.

3. **Root Context (DOC03)** — Repository root must have `CLAUDE.md` and `AGENTS.md` that reference the documentation location, ensuring agents always have context from a fresh start.

## Configuration

All settings live in `.apss/config.toml` under the `[docs]` section. Zero-config works — all defaults are sensible.

## CLI

```bash
aps run docs validate [path]         # Validate documentation structure
aps run docs index [path]            # Preview auto-generated indexes
aps run docs index [path] --write    # Write indexes into README.md files
```

## Category

Governance — this is about consistency and process, not code analysis.
