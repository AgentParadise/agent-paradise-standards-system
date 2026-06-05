# APS-V1-0000.CF01: Config Contribution Schema (Normative)

**Version**: 1.0.0
**Status**: Active
**Parent**: APS-V1-0000.CF01 (Project Configuration)

Sibling normative spec to `01_spec.md` and `02_slug_registry.md`.
Equal precedence under APS-V1-0000 §1.1.

This document specifies the schema format each standard ships to
contribute its section to APSS.yaml. It is the APSS analog of VS Code's
`contributes.configuration`.

## Terminology

RFC 2119 keywords apply, as in `01_spec.md`.

---

## 1. Why Contribution Schemas

APSS.yaml has one top-level key per registered slug (see
`02_slug_registry.md`). Each standard owns the contents of its own
key. To make the file self-documenting, machine-validatable, and
editor-friendly, every standard MUST ship a typed schema describing:

- which keys live under its slug,
- their types,
- their defaults,
- whether they are required,
- one-line docs strings (for hover, generated docs, and templates).

The schema is the contract between the standard and the meta-validator.
The meta-validator does NOT need to know the semantics of any
standard's keys; it only needs the schema to know:

- which keys are valid under that slug,
- the right Rust type to deserialize into,
- where to delegate semantic validation (see
  `04_validation_delegation.md`).

---

## 2. Source of Truth

Each standard's contribution schema is derived from its Rust
`StandardConfig` type (APS-V1-0000 §8.3.1). This is the only source
of truth. Hand-edited schema files are not permitted; CI MUST reject
any `config.schema.json` that does not round-trip from the Rust type.

```
src/
  lib.rs        # re-exports Config
  config.rs     # struct MyConfig { ... } impl StandardConfig
config.schema.json   # generated, checked in for tooling consumers
```

The Rust type carries the authoritative defaults, types, and
documentation (via doc comments and `serde` attributes). The JSON
Schema file is a derived artifact, committed for editor tooling and
external consumers that cannot run Rust.

### 2.1 Required Trait Surface

Every standard that registers a slug MUST implement `StandardConfig`,
or use the `NoConfig` marker. The trait was introduced in
APS-V1-0000 §8.3.1 and is extended here for contribution purposes:

```rust
pub trait StandardConfig:
    DeserializeOwned + Serialize + Default + ConfigContribution
{
    fn validate(&self) -> Diagnostics;
    fn json_schema() -> serde_json::Value;
    fn toml_template() -> String;
}

pub trait ConfigContribution {
    /// The slug under which this standard's section appears in
    /// APSS.yaml. Must equal the slug declared in metadata.
    fn slug() -> &'static str;

    /// Stable, human-readable title for the section.
    fn title() -> &'static str;

    /// One sentence summary used in generated docs and IDE hover.
    fn summary() -> &'static str;

    /// Optional substandard contributions. Each entry MUST match a
    /// substandard slug declared in `substandard.toml`.
    fn substandard_schemas() -> Vec<SubstandardContribution> {
        Vec::new()
    }
}

pub struct SubstandardContribution {
    pub slug: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub schema: serde_json::Value,
}
```

Notes:

- `toml_template()` emits documented TOML snippets for
  `[standards.<slug>.config]`.
- `slug()`, `title()`, and `summary()` are needed at code-generation
  time and are therefore associated functions, not instance methods.
- `substandard_schemas()` lets the parent standard carry its
  substandards' schemas without each substandard having to own a
  separate `StandardConfig` type. Substandards MAY ship their own
  `StandardConfig` instead; both are allowed (see `05_substandard_nesting.md`).

### 2.2 The `NoConfig` Marker

Standards that take no configuration MUST use `NoConfig`. This still
registers a slug and a (trivially empty) contribution schema, so that
APSS.yaml can still toggle the standard on or off via the universal
`disable` key (§3.1.1) and so that the meta-validator can still
report unknown keys for that slug.

---

## 3. APSS.yaml Section Shape

### 3.1 Universal Keys

CF01 reserves the following keys inside every standard's section.
Standards MUST NOT redefine them; meta-validation enforces this.

| Key | Type | Default | Purpose |
|-----|------|---------|---------|
| `disable` | bool | `false` | Disables the standard for this project (or workspace member). Carries over from EXP-V1-0004. |
| `version` | string | resolved by installer | Optional Cargo-style semver requirement that pins the standard's version. If absent, the workspace lockfile decides. |

The `disable` key is the universal off switch. An active standard
requires no section in APSS.yaml; a section exists only to override
or disable. Tooling MUST accept `disable: false` as a no-op
(useful for documentation purposes when explicitly opting in).

### 3.2 Standard-Owned Keys

All other keys under a slug are defined by the standard's contribution
schema. The schema MUST follow the JSON Schema 2020-12 dialect with
the following restrictions and conventions:

- `type` MUST be one of `object`, `string`, `integer`, `number`,
  `boolean`, `array`. `null` is allowed only as a union member.
- Top-level for the slug section is always `type: object`.
- `additionalProperties` MUST be `false` at every object level the
  standard controls. This is what makes unknown keys errors instead
  of silent ignores.
- Every property SHOULD have a `description` and a `default`. The
  meta-validator emits a warning (`CF_SCHEMA_NO_DESCRIPTION`) for
  any property missing a description so that generated docs do not
  ship with blanks.
- `$ref` is allowed only into the schema's own `$defs`. External
  refs are forbidden because the schema must be self-contained for
  the unified installer's offline mode (see `06_unified_install_seam.md` §4).

