# Agent Paradise Standards System (APSS)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![CI](https://github.com/AgentParadise/agent-paradise-standards-system/actions/workflows/ci.yml/badge.svg)](https://github.com/AgentParadise/agent-paradise-standards-system/actions)

**Executable, evolvable standards for agentic engineering.** APSS standards are versioned Rust crates with automated validation, not static documents. They install from crates.io and run as a project-local CLI.

## Available Standards

### Official

| Standard | Description | Run it |
|----------|-------------|--------|
| [APS-V1-0001 Code Topology](standards/v1/APS-V1-0001-code-topology/docs/00_overview.md) | Architectural metrics: complexity, coupling, module structure. Produces `.topology/` artifacts and visualizations. | `apss run code-topology analyze .` |
| [APS-V1-0002 Architecture Fitness](standards/v1/APS-V1-0002-architecture-fitness/docs/00_overview.md) | Declarative architectural assertions (threshold, dependency, structural rules) over topology artifacts, with per-dimension scoring. | `apss run architecture-fitness validate .` |
| [APS-V1-0003 Documentation](standards/v1/APS-V1-0003-documentation/docs/00_overview.md) | Documentation and context engineering: ADR enforcement, README index validation, agent context files. | `apss run documentation validate .` |

The slug in the first command word is the **canonical slug** and must match the standard key in your `apss.yaml`.

### Experimental

| Standard | Description |
|----------|-------------|
| [EXP-V1-0002 TODO Tracker](standards-experimental/v1/EXP-V1-0002-todo-tracker/docs/00_overview.md) | Scans TODO/FIXME comments, enforces issue references |

### Governance

| Standard | Description |
|----------|-------------|
| [APS-V1-0000 Meta-Standard](standards/v1/APS-V1-0000-meta/docs/00_overview.md) | Defines the structure, validation, distribution, and lifecycle for all V1 standards. Not published to crates.io. |

## Using APSS in Your Project

APSS uses crates.io as its distribution transport (see [ADR-0002](standards/v1/APS-V1-0000-meta/docs/adrs/0002-crates-io-distribution.md)):

- **crates.io delivers the tooling**: the `apss` CLI, built on `apss-core`.
- **crates.io delivers the standards**: each official standard publishes as one crate (for example `apss-v1-0001-code-topology`), and its substandards ship as cargo features of that crate. `apss install` resolves standards from crates.io, no APSS checkout required.

```bash
# One-time: install the global CLI (1.1.0 or newer resolves from crates.io)
cargo install apss

# In your repo: declare one or more standards, then install
apss init --standard code-topology   # generates apss.yaml (set the scaffolded id to APS-V1-0001)
apss install     # resolves standards from crates.io, writes apss.lock, builds a project-local binary, installs the git hook
apss validate    # validate the project config and standard structure (also runs from the pre-commit hook)
apss status      # show project configuration and status
apss run code-topology analyze .   # run a standard's command through the project-local binary
```

`apss.yaml` lists standards by their canonical slug and id, for example:

```yaml
standards:
  code-topology:        { id: APS-V1-0001, version: ">=0.2.0" }
  architecture-fitness: { id: APS-V1-0002, version: ">=1.0.0" }
  documentation:        { id: APS-V1-0003, version: ">=1.0.0" }
```

For offline or air-gapped installs, build a bundle locally and point the install at it. Bundles are the optional offline and catalog format, not the default transport:

```bash
apss install --bundle-dir /path/to/bundles
```

Commit `apss.yaml` and `apss.lock`. The generated `.apss/` runtime is build output and stays out of git. Contributors to your repo can read, edit, and commit without installing the global CLI. A step-by-step guide is in [docs/runbooks/visualize-your-codebase.runbook.md](docs/runbooks/visualize-your-codebase.runbook.md).

The full lifecycle is specified in the [DI01 distribution substandard](standards/v1/APS-V1-0000-meta/substandards/DI01-distribution/docs/01_spec.md) and the [package manager lifecycle doc](standards/v1/APS-V1-0000-meta/substandards/DI01-distribution/docs/03_package_manager_lifecycle.md).

## Architecture

Standards **produce artifacts**, and their substandards **consume those artifacts and produce further output**. Substandards ship as cargo features of the parent standard crate; the feature name equals the substandard profile code.

