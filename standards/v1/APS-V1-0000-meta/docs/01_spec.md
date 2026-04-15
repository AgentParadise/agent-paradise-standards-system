# APS-V1-0000 — Meta-Standard (Canonical Specification)

**Version**: 1.2.0
**Status**: Active  
**Category**: Governance

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Scope and Authority

This document defines the normative rules for:

- APS official standards
- APS substandards
- APS experimental standards
- APS templates
- APS agent assets

**Normative** means: any APS tooling and CI enforcement MUST implement these rules for APS-V1.

### 1.1 Precedence

If a conflict exists between prose and executable artifacts, the following precedence applies:

1. Enforced tooling/validators (when present)
2. Protobuf-defined contract artifacts (for technical standards)
3. This spec (`docs/01_spec.md`)

### 1.2 Canonical Source

The **filesystem is the canonical source of truth**. Registry views and indexes are derived outputs generated from the filesystem + metadata files. Registries MUST NOT be treated as authoritative.

---

## 2. Core Definitions

### 2.1 Standard

A **standard** is a versioned Rust crate with an immutable standard ID and a package structure required by this meta-standard.

### 2.2 Substandard

A **substandard** is a first-class package co-located under a parent standard, with its own versioning, examples, tests, and agent assets. Substandards MUST conform to the parent standard where applicable.

### 2.3 Experimental Standard

An **experimental standard** is an incubating package used for iteration and community feedback. Experiments:

- Are never considered official
- Are never automatically enforced on downstream repositories
- Follow the same structure as official standards
- Can be promoted to official after peer review and security audit

### 2.4 Template

A **template** is a scaffold used by tooling to create standards, substandards, or adoption assets. Templates are optional and co-located inside the standard (or substandard) they belong to.

### 2.5 APS-V1 Compatibility Contract

APS-V1 is an ecosystem contract. APS-V1 evolution MUST preserve backward compatibility for APS-V1 consumers. Changes that break the APS-V1 contract require APS-V2.

---

## 3. Repository Layout

The APS System repository MUST use this top-level layout:

```
agent-paradise-standards-system/
├── crates/
│   ├── aps-core/               # Core engine
│   └── aps-cli/                # CLI entrypoint
├── standards/v1/               # Official V1 standards
├── standards-experimental/v1/  # Experimental V1 standards
└── generated/                  # Derived artifacts (gitignored)
```

### 3.1 Official Standards Path

Official standards MUST be located at:

```
standards/v1/APS-V1-XXXX-<slug>/
```

Where:

- `APS-V1-XXXX` is a 4-digit number with leading zeros
- `<slug>` is filesystem-safe and kebab-case recommended

Example: `standards/v1/APS-V1-0002-knowledge-base/`

### 3.2 Experimental Path

Experimental standards MUST be located at:

```
standards-experimental/v1/EXP-V1-XXXX-<slug>/
```

Experiments MUST NOT appear in official registry views.

### 3.3 Derived Views

Tooling MAY generate registry views from the canonical filesystem layout:

```
generated/v1/views/
├── standards.json
├── standards.toml
├── experiments.json
└── by-category/
```

These views are derived outputs, NOT the source of truth.

---

## 4. Standard IDs and Naming

### 4.1 Standard ID

Official standard IDs MUST be:

```
APS-V1-XXXX
```

The ID is immutable once assigned.

### 4.2 Substandard ID (Qualified)

Substandard IDs MUST be:

```
APS-V1-XXXX.YYYY
```

Where `YYYY` is a short, uppercase alphanumeric profile identifier (e.g., `GH01`, `PY01`).

Example: `APS-V1-0001.VZ01`

Substandard IDs are immutable.

#### 4.2.1 Substandard Directory Naming

Substandard directories SHOULD be prefixed with the parent standard number for discoverability in flat listings (Cargo workspace, search results):

```
substandards/0001-VIZ01-dashboard/     # RECOMMENDED — parent visible
substandards/VIZ01-dashboard/          # ALLOWED — parent inferred from path
```

The prefix is the 4-digit parent standard number (matching `APS-V1-XXXX`).

### 4.3 Experiment ID

