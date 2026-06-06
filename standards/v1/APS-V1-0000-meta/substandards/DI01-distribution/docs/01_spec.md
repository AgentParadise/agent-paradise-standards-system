# APS-V1-0000.DI01  -  Distribution & Installation (Specification)

**Version**: 1.0.0
**Status**: Active
**Parent**: APS-V1-0000 (Meta-Standard)

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Scope

This substandard defines:

- How standards are packaged and published as APSS-native bundles
- How Rust crates may be used inside bundles as implementation artifacts
- The bootstrap CLI binary used for project onboarding (canonical binary name
  is being resolved in repo issue 64; this spec refers to it as the
  "bootstrap" where the name can be avoided)
- The installation workflow that reads the `APSS.yaml` manifest defined by
  CF01 and resolves, fetches, locks, and composes the standards it declares
- The lockfile format (`apss.lock`)
- Code generation for composed project-local binaries
- The seam between CF01 (manifest), DI01 (resolution and packaging), and each
  standard's install contract that the unified installer invokes

---

## 2. Standard Bundle Publishing

### 2.1 Distribution Boundary

Standards and substandards MUST be distributed as APSS-native bundles.
The APSS CLI, bootstrap binary, GitHub Action, and language-specific
wrappers MAY be distributed through crates.io, npm, binary releases, or other
tooling package managers. A standard's Rust crate MUST NOT be the
user-facing distribution unit unless the package is explicitly marked as a
tooling crate or independent Rust library.

This separates two concerns:

- Tooling distribution installs the APSS tools.
- Standard distribution installs APSS standards and explicitly declared
  experiments into a consumer repository.

### 2.2 Bundle Naming Convention

Bundle directories and archive filenames MUST follow:

```
APS-V1-NNNN-<slug>-<version>.apss
```

For official standards, `NNNN` is the 4-digit standard ID, `<slug>` is the
kebab-case slug, and `<version>` is the SemVer version from the package
metadata. Experimental standard bundles MUST use the same shape with
`EXP-V1-NNNN` as the ID prefix:

```
EXP-V1-NNNN-<slug>-<version>.apss
```

Examples:
- `APS-V1-0001-code-topology-1.0.0.apss`
- `APS-V1-0000.DI01-distribution-1.0.0.apss`
- `EXP-V1-0003-fitness-functions-0.1.0.apss`

### 2.3 Required Bundle Contents

Every bundle MUST contain:

- `bundle.toml`, the APSS bundle manifest
- The source package metadata file, such as `standard.toml` or
  `substandard.toml`
- `docs/` when documentation exists in the source package
- Runtime or validation artifacts required by the standard
- Implementation source required to build or execute the standard

Generated build outputs such as `target/` MUST NOT be included.

### 2.4 Bundle Manifest

`bundle.toml` MUST use this schema:

```toml
schema = "apss.bundle/v1"
id = "APS-V1-0001"
name = "Code Topology"
slug = "code-topology"
version = "1.0.0"
kind = "standard"
metadata_file = "standard.toml"

[source]
package_path = "standards/v1/APS-V1-0001-code-topology"
repository = "https://github.com/AgentParadise/agent-paradise-standards-system"

[payload]
metadata = "standard.toml"
docs = "docs"
implementation = "."
```

Substandard bundles MUST set `kind = "substandard"` and use the substandard
ID, name, slug, version, and `metadata_file = "substandard.toml"`.

Experimental standard bundles MUST set `kind = "experiment"` and use
`metadata_file = "experiment.toml"`.

Promoted official standard bundles MAY include a promotion alias table:

```toml
[[promoted_from]]
id = "EXP-V1-0003"
slug = "fitness-functions"
last_experiment_version = "0.4.0"
compatibility = "config-compatible"
```

The `compatibility` value MUST be one of:

- `config-compatible` when the promoted standard accepts the experiment's
  config unchanged.
- `migration-adapter` when the promoted standard ships code that translates
  the experiment config.
- `manual-migration-required` when automatic resolution is unsafe.