```
APS-V1-0001 (Code Topology)
├── Produces: .topology/metrics/*.json, .topology/graphs/*.json
│
├── RS01 (lang-rust)         Rust source to topology data (input adapter)
├── VZ01 (viz-dashboard)     .topology/ to an HTML dashboard (CodeCity, 3D, clusters)
├── MM01 (viz-mermaid)       .topology/graphs/ to Mermaid dependency diagrams
├── FD01 (force-directed)    .topology/metrics/ to a WebGL 3D coupling visualization
└── CI01 (github-actions)    .topology/ to a PR comment with diff analysis

APS-V1-0002 (Architecture Fitness)
└── Consumes .topology/ artifacts, evaluates declarative rules across 8 dimensions
    (ST01 structural, MD01 modularity, MT01 maintainability, SC01 security, and more)
```

## Repository Structure

```
agent-paradise-standards-system/
├── crates/
│   ├── apss-core/                    # Core engine (diagnostics, discovery, registry, config, lockfile, codegen)
│   ├── aps-cli/                      # Development CLI (apss-dev): v1 validate/create/promote/bundle, run
│   └── apss-bootstrap/               # The published `apss` consumer CLI: init, install, status, validate, run
├── standards/v1/
│   ├── APS-V1-0000-meta/             # Meta-standard (v1.5.0): defines all V1 rules; not published
│   ├── APS-V1-0001-code-topology/    # Code topology + substandards (RS01, VZ01, MM01, FD01, CI01)
│   ├── APS-V1-0002-architecture-fitness/  # Architecture fitness + dimension substandards
│   └── APS-V1-0003-documentation/    # Documentation + substandards (AD01, PV01, RT01)
└── standards-experimental/v1/
    ├── EXP-V1-0001-code-topology/    # Promoted to APS-V1-0001 (historical, slated for removal)
    └── EXP-V1-0002-todo-tracker/     # TODO/FIXME tracking (incubating)
```

## Package Types

| Type | ID Format | Purpose |
|------|-----------|---------|
| **Standard** | `APS-V1-XXXX` | Published crate. Produces artifacts, defines rules. Full structure (`docs/`, `examples/`, `tests/`, `agents/skills/`, `src/`). |
| **Substandard** | `APS-V1-XXXX.YY##` | Feature module of the parent crate (feature name = the `YY##` code). Keeps `substandard.toml` + `docs/` as its governed identity. |
| **Experiment** | `EXP-V1-XXXX` | Incubating. Held to the same validation bar; promoted to an official ID when ready. |

## Two CLIs

- **`apss`** (published, `crates/apss-bootstrap`): the consumer CLI. `apss init`, `apss install`, `apss validate`, `apss status`, `apss run <canonical-slug> <command>`. This is what end users install.
- **`apss-dev`** (development only, `crates/aps-cli`, not published): repo-authoring tooling. Validates and scaffolds standards in THIS repo. It also runs standards via slug aliases (for example `apss-dev run topology ...`) that the consumer `apss` does not accept.

### Authoring standards (apss-dev)

```bash
cargo run -p aps-cli --bin apss-dev -- v1 validate repo            # validate all packages
cargo run -p aps-cli --bin apss-dev -- v1 validate distribution    # distribution + registered-command checks
cargo run -p aps-cli --bin apss-dev -- v1 create standard my-slug  # scaffold a new standard
cargo run -p aps-cli --bin apss-dev -- v1 promote EXP-V1-0002      # promote an experiment to official
```

## Development

```bash
just check                 # format, lint, typecheck, test, build, aps-validate
just qa                    # the full CI-equivalent gate
cargo test --workspace
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
```

## Documentation

- [Meta-Standard Spec](standards/v1/APS-V1-0000-meta/docs/01_spec.md): normative rules for all V1 packages
- [ADR-0002: crates.io distribution](standards/v1/APS-V1-0000-meta/docs/adrs/0002-crates-io-distribution.md): the distribution model
- [Code Topology Overview](standards/v1/APS-V1-0001-code-topology/docs/00_overview.md)
- [Architecture Fitness Overview](standards/v1/APS-V1-0002-architecture-fitness/docs/00_overview.md)
- [Documentation Standard Overview](standards/v1/APS-V1-0003-documentation/docs/00_overview.md)
- [Visualize Your Codebase runbook](docs/runbooks/visualize-your-codebase.runbook.md): hand-to-a-friend guide
- [Experimental Standards](standards-experimental/v1/README.md): incubation and promotion

## License

MIT: See [LICENSE](LICENSE) for details.
