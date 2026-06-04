---
name: "agent-paradise-standards-system"
description: "Executable, evolvable standards for agentic engineering. Use to run architecture analysis, validate fitness functions, and manage documentation governance (ADRs, indexing, backlinking)."
---

# Agent Paradise Standards System (APS)

Executable, evolvable standards for agentic engineering. APS standards are versioned Rust crates with automated validation, not static documents.

This repository dogfoods its own [Documentation Standard (EXP-V1-0004)](standards-experimental/v1/EXP-V1-0004-documentation/docs/00_overview.md). Agents reading this repo MUST follow the [Mandatory Rules in AGENTS.md](AGENTS.md).

## Skills

### `aps run topology`
Analyze codebase architecture, complexity, and coupling. Produces `.topology/` artifacts.
- **Spec**: [APS-V1-0001 Code Topology](standards/v1/APS-V1-0001-code-topology/docs/01_spec.md)
- **Usage**: `aps run topology analyze .`

### `aps run fitness`
Check architecture fitness thresholds against topology artifacts.
- **Spec**: [EXP-V1-0003 Fitness Functions](standards-experimental/v1/EXP-V1-0003-fitness-functions/docs/01_spec.md)
- **Usage**: `aps run fitness validate .`

### `aps run docs`
Manage documentation governance: ADRs, indexing, backlinking, and pre-commit hooks.
- **Spec**: [EXP-V1-0004 Documentation](standards-experimental/v1/EXP-V1-0004-documentation/docs/01_spec.md)
- **Usage**: `aps run docs validate .` or `aps run docs install .`

### `aps v1`
Authoring tools for standards: validate repo, create packages, promote experiments.
- **Spec**: [APS-V1-0000 Meta-Standard](standards/v1/APS-V1-0000-meta/docs/01_spec.md)
- **Usage**: `aps v1 validate repo`

## Architecture

Standards **produce artifacts** (like `.topology/`) which substandards and other standards then **consume**.

- **Official Standards**: [`standards/v1/`](standards/v1/)
- **Experimental Standards**: [`standards-experimental/v1/`](standards-experimental/v1/)
- **Architecture Decision Records**: [`docs/adrs/`](docs/adrs/) (managed by ADR01 substandard)

## Quick Start

```bash
# Build the CLI
cargo build --release -p aps-cli

# List available standards
aps run --list

# Analyze a codebase
aps run topology analyze .

# Install the documentation pre-commit hook
aps run docs install .
```

## Documentation

- [Agent Orientation (AGENTS.md)](AGENTS.md)
- [Claude Code Context (CLAUDE.md)](CLAUDE.md)
- [Documentation Standard Overview](standards-experimental/v1/EXP-V1-0004-documentation/docs/00_overview.md)
- [Meta-Standard Spec](standards/v1/APS-V1-0000-meta/docs/01_spec.md)

## License

MIT. See [LICENSE](LICENSE) for details.