### 2.5 Implementation Crate Naming

When a bundle contains a Rust implementation crate, the crate SHOULD follow
the existing APSS naming convention:

```
apss-v1-NNNN-<slug>
```

Substandard implementation crates SHOULD follow:

```
apss-v1-NNNN-<profile>-<slug>
```

These names are implementation details used by composition and build
generation. They are not the public standard package identifiers.

### 2.6 Required Exports

Rust implementation crates used by bundles MUST export:

```rust
pub fn register(registry: &mut dyn apss_core::StandardRegistry) {
    // Register this standard's CLI handler
}
```

### 2.7 Dependencies

Rust implementation crates MUST depend on `apss-core` for shared traits.

### 2.8 Configuration Export

Standard implementation crates MUST export a type implementing
`StandardConfig` or use `NoConfig`. See CF01 and meta-standard Section 8.3.

---

## 3. Bootstrap Binary

### 3.1 Purpose

The bootstrap binary is a lightweight CLI installed globally via
`cargo install <bootstrap>`. It handles project onboarding and unified
installation. The canonical binary name is being resolved in repo issue 64;
this spec uses `<bootstrap>` where the name can be avoided.

### 3.2 Bootstrap Commands

| Command | Description |
|---------|-------------|
| `<bootstrap> init` | Create `APSS.yaml` |
| `<bootstrap> install` | Read `APSS.yaml`, resolve, run per-standard install contracts, build composed binary |
| `<bootstrap> install --check` | Report what install would do without writing |
| `<bootstrap> install --locked` | CI mode, fail if lockfile would change |
| `<bootstrap> install --update <slug>` | Update one standard |
| `<bootstrap> install --offline` | Use only cached crates |
| `<bootstrap> uninstall <slug>` | Invoke a single standard's uninstall contract |
| `<bootstrap> status` | Show project config and installed versions |
| `<bootstrap> validate` | Validate project against all standards |
| `<bootstrap> validate --config-only` | Validate only the manifest |
| `<bootstrap> config show <slug>` | Show resolved config for a standard |
| `<bootstrap> config schema <slug>` | Show JSON Schema for config |
| `<bootstrap> config template` | Generate config with defaults |
| `<bootstrap> run <slug> <cmd>` | Delegate to composed binary |

### 3.3 Delegation

When the bootstrap receives `<bootstrap> run ...`, it delegates to the
composed binary at `.apss/bin/<bootstrap>`. If the binary does not exist, it
prints a helpful error directing the user to run `<bootstrap> install`.

---

## 4. Installation Workflow

The unified installer reads the `APSS.yaml` manifest (CF01 Section 2),
resolves the standards it declares, drives each resolved standard's install
contract, then composes the project-local binary. CF01 owns the manifest;
DI01 owns resolution and the lockfile; each standard owns its install
contract. The pipeline below stitches the three together.

### 4.1 Install Pipeline

1. Parse and validate `APSS.yaml` via CF01 (with cascade applied for
   workspaces). Refuse to proceed on any error-severity diagnostic.
2. Resolve version ranges against the APSS bundle registry index, producing
   one `ResolvedStandard` per `standards.<slug>` entry in the manifest.
3. Apply promoted-experiment aliases for any requested `EXP-V1-XXXX` package
   whose registry entry points to an official `APS-V1-XXXX` replacement
   (Section 4.2).
