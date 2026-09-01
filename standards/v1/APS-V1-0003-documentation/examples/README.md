---
name: "Documentation Standard Examples"
description: "Example configuration and compliant project structure for APS-V1-0003"
---

# Examples

## Example Configuration

Configuration lives in a single root-level `apss.yaml` owned by the meta-standard (APS-V1-0000.CF01). This standard's canonical slug is `documentation` (the `docs` and `doc` spellings are dev-CLI aliases only); it contributes the `docs:` section. A minimal `apss.yaml` for a project adopting the documentation standard:

```yaml
schema: apss.project/v1

docs:
  root: docs
  adr:
    directory: adrs
    required_adr_keywords:
      - security
```

The full default `docs:` section (every key) is in [`apss.yaml`](apss.yaml).

## Example Compliant Directory Structure

```
my-project/
├── apss.yaml                    # APSS configuration (meta-standard owned)
├── docs/
│   ├── README.md                # Has ## Index auto-generated
│   ├── AGENTS.md                # Canonical agent context for this directory
│   ├── CLAUDE.md                # Symlink to AGENTS.md
│   └── adrs/
│       ├── README.md            # Has ## Index of ADRs
│       ├── AGENTS.md            # Canonical agent context for ADRs
│       ├── CLAUDE.md            # Symlink to AGENTS.md
│       ├── ADR-001-initial-architecture.md
│       └── ADR-002-auth-strategy.md
├── CLAUDE.md                    # Root context, references docs/
├── AGENTS.md                    # Root agent context
└── src/
    └── ...
```

The standard ships no `GEMINI.md`; Gemini reads `AGENTS.md` natively. The docs-area `AGENTS.md` files are scaffolded by the installer when absent and never overwritten on subsequent installs (see `docs/02_install_contract.md` Section 1.5).

(The `.apss/` dotdir, if it exists, holds generated artifacts only such as cached indexes; it MUST NOT hold configuration.)

## Example ADR Front Matter

```markdown
---
name: "Initial Architecture"
description: "Defines the foundational system architecture and key technology choices"
status: accepted
---

# ADR-001: Initial Architecture

**Date:** 2026-01-15

## Context
...
```

## Example README.md with Auto-Generated Index

```markdown
# Architecture Decision Records

Overview of all architectural decisions for this project.

## Index

| Document | Description |
|----------|-------------|
| [Initial Architecture](ADR-001-initial-architecture.md) | Defines the foundational system architecture |
| [Auth Strategy](ADR-002-auth-strategy.md) | Authentication and authorization approach |
```

## Example AGENTS.md (Directory-Level Pointer)

```markdown
---
name: "adrs"
description: "AI context for Architecture Decision Records"
---

See [README.md](README.md) for full index and overview of this directory.
```

`CLAUDE.md` in the same directory is a committed, byte-identical copy of `AGENTS.md` (a regular file, not a symlink and not an `@AGENTS.md` import stub) so Claude Code reads the same content on every platform. Edit `AGENTS.md`; `CLAUDE.md` is regenerated from it.
