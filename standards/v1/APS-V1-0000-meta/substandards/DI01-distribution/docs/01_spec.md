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
version = "0.1.2"
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

## 8. Error Codes

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