4. Write or update `apss.lock` (Section 5).
5. For each `ResolvedStandard`, load its install contract
   (`docs/02_install_contract.md` in the standard's package) and ask it for
   an install plan in dry-run mode.
6. Apply each install plan in dependency order. Standards may install git
   hooks, scaffolds, validators, and other artifacts per their contract.
7. Reconcile removals: any standard previously present in `apss.lock` but
   absent or `enabled: false` in the current manifest MUST have its
   uninstall contract invoked. Removal MUST leave operator data and source
   code untouched.
8. Generate `.apss/build/Cargo.toml` with resolved bundle implementation
   dependencies.
9. Generate `.apss/build/src/main.rs` with `register()` calls.
10. Run `cargo build --release --manifest-path .apss/build/Cargo.toml`.
11. Copy binary to `.apss/bin/<bootstrap>`.

The pipeline MUST be idempotent: re-running with an unchanged manifest and
registry MUST be a no-op (no file rewrites, no rebuild, exit zero).

### 4.2 Promoted Experiment Resolution

DI01 MUST support experimental standards declared in `APSS.yaml`. If an
`EXP-V1-XXXX` package exists in the registry, resolution proceeds normally
and the resulting package is marked experimental in the lockfile.

If the requested experiment has been promoted and the registry exposes a
promotion alias, DI01 MUST resolve the request to the promoted
`APS-V1-XXXX` package when compatibility is `config-compatible` or
`migration-adapter`. The installer MUST emit a warning diagnostic that
includes:

- The requested experimental ID and slug.
- The resolved official ID and slug.
- The compatibility mode.
- A recommendation to update `APSS.yaml`.

If compatibility is `manual-migration-required`, DI01 MUST fail resolution
with an error diagnostic and a migration path. It MUST NOT silently skip the
standard, drop validation, or leave the project unenforced.

Promoted alias resolution MUST be deterministic. Given the same registry
index, `APSS.yaml`, and lockfile, resolution MUST produce the same official
package and diagnostics.

### 4.3 Locked Mode

`<bootstrap> install --locked` MUST fail if the resolved versions or the
per-standard install plans would change `apss.lock`. This is intended for
CI environments.

### 4.4 Per-standard Install Contracts

Every standard MUST ship `docs/02_install_contract.md`, which is the
per-standard lifecycle hook the unified installer invokes. The contract
MUST define `install`, `uninstall`, and a `plan` mode, each with stable
diagnostics. The `StandardCli` trait (CL01) is the in-process API the
installer uses to reach the contract.

The per-standard escape hatch MUST remain supported as a debugging aid:

```
<bootstrap> run <slug> install
<bootstrap> run <slug> uninstall
```

Documentation MUST present the unified `install` command as the primary path
and the per-standard form as a secondary escape hatch.

### 4.5 CF01 to DI01 Seam

The boundary between CF01 and DI01 is explicit:

- CF01 hands DI01 an ordered list of `(slug, id, version, substandards)`
  tuples derived from the manifest, the cascade, and the slug registry.
- DI01 returns a `ResolvedStandard` per tuple, containing the pinned
  version, the checksum, source descriptor (registry, bundle path, git), and
  any promoted-experiment alias that was applied.
- The installer then drives the per-standard install contracts using those
  `ResolvedStandard` values.
- DI01 owns no knowledge of standard configuration content. CF01 owns no
  knowledge of how a version range is resolved against a registry. Each
  standard owns its own install contract content.

---

## 5. Lockfile Format

### 5.1 Location

The lockfile MUST be at `apss.lock` in the project root, next to
`APSS.yaml`.

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
requested_id = "EXP-V1-0003"
requested_slug = "fitness-functions"
resolved_from = "promoted-experiment"

substandards = [
    { profile = "RS01", crate_name = "apss-v1-0001-rs01-rust", version = "1.0.0", checksum = "sha256:..." },
]
```

### 5.3 Source Types

The `source` field supports:
- `registry+<url>`  -  fetched from a crate registry
- `path+<relative>`  -  local path (for development)
- `git+<url>?rev=<sha>`  -  git source

### 5.4 Version Control

`apss.lock` SHOULD be committed to version control for reproducibility. `.apss/build/` SHOULD be gitignored.

If DI01 resolves a promoted experiment, `apss.lock` MUST record both the
requested experimental identity and the resolved official identity. This
allows future installs to detect whether the operator has updated
`APSS.yaml`, and it makes audit trails clear during experiment promotion.

---

## 6. Code Generation

### 6.1 Generated Crate

`<bootstrap> install` generates a minimal Rust crate at `.apss/build/`:

```
.apss/
├── build/
│   ├── Cargo.toml     # Generated deps
│   └── src/
│       └── main.rs    # Generated register() + dispatch
└── bin/
    └── <bootstrap>    # Compiled binary
```

### 6.2 Determinism

Code generation MUST be deterministic: the same `APSS.yaml` and `apss.lock`
MUST produce identical generated files.

---

## 7. .gitignore Recommendations

Consumer projects SHOULD add:

```gitignore
.apss/build/
.apss/bin/
```

And SHOULD commit:
- `APSS.yaml`
- `apss.lock`

---

## 8. Versioning Model

### 8.1 Version Tiers

The system has two independent version tracks:

| Tier | Scope | Pattern | Source of truth |
|------|-------|---------|-----------------|
| **System** | `apss-core`, `aps-cli`, `apss` bootstrap | `1.x.y` | `[workspace.package].version` in root `Cargo.toml` |
| **Standard** | Each standard/substandard independently | SemVer | `standard.toml` / `substandard.toml` version field |

The system version MUST track `1.x.y` to align with `APS-V1`. It is bumped on
any change to system crates (`apss-core`, `aps-cli`, `apss`).

Standard and substandard versions are independent: a standard MAY be at
`3.0.0` while the system is at `1.2.0`. Consumer projects pin standard
versions in `APSS.yaml` via semver ranges.

Experimental standards MAY be distributed and pinned with `EXP-V1-XXXX`
identities. Promotion to an official standard creates a new official identity
and MAY keep the experiment's version lineage or reset to `1.0.0`. The
promotion alias, not SemVer alone, defines the compatibility bridge from the
experimental identity to the official identity.

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
- If any system crate (`crates/apss-core`, `crates/aps-cli`, `crates/apss-bootstrap`)
  has changed, the workspace version MUST have been bumped
- The release gate MUST fail if a version bump is missing

### 8.4 Backward Compatibility

Published crate versions MUST follow SemVer:

- A consumer project using `apss-v1-0001 = ">=1.0, <2.0"` MUST continue to
  work with any `1.x.y` release of that standard
- System crate updates (e.g., `apss-core` `1.1.0` → `1.2.0`) MUST NOT break
  previously published standards  -  the `apss-core` API is a stability contract

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
2. `apss-dev v1 validate distribution` passes (hard gate, not advisory)
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
4. Publish APSS tooling to crates.io when the system version changed:
   - Tier 1: `apss-core`
   - Tier 2: `apss` bootstrap binary
5. Publish standards as APSS bundles when standard versions changed.
6. Previously published versions remain available  -  consumers are not forced
   to upgrade

### 9.4 Publish Scope

Only APSS tooling crates are published to crates.io by default. CF01, DI01,
CL01, SS01, and standard implementation crates are standards packages, not
public product crates, unless explicitly marked as independent Rust libraries.

Standard distribution MUST use APSS bundles. The system MUST work with any
combination of previously published standard bundle versions within their
declared semver compatibility ranges.

---

## 10. Error Codes

| Code | Severity | Rule |
|------|----------|------|
| `DI_MISSING_REGISTER_FN` | Error | Crate must export `register()` |
| `DI_INVALID_CRATE_NAME` | Error | Must follow naming convention |
| `DI_MISSING_APSS_CORE_DEP` | Error | Must depend on `apss-core` |
| `DI_LOCKFILE_INTEGRITY` | Error | Checksum mismatch |
| `DI_LOCKFILE_PARSE_ERROR` | Error | Invalid lockfile format |
| `DI_BUILD_DIR_MISSING` | Error | Build dir missing |
| `DI_BINARY_STALE` | Warning | Binary older than lockfile |
| `DI_BINARY_MISSING` | Warning | Lockfile exists, no binary |
| `DI_VERSION_MISMATCH` | Error | Cargo.toml vs metadata version |
| `DI_MISSING_PUBLISH_METADATA` | Warning | Missing description/license/repository |
| `DI_PUBLISH_DISABLED` | Warning | `publish = false` on distributable crate |