Experiment IDs MUST be:

```
EXP-V1-XXXX
```

Experiment IDs are immutable.

### 4.4 Filesystem Safety

All directory names and filenames MUST be cross-platform safe, including Windows. Avoid reserved characters such as `:` and reserved device names.

---

## 5. Package Structure Requirements

### 5.1 Required Directories (Standards and Experiments)

Every official standard and experimental standard MUST include:

```
<package>/
├── docs/
├── examples/
├── tests/
├── agents/
│   └── skills/
└── src/                 # Rust source (Standard trait impl)
```

### 5.2 Required Directories (Substandards)

Substandards consume their parent standard's artifacts and produce further output. They have reduced structural requirements — the `docs/01_spec.md` serves as both normative specification and agent-readable knowledge about what the substandard consumes and produces.

Every substandard MUST include:

```
<package>/
├── docs/                # MUST contain 01_spec.md
└── src/                 # Rust source
```

The following directories are OPTIONAL for substandards (they inherit context from the parent):

- `examples/` — MAY be present; parent standard examples cover end-to-end usage
- `tests/` — MAY be present as a directory; see §11.2 for inline test alternative
- `agents/skills/` — MAY be present; the substandard's `docs/01_spec.md` serves as agent context

### 5.3 Optional Directories

The following are optional for all package types, but if present MUST conform to this spec:

- `templates/` — Scaffolding templates
- `proto/` — Protobuf contracts (REQUIRED for technical standards)
- `evolution/` — Evolution packs for major version bumps

### 5.4 Category-based Requirements

Standards and substandards MUST declare a `category` in metadata.

**For `technical` category:**

- MUST include `proto/`
- MUST support contract validation via protobuf descriptors
- MUST support breaking-change detection

**For `governance`, `design`, `process`, `security` categories:**

- `proto/` is OPTIONAL
- Tests MAY be lint-style, structural, or content-validating

---

## 6. Metadata Files

### 6.1 `standard.toml`

Official standards MUST include `standard.toml` at the package root:

```toml
schema = "aps.standard/v1"

[standard]
id = "APS-V1-XXXX"           # Required, immutable
name = "Human-Readable Name" # Required
slug = "kebab-case-slug"     # Required, filesystem-safe
version = "1.0.0"            # Required, SemVer
category = "governance"      # Required: governance|technical|design|process|security
status = "active"            # Required: active|deprecated|experimental

[aps]
aps_major = "v1"             # Required

[ownership]
maintainers = ["org/team"]   # Required, at least one
```

### 6.2 `substandard.toml`

Substandards MUST include `substandard.toml` at the package root:

```toml
schema = "aps.substandard/v1"

[substandard]
id = "APS-V1-XXXX.YYYY"      # Required, qualified ID
name = "Profile Name"        # Required
slug = "profile-slug"        # Required
version = "1.0.0"            # Required, SemVer
parent_id = "APS-V1-XXXX"    # Required
parent_major = "1"           # Required, aligns with parent major

[ownership]
maintainers = ["org/team"]
```

### 6.3 `experiment.toml`

Experiments MUST include `experiment.toml` at the package root:

```toml
schema = "aps.experiment/v1"

[experiment]
id = "EXP-V1-XXXX"           # Required
name = "Experiment Name"     # Required
slug = "experiment-slug"     # Required
version = "0.1.0"            # Required, typically 0.x for experiments
category = "technical"       # Required

[aps]
aps_major = "v1"

[ownership]
maintainers = ["org/team"]

# Added after promotion (do not remove)
[promotion]
promoted_to = "APS-V1-YYYY"  # Official standard ID
promoted_at = "2025-12-15"   # Date of promotion
```

---

## 7. Substandard Conformance Model

### 7.1 Validation Order

A substandard MUST be validated in this order:

1. Parent standard validations (where applicable)
2. Substandard-specific validations
3. Example validations

### 7.2 Scoped Applicability

Parent validations MUST be written with scoping so that non-applicable rules can be skipped without weakening enforcement. Tooling MUST report which rules were skipped and why.

### 7.3 Meta Substandard as Ecosystem Validators

Substandards of the meta standard (`APS-V1-0000`) serve a dual role:

