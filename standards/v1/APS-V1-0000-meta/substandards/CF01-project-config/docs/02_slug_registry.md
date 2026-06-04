# APS-V1-0000.CF01: Slug Registry (Normative)

**Version**: 1.0.0
**Status**: Active
**Parent**: APS-V1-0000.CF01 (Project Configuration)

This document is a sibling normative spec to `01_spec.md` and has
equal precedence with it under the meta-standard's precedence rule
(§1.1 of APS-V1-0000). It defines the slug registry that backs the
single-file apss.yaml configuration model.

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT",
"SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this
document are to be interpreted as described in
[RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Purpose

apss.yaml uses short slugs as top-level keys to namespace each
standard's configuration section. The registry is the single source
of truth that maps slugs to standard IDs and prevents collisions.
Every standard in the repository, including experimental standards,
MUST appear in the registry.

The registry exists to:

1. Guarantee that no two standards can claim the same top-level key
   in apss.yaml.
2. Allow the unified installer (Addendum 1, see `06_unified_install_seam.md`)
   to resolve a manifest entry like `docs:` back to its owning crate
   without scanning the filesystem.
3. Power editor tooling (autocomplete, schema lookup) by exposing a
   stable list of valid top-level sections.
4. Catch typos in apss.yaml early: unknown top-level keys are errors
   (see `04_validation_delegation.md`).

---

## 2. Where Slugs Live

### 2.1 Per-Standard Source of Truth

Each standard's existing metadata file is the canonical source for
its slug:

| Package type | File | Field |
|--------------|------|-------|
| Standard | `standard.toml` | `[standard] slug` |
| Substandard | `substandard.toml` | `[substandard] slug` |
| Experiment | `experiment.toml` | `[experiment] slug` |

These fields are already required by APS-V1-0000 §6. This substandard
tightens their rules and adds registry-level cross-checks.

The filesystem is the canonical source of truth (APS-V1-0000 §1.2),
so the registry is derived from these files, not the other way
around. There is no separate `slug.toml`.

### 2.2 Generated Registry Artifact

Tooling MUST be able to emit a generated registry view at:

```
generated/v1/slug_registry.json
```

The artifact is a derived view (APS-V1-0000 §15) and SHOULD be
gitignored. It exists for tooling consumers (the unified installer,
language servers, schema generators) that need fast lookups without
re-parsing every metadata file.

The artifact MUST include a `GENERATED` header and a regeneration
command, per APS-V1-0000 §15.2.

#### 2.2.1 Artifact Schema

```json
{
  "schema": "apss.slug_registry/v1",
  "generated_at": "2026-06-04T22:47:00Z",
  "entries": [
    {
      "slug": "docs",
      "standard_id": "EXP-V1-0004",
      "kind": "experiment",
      "crate": "apss-v1-0004-docs",
      "manifest_path": "standards-experimental/v1/EXP-V1-0004-docs/experiment.toml",
      "config_schema_path": "standards-experimental/v1/EXP-V1-0004-docs/config.schema.json",
      "substandards": [
        { "code": "ADR", "slug": "adr" },
        { "code": "PVS", "slug": "purpose-and-vision" }
      ]
    }
  ]
}
```

Field rules:

- `slug` MUST be the standard's slug; substandards do NOT get their
  own top-level entry (see §3.3 and `05_substandard_nesting.md`).
- `kind` MUST be one of `standard`, `experiment`. Substandards
  appear only inside the parent's `substandards` array.
- `substandards[].code` is the qualified substandard code (e.g.
  `ADR`, `PVS`), matching the `[A-Z]+\d*` portion of the substandard
  ID (APS-V1-0000 §4.2).
- `substandards[].slug` is the kebab-case slug declared in the
  substandard's `substandard.toml`; it is used as a nested key under
  the parent slug in apss.yaml.

#### 2.2.2 Regeneration

The generated artifact MUST be regenerable from the filesystem alone.
The regeneration command is part of the unified installer's
resolve step (see `06_unified_install_seam.md` §3.1). Tooling MAY
expose the regeneration command directly:

```
<bootstrap> v1 generate slug-registry
```

where `<bootstrap>` is the project-local entry point; see Note on
binary naming below.

> Note on binary naming. The binary name is intentionally unfixed in
> this spec while repo issue 64 (APS vs APSS naming) is open. All
> examples use `<bootstrap>` as a placeholder. Once issue 64 resolves,
> the name MAY be substituted everywhere without changing the
> mechanism.

---

## 3. Slug Format Rules

### 3.1 Lexical Format

A slug MUST match the regular expression:

```
^[a-z][a-z0-9]*(-[a-z0-9]+)*$
```

In prose: lowercase, kebab-case, starts with a letter, ends with a
letter or digit, no leading or trailing hyphens, no double hyphens.

Additional limits:

- A slug MUST be at least 2 characters and at most 32 characters.
- A slug MUST NOT contain ASCII characters outside the regex.
- A slug MUST NOT change after the standard's first published
  version (it is part of the public manifest surface).

Slugs are case-sensitive per the regex (always lowercase). Tooling
MUST NOT auto-correct case; mismatches MUST raise an error.

### 3.2 Reserved Slugs

The following top-level keys are reserved by CF01 and MUST NOT be
used as a standard's slug:

| Reserved | Owner | Purpose |
|----------|-------|---------|
| `schema` | CF01 | apss.yaml schema identifier |
| `project` | CF01 | Project identity |
| `workspace` | CF01 | Monorepo membership and cascade |
| `tool` | CF01 | Installer/tooling settings |
| `apss` | CF01 | Reserved for future meta-level keys |
| `standards` | CF01 | Reserved for future explicit pinning |
| `extends` | CF01 | Reserved for future cascade pointers |

The reserved list MAY grow in minor versions of CF01. Adding a new
reserved name is a backward-compatible change only if no shipped
standard already uses that slug; otherwise it requires a major bump
of CF01.

### 3.3 Substandard Slug Rules

Substandards MUST declare a slug in `substandard.toml`, but the slug
is scoped under the parent standard. Substandard slugs MUST be unique
within their parent's substandard set. Two substandards under
different parents MAY share a slug (e.g. both `docs.adr` and
`fitness.adr` would be legal).

Substandards MUST NOT appear as top-level keys in apss.yaml. The
registry artifact reflects this by nesting them inside the parent's
`substandards` array (see §2.2.1). The full apss.yaml shape for
substandard toggles is specified in `05_substandard_nesting.md`.

### 3.4 Experiment Slug Lifecycle

Experiments register slugs in `experiment.toml` and appear in the
registry with `kind = "experiment"`. On promotion to an official
standard (APS-V1-0000 §14.3):

- The experiment's slug SHOULD carry over to the promoted standard
  to avoid breaking consumer apss.yaml files.
- If the slug changes during promotion, the experiment's
  `experiment.toml` MUST record both the original slug and the
  promoted slug under `[promotion]`, and the meta-validator MUST
  emit `CF_SLUG_RENAMED_ON_PROMOTION` as a warning so downstream
  consumers can update.

---

## 4. Registry Construction

### 4.1 Discovery Roots

The meta-validator builds the registry by scanning both:

```
standards/v1/APS-V1-XXXX-*/standard.toml
standards-experimental/v1/EXP-V1-XXXX-*/experiment.toml
```

and, for each package found, recursively:

```
<package>/substandards/*/substandard.toml
```

Substandard directory naming follows APS-V1-0000 §4.2.1.

### 4.2 Deterministic Ordering

The generated registry MUST be sorted by `standard_id` ascending
(lexical) so that the artifact is byte-stable across runs on the
same filesystem state. Within a standard, `substandards[]` MUST be
sorted by `code` ascending. This is required for code generation
determinism (DI01 §6.2) and for human-reviewable diffs.

### 4.3 Cross-Standard Validation

After discovery, the meta-validator MUST run the checks listed in §5
across the full registry before emitting the artifact. If any check
fails the artifact MUST NOT be written and the validator MUST exit
with the appropriate error code.

---

## 5. Meta-Validation Rules

The following rules MUST be enforced by `<bootstrap> v1 validate repo`
(see `07_qa_checks.md` for the QA harness):

| Code | Severity | Rule |
|------|----------|------|
| `CF_SLUG_MISSING` | Error | Standard or experiment metadata is missing a `slug` field. |
| `CF_SLUG_INVALID_FORMAT` | Error | Slug does not match the regex in §3.1, or violates length limits. |
| `CF_SLUG_RESERVED` | Error | Slug is on the reserved list in §3.2. |
| `CF_SLUG_COLLISION` | Error | Two standards/experiments share the same slug. The error message MUST list all conflicting standard IDs and their manifest paths. |
| `CF_SUBSTANDARD_SLUG_COLLISION` | Error | Two substandards under the same parent share the same slug. |
| `CF_SLUG_NOT_IN_REGISTRY` | Error | A standard exists on disk but is missing from the generated registry artifact (artifact is stale). |
| `CF_REGISTRY_HAS_PHANTOM` | Error | The generated registry references a standard that does not exist on disk (artifact is stale). |
| `CF_SLUG_RENAMED_ON_PROMOTION` | Warning | Promoted experiment's slug differs from the published experimental slug (§3.4). |
| `CF_SLUG_TOO_GENERIC` | Warning | Slug is one of a small set of reserved-ish generic names (`config`, `core`, `meta`, `system`). Allowed but discouraged. |

Errors here are blocking: `<bootstrap> v1 validate repo` MUST exit
with code 1 if any error-severity slug rule fails. Warnings MUST
exit with code 2 only if no errors are present (APS-V1-0000 §16.3).

### 5.1 Completeness Check

The registry MUST be complete: every directory under `standards/v1/`
or `standards-experimental/v1/` that contains a valid metadata file
MUST be represented in the registry artifact. The completeness
check is what `CF_SLUG_NOT_IN_REGISTRY` enforces. CI MUST run this
check on every PR (see `07_qa_checks.md` §3.2).

### 5.2 Substandard Coverage

For every standard with a non-empty `substandards/` directory, every
substandard MUST appear under its parent's `substandards` array in
the registry. Missing entries raise `CF_SLUG_NOT_IN_REGISTRY`
qualified with the substandard ID.

---

## 6. Reading the Registry from apss.yaml

When the unified installer or validator parses apss.yaml, it looks
up each top-level key in the registry to find the owning crate:

1. Reserved key (§3.2): handled by CF01 directly.
2. Key matches a registry slug: delegate validation to the owning
   crate's validator (`04_validation_delegation.md`).
3. Key matches no slug and no reserved name: error
   `CF_UNKNOWN_TOP_LEVEL_KEY` (defined in
   `04_validation_delegation.md`).

The registry artifact's `manifest_path` and `config_schema_path`
fields are the bridge from a slug to the standard's contribution
schema; see `03_contribution_schema.md` for the schema format.

---

## 7. Examples

### 7.1 Minimal Valid Entry (Standard)

```toml
# standards/v1/APS-V1-0001-code-topology/standard.toml
[standard]
id = "APS-V1-0001"
slug = "topology"
# other required fields per APS-V1-0000 §6.1
```

Generated entry:

```json
{
  "slug": "topology",
  "standard_id": "APS-V1-0001",
  "kind": "standard",
  "crate": "apss-v1-0001-code-topology",
  "manifest_path": "standards/v1/APS-V1-0001-code-topology/standard.toml",
  "config_schema_path": "standards/v1/APS-V1-0001-code-topology/config.schema.json",
  "substandards": []
}
```

### 7.2 Experiment with Substandards

```toml
# standards-experimental/v1/EXP-V1-0004-docs/experiment.toml
[experiment]
id = "EXP-V1-0004"
slug = "docs"
```

Each substandard declares its slug:

```toml
# standards-experimental/v1/EXP-V1-0004-docs/substandards/ADR-adr/substandard.toml
[substandard]
id = "EXP-V1-0004.ADR"
slug = "adr"
```

Generated entry:

```json
{
  "slug": "docs",
  "standard_id": "EXP-V1-0004",
  "kind": "experiment",
  "crate": "apss-v1-0004-docs",
  "substandards": [
    { "code": "ADR", "slug": "adr" },
    { "code": "PVS", "slug": "purpose-and-vision" },
    { "code": "RTS", "slug": "retrospectives" }
  ]
}
```

Consumer apss.yaml then references:

```yaml
docs:
  adr:
    disable: false
  purpose-and-vision:
    disable: true
```

Substandard nesting is fully specified in
`05_substandard_nesting.md`.

---

## 8. Backward Compatibility

This document is introduced at CF01 v1.0 and is therefore not a
breaking change relative to a published CF01. The slug field
requirement existed under APS-V1-0000 §6 prior to this document;
this substandard sharpens the rules but does not introduce a new
required metadata field.

If the reserved slug list grows in a future minor version, the
meta-validator MUST emit `CF_SLUG_RESERVED` for any standard
whose slug becomes reserved AFTER its first publish, and the
release pipeline (DI01 §9) MUST treat that as a blocking error.
This is the safety valve that prevents silent renames.
