# DI01 Package Manager Lifecycle

## 1. Purpose

APSS distribution has two related but separate responsibilities:

1. Provide a global management CLI for maintainers and agents.
2. Provide repo-local enforcement that works for contributors who only clone the repository.

The global CLI is useful for setup, add, remove, update, install, and repair operations. It MUST NOT be required for ordinary contributors to read, edit, build, or commit in a repository unless that repository explicitly opts into that requirement.

## 2. Roles

### 2.1 Global APSS CLI

The global CLI is installed by maintainers or agents:

```bash
cargo install apss
```

It provides package-manager behavior:

```bash
apss init
apss add <standard>[@<version-req>]
apss remove <standard>
apss install
apss install --bundle-dir <path>
apss update [<standard>]
apss validate
apss status
apss run <standard> <command> [args...]
```

The global CLI MAY also be installed temporarily in CI.

### 2.2 Repo-Local Runtime

Each repository MAY have a generated repo-local runtime:

```text
.apss/bin/apss
```

The repo-local runtime is composed from the exact standards pinned by `apss.lock`. It is intended for reproducible standard execution inside that repository.

The repo-local runtime is generated output. It SHOULD NOT be committed.

### 2.3 Contributors

Contributors who clone a repository SHOULD be able to contribute without globally installing APSS.

Repositories SHOULD commit enough source-of-truth files for APSS enforcement to be understandable and reproducible:

- APSS project config.
- `apss.lock`.
- Human-readable documentation links.
- Lightweight generated hook or CI wrappers, if the repository chooses to commit them.

Heavy build output MUST NOT be committed.

## 3. Source Of Truth

### 3.1 User-Owned Files

The APSS project config is user-owned. Humans and agents may edit it directly.

Examples:

```text
APSS.yaml
apss.lock
```

The config SHOULD use a friendly header:

```text
# APSS project configuration.
# Edit this file to add, remove, or configure standards.
# Use `apss add`, `apss remove`, or `apss install` to keep generated files in sync.
```

### 3.2 APSS-Managed Files

APSS-managed files are generated from config and lockfile state. They SHOULD include a managed-file header.

Examples:

```text
.git/hooks/pre-commit
.github/workflows/apss.yml
.apss/build/Cargo.toml
.apss/build/src/main.rs
```

Managed files SHOULD use this header shape, adapted to the file format:

```text
Managed by APSS. Do not edit manually.
Regenerate with: apss install
Source: APSS.yaml
Docs: https://github.com/AgentParadise/agent-paradise-standards-system
```

APSS MUST preserve non-managed content when updating a file that already exists. Managed regions SHOULD be delimited with begin and end markers.

## 4. Add, Remove, Install, Update

### 4.1 Add

`apss add <standard>[@<version-req>]` SHOULD:

1. Resolve the standard slug to a package source.
2. Add or update the standard entry in the APSS project config.
3. Resolve dependencies and substandards.
4. Update `apss.lock`.
5. Regenerate managed files.
6. Rebuild the repo-local runtime.
7. Run `apss validate`.

### 4.2 Remove

`apss remove <standard>` SHOULD:

1. Remove or disable the standard in config.
2. Remove the package from `apss.lock` if no longer required.
3. Remove APSS-managed files owned only by that standard.
4. Rebuild the repo-local runtime.
5. Run `apss validate`.

### 4.3 Install

`apss install` SHOULD:

1. Read the APSS project config.
2. Read `apss.lock` if present.
3. Resolve missing APSS bundles or fail if `--locked` is set.
4. Generate the repo-local build crate.
5. Build the repo-local runtime.
6. Install managed enforcement files, including hooks if enabled.
7. Validate installation state.

`apss install --bundle-dir <path>` SHOULD consume a local APSS bundle
directory for distribution testing before registry publication exists. It
MUST be treated as a local source and MUST NOT hide unresolved registry
state in release-ready installs.

### 4.4 Update

`apss update [<standard>]` SHOULD:

1. Re-resolve selected standards within configured version constraints.
2. Update `apss.lock`.
3. Rebuild the repo-local runtime.
4. Regenerate managed files.
5. Run `apss validate`.

## 5. Lockfile Semantics

`apss.lock` MUST pin exact standard bundle state.

Published registry entries SHOULD include:

- Standard ID.
- Slug.
- Bundle package name.
- Exact version.
- Registry source.
- Checksum.
- Enabled substandards.

Local development entries MAY use path or bundle sources:

```text
path+file:///absolute/path/to/standard
bundle+file:///absolute/path/to/APS-V1-0001-code-topology-1.0.0.apss
```

Local path entries SHOULD be clearly marked as development sources. They are useful for testing, but they are not reproducible across machines unless the path source is available.

`UNRESOLVED` values are acceptable only during early implementation and MUST NOT be allowed in release-ready locked installs.

## 6. Build And Cache Layout

APSS SHOULD avoid storing heavy Cargo target output inside the repository.

Recommended layout:

