# APS-V1-0000.CF01: Validation Delegation Protocol (Normative)

**Version**: 1.0.0
**Status**: Active
**Parent**: APS-V1-0000.CF01 (Project Configuration)

Sibling normative spec to `01_spec.md`, `02_slug_registry.md`, and
`03_contribution_schema.md`. Equal precedence under
APS-V1-0000 §1.1.

This document defines how APSS.yaml is validated end to end across the
meta-validator and the per-standard validators.

## Terminology

RFC 2119 keywords apply.

---

## 1. The Three Roles

APSS.yaml validation has three roles, deliberately separated so that
each standard owns its own contract surface and the meta-standard
does not become a god object:

1. **Meta-validator (CF01).** Owns the file: opens it, parses TOML,
   resolves the workspace cascade (see `01_spec.md` §4 as rewritten
   by the APSS.yaml migration), checks reserved keys and the slug
   registry. Then routes each registered slug to its owning validator.
2. **Standard validator (per slug).** Owns one top-level key. Receives
   the parsed config object (already deserialized into the standard's
   `StandardConfig` type) and runs `StandardConfig::validate()` plus
   any standard-specific cross-checks.
3. **Substandard validator (per nested key).** Owns one nested key
   under the parent slug. Runs its own `validate()` on the substandard
   config, if any.

The meta-validator MUST NOT inspect the semantics of any standard's
configuration. The per-standard validator MUST NOT touch keys that
do not belong to it.

---

## 2. End-to-End Validation Order

`<bootstrap> v1 validate` runs the following steps in order. Each step
gates the next: a failure halts subsequent steps for that section
(but other sections continue, so the user sees as many diagnostics
as possible per run).

```
1. Parse TOML        (CF01)   error code CF_TOML_PARSE_ERROR
2. Cascade resolve   (CF01)   per 01_spec.md §4
3. Reserved keys     (CF01)   project, workspace, tool, etc.
4. Registry lookup   (CF01)   per 02_slug_registry.md §6
5. Schema check      (CF01)   per slug, structural only
6. Deserialize       (owner)  into StandardConfig type
7. Semantic validate (owner)  StandardConfig::validate()
8. Substandard pass  (owner)  per 05_substandard_nesting.md
9. Cross-standard    (CF01)   project-wide consistency
```

Steps 5 through 8 are per-section and MUST be runnable in parallel
across slugs for performance; CF01 imposes no ordering between them.
Step 9 runs once after all per-section work is done.

### 2.1 What "Structural Only" Means in Step 5

Step 5 applies the JSON Schema from
`03_contribution_schema.md` to the raw TOML node for that slug. It
checks:

- the section is a TOML mapping,
- only known keys appear (`additionalProperties: false`),
- types match,
- required keys are present.

It does NOT check value ranges, cross-field consistency, or anything
the schema cannot express. That is the standard validator's job.

### 2.2 Cross-Standard Consistency (Step 9)

The meta-validator runs the following cross-section checks after all
sections have been validated independently:

| Code | Severity | Rule |
|------|----------|------|
| `CF_INCOMPATIBLE_STANDARDS` | Error | Two standards declare a mutual incompatibility (via their contribution schema `$id` collision metadata) and both are enabled. |
| `CF_DUPLICATE_SLUG_LIVE` | Error | The slug registry reports a collision (`CF_SLUG_COLLISION`) and that collision affects the active set. This is a hard fail even if the upstream registry check was already raised. |
| `CF_DISABLED_BUT_REQUIRED` | Error | Standard A declares `requires = ["b"]` in its manifest and standard B is `disable: true`. |
| `CF_VERSION_RANGE_CONFLICT` | Error | Carried over from `01_spec.md` §6 cascade rules. |

Mutual-incompatibility metadata is OPTIONAL. Standards MAY declare
`[contribution.conflicts_with]` in their metadata file; if absent,
this check is a no-op for that standard.

---

## 3. Delegation Contract

### 3.1 What CF01 Hands Off

For each registered slug present in APSS.yaml, CF01 passes the
following to the owning validator:

```rust
pub struct DelegatedSection<'a> {
    /// Path to the APSS.yaml that produced this section.
    pub source_path: &'a Path,

    /// Resolved cascade level: 0 = workspace root, n = nth child.
    pub cascade_level: usize,

    /// Slug for this section.
    pub slug: &'a str,

    /// Parsed TOML value for this section, post-cascade-merge,
    /// BEFORE deserialization. Owners deserialize into their own
    /// StandardConfig type.
    pub raw: &'a toml::Value,

    /// Snapshot of the universal keys, lifted by CF01 for convenience.
    /// disable: from `disable`, defaults to false.
    /// version: from `version`, None if absent.
    pub universal: UniversalKeys,
}

pub struct UniversalKeys {
    pub disable: bool,
    pub version: Option<semver::VersionReq>,
}
```

Notes:

- `raw` already has the cascade applied per `01_spec.md` §4 merge
  rules. Owners do not re-implement cascade.
- `universal` is provided so owners do not duplicate parsing of
  `disable` and `version` and stay consistent with CF01's
  interpretation. Owners MAY still read these directly if they need
  raw error positions for diagnostics.
- The `source_path` and `cascade_level` are required for diagnostic
  spans pointing back into the right file in a monorepo.

### 3.2 What Owners Return

```rust
pub trait DelegatedValidator {
    fn slug() -> &'static str;
    fn validate_section(section: DelegatedSection<'_>) -> Diagnostics;
}
```

The returned `Diagnostics` MUST:

- include the standard ID in every diagnostic's `code` field via the
  standard's normal error prefix (e.g. `DOCS_*`, `FIT_*`),
