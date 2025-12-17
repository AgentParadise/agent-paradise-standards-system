# EXP-V1-0002: Consumer SDK

**Status**: Experimental  
**Version**: 0.1.0

## Summary

The Consumer SDK enables downstream repositories to adopt, validate, and sync APS standards via a manifest file. It bridges the gap between standard authors (who define specifications) and standard consumers (who implement them).

## Problem

APS is currently **author-focused**. There's no standard way for a project to:

- Declare which standards it adopts
- Generate required artifacts
- Validate compliance
- Help AI agents understand available capabilities

## Solution

A consumer-side SDK providing:

1. **Manifest file** (`.aps/manifest.toml`) — Declares adopted standards and versions
2. **Agent index** (`.aps/index.json`) — Auto-generated file for agent discovery
3. **CLI commands** — `init`, `add`, `sync`, `check`, `update`

## Quick Start

```bash
# Initialize in your project
aps init --name my-project

# Add a standard
aps add code-topology@0.1.0

# Generate artifacts
aps sync

# Validate compliance
aps check
```

## Key Concepts

### Consumer vs. Author

| Role | Focus | Commands |
|------|-------|----------|
| **Author** | Create/maintain standards | `aps v1 create`, `aps v1 validate` |
| **Consumer** | Adopt/comply with standards | `aps init`, `aps add`, `aps sync` |

### Manifest File

The manifest declares what standards a project adopts:

```toml
schema = "aps.manifest/v1"

[project]
name = "my-project"
aps_version = "v1"

[standards.code-topology]
id = "EXP-V1-0001"
version = "0.1.0"
source = "github:AgentParadise/agent-paradise-standards-system"
artifacts = [".topology/"]
```

### Agent Index

The index (`.aps/index.json`) is auto-generated and provides:

- List of adopted standards and versions
- Available agent skills from each standard
- Artifact locations for each standard
- Token estimates for context budgeting

## Use Cases

### 1. Code Quality Standards

Adopt code-topology to track complexity and coupling:

```bash
aps add code-topology@0.1.0
aps sync  # Generates .topology/
```

### 2. Documentation Standards

Adopt doc-index to maintain ADR indices:

```bash
aps add doc-index@0.1.0
aps sync  # Generates docs/adrs/INDEX.md
```

### 3. Dogfooding (Development)

Use a local submodule for standard development:

```toml
[sources.overrides]
"github:AgentParadise/agent-paradise-standards-system" = { 
  type = "submodule",
  path = "lib/aps"
}
```

## Related

- [01_spec.md](./01_spec.md) — Full specification
- [EXP-V1-0001: Code Topology](../../EXP-V1-0001-code-topology/docs/00_overview.md) — Example adoptable standard
