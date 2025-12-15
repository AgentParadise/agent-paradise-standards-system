# Agent Paradise Standards System (APS)

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

## Key Principles

| Principle | Description |
|-----------|-------------|
| **Filesystem is canonical** | `standards/v1/**` + metadata files are the source of truth |
| **Registries are derived** | Generated views, not authoritative |
| **Code is the standard** | Rust crates + protos ARE the standard, not docs |
| **Self-validating** | Each standard can validate itself and consumers |

## Quick Start

```bash
# Validate the entire V1 repo structure
cargo run --bin aps -- v1 validate repo

# Validate a specific standard
cargo run --bin aps -- v1 validate standard APS-V1-0000

# Create a new standard
cargo run --bin aps -- v1 create standard my-new-standard

# Create a new experiment
cargo run --bin aps -- v1 create experiment my-experiment
```

## Documentation

- [APS-V1-0000 Meta-Standard](standards/v1/APS-V1-0000-meta/docs/01_spec.md) — Canonical specification
- [Experimental Standards](standards-experimental/v1/README.md) — Incubation rules and promotion

## License

MIT OR Apache-2.0
