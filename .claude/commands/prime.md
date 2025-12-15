---
description: Prime context for Agent Paradise Standards System
argument-hint: "[optional: focus area like 'cli', 'core', 'templates', 'meta-standard']"
model: sonnet
allowed-tools: Read, Glob, Grep
---

# Prime

Quickly understand the Agent Paradise Standards System (APS) codebase structure and patterns.

## Purpose

Build working context for this Rust-based standards system by reading key files and understanding the architecture. APS is an **executable, evolvable standards framework** where standards are versioned Rust crates with automated validation via CLI.

## Variables

FOCUS_AREA: $ARGUMENTS

## Codebase Structure

```
agent-paradise-standards-system/
├── crates/
│   ├── aps-core/                 # Core engine (discovery, templates, validation)
│   │   └── src/
│   │       ├── lib.rs            # Module exports
│   │       ├── discovery.rs      # Package discovery (standards/experiments)
│   │       ├── diagnostics.rs    # Error/warning reporting
│   │       ├── metadata.rs       # TOML parsing for standard/substandard/experiment
│   │       ├── templates.rs      # Handlebars template rendering
│   │       ├── promotion.rs      # Experiment → Standard promotion
│   │       ├── versioning.rs     # SemVer version management
│   │       └── views.rs          # Registry generation
│   └── aps-cli/                  # CLI: `aps v1 validate|create|promote|list`
│       ├── src/main.rs           # CLI entrypoint with clap
│       └── tests/                # Integration tests
│
├── standards/v1/                 # Official V1 standards
│   └── APS-V1-0000-meta/         # Meta-standard (defines all V1 rules)
│       ├── standard.toml         # Package metadata
│       ├── docs/01_spec.md       # Canonical specification
│       ├── src/lib.rs            # Standard trait implementation
│       ├── substandards/         # Child profiles
│       └── templates/            # Scaffolding templates
│           ├── standard/skeleton/
│           ├── substandard/skeleton/
│           └── experiment/skeleton/
│
├── standards-experimental/v1/    # Experimental standards (incubation)
│
├── justfile                      # Task runner (just check, just aps-validate)
├── Cargo.toml                    # Workspace manifest
└── AGENTS.md                     # RIPER-5 mode protocol
```

## Key Files

| File | Purpose | Read Priority |
|------|---------|---------------|
| `README.md` | Project overview, CLI usage, architecture | 1 (high) |
| `AGENTS.md` | RIPER-5 mode protocol for agents | 1 (high) |
| `Cargo.toml` | Workspace members, dependencies | 1 (high) |
| `justfile` | All dev/QA/APS commands | 1 (high) |
| `crates/aps-cli/src/main.rs` | CLI entrypoint, command structure | 2 (medium) |
| `crates/aps-core/src/lib.rs` | Core module exports | 2 (medium) |
| `standards/v1/APS-V1-0000-meta/docs/01_spec.md` | Canonical meta-standard spec | 2 (medium) |
| `standards/v1/APS-V1-0000-meta/standard.toml` | Example metadata format | 3 (reference) |
| `crates/aps-core/src/discovery.rs` | Package discovery logic | 3 (reference) |
| `crates/aps-core/src/templates.rs` | Template rendering | 3 (reference) |

## Patterns

- **Language:** Rust 2024 Edition (rust-version = "1.85")
- **Package Manager:** Cargo with workspace
- **Task Runner:** Just (`just check`, `just aps-validate`, `just aps-full`)
- **Testing:** `cargo test --workspace` or `just test`
- **Linting:** `cargo clippy --workspace --all-targets -- -D warnings`
- **Formatting:** `cargo fmt --all`
- **Build:** `cargo build --release`
- **CLI Install:** `cargo install --path crates/aps-cli`

### Naming Conventions

- Standard IDs: `APS-V1-XXXX` (4-digit, zero-padded)
- Substandard IDs: `APS-V1-XXXX.YYYY` (qualified)
- Experiment IDs: `EXP-V1-XXXX`
- Directory names: `APS-V1-XXXX-slug` (kebab-case slug)
- Metadata files: `standard.toml`, `substandard.toml`, `experiment.toml`

### Architecture Principles

