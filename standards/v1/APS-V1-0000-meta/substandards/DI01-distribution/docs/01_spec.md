# APS-V1-0000.DI01 — Distribution & Installation (Specification)

**Version**: 1.0.0
**Status**: Active
**Parent**: APS-V1-0000 (Meta-Standard)

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Scope

This substandard defines:

- How standards are packaged and published as independent Rust crates
- The bootstrap CLI binary (`apss`) for project onboarding
- The installation workflow (`apss install`)
- The lockfile format (`apss.lock`)
- Code generation for composed project-local binaries

---

## 2. Standard Crate Publishing

### 2.1 Crate Naming Convention

Standard crates MUST follow the naming pattern:

```
apss-v1-NNNN-<slug>
```

Where `NNNN` is the 4-digit standard ID and `<slug>` is the kebab-case slug.

Examples:
- `apss-v1-0001-code-topology`
- `apss-v1-0003-fitness-functions`

### 2.2 Substandard Crate Naming

Substandard crates MUST follow:

```
apss-v1-NNNN-<profile>-<slug>
```

Examples:
- `apss-v1-0001-rs01-rust`
- `apss-v1-0001-ci01-github-actions`

### 2.3 Required Exports

Standard crates MUST export:

```rust
pub fn register(registry: &mut dyn aps_core::StandardRegistry) {
    // Register this standard's CLI handler
}
```

### 2.4 Dependencies

Standard crates MUST depend on `aps-core` for shared traits.

### 2.5 Configuration Export

Standard crates MUST export a type implementing `StandardConfig` (or use `NoConfig`). See CF01 and meta-standard §8.3.

---

## 3. Bootstrap Binary

### 3.1 Purpose

The bootstrap binary is a lightweight CLI installed globally via `cargo install apss`. It handles project onboarding and standard installation.

### 3.2 Bootstrap Commands

| Command | Description |
|---------|-------------|
| `apss init` | Create `apss.toml` |
| `apss install` | Resolve + build composed binary |
| `apss install --locked` | CI mode — fail if lockfile changes |
| `apss install --update <slug>` | Update one standard |
| `apss install --offline` | Use only cached crates |
| `apss status` | Show project config + installed versions |
| `apss validate` | Validate project against all standards |
| `apss validate --config-only` | Validate only `apss.toml` |
| `apss config show <slug>` | Show resolved config for a standard |
| `apss config schema <slug>` | Show JSON Schema for config |
| `apss config template` | Generate config with defaults |
| `apss run <slug> <cmd>` | Delegate to composed binary |

### 3.3 Delegation

When the bootstrap receives `apss run ...`, it delegates to the composed binary at `.apss/bin/apss`. If the binary doesn't exist, it prints a helpful error directing the user to run `apss install`.

---

## 4. Installation Workflow

### 4.1 `apss install` Steps

1. Parse `apss.toml` (with cascading if workspace)
2. Resolve version ranges against the registry index
3. Write/update `apss.lock`
4. Generate `.apss/build/Cargo.toml` with resolved dependencies
5. Generate `.apss/build/src/main.rs` with `register()` calls
6. Run `cargo build --release --manifest-path .apss/build/Cargo.toml`
7. Copy binary to `.apss/bin/apss`

### 4.2 Locked Mode

`apss install --locked` MUST fail if the resolved versions would change `apss.lock`. This is intended for CI environments.

---

## 5. Lockfile Format

### 5.1 Location

The lockfile MUST be at `apss.lock` in the project root, next to `apss.toml`.

### 5.2 Schema

```toml
schema = "apss.lock/v1"

[core]
version = "1.0.0"
checksum = "sha256:..."

[[package]]
id = "APS-V1-0001"
slug = "topology"
crate_name = "apss-v1-0001-code-topology"
version = "1.2.0"
checksum = "sha256:..."
source = "registry+https://crates.io"

substandards = [
    { profile = "RS01", crate_name = "apss-v1-0001-rs01-rust", version = "1.0.0", checksum = "sha256:..." },
]
```

### 5.3 Source Types

The `source` field supports:
- `registry+<url>` — fetched from a crate registry
- `path+<relative>` — local path (for development)
- `git+<url>?rev=<sha>` — git source

### 5.4 Version Control

`apss.lock` SHOULD be committed to version control for reproducibility. `.apss/build/` SHOULD be gitignored.

---

## 6. Code Generation

### 6.1 Generated Crate

`apss install` generates a minimal Rust crate at `.apss/build/`:

```
.apss/
├── build/
│   ├── Cargo.toml     # Generated deps
│   └── src/
│       └── main.rs    # Generated register() + dispatch
└── bin/
    └── apss           # Compiled binary
```

### 6.2 Determinism

Code generation MUST be deterministic — the same `apss.toml` + `apss.lock` MUST produce identical generated files.