- include byte spans pointing into `section.source_path` so the
  CLI can render them with line/column,
- never include diagnostics for keys outside `section.slug`.

If an owner returns diagnostics whose codes do not start with the
standard's declared error prefix (see APS-V1-0000 §16.2), the meta-
validator MUST emit `CF_OWNER_PREFIX_VIOLATION` (warning, not error)
to flag the protocol breach without dropping the diagnostics.

### 3.3 Disabled Standards

If `universal.disable` is true, the meta-validator MUST:

- still pass the section to the owner (for schema-level checks on
  the remaining keys, so typos in disabled sections do not silently
  rot),
- treat the standard as inactive for any cross-standard consistency
  check (step 9): inactive standards do not consume `requires`,
  do not contribute to `CF_INCOMPATIBLE_STANDARDS`, etc.

This mirrors how a disabled VS Code extension still has its
settings validated.

---

## 4. Unknown Keys are Errors

The meta-standard's default-on philosophy (`01_spec.md` §5, brief
binding decision 5) requires that any active standard works without
needing an APSS.yaml section. The flip side is that any top-level
key in APSS.yaml MUST be one of:

- a reserved CF01 key (see `02_slug_registry.md` §3.2),
- a registered slug from the generated registry.

Anything else is an error:

| Code | Severity | Rule |
|------|----------|------|
| `CF_UNKNOWN_TOP_LEVEL_KEY` | Error | A top-level key in APSS.yaml is neither reserved nor a registered slug. Diagnostic MUST include the closest registered slug as a suggestion if edit distance is below 3. |
| `CF_UNKNOWN_NESTED_KEY` | Error | A key inside a registered section is not in that section's contribution schema (`additionalProperties: false`). |
| `CF_UNKNOWN_SUBSTANDARD_KEY` | Error | A nested key under a registered slug claims to be a substandard but no substandard with that slug exists for the parent standard. |

This is deliberate: unknown-key permissiveness is what makes config
files rot over decades. APSS.yaml errors loudly on the first run after
a typo is introduced.

The `--allow-unknown-keys` flag MUST NOT exist. Migration paths for
removed keys are handled by per-standard deprecation diagnostics
emitted from `validate()`, not by relaxing the meta-validator.

---

## 5. Disabled and Missing Sections

| State | APSS.yaml | Behavior |
|-------|-----------|----------|
| Active, default config | section absent | Standard runs with `Default` config. |
| Active, overridden | section present, no `disable` | Standard runs with section merged onto `Default`. |
| Explicitly active | `disable: false` in section | Same as above; the `disable: false` is documentation. |
| Disabled | `disable: true` in section | Standard skipped from runtime; structural validation of the section still runs (§3.3). |

Section presence is never required. The meta-validator MUST NOT
emit `CF_EMPTY_STANDARDS` or any similar diagnostic just because
APSS.yaml has no sections for active standards: that is the
expected state for greenfield projects.

---

## 6. CLI Surface

The CF01 validation delegation is exposed at the CLI through:

```
<bootstrap> v1 validate                # full repo + project pass
<bootstrap> v1 validate --project      # only APSS.yaml + sections
<bootstrap> v1 validate --slug <slug>  # only one section, useful for editor LSPs
```

`--slug` MUST still parse the entire file (so cascade is correct)
but only emit diagnostics whose path roots at the named section.

The placeholder `<bootstrap>` reflects repo issue 64 (APS vs APSS naming)
and resolves once that issue closes.

---

## 7. Error Code Summary (New)

The following codes are introduced by this document. They are in
addition to the existing CF01 codes from `01_spec.md` §6.

| Code | Severity | Source |
|------|----------|--------|
| `CF_TOML_PARSE_ERROR` | Error | Step 1 |
| `CF_UNKNOWN_TOP_LEVEL_KEY` | Error | Step 4 / §4 |
| `CF_UNKNOWN_NESTED_KEY` | Error | Step 5 / §4 |
| `CF_UNKNOWN_SUBSTANDARD_KEY` | Error | Step 5 / §4 |
| `CF_INCOMPATIBLE_STANDARDS` | Error | Step 9 |
| `CF_DUPLICATE_SLUG_LIVE` | Error | Step 9 |
| `CF_DISABLED_BUT_REQUIRED` | Error | Step 9 |
| `CF_OWNER_PREFIX_VIOLATION` | Warning | §3.2 |

The existing `CF_INVALID_CONFIG_VALUE` and
`CF_CONFIG_VALIDATION_FAILED` codes from `01_spec.md` §6 are reused
unchanged at steps 6 and 7 respectively.

---

## 8. Worked Example

Consumer APSS.yaml:

```yaml
schema: apss.project/v1

project:
  name: my-service
  apss_version: v1

docs:
  enforce_adr: true
  adr_dir: docs/adrs
  adr:
    disable: false
  retrospectives:
    disable: true

fintess:                # typo
  threshold: 0.7
```

Diagnostics (in order):

1. `CF_UNKNOWN_TOP_LEVEL_KEY` on `fintess` with hint
   "did you mean 'fitness'?"
2. Inside `docs`, the meta-validator runs the docs standard's
   delegated validator on the section. The substandards `adr` and
   `retrospectives` are dispatched per `05_substandard_nesting.md`.
3. If `docs` declares no `enforce_adr` key in its contribution
   schema, `CF_UNKNOWN_NESTED_KEY` fires for it. Otherwise the
   value `true` passes structural validation and the docs standard's
   `validate()` runs.

Exit code: 1 (errors present). The fitness section was a typo, so the
fitness standard never runs; that is correct under the no-permissive
rule and avoids the failure mode where a misnamed section silently
loses configuration.