1. **Specification** — They define normative rules for a specific concern (e.g., CLI contracts, project configuration, distribution).
2. **Validation** — They MUST provide executable validation that can be applied to other standards and substandards to verify compliance.

Meta substandards MUST:

- Implement validation rules as part of their Rust crate (via the `Standard` trait or domain-specific traits)
- Define error codes (§16.2) for each compliance rule they enforce
- Be composable — their validation SHOULD be invocable independently or as part of a full `aps v1 validate repo` sweep

Meta substandards SHOULD:

- Provide generation capabilities where applicable (e.g., scaffolding, schema generation, default config generation)
- Define traits that other standards implement, creating a contract that meta substandards can validate against

This ensures the meta standard's authority is enforced programmatically, not just documented.

---

## 8. Rust Crate Requirements

### 8.1 Standard Trait

Every standard crate MUST implement the `Standard` trait:

```rust
pub trait Standard {
    /// Validate a package against this standard's rules.
    fn validate_package(&self, path: &Path) -> Diagnostics;
    
    /// Validate an entire repository against this standard's rules.
    fn validate_repo(&self, path: &Path) -> Diagnostics;
}
```

### 8.2 Crate Structure

Standard crates MUST include:

- `Cargo.toml` — Crate manifest
- `src/lib.rs` — Library root with Standard trait impl

Standard crates SHOULD include:

- `src/validate.rs` — Validation rules
- `src/templates.rs` — Template rendering (if templates exist)

### 8.3 Configuration Contract

Standards and substandards that accept runtime configuration MUST define a typed configuration struct in their crate. Standards that accept no configuration MUST use the `NoConfig` marker type.

#### 8.3.1 StandardConfig Trait

Configurable standards MUST implement the `StandardConfig` trait:

```rust
pub trait StandardConfig: DeserializeOwned + Serialize + Default {
    /// Validate config values beyond type checking.
    fn validate(&self) -> Diagnostics;

    /// Generate a JSON Schema for this config.
    fn json_schema() -> serde_json::Value;

    /// Generate a commented TOML snippet showing defaults.
    fn toml_template() -> String;
}
```

This trait enables:

- **Type-safe config validation** — Consumer `apss.toml` config blocks are deserialized into the standard's config type, catching type errors at parse time.
- **Semantic validation** — The `validate()` method checks value ranges, cross-field consistency, and other constraints beyond what the type system expresses.
- **Schema generation** — `json_schema()` produces a JSON Schema for IDE completion and external tooling.
- **Scaffolding** — `toml_template()` generates documented default config snippets for `apss init`.

#### 8.3.2 Config Module Convention

Standards SHOULD place their config type in `src/config.rs` and re-export it from `src/lib.rs`:

```
src/
├── lib.rs          # re-exports Config
└── config.rs       # struct MyStandardConfig { ... }
```

#### 8.3.3 Config Schema File

Standards that implement `StandardConfig` SHOULD include a generated `config.schema.json` at the crate root. This file:

- MUST be regenerable from the Rust type via `StandardConfig::json_schema()`
- SHOULD be kept in sync (CI MAY validate staleness)
- Enables IDE completion and external tooling for `apss.toml`

#### 8.3.4 Validation Scope

The `validate()` method handles semantic validation beyond what the type system can express:

- Value ranges (e.g., `coupling_threshold` between 0.0 and 1.0)
- Path existence checks
- Cross-field consistency (e.g., if feature X is enabled, field Y is required)

Type-level validation (required fields, correct types) is handled automatically by serde deserialization.

---

## 9. Protobuf Contracts (Technical Standards)

### 9.1 Protobuf as Source of Truth

For `technical` standards/substandards:

- Protobuf definitions are the canonical machine contract
- Generated artifacts (schemas, bindings) MUST be derived from protobuf

### 9.2 Breaking-Change Detection

For `technical` standards/substandards:

- CI MUST compile protobuf descriptors and compare against baseline
- If a breaking change is detected, CI MUST FAIL unless:
  - A SemVer major bump is present, AND
  - An evolution pack exists (see §10)
- CI MUST print a human-readable explanation of the breaking change

