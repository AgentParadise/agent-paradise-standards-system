---
name: "Documentation Standard Examples"
description: "Example configuration and compliant project structure for EXP-V1-0004"
---

# Examples

## Example Configuration

Configuration lives in a single root-level `apss.yaml` owned by the meta-standard (APS-V1-0000.CF01). This standard registers the slug `docs` and contributes the `docs:` section. A minimal apss.yaml for a project adopting the documentation standard:

```yaml
schema: apss.config/v1

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
│   ├── CLAUDE.md                # "See README.md for index"
│   ├── AGENTS.md                # "See README.md for index"
│   └── adrs/
│       ├── README.md            # Has ## Index of ADRs
│       ├── CLAUDE.md
│       ├── AGENTS.md
│       ├── ADR-001-initial-architecture.md
│       └── ADR-002-auth-strategy.md
├── CLAUDE.md                    # Root context, references docs/
├── AGENTS.md                    # Root agent context
└── src/
    └── ...
```

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

## Example CLAUDE.md (Directory-Level Pointer)

```markdown
---
name: "adrs"
description: "AI context for Architecture Decision Records"
---

See [README.md](README.md) for full index and overview of this directory.
```
