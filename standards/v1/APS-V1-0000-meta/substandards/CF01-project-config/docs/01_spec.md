# APS-V1-0000.CF01 — Project Configuration (Specification)

**Version**: 1.0.0
**Status**: Active
**Parent**: APS-V1-0000 (Meta-Standard)

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Scope

This substandard defines:

- The `apss.toml` project configuration schema
- Cascading configuration rules for monorepos
- Validation rules for consumer config files
- Requirements for standards to define typed configuration surfaces

---

## 2. Configuration File

### 2.1 Filename and Location

Consumer projects MUST place their configuration at `apss.toml` in the project root.

### 2.2 Schema

The schema identifier MUST be `"apss.project/v1"`.

### 2.3 Full Schema

```toml
schema = "apss.project/v1"

[project]
name = "my-service"              # REQUIRED. Human-readable project name.
apss_version = "v1"              # REQUIRED. APSS major version.

[standards.<slug>]
id = "APS-V1-XXXX"              # REQUIRED. Standard ID.
version = ">=1.0.0, <2.0.0"     # REQUIRED. Semver requirement (Cargo-style).
enabled = true                   # OPTIONAL. Default: true.
substandards = ["RS01", "CI01"]  # OPTIONAL. If omitted, all substandards enabled.

[standards.<slug>.config]
# Standard-specific config. Schema defined by each standard's StandardConfig.

[workspace]
members = ["packages/*"]         # OPTIONAL. Glob patterns for child configs.
exclude = ["packages/legacy-*"]  # OPTIONAL. Glob patterns to exclude.

[tool]
bin_dir = ".apss/bin"            # OPTIONAL. Default: ".apss/bin".
registry = "https://crates.io"  # OPTIONAL. Default: crates.io.
offline = false                  # OPTIONAL. Default: false.
log_level = "warn"               # OPTIONAL. Default: "warn".
```

---

## 3. Field Validation Rules

### 3.1 Project Identity

- `project.name` MUST be a non-empty string
- `project.apss_version` MUST be `"v1"` (the only currently supported version)

### 3.2 Standard Entries

Each key under `[standards]` is a slug used for CLI dispatch.

- `id` MUST match the pattern `APS-V1-\d{4}`
- `version` MUST be a valid semver version requirement
- Each standard ID MUST appear under exactly one slug (no duplicates)
- `substandards` entries MUST match the pattern `[A-Z]{2}\d{2}`

### 3.3 Standard-Specific Configuration

The `[standards.<slug>.config]` table is opaque to CF01. Its schema is defined by each standard's `StandardConfig` implementation (see §5).

CF01 validates config blocks by:
1. Attempting deserialization into the standard's config type
2. Running `StandardConfig::validate()` on the result
3. Reporting any type errors or validation failures

---

## 4. Cascading Configuration (Monorepos)

### 4.1 Discovery

1. Walk up from the current directory to find an `apss.toml` with `[workspace]`
2. The first such file is the **root config**
3. Child `apss.toml` files (without `[workspace]`) are **leaf configs**

### 4.2 Merge Rules

| Section | Rule |
|---------|------|
| `project.name` | Child wins |
| `project.apss_version` | MUST match root (error if different) |
| `standards.<slug>` absent in child | Inherited from root |
| `standards.<slug>` present in child | Child fully replaces root's entry (no deep merge) |
| `standards.<slug>.enabled = false` | Disables for this member only |
| `[workspace]` | MUST NOT appear in child configs |
| `[tool]` | Child overrides individual fields |

### 4.3 Version Range Intersection

When both root and child specify version requirements for the same standard, the resolved version MUST satisfy both ranges. If the intersection is empty, validation MUST emit `CF_VERSION_RANGE_CONFLICT`.

---

## 5. StandardConfig Trait Contract

### 5.1 Requirement

Every standard and substandard that accepts configuration MUST implement `StandardConfig` (defined in `aps-core`). Standards with no configuration MUST use `NoConfig`.

### 5.2 Validation by CF01

CF01 validates standard crates in the APS repo to ensure:

- A `StandardConfig` type is exported (or `NoConfig`)
- The type implements `Default`
- `config.schema.json` matches `json_schema()` output (if present)

### 5.3 Config Module Convention

Standards SHOULD define their config in `src/config.rs` and re-export from `src/lib.rs`.

---

## 6. Error Codes

### 6.1 Consumer Config Validation

| Code | Severity | Rule |
|------|----------|------|
| `CF_MISSING_SCHEMA` | Error | Schema must be `"apss.project/v1"` |
| `CF_MISSING_PROJECT_NAME` | Error | `project.name` required, non-empty |
| `CF_INVALID_APSS_VERSION` | Error | Must be `"v1"` |
| `CF_MISSING_STANDARD_ID` | Error | Each standard needs `id` |
| `CF_INVALID_STANDARD_ID` | Error | Must match `APS-V1-\d{4}` |
| `CF_MISSING_VERSION_REQ` | Error | Each standard needs `version` |
| `CF_INVALID_VERSION_REQ` | Error | Must be valid semver requirement |
| `CF_DUPLICATE_STANDARD_ID` | Error | Same ID under two slugs |
| `CF_INVALID_SUBSTANDARD_CODE` | Error | Must match `[A-Z]{2}\d{2}` |
| `CF_WORKSPACE_IN_CHILD` | Error | Child must not have `[workspace]` |
| `CF_APSS_VERSION_MISMATCH` | Error | Child vs root version mismatch |
| `CF_VERSION_RANGE_CONFLICT` | Error | Empty version intersection |
| `CF_INVALID_CONFIG_VALUE` | Error | Config deserialization failed |
| `CF_CONFIG_VALIDATION_FAILED` | Error | Config semantic validation failed |
| `CF_EMPTY_STANDARDS` | Warning | No standards declared |
| `CF_NO_LOCKFILE` | Warning | No apss.lock found |
| `CF_LOCKFILE_STALE` | Warning | Config newer than lockfile |

### 6.2 Standard Config Surface Validation

| Code | Severity | Rule |
|------|----------|------|
| `CF_MISSING_CONFIG_TYPE` | Error | No `StandardConfig` export |
| `CF_NO_CONFIG_DEFAULTS` | Error | Missing `Default` impl |
| `CF_CONFIG_SCHEMA_STALE` | Warning | JSON schema out of date |
| `CF_NO_CONFIG_VALIDATION` | Warning | `validate()` is a no-op |