---

## 10. Evolvability and Evolution Packs

### 10.1 APS-V1 Major Backward Compatibility

APS-V1 (ecosystem major) MUST remain backward compatible over time. Breaking changes to the APS-V1 contract require APS-V2.

### 10.2 Standard SemVer Rules

Each standard and substandard MUST use SemVer:

- Minor/Patch releases MUST be backward compatible
- Breaking changes require a SemVer major bump

#### 10.2.1 Version Format

Versions MUST use the format `MAJOR.MINOR.PATCH`:

- **MAJOR**: Increment for breaking changes to schema, artifacts, or behavior
- **MINOR**: Increment for backward-compatible new features
- **PATCH**: Increment for backward-compatible bug fixes

#### 10.2.2 Backwards Compatibility Flag

The `backwards_compat` field indicates API stability:

```toml
[standard]
backwards_compat = true   # No breaking changes from previous version
backwards_compat = false  # Contains breaking changes
```

**Rule**: `backwards_compat: false` REQUIRES a MAJOR version increment.

CLI validation MUST fail if:
- `backwards_compat: false` but MAJOR version is 0 for non-experiments
- `backwards_compat: false` but version was not bumped from previous release

#### 10.2.3 Experiment Versioning

Experiments (EXP-*) MAY use `0.x.x` versions freely:

```toml
[experiment]
version = "0.1.0"  # Pre-1.0 indicates experimental status
```

Upon promotion to official standard, version MAY:
- Reset to `1.0.0` if significant changes occurred
- Continue from experiment version if mature

#### 10.2.4 Substandard Versioning

Substandards:
- MUST inherit the APS major version from their parent (e.g., `V1`)
- MAY version independently from parent standard
- SHOULD align major versions with parent for clarity

### 10.3 Required Evolution Pack on Major Bump

If a standard or substandard introduces a SemVer major bump, it MUST include:

```
evolution/major/<version>/
├── rationale.toml      # Why the breaking change was necessary
├── compatibility.toml  # What breaks and what doesn't
└── migration.md        # How to migrate
```

CI MUST fail if:

- A breaking change is detected but version is not major-bumped
- A major bump occurs without the evolution pack
- The evolution pack is missing required fields

---

## 11. Examples and Tests Requirements

### 11.1 Examples

For standards and experiments, `examples/` MUST contain at least one example that demonstrates valid usage. An `examples/README.md` SHOULD index available examples.

For substandards, `examples/` is OPTIONAL. Substandards inherit end-to-end usage examples from the parent standard.

### 11.2 Tests

Every package MUST have automated test coverage. Tests MAY be provided via either:

1. **Integration test files** in a `tests/` directory (e.g., `tests/my_test.rs`), OR
2. **Inline test modules** in source files (e.g., `#[cfg(test)] mod tests` in `src/lib.rs`)

At least one of these forms MUST be present. CI MUST run these tests for all packages.

---

## 12. Agent Skills Namespace

### 12.1 Required Skills

Standards and experiments MUST provide agent assets under `agents/skills/`. This directory MUST include at least one skill file or README usable by an agent.

For substandards, `agents/skills/` is OPTIONAL. The substandard's `docs/01_spec.md` serves as agent-readable context — it specifies what artifacts the substandard consumes and produces, which is sufficient for agent reasoning.

### 12.2 Reserved Directories

The following directories are RESERVED for future APS versions:

- `agents/commands/`
- `agents/tools/`

Tooling SHOULD treat unknown directories under `agents/` as warnings.

---

## 13. Templates (Optional)

If `templates/` exists inside a standard/substandard package:

- Templates MUST be deterministic and CLI-renderable
- Templates SHOULD scaffold packages that pass validation immediately
- Templates MUST NOT be stored only at repo root (co-locate with the standard)

### 13.1 Template Structure

```
templates/
├── standard/
│   ├── template.toml     # Template metadata
│   └── skeleton/         # Files to scaffold
├── substandard/
│   ├── template.toml
│   └── skeleton/
└── experiment/
    ├── template.toml
    └── skeleton/
```

---

## 14. Experimental Standards Lifecycle

