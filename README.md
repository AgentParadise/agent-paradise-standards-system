# Agent Paradise Standards System (APS)

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![CI](https://github.com/AgentParadise/agent-paradise-standards-system/actions/workflows/ci.yml/badge.svg)](https://github.com/AgentParadise/agent-paradise-standards-system/actions)

The Agent Paradise Standards System (APS) is an **executable, evolvable standards framework** designed for agentic engineering at scale.

## What is APS?

APS standards are not static documents — they are **versioned Rust crates** with:

- **Protobuf contracts** for technical standards
- **Automated validation** via the `aps` CLI
- **Substandards** for specialized profiles
- **Experimental track** for incubation
- **Agent-native skills** for AI integration

The **meta-standard (APS-V1-0000)** defines how all V1 standards, substandards, and experiments are structured, validated, and evolved.

## Repository Structure

```
agent-paradise-standards-system/
├── crates/
│   ├── aps-core/           # Core engine (diagnostics, discovery, templates)
│   └── aps-cli/            # CLI: aps v1 validate, create, promote
│
├── standards/v1/           # Official V1 standards
│   └── APS-V1-0000-meta/   # Meta-standard (defines all V1 rules)
│
├── standards-experimental/v1/  # Experimental standards (incubation)
│
├── generated/              # Derived artifacts (gitignored)
│
└── lib/agentic-primitives/ # Agentic primitives submodule
```

## Quick Start

```bash
# Build the CLI
cargo build --release

# Install locally (optional)
cargo install --path crates/aps-cli

# Or run via cargo
cargo run --bin aps -- v1 --help
```

## CLI Commands

### Validation

```bash
# Validate the entire V1 repo structure
aps v1 validate repo

# Validate a specific standard
aps v1 validate standard APS-V1-0000

# Validate an experiment
aps v1 validate experiment EXP-V1-0001

# JSON output for CI
aps v1 validate repo --format json
```

### Creating Packages

```bash
# Create a new official standard
aps v1 create standard my-new-standard

# Create an experimental standard
aps v1 create experiment my-experiment

# Create a substandard (profile)
aps v1 create substandard APS-V1-0001 GH01
```

### Lifecycle Management

```bash
# Promote an experiment to official standard
aps v1 promote EXP-V1-0001

# Show package version
aps v1 version show APS-V1-0000

# Bump version (major, minor, or patch)
aps v1 version bump APS-V1-0000 patch
```

### Views & Registry

```bash
# Generate derived views (registry.json, INDEX.md)
aps v1 generate views

# List all packages
aps v1 list
```

## Key Principles

| Principle | Description |
|-----------|-------------|
| **Filesystem is canonical** | `standards/v1/**` + metadata files are the source of truth |
| **Registries are derived** | Generated views, not authoritative |
| **Code is the standard** | Rust crates + protos ARE the standard, not docs |
| **Self-validating** | Each standard can validate itself and consumers |

## Package Types

| Type | ID Format | Location | Description |
|------|-----------|----------|-------------|
| Standard | `APS-V1-XXXX` | `standards/v1/` | Official, stable standards |
| Substandard | `APS-V1-XXXX.YY00` | `<parent>/substandards/` | Specialized profiles |
| Experiment | `EXP-V1-XXXX` | `standards-experimental/v1/` | Incubating standards |

## Development

```bash
# Run all checks (format, lint, test)
just check

# Fix formatting and run checks
just check-fix

# Run only tests
cargo test --workspace

# Build release
cargo build --release
```

## Documentation

- [APS-V1-0000 Meta-Standard](standards/v1/APS-V1-0000-meta/docs/01_spec.md) — Canonical specification
- [Experimental Standards](standards-experimental/v1/README.md) — Incubation rules and promotion
- [Templates](standards/v1/APS-V1-0000-meta/templates/README.md) — Package scaffolding templates

## License

Apache-2.0 — See [LICENSE](LICENSE) for details.
