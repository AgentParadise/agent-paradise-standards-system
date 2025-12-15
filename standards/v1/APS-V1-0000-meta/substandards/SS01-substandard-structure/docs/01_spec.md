# APS-V1-0000.SS01 — Substandard Structure (Canonical Specification)

**Version**: 1.0.0  
**Status**: Active  
**Parent**: APS-V1-0000 (Meta-Standard)

---

> **RFC 2119 Keywords**: The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

---

## 1. Scope

This substandard defines the structural requirements for all APS substandards within the V1 ecosystem. It specifies:

- Package layout and required files
- Metadata schema (`substandard.toml`)
- ID format and parent referencing
- Versioning and compatibility rules
- Validation requirements

## 2. Definitions

### 2.1 Substandard

A domain-specific extension package that inherits from and extends a parent standard. Substandards provide specialized implementations while maintaining structural consistency.

### 2.2 Parent Standard

The standard that a substandard extends. Every substandard MUST have exactly one parent standard.

### 2.3 Profile Code

A two-letter uppercase code identifying the substandard type within its parent:
- `SS` — Structure/Schema definitions
- `GH` — GitHub integrations
- `PY` — Python implementations
- `TS` — TypeScript implementations
- `RS` — Rust implementations
- `GO` — Go implementations
- `SK` — Agent Skills
- Custom codes as needed

## 3. ID Format

### 3.1 Pattern

Substandard IDs MUST match: `APS-V1-XXXX.YY##`

Where:
- `APS-V1-XXXX` — Parent standard ID
- `.` — Separator (required)
- `YY` — Two uppercase letters (profile code)
- `##` — Two-digit sequence number (01-99)

### 3.2 Examples

```
APS-V1-0000.SS01  — First structure substandard of meta-standard
APS-V1-0001.GH01  — First GitHub profile of APS-V1-0001
APS-V1-0001.PY01  — First Python binding of APS-V1-0001
APS-V1-0002.TS02  — Second TypeScript variant of APS-V1-0002
```

## 4. Package Layout

Substandards MUST follow this directory structure:

```
standards/v1/{parent-id}-{parent-slug}/
  substandards/
    {profile-code}-{slug}/
      substandard.toml      # REQUIRED: Metadata
      Cargo.toml            # REQUIRED: Rust crate manifest
      src/
        lib.rs              # REQUIRED: Crate entrypoint
      docs/
        00_overview.md      # RECOMMENDED: Overview
        01_spec.md          # REQUIRED: Normative specification
      examples/
        README.md           # REQUIRED: Example index
      tests/
        README.md           # REQUIRED: Test requirements
      agents/
        skills/
          README.md         # REQUIRED: Agent skill instructions
      templates/            # OPTIONAL: Substandard-specific templates
```

## 5. Metadata Schema

### 5.1 substandard.toml

```toml
schema = "aps.substandard/v1"

[substandard]
id = "APS-V1-0000.SS01"           # REQUIRED: Substandard ID
name = "Substandard Structure"     # REQUIRED: Human-readable name
slug = "substandard-structure"     # REQUIRED: Filesystem-safe slug
version = "1.0.0"                  # REQUIRED: SemVer version
parent_id = "APS-V1-0000"          # REQUIRED: Parent standard ID
parent_major = "1"                 # REQUIRED: Parent major version alignment

[ownership]
maintainers = ["AgentParadise"]    # REQUIRED: Maintainer list
```

### 5.2 Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `schema` | string | MUST be `"aps.substandard/v1"` |
| `substandard.id` | string | MUST match ID pattern |
| `substandard.name` | string | Human-readable name |
| `substandard.slug` | string | Kebab-case, filesystem-safe |
| `substandard.version` | string | SemVer format (X.Y.Z) |
| `substandard.parent_id` | string | MUST reference existing standard |
| `substandard.parent_major` | string | Parent version alignment |
| `ownership.maintainers` | array | At least one maintainer |

## 6. Compatibility Rules

### 6.1 Within Parent Major Version

Unlike standards, substandards MAY introduce breaking changes within a parent's major version. This allows:

- Rapid iteration on domain-specific implementations
- Platform-specific optimizations
- Language idiom alignment

### 6.2 Cross-Parent Compatibility

When the parent standard increments its major version:
- Substandards SHOULD provide migration guidance
- Substandards MAY require updates to align with new parent

### 6.3 Version Independence

Substandard versions are independent of parent versions:
- Substandard at `2.0.0` can exist under parent at `1.5.0`
- Version reflects substandard changes, not parent changes

## 7. Validation Requirements

### 7.1 Structural Validation

Substandards MUST pass all structural checks from the parent meta-standard:
- Required directories exist
- Required files present
- Rust crate compiles

### 7.2 Metadata Validation

The following MUST be validated:
- `id` matches pattern `APS-V1-\d{4}\.[A-Z]{2}\d{2}`
- `parent_id` matches the ID prefix (before `.`)
- `parent_id` references an existing standard
- `version` is valid SemVer

### 7.3 Location Validation

Substandards MUST be located at:
```
standards/v1/{parent-id}-{parent-slug}/substandards/{profile-code}-{slug}/
```

## 8. Error Codes

| Code | Severity | Description |
|------|----------|-------------|
| `INVALID_SUBSTANDARD_ID` | Error | ID doesn't match `APS-V1-XXXX.YY##` |
| `INVALID_PARENT_REF` | Error | `parent_id` doesn't match ID prefix |
| `PARENT_NOT_FOUND` | Error | Referenced parent doesn't exist |
| `SUBSTANDARD_WRONG_LOCATION` | Error | Not under parent's `substandards/` |

## 9. Templates

Substandards inherit template structure from the parent meta-standard. The `templates/substandard/` skeleton provides:

- `substandard.toml` template with Handlebars variables
- Standard directory structure
- Placeholder documentation

### 9.1 Template Variables

| Variable | Description |
|----------|-------------|
| `{{id}}` | Full substandard ID |
| `{{name}}` | Human-readable name |
| `{{slug}}` | Filesystem-safe slug |
| `{{version}}` | Initial version |
| `{{parent_id}}` | Parent standard ID |
| `{{parent_major}}` | Parent major version |
| `{{maintainers}}` | Maintainer list |

## 10. Conformance

A substandard is conformant if:

1. Package structure matches Section 4
2. Metadata validates per Section 5
3. ID format matches Section 3
4. Located correctly per Section 7.3
5. Parent reference is valid per Section 5.2