### 14.1 Purpose

Experiments are used for rapid iteration and community feedback before committing to official support.

### 14.2 Non-Official Status

Experiments:

- MUST NOT be included in official registry views
- MUST NOT be automatically enforced on downstream repos
- MUST follow the same package structure as official standards
- MUST pass the same validation as official standards

### 14.3 Promotion

An experiment MAY be promoted to an official APS standard via:

```bash
aps v1 promote EXP-V1-XXXX --to APS-V1-YYYY
```

If promoted:

- The official standard is created under `standards/v1/APS-V1-YYYY-*/`
- The experiment MUST remain in `standards-experimental/v1/`
- The experiment MUST record promotion metadata in `experiment.toml`

---

## 15. Derived Views (Registries)

### 15.1 Generated, Not Authoritative

Tooling MAY generate registry views from the canonical filesystem:

```bash
aps v1 generate views --format json|toml|md
aps v1 generate views --filter official|experimental|all
```

### 15.2 View Outputs

Generated views are placed in:

```
generated/v1/views/
```

This directory SHOULD be gitignored. Views MUST include a header:

```
# GENERATED — DO NOT EDIT
# Regenerate with: aps v1 generate views
```

---

## 16. Compliance and Failure Reporting

### 16.1 Validation Requirements

Validation tooling and CI MUST:

- Fail fast on violations that could affect downstream consumers
- Provide a reasoned, human-readable error message
- Include remediation guidance when possible

### 16.2 Error Codes

All validation errors MUST include a unique error code for CI integration.

Error codes SHOULD be:
- Human-readable (e.g., `MISSING_REQUIRED_DIR`, not `E001`)
- SCREAMING_SNAKE_CASE for grep-ability and consistency
- Distinct from standard IDs (no `APS-V1-` prefix)

**Examples**:
```
MISSING_REQUIRED_DIR: Missing required directory: examples/
MISSING_METADATA_FILE: Missing metadata file
INVALID_STANDARD_ID: Invalid standard.toml: malformed 'id' field
BREAKING_CHANGE_NO_MAJOR_BUMP: Breaking change detected without major version bump
PROTO_DESCRIPTOR_MISMATCH: Protobuf descriptor mismatch
```

### 16.3 Exit Codes

CLI validation commands MUST use:

- `0` — All checks passed
- `1` — Errors found
- `2` — Warnings only (no errors)

---

## 17. Package Distribution

Distribution, installation, and CLI composition for consumer projects are defined by `APS-V1-0000.DI01` (Distribution & Installation substandard).

The DI01 substandard specifies:

- How standards are published as independent Rust crates
- The bootstrap CLI (`apss`) for project onboarding
- Version resolution and lockfile format (`apss.lock`)
- Composed binary generation for project-local CLI

### 17.1 Project Configuration

Consumer projects declare which standards they adopt via `apss.toml`, defined by `APS-V1-0000.CF01` (Project Configuration substandard).

The CF01 substandard specifies:

- The `apss.toml` schema for declaring standards, versions, and config
- Cascading configuration for monorepos
- Typed configuration validation via the `StandardConfig` trait (§8.3)

See ADR 0001 (Versioning Strategy) for detailed semantics.

---

## Appendix A: Validation Checklist

### Standards and Experiments

- [ ] `standard.toml` or `experiment.toml` with valid schema
- [ ] `docs/01_spec.md` (normative spec)
- [ ] `examples/` with at least one example
- [ ] Test coverage (integration tests in `tests/` or inline `#[cfg(test)]`)
- [ ] `agents/skills/` with at least one skill or README
- [ ] `Cargo.toml` and `src/lib.rs` (Rust crate)
- [ ] Implements `Standard` trait

### Substandards

- [ ] `substandard.toml` with valid schema and parent reference
- [ ] `docs/01_spec.md` (specifies consumed/produced artifacts)
- [ ] Test coverage (integration tests in `tests/` or inline `#[cfg(test)]`)
- [ ] `Cargo.toml` and `src/lib.rs` (Rust crate)

### Technical category (additional)

- [ ] `proto/` with protobuf contracts
- [ ] Breaking-change detection enabled