### 3.3 Worked Example

```yaml
# APSS.yaml fragment, contributed by the docs standard
docs:
  enforce_adr: true
  adr_dir: docs/adrs
  required_sections:
    - context
    - decision
    - consequences
```

derives from a `DocsConfig` struct whose generated schema is:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "apss.contribution/docs/v1",
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "disable":  { "type": "boolean", "default": false,
                  "description": "Disable the docs standard." },
    "version":  { "type": "string",
                  "description": "Optional semver pin for the docs crate." },
    "enforce_adr": {
      "type": "boolean",
      "default": true,
      "description": "Require ADRs for any change touching architectural seams."
    },
    "adr_dir": {
      "type": "string",
      "default": "docs/adrs",
      "description": "Directory where ADRs are stored."
    },
    "required_sections": {
      "type": "array",
      "items": { "type": "string" },
      "default": ["context", "decision", "consequences"],
      "description": "ADR headings that must be present."
    }
  }
}
```

The `$id` follows the pattern `apss.contribution/<slug>/v<major>`,
where `<major>` tracks the contribution schema's own version
(usually equal to the standard's major). The `$id` is what the
meta-validator uses to detect when a schema's contract has changed
incompatibly across releases.

---

## 4. Substandard Contributions

Substandards extend the parent section, not the top level. Their
schemas appear under the parent slug as nested objects keyed by the
substandard's slug. See `05_substandard_nesting.md` for the full
shape; this section covers what each substandard MUST contribute.

A substandard MUST contribute either:

1. its own `StandardConfig` implementation, which gets a generated
   `config.schema.json` at the substandard crate root, AND its entry
   appears under the parent's `substandard_schemas()`, OR
2. only a static `SubstandardContribution` returned from the
   parent's `substandard_schemas()`. This is the minimal form for
   substandards that exist only as toggles (`disable: true|false`).

In either case the substandard's `slug()` MUST equal the slug in
`substandard.toml` so that the slug registry and the contribution
schema agree.

---

## 5. Generation, Determinism, and Staleness

### 5.1 Generation Command

The contribution schema artifact MUST be regenerable via:

```
<bootstrap> v1 config schema <slug>
```

This is the same command surface as DI01 §3.2 `apss config schema <slug>`;
that command is preserved verbatim and remains the documented entry
point. The placeholder `<bootstrap>` reflects the open APS vs APSS naming
question (repo issue 64) and is replaced by the resolved name once
that issue closes.

### 5.2 Determinism

`StandardConfig::json_schema()` MUST be a pure function of the Rust
type. Implementations MUST NOT include timestamps, file paths, host
names, or other environment-dependent data. The generated
`config.schema.json` MUST be byte-stable across runs.

### 5.3 Staleness Detection

CI MUST detect when `config.schema.json` is out of date with respect
to the Rust source:

| Code | Severity | Trigger |
|------|----------|---------|
| `CF_SCHEMA_STALE` | Error | Committed `config.schema.json` differs from `json_schema()` output. |
| `CF_SCHEMA_MISSING` | Error | Standard implements `StandardConfig` but no `config.schema.json` is committed. |
| `CF_SCHEMA_NO_NOCONFIG_MARKER` | Error | Standard uses `NoConfig` but ships a non-trivial schema. |
| `CF_SCHEMA_NO_DESCRIPTION` | Warning | A schema property has no `description` field. |
| `CF_SCHEMA_NO_DEFAULT` | Warning | A non-required property has no `default`. |
| `CF_SCHEMA_NON_LOCAL_REF` | Error | Schema uses a `$ref` pointing outside its own `$defs`. |
| `CF_SCHEMA_INVALID_DIALECT` | Error | Schema declares a `$schema` other than the supported JSON Schema dialect. |

The release gate (DI01 §9.2) MUST run schema staleness checks for
every standard whose source has changed since the last release tag.

---

## 6. Tooling Targets

The contribution schema powers four concrete tooling targets:

1. **Generated config docs.** Tooling MAY render a per-standard docs
   page from the schema (`<bootstrap> v1 config docs <slug>`), removing
   the need for hand-written reference tables that drift.
2. **Editor autocomplete and hover.** Editors that understand JSON
   Schema can be pointed at the per-slug `$id` for completion. The
   unified installer SHOULD also emit an aggregate
   `generated/v1/apss.schema.json` that unions every standard's schema
   under its slug for editor integrations.
3. **Validation.** The meta-validator uses the schema for structural
   validation (types, required, additionalProperties). Semantic
   validation is delegated to `StandardConfig::validate()`; see
   `04_validation_delegation.md`.
4. **Scaffolding.** `<bootstrap> v1 config template` emits a TOML
   skeleton from every active standard's `toml_template()`, used by
   the unified installer's first-run flow
   (`06_unified_install_seam.md` §2.4).

---

## 7. Backward Compatibility

This document tightens the `StandardConfig` contract introduced in
APS-V1-0000 §8.3. Existing implementations that derived
`StandardConfig` without `ConfigContribution` MUST add the
`ConfigContribution` impl during the APSS.yaml migration window
described in the migration note attached to `01_spec.md`. The
meta-validator MUST emit `CF_MISSING_CONTRIBUTION_TRAIT` as an
error for any standard that lacks `ConfigContribution` after the
window closes.

For published v1 standards, the migration is a minor bump (no
breaking changes to consumers, since the trait extension is purely
additive on the standards side and APSS.yaml content is unchanged
by adding it).
