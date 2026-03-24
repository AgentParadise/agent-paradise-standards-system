# Agent Paradise Standards System (APS)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![CI](https://github.com/AgentParadise/agent-paradise-standards-system/actions/workflows/ci.yml/badge.svg)](https://github.com/AgentParadise/agent-paradise-standards-system/actions)

**Executable, evolvable standards for agentic engineering.** APS standards are versioned Rust crates with automated validation, not static documents.

## Available Standards

### Official

| Standard | Description | CLI |
|----------|-------------|-----|
| [APS-V1-0001 Code Topology](standards/v1/APS-V1-0001-code-topology/docs/00_overview.md) | Architectural metrics: complexity, coupling, module structure. Produces `.topology/` artifacts. | `aps run topology analyze .` |

### Experimental

| Standard | Description | CLI |
|----------|-------------|-----|
| [EXP-V1-0002 TODO Tracker](standards-experimental/v1/EXP-V1-0002-todo-tracker/docs/00_overview.md) | Scans TODO/FIXME comments, enforces issue references | `aps run todos scan` |
| [EXP-V1-0003 Fitness Functions](standards-experimental/v1/EXP-V1-0003-fitness-functions/docs/00_overview.md) | Declarative architecture fitness thresholds against topology artifacts | `aps run fitness validate .` |

### Governance

| Standard | Description |
|----------|-------------|
| [APS-V1-0000 Meta-Standard](standards/v1/APS-V1-0000-meta/docs/00_overview.md) | Defines the structure, validation, and lifecycle for all V1 standards |

## Architecture

Standards **produce artifacts** → substandards **consume them and produce further output**.

```
APS-V1-0001 (Code Topology)
├── Produces: .topology/metrics/*.json, .topology/graphs/*.json
│
├── LANG01-rust          Rust source → topology data (input adapter)
├── VIZ01-dashboard      .topology/ → HTML dashboard with CodeCity, 3D, clusters
├── VIZ01-mermaid        .topology/graphs/ → Mermaid dependency diagrams
├── 3D01-force-directed  .topology/metrics/ → WebGL 3D coupling visualization
└── CI01-github-actions  .topology/ → PR comment with diff analysis
```

```
EXP-V1-0003 (Fitness Functions)
├── Consumes: .topology/metrics/*.json
└── Produces: fitness-report.json (threshold violations, exceptions, ratchets)
```

## Quick Start

```bash
# Build the CLI
cargo build --release -p aps-cli

# List available standards
aps run --list

# Analyze a codebase
aps run topology analyze . --output .topology

# Check architecture fitness thresholds
aps run fitness validate .

# Validate this repo's own standard structure
aps v1 validate repo
```

## Repository Structure

```
agent-paradise-standards-system/
├── crates/
│   ├── aps-core/                      # Core engine (diagnostics, discovery, templates)
│   └── aps-cli/                       # CLI: aps run, aps v1 validate/create/promote
├── standards/v1/
│   ├── APS-V1-0000-meta/             # Meta-standard (v1.1.0) — defines all V1 rules
│   └── APS-V1-0001-code-topology/    # Code topology + 5 substandards
├── standards-experimental/v1/
│   ├── EXP-V1-0001-code-topology/    # Promoted → APS-V1-0001 (historical)
│   ├── EXP-V1-0002-todo-tracker/     # TODO/FIXME tracking
│   └── EXP-V1-0003-fitness-functions/ # Architecture fitness thresholds
└── lib/agentic-primitives/            # Agent building blocks (submodule)
```

## Package Types

| Type | ID Format | Structure | Purpose |
|------|-----------|-----------|---------|
| **Standard** | `APS-V1-XXXX` | Full: `docs/`, `examples/`, `tests/`, `agents/skills/`, `src/` | Produces artifacts, defines rules |
| **Substandard** | `APS-V1-XXXX.YY##` | Reduced: `docs/`, `src/` | Consumes parent artifacts, produces further output |
| **Experiment** | `EXP-V1-XXXX` | Full (same as standard) | Incubating — not enforced on consumers |

Substandards inherit agent context and examples from their parent. Their `docs/01_spec.md` serves as agent-readable knowledge about what they consume and produce.

## CLI Commands

### Running Standards

```bash
aps run --list                          # List available standards
aps run topology analyze .              # Analyze codebase topology
aps run topology validate .topology/    # Validate artifact freshness
aps run topology diff base/ pr/         # Compare topology snapshots
aps run topology viz .topology/         # Generate visualizations
aps run fitness validate .              # Check fitness thresholds
```

### Authoring Standards

```bash
aps v1 validate repo                    # Validate all packages
aps v1 validate standard APS-V1-0001    # Validate one standard
aps v1 create standard my-standard      # Scaffold a new standard
aps v1 create experiment my-idea        # Scaffold an experiment
aps v1 promote EXP-V1-0001             # Promote experiment → official
aps v1 version bump APS-V1-0001 minor   # Bump version
```

## Development

```bash
# Run all checks
just check

# Run tests
cargo test --workspace

# Validate repo structure
cargo run -p aps-cli -- v1 validate repo
```

## Documentation

- [Meta-Standard Spec (v1.1.0)](standards/v1/APS-V1-0000-meta/docs/01_spec.md) — Normative rules for all V1 packages
- [Code Topology Overview](standards/v1/APS-V1-0001-code-topology/docs/00_overview.md) — Architecture metrics and visualization
- [Fitness Functions Overview](standards-experimental/v1/EXP-V1-0003-fitness-functions/docs/00_overview.md) — Declarative fitness thresholds
- [Templates](standards/v1/APS-V1-0000-meta/templates/README.md) — Package scaffolding
- [Experimental Standards](standards-experimental/v1/README.md) — Incubation and promotion

## License

MIT — See [LICENSE](LICENSE) for details.