```text
repo/
  APSS.yaml
  apss.lock
  .apss/
    bin/
      apss
    build/
      Cargo.toml
      src/main.rs
```

Heavy target output SHOULD use an external cache:

```text
~/.cache/apss/builds/<repo-hash>/target
```

Alternative acceptable behavior:

1. Build in `.apss/build/target`.
2. Copy `.apss/bin/apss`.
3. Remove `.apss/build/target` after successful install.

The external cache approach is preferred because it speeds repeated installs while keeping repositories small.

## 7. Enforcement

### 7.1 Git Hooks

APSS SHOULD install managed git hooks during `apss install`.

Hook installation is controlled by `tool.hooks` in `APSS.yaml`:

```yaml
tool:
  hooks:
    pre_commit: true
```

`pre_commit` defaults to `true`. If `pre_commit: false`, or if the user
passes an install-time skip flag such as `--no-hooks`, the installer MUST emit
a clear warning that commit-time APSS validation is disabled. The warning is
intentional because disabling hooks should be temporary and reserved for cases
where validation blocks urgent refactoring or recovery work.

For consumer repositories, the default pre-commit hook SHOULD run:

```bash
apss validate
```

If global `apss` is unavailable, the hook MAY fall back to repo-local `.apss/bin/apss` for checks that the repo-local runtime can perform.

For an APSS standards repository, the managed pre-commit hook MUST run the
full local QA gate before allowing a commit:

```bash
just qa
```

If `just` is unavailable, the hook MUST fall back to the equivalent direct
commands:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace --all-targets
cargo test --workspace
cargo build --workspace --release
cargo run -p aps-cli --bin apss-dev -- v1 validate repo
cargo run -p aps-cli --bin apss-dev -- v1 validate distribution
```

This ensures the meta-standard validates every discovered standard,
substandard, and experiment; DI01 validates distribution readiness; and the
Rust workspace remains formatted, lint-clean, type-checkable, tested, and
release-buildable.

Hooks MUST be managed in a way that preserves user-authored hook content.

### 7.2 CI

Repositories SHOULD enforce standards in CI. CI MAY install global APSS temporarily, or use repo-local generated wrappers when available.

CI SHOULD run:

```bash
apss install --locked
apss validate
```

### 7.3 Standard Validators

Each standard SHOULD expose standard-specific validators through its `register()` implementation.

`apss validate` SHOULD aggregate:

1. APSS config validation.
2. Lockfile validation.
3. Installation validation.
4. Standard-specific validators.

## 8. Difficulty Estimate

| Area | Difficulty | Notes |
| --- | --- | --- |
| Clean target output after install | Small | Delete `.apss/build/target` or set external target dir. Low risk. |
| Managed file headers | Small | Central helper plus hook and codegen updates. |
| `apss add` and `apss remove` config mutation | Medium | Requires safe editing of TOML without corrupting user intent. |
| `apss update` lifecycle | Medium | Depends on resolver and lockfile maturity. |
| External APSS build cache | Medium | Needs stable repo hash, cache invalidation, and cleanup behavior. |
| crates.io resolver | Medium to Large | Needs version selection, registry metadata, checksum handling, and offline behavior. |
| Lockfile hardening | Medium | Replace unresolved sentinels, enforce `--locked`, distinguish registry, git, and path sources. |
| Real standard command handlers | Medium to Large | Each standard must expose useful validators and generators. |
| Full `apss validate` aggregation | Medium | Needs common validator interface and clear diagnostics. |
| Hook lifecycle management | Medium | Install, update, preserve user content, remove owned blocks, support custom hooks path. |
| Contributor no-global-install path | Medium | Requires CI and hook fallbacks that are useful without global APSS. |
| crates.io publish readiness | Large | Requires package metadata, release order, dry-run packaging, install tests, and resolver correctness. |

## 9. Implementation Milestones

### Milestone 1: Repo Size And Managed Files

- Move build target output to an external cache or delete it after install.
- Add standardized managed-file headers.
- Keep APSS config user-owned and editable.

### Milestone 2: Local End-To-End Enforcement

- Keep `apss install --local-repo`.
- Install managed pre-commit hooks.
- Add at least one real standard validator command.
- Make `apss validate` call standard-specific validators.
- Prove with a Hello World repository.

### Milestone 3: Package Manager Commands

- Implement `apss add`.
- Implement `apss remove`.
- Implement `apss update`.
- Ensure commands mutate config, lockfile, generated files, and runtime consistently.

### Milestone 4: Resolver And Lockfile

- Resolve crates.io package versions.
- Resolve git and path sources.
- Write exact lockfile entries.
- Reject `UNRESOLVED` entries in locked installs.

### Milestone 5: Publish Dry Run

- Run `cargo package` for all publishable crates.
- Install APSS globally from a local package artifact.
- Run end-to-end install and validation in a clean example repo.

### Milestone 6: crates.io Release

- Publish crates in dependency order.
- Validate `cargo install apss`.
- Validate `apss add`, `apss install --locked`, hooks, and CI in a fresh clone.
