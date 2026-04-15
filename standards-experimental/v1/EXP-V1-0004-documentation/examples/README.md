---
name: "Documentation Standard Examples"
description: "Example configuration and compliant project structure for EXP-V1-0004"
---

# Examples

## Example Configuration

A minimal `.apss/config.toml` for a project adopting the documentation standard:

```toml
schema = "apss.config/v1"

[docs]
root = "docs"

[docs.adr]
directory = "adrs"
required_adr_keywords = ["security"]
```

## Example Compliant Directory Structure

```
my-project/
├── .apss/
│   └── config.toml              # APSS configuration
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
├── CLAUDE.md                    # Root context — references docs/
├── AGENTS.md                    # Root agent context
└── src/
    └── ...
```

## Example ADR Front Matter

```markdown
---
name: "Initial Architecture"
description: "Defines the foundational system architecture and key technology choices"
---

# ADR-001: Initial Architecture

**Status:** Accepted
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