---

## 7. .gitignore Recommendations

Consumer projects SHOULD add:

```gitignore
.apss/build/
.apss/bin/
```

And SHOULD commit:
- `apss.toml`
- `apss.lock`

---

## 8. Versioning Model

### 8.1 Version Tiers

The system has two independent version tracks:

| Tier | Scope | Pattern | Source of truth |
|------|-------|---------|-----------------|
| **System** | `aps-core`, `aps-cli`, `apss` bootstrap | `1.x.y` | `[workspace.package].version` in root `Cargo.toml` |
| **Standard** | Each standard/substandard independently | SemVer | `standard.toml` / `substandard.toml` version field |

The system version MUST track `1.x.y` to align with `APS-V1`. It is bumped on
any change to system crates (`aps-core`, `aps-cli`, `apss`).

Standard and substandard versions are independent — a standard MAY be at `3.0.0`
while the system is at `1.2.0`. Consumer projects pin standard versions in
`apss.toml` via semver ranges.

### 8.2 Version Consistency

For each standard/substandard/experiment:

- The version in `Cargo.toml` MUST match the version in the metadata file
  (`standard.toml`, `substandard.toml`, or `experiment.toml`)
- Standards using `version.workspace = true` in `Cargo.toml` are exempt
  (workspace version is managed centrally)
- The `DI_VERSION_MISMATCH` error is raised if these diverge

### 8.3 Version Bump Enforcement

When merging from `main` to `release`:

- If any file within a standard's directory has changed since the last release,
  the standard's version MUST have been bumped
- If any system crate (`crates/aps-core`, `crates/aps-cli`, `crates/apss-bootstrap`)
  has changed, the workspace version MUST have been bumped
- The release gate MUST fail if a version bump is missing

### 8.4 Backward Compatibility

Published crate versions MUST follow SemVer:

- A consumer project using `apss-v1-0001 = ">=1.0, <2.0"` MUST continue to
  work with any `1.x.y` release of that standard
- System crate updates (e.g., `aps-core` `1.1.0` → `1.2.0`) MUST NOT break
  previously published standards — the `aps-core` API is a stability contract

---

## 9. Release Pipeline

### 9.1 Release Flow

```
main ──PR──► release branch
               │
               ├── release-gate (required checks)
               └── on merge → release-create
```

### 9.2 Release Gate (PR to release)

The release gate MUST validate:

1. `just ci` passes (format, lint, typecheck, test, build, aps-validate)
2. `aps v1 validate distribution` passes (hard gate, not advisory)
3. Version bump detected for every changed standard/substandard
4. System version bumped if any system crate changed
5. `cargo audit` passes (supply chain security)
6. PR body contains a changelog section

### 9.3 Release Creation (merge to release)

On merge to `release`:

1. Manual approval via GitHub Environment (`release-publish`)
2. Create git tags:
   - System tag: `v1.x.y` (if system version changed)
   - Per-standard tags: `APS-V1-NNNN-vX.Y.Z` (for each bumped standard)
   - Per-substandard tags: `APS-V1-NNNN.PP01-vX.Y.Z` (for each bumped substandard)
3. Create GitHub Release with changelog from PR body
4. Publish to crates.io (only changed crates, in dependency order):
   - Tier 1: `aps-core` (if changed)
   - Tier 2: meta substandard crates — CF01, DI01, CL01, SS01 (if changed)
   - Tier 3: standard crates (if changed)
   - Tier 4: `apss` bootstrap binary (if changed)
5. Previously published versions remain available — consumers are not forced
   to upgrade

### 9.4 Publish Scope

Only crates with a version bump since the last release tag are published. The
system MUST work with any combination of previously published standard versions
within their declared semver compatibility ranges.

---

## 10. Error Codes

| Code | Severity | Rule |
|------|----------|------|
| `DI_MISSING_REGISTER_FN` | Error | Crate must export `register()` |
| `DI_INVALID_CRATE_NAME` | Error | Must follow naming convention |
| `DI_MISSING_APS_CORE_DEP` | Error | Must depend on `aps-core` |
| `DI_LOCKFILE_INTEGRITY` | Error | Checksum mismatch |
| `DI_LOCKFILE_PARSE_ERROR` | Error | Invalid lockfile format |
| `DI_BUILD_DIR_MISSING` | Error | Build dir missing |
| `DI_BINARY_STALE` | Warning | Binary older than lockfile |
| `DI_BINARY_MISSING` | Warning | Lockfile exists, no binary |
| `DI_VERSION_MISMATCH` | Error | Cargo.toml vs metadata version |
| `DI_MISSING_PUBLISH_METADATA` | Warning | Missing description/license/repository |
| `DI_PUBLISH_DISABLED` | Warning | `publish = false` on distributable crate |
