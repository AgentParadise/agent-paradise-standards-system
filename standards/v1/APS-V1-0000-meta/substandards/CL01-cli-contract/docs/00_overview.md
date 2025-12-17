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
aps run topology analyze .
aps run topology validate .topology/
aps run topology diff base/ pr/

# Discovery
aps run --list                    # Show available standards
aps run topology --help           # Show topology commands
```

## Key Concepts

### Command Hierarchy

```
aps
├── run <slug> <command>        # Run standard CLI
├── v1                          # v1 authoring commands
└── v2                          # Future v2 commands
```

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
- [EXP-V1-0001](../../../standards-experimental/v1/EXP-V1-0001-code-topology/) — Example implementation
