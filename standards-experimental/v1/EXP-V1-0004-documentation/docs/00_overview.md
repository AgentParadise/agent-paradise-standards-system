---
name: "Documentation and Context Engineering"
description: "Context engineering for technical docs: structured indexes, agent context files, and ADR enforcement"
---

# EXP-V1-0004 — Documentation and Context Engineering

A configurable standard for context engineering within a project's technical documentation directory. The docs root defaults to `docs/` but is configurable. Every Markdown file carries YAML front matter, and the standard auto-generates `## Index` tables from that metadata — making docs structured, searchable, and machine-readable for validators, generators, vectorization, and fast navigation by both humans and AI agents.

## Problem

- Technical documentation lacks structure, making it hard for agents and developers to find the right document quickly.
- Documentation drifts out of sync with actual file contents, making indexes unreliable.
- No standard way for AI agents to orient themselves within a project's docs from a fresh start.

## Solution

The standard enforces structure and frontmatter-driven indexing across the docs directory:

1. **Frontmatter + Auto-Generated Indexes (DOC02)** — Every `.md` file has YAML front matter (`name`, `description`, configurable fields). Every directory has a `README.md` with an auto-generated `## Index` table built from that metadata. `CLAUDE.md` and `AGENTS.md` per directory provide lightweight context pointers for AI agents.

2. **Root Context (DOC03)** — Repository root has `CLAUDE.md` and `AGENTS.md` that reference the docs location, ensuring agents always find docs from a fresh start.

Domain-specific rules live in **substandards** that inherit the parent's index format:

3. **ADR Enforcement ([EXP-V1-0004.ADR01](substandards/ADR01-architecture-decision-records/docs/00_overview.md))** — Architecture Decision Records inside the docs directory with enforced naming, lifecycle status, required topics, and dead reference detection.

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
