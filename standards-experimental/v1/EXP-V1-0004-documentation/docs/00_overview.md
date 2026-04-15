---
name: "Documentation and Context Engineering"
description: "Context engineering for technical docs: structured indexes, agent context files, and ADR enforcement"
---

# EXP-V1-0004 — Documentation and Context Engineering

A configurable standard for context engineering within a project's technical documentation directory. Enforces structured, searchable docs with auto-generated indexes optimized for both human developers and AI agents — enabling generators, validators, vectorization, and fast navigation.

## Problem

- Technical documentation lacks structure, making it hard for agents and developers to find the right document quickly.
- Documentation drifts out of sync with actual file contents, making indexes unreliable.
- Architectural decisions are scattered or undocumented, leading to redundant work and conflicting implementations.

## Solution

The standard defines a configurable `docs/` directory structure with two enforcement domains and one substandard:

1. **README/Index Enforcement (DOC02)** — Every directory under `docs/` has a `README.md` with an auto-generated `## Index` section built from YAML front matter of `.md` files. `CLAUDE.md` and `AGENTS.md` in each directory serve as lightweight context pointers for AI agents.

2. **Root Context (DOC03)** — Repository root has `CLAUDE.md` and `AGENTS.md` that reference the documentation location, ensuring agents always have context from a fresh start.

3. **ADR Enforcement (EXP-V1-0004.ADR01)** — A substandard for Architecture Decision Records that live inside the docs directory. Enforces `ADR-XXX-<name>.md` naming, required front matter (`name`, `description`, `status`), configurable required topics, and dead reference detection. ADRs follow the Fowler lifecycle: proposed, accepted, deprecated, superseded — they are never revised, only superseded.

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
