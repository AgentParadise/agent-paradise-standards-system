# CLI01 — CLI Contract

**Status**: Active  
**Parent**: APS-V1-0000 (Meta Standard)

## Purpose

This substandard defines how APS standards expose their functionality through the command-line interface. It provides:

1. **Command patterns** — Consistent naming and argument conventions
2. **Output formats** — Structured JSON output for automation
3. **Exit codes** — Semantic return codes for CI integration
4. **Rust traits** — `StandardCli` trait for standard integration

## Quick Start

```bash
# Run a standard's CLI
apss run topology analyze .
apss run topology validate .topology/
apss run topology diff base/ pr/

# Discovery
apss run --list                    # Show available standards
apss run topology --help           # Show topology commands
```

## Key Concepts

### Command Hierarchy

This contract's `run` dispatch is implemented by two separate binaries, not one:

```
apss                             # published, consumer-facing (cargo install apss)
└── run <slug> <command>        # Run standard CLI

apss-dev                         # this repo's own aps-cli crate, never published
└── v1                          # v1 authoring commands (repo-internal only)

(v2)                             # Future v2 commands, binary not yet decided
```

Consumer projects only ever see `apss`; `apss-dev` is repo-internal tooling for
authoring standards and has no consumer-facing equivalent.

### Standard Commands

Every standard with artifacts SHOULD expose:

| Command | Description |
|---------|-------------|
| `analyze` | Generate artifacts from codebase |
| `validate` | Validate existing artifacts |
| `check` | Check repo compliance |
| `diff` | Compare two artifact sets |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Errors found |
| 2 | Warnings only |

## Related

- [01_spec.md](./01_spec.md) — Full specification
- [EXP-V1-0001](../../../../../../standards-experimental/v1/EXP-V1-0001-code-topology/): Example implementation