1. **Filesystem is canonical** — `standards/v1/**` + metadata files are the source of truth
2. **Registries are derived** — Generated views, not authoritative
3. **Code is the standard** — Rust crates + protos ARE the standard, not docs
4. **Self-validating** — Each standard can validate itself and consumers

### Key Traits

```rust
pub trait Standard {
    fn validate_package(&self, path: &Path) -> Diagnostics;
    fn validate_repo(&self, path: &Path) -> Diagnostics;
}
```

## CLI Commands

```bash
# Validate entire repo
aps v1 validate repo

# Validate specific package
aps v1 validate standard APS-V1-0000

# Create new packages
aps v1 create standard my-new-standard
aps v1 create experiment my-experiment
aps v1 create substandard APS-V1-0001 GH01

# Promote experiment to standard
aps v1 promote EXP-V1-0001

# Version management
aps v1 version show APS-V1-0000
aps v1 version bump APS-V1-0000 patch

# List all packages
aps v1 list

# Generate registry views
aps v1 generate views
```

## Just Commands

```bash
# QA
just check          # format + lint + test
just check-fix      # auto-format + lint

# APS-specific
just aps-validate   # Validate all V1 standards
just aps-list       # List discovered packages
just aps-test       # Run APS integration tests
just aps-full       # Full validation suite

# Development
just build          # Debug build
just build-release  # Release build
just init           # Initialize dev environment
```

## Workflow

1. **Read Foundation**
   - Read `README.md` for project overview
   - Read `AGENTS.md` for RIPER-5 protocol
   - Read `justfile` for available commands

2. **Understand Core**
   - Read `crates/aps-core/src/lib.rs` for module structure
   - Read `crates/aps-cli/src/main.rs` for CLI commands
   
3. **Understand Standards**
   - Read `standards/v1/APS-V1-0000-meta/docs/01_spec.md` for specification
   - Read `standards/v1/APS-V1-0000-meta/standard.toml` for metadata format

4. **If FOCUS_AREA provided**
   - `cli` → Deep dive into `crates/aps-cli/`
   - `core` → Deep dive into `crates/aps-core/`
   - `templates` → Read `crates/aps-core/src/templates.rs` and `standards/v1/APS-V1-0000-meta/templates/`
   - `meta-standard` → Read all files in `standards/v1/APS-V1-0000-meta/`
   - `discovery` → Read `crates/aps-core/src/discovery.rs`
   - `validation` → Read meta-standard validation logic

## Package Types

| Type | ID Format | Location | Metadata File |
|------|-----------|----------|---------------|
| Standard | `APS-V1-XXXX` | `standards/v1/` | `standard.toml` |
| Substandard | `APS-V1-XXXX.YYYY` | `<parent>/substandards/` | `substandard.toml` |
| Experiment | `EXP-V1-XXXX` | `standards-experimental/v1/` | `experiment.toml` |

## Required Package Structure

Every standard/substandard MUST include:

```
<package>/
├── docs/           # Documentation (01_spec.md required)
├── examples/       # At least one example
├── tests/          # Automated tests
├── agents/skills/  # Agent assets
├── src/lib.rs      # Standard trait impl
├── Cargo.toml      # Crate manifest
└── standard.toml   # Metadata (or substandard.toml/experiment.toml)
```

## Report

## APS Context

**Purpose:** Executable, evolvable standards framework for agentic engineering at scale
**Stack:** Rust 2024 Edition, Cargo workspace, Clap CLI, Handlebars templates

### Structure Understanding
- `crates/aps-core/` — Core engine with discovery, templates, diagnostics, versioning
- `crates/aps-cli/` — Command-line interface built with Clap
- `standards/v1/` — Official standards (currently APS-V1-0000-meta)
- `standards-experimental/v1/` — Incubating experiments
- Meta-standard (APS-V1-0000) defines structure for all V1 packages

### Ready to Work On
Based on this context, I can now help with:
- Creating new standards/experiments via `aps v1 create`
- Implementing the `Standard` trait for new packages
- Extending CLI commands in `aps-cli`
- Adding validation rules to `aps-core`
- Working with templates and package scaffolding
- Understanding and modifying the meta-standard specification
- Debugging package discovery and validation issues

