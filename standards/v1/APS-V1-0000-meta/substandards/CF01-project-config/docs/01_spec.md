# APS-V1-0000.CF01 Project Configuration (Specification)

**Version**: 2.0.0
**Status**: Active
**Parent**: APS-V1-0000 (Meta-Standard)

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Scope

This substandard defines a single, unified configuration mechanism that serves
three concerns at once: project configuration, standard activation, and the
manifest that drives installation. The model is borrowed from the VS Code
settings architecture: ONE shared configuration file whose top-level structure
is owned by the meta-standard, into which each standard REGISTERS a namespaced
section, with validation delegated to each standard's own validator.

Concretely, CF01 defines:

- The `APSS.yaml` manifest at the project root, its schema identifier, and its
  top-level structure.
- The core sections owned by CF01: project identity, the standards list, the
  workspace declaration, and the tool block.
- Cascading rules for nested `APSS.yaml` files in monorepos.
- A slug registry that every standard in the ecosystem registers into, and the
  meta-validation rules over that registry (Section 3, owned by the registry
  work block).
- A config contribution schema that each standard ships so its section can be
  type checked, documented, and validated without CF01 knowing the details
  (Section 5, owned by the registry work block).
- A validation delegation protocol that lets each standard's validator own its
  section while the meta validator aggregates results (Section 6, owned by the
  registry work block).
- The substandard nesting convention used inside a parent slug (Section 7,
  owned by the registry work block).
- The seam between this manifest, the resolution layer in DI01, and each
  standard's install contract, so that one command can install, update, and
  remove everything declared in the manifest (Section 8).
- A migration from the prior `apss.toml` and `.apss/config.toml` homes
  (Section 9).
- The QA checks that enforce all of the above across `standards/` and
  `standards-experimental/` (Section 10, owned by the registry work block).

The manifest is the single source of truth for what the project considers
active, configured, and installed. It is the file an operator reads to know
"what standards does this project use, with what config, at which versions",
and it is the file the installer reads to materialize that intent on disk.

---

## 2. The `APSS.yaml` Manifest

### 2.1 Filename and Location

A consumer project MUST place its configuration at `APSS.yaml` in the project
root.

- The file MUST be valid YAML as parsed by the Rust `serde_yaml` crate.
- The extension MUST be `.yaml`.
- The file MUST live at the project root, defined as the directory containing
  the project's version control root (`git rev-parse --show-toplevel`) or,
  when no VCS root is available, the directory in which the operator invokes
  the bootstrap CLI.
- A project MAY add nested `APSS.yaml` files inside workspace members. Their
  semantics are defined in Section 4.

The `.apss/` dot directory is reserved for GENERATED artifacts (resolved
indexes, build outputs, cached schemas, the composed binary). Configuration
MUST NOT live under `.apss/`. Tooling MUST refuse to read configuration from
`.apss/config.toml` or any path under `.apss/`.

### 2.2 Schema Identifier

The top-level `schema` key MUST be the literal string `apss.project/v1`.

### 2.3 Top-level Structure

The manifest has two kinds of top-level keys:

1. **Core keys** owned by CF01. These define project identity, the standards
   the project declares as active, the workspace shape, and tool-level
   configuration. Every core key is specified in this document.
2. **Slug keys** contributed by individual standards. Each standard registers
   a unique slug in the slug registry (Section 3), and the value of that slug
   key holds the standard's configuration for this project.

```yaml
schema: apss.project/v1

project:
  name: my-service
  apss_version: v1

standards:
  code-topology:
    id: APS-V1-0001
    version: ">=1.0.0, <2.0.0"
    substandards: ["RS01", "CI01"]
    config:
      output_dir: .topology
      languages: ["rust", "python"]

workspace:
  members: ["packages/*"]

tool:
  bin_dir: .apss/bin
```

Every top-level key MUST be either a core key listed above or a slug
registered in the slug registry. Unknown top-level keys MUST be rejected as
`CF_UNKNOWN_TOP_LEVEL_KEY` (see Section 6 for the delegation protocol that
catches this).

### 2.4 Core Section: `project`

```yaml
project:
  name: my-service      # REQUIRED. Non-empty string.
  apss_version: v1      # REQUIRED. APSS major version.
```

Rules:

- `project.name` MUST be a non-empty string.
- `project.apss_version` MUST be `"v1"`. Future major versions of APSS will
  introduce new permitted values; tooling MUST reject unknown values.

### 2.5 Core Section: `standards`

`standards` is a mapping from slug to a per-standard activation entry. The
slug MUST be one registered in the slug registry (Section 3).

```yaml
standards:
  docs:
    id: "EXP-V1-0004"             # REQUIRED. Standard or experiment ID.
    version: ">=1.0.0, <2.0.0"    # REQUIRED. SemVer requirement (Cargo style).
    enabled: true                  # OPTIONAL. Default: true.
    substandards: ["ADR01"]        # OPTIONAL. If omitted, all enabled.

  fitness:
    id: "EXP-V1-0003"
    version: "^0.3.0"
```

Field rules:

- `id` MUST match either `APS-V1-\d{4}` (official) or `EXP-V1-\d{4}`
  (experimental).
- `version` MUST be a valid SemVer requirement.
- Each `id` MUST appear under at most one slug. Two slugs pointing at the
  same standard ID MUST be rejected as `CF_DUPLICATE_STANDARD_ID`.
- An `EXP-V1-\d{4}` entry explicitly opts the project into enforcing an
  experimental standard. Experiments MUST NOT be installed, validated, or
  enforced unless they are declared in `APSS.yaml`.
- `enabled: false` keeps the entry in the manifest but disables the standard
  for this project. The unified installer (Section 8) MUST uninstall the
  standard's artifacts cleanly when this flag flips to false.
- `substandards`, if present, lists profile codes the project explicitly
  activates (e.g. `["RS01", "CI01"]`). If omitted, every substandard
  shipped by the standard is enabled.
- Each substandard code MUST match `[A-Z]{2}\d{2}`.

The `standards` map is the dependency declaration that the unified installer
reads to materialize the project. The standard's own configuration does NOT
live here; it lives in the top-level slug key (Section 2.7 below).

### 2.6 Core Section: `workspace`

`workspace` declares this manifest as the root of a monorepo cascade. Its
presence triggers the discovery and merge rules in Section 4.

```yaml
workspace:
  members: ["packages/*"]          # OPTIONAL. Glob patterns for child configs.
  exclude: ["packages/legacy-*"]   # OPTIONAL. Glob patterns to exclude.
```

Rules:

- `workspace` MUST NOT appear in a child `APSS.yaml`. A manifest with
  `workspace` is by definition a root manifest.
- `members` patterns MUST resolve to directories that contain an
  `APSS.yaml`. A pattern that matches no directory MUST raise
  `CF_EMPTY_WORKSPACE_GLOB` as a warning.

### 2.7 Standard Sections (slug keys)

Each active standard contributes ONE top-level section keyed by its slug. The
value is owned by the standard; CF01 treats it as opaque YAML during parsing
and hands it to the standard's validator (Section 6).

Default-on philosophy:

- An active standard requires NO section in `APSS.yaml`. Defaults from the
  standard's contributed schema apply.
- A section exists only to OVERRIDE a default or to DISABLE a feature.
- The `disable: false` flag is the convention for opt-out, both at the top of
  a section and at the head of any nested substandard block (Section 7).

Snake case rule:

- Scalar field names inside any standard section MUST be `snake_case`, to
  match the Rust struct field names the validator deserialises into.

Example (taken from the EXP-V1-0004 docs section after the re-home in commit
`1784797`):

```yaml
docs:
  disable: false
  root: "docs"

  index:
    disable: false
    auto_generate: true

  adr:
    disable: false
    directory: "adrs"
    naming_pattern: "ADR-\\d{3,5}-[a-zA-Z0-9-]+\\.md"
```

### 2.8 Core Section: `tool`

`tool` carries preferences for the bootstrap CLI and supporting tooling.
Every key is optional with documented defaults.

```yaml
tool:
  bin_dir: ".apss/bin"             # OPTIONAL. Default: ".apss/bin".
  registry: "https://crates.io"   # OPTIONAL. Default: crates.io.
  offline: false                   # OPTIONAL. Default: false.
  log_level: "warn"                # OPTIONAL. Default: "warn".
```

The values map directly onto resolution and installation flags described in
DI01 and Section 8.

---

## 3. Slug Registry

The slug registry is the single source of truth that maps every registered
slug to its owning standard or experiment. Its normative definition lives in
the sibling spec `02_slug_registry.md`, which has equal precedence with this
document. See that file for the registration source of truth, the generated
registry artifact, the format and reservation rules, and the meta-validation
rules that enforce uniqueness and completeness across `standards/` and
`standards-experimental/`.

---

## 4. Cascade (Monorepos)

### 4.1 Discovery

To resolve the active configuration for a given working directory, tooling
MUST:

1. Walk upward from the working directory looking for an `APSS.yaml`
   containing a `workspace` key. The first such file is the **root
   manifest**.
2. While walking, every `APSS.yaml` without a `workspace` key encountered
   between the working directory and the root manifest is a **child
   manifest**, in deepest-to-shallowest order.
3. If no manifest with a `workspace` key is found, the closest `APSS.yaml`
   to the working directory is the root manifest and there are no child
   manifests.

The "nearest manifest wins" rule layers like VS Code user and workspace
settings: the user-wide configuration is the root manifest, and the
workspace-scoped override is the nearest child manifest. The closer the
manifest is to the working directory, the higher its precedence.

### 4.2 Merge Rules

Merging happens in two passes: core sections (this table) and standard
sections (Section 4.3).

| Key | Rule |
|-----|------|
| `schema` | MUST be identical across root and every child. Mismatch is `CF_SCHEMA_MISMATCH`. |
| `project.name` | Child wins. A child MAY rename itself. |
| `project.apss_version` | MUST match the root exactly. Mismatch is `CF_APSS_VERSION_MISMATCH`. |
| `standards.<slug>` absent in child | Inherited from the parent (root or nearer child). |
| `standards.<slug>` present in child | Child entry fully replaces the inherited entry (no deep merge). |
| `standards.<slug>.enabled = false` in child | Disables that standard for this member subtree only. |
| `workspace` | MUST NOT appear in a child. Presence in a child is `CF_WORKSPACE_IN_CHILD`. |
| `tool.<key>` | Child overrides individual keys; unset child keys inherit. |

### 4.3 Merging Standard Sections

Standard sections are merged per slug. The standard's contributed schema
(Section 5) declares for each field whether the merge is "child replaces" or
"deep merge per key". CF01 itself performs no deep merge on standard
sections; it delegates the merge decision to the standard's validator via the
contribution schema. A standard that does not declare merge semantics for a
field defaults to "child replaces".

### 4.4 Version Range Intersection

When both root and a child declare a `version` for the same slug, the
resolved version requirement MUST be the intersection of the two ranges.
If the intersection is empty, tooling MUST emit `CF_VERSION_RANGE_CONFLICT`
and refuse to install.

### 4.5 Disable Inheritance

If a parent sets `standards.<slug>.enabled = false`, a child MAY re-enable
the standard by setting `enabled = true`. This lets a monorepo disable a
standard globally and opt one package back in. The same rule applies to
section-level `disable: false` flags inside standard sections, scoped to the
standard's own merge rules.

### 4.6 Generated Artifact Scope

Each manifest (root or child) owns its own `.apss/` artifact directory,
sibling to the manifest file. Tools MUST NOT write generated artifacts under
a different manifest's `.apss/` directory.

---

## 5. Config Contribution Schema

Each standard ships a contribution schema that declares its slug, the keys
it accepts under that slug, their types, defaults, and merge semantics for
the cascade. The normative definition of the contribution schema lives in
the sibling spec `03_contribution_schema.md`, which has equal precedence
with this document. CF01 treats each standard section as opaque YAML and
defers to the contribution schema for typing, defaulting, and merge.

---

## 6. Validation Delegation Protocol

The meta validator validates the core sections defined in Section 2 itself,
and delegates each registered slug section to its owning standard's
validator via the `StandardConfig` contract. Unknown top-level keys are
errors. The normative protocol lives in the sibling spec
`04_validation_delegation.md`, including the in-process trait surface, the
diagnostics aggregation rules, and the ordering between core and delegated
validation.

---

## 7. Substandard Nesting Convention

Substandards nest under their parent slug as kebab-case keys (for example
`docs.adr`, `docs.purpose-and-vision`, `docs.retrospectives`). Substandards
do not receive top-level slugs. The per-substandard block uses the same
`disable: false` convention as the parent section. The normative rules,
disable-inheritance matrix, and worked examples live in the sibling spec
`05_substandard_nesting.md`.

---

## 8. Manifest-Driven Installation (Summary)

The operator-approved Addendum 1 of the unified-config brief makes
configuration, distribution, and installation a single system. `APSS.yaml`
is the manifest, the unified installer is the glue, and each active
standard ships an install contract that the installer invokes.

The npm-style model is the binding analogy: `APSS.yaml` is to APSS what
`package.json` is to npm. The `standards` map (Section 2.5) is the
dependency declaration; one install command reads the manifest, resolves
versions via DI01, then drives each per-standard install contract; removing
an entry and re-running uninstalls cleanly; disabling via `enabled: false`
is equivalent to removal for on-disk artifacts but preserves the operator's
intent to keep the entry on the manifest.

Three ownership boundaries make the seam explicit:

- **CF01 owns the manifest** (this document and Section 2): the file, the
  schema, the slug keys, the activation entries, and the cascade.
- **DI01 owns resolution and packaging** (see `DI01/docs/01_spec.md` Section 4):
  where standards come from, how versions are pinned in the lockfile, how
  the composed binary is built.
- **Each standard owns its install contract** (`docs/02_install_contract.md`
  in the standard's package): the per-standard lifecycle hook the installer
  invokes for `install`, `uninstall`, and `plan`.

The normative install pipeline, the per-standard install contract surface,
the trait recommendation (`Installable`), the removal semantics, the
failure-handling rules, and the explicit DI01 seam table live in the
sibling spec `06_unified_install_seam.md`, which has equal precedence with
this document. DI01's matching pipeline lives in `DI01/docs/01_spec.md`
Section 4.

The canonical binary name is being resolved separately in repo issue 64
(APS vs APSS). This spec uses `<bootstrap>` (or the term "the unified
installer") wherever the binary name can be avoided.

### 8.1 Experimental Standards and Promotion Aliases

Consumer manifests MAY declare experimental standards by using an
`EXP-V1-XXXX` ID in `standards.<slug>.id`. This is explicit opt-in:
experimental standards are not part of the default baseline.

If a declared experiment has been promoted to an official standard, CF01 and
DI01 MUST treat the promotion as a compatibility alias:

1. CF01 accepts the `EXP-V1-XXXX` ID as valid manifest input.
2. DI01 resolves the alias to the promoted `APS-V1-XXXX` package.
3. The installer emits a warning diagnostic that names both IDs and the
   replacement slug.
4. The lockfile records the resolved official ID and SHOULD retain the
   original requested ID for auditability.
5. The project continues to validate using the promoted standard without a
   manual manifest edit.

The warning SHOULD recommend updating `APSS.yaml` from the experimental ID to
the official ID. It MUST NOT silently rewrite operator-owned configuration
without an explicit command or flag.

Promotion aliases MUST preserve the prior experiment's config schema for at
least one compatible official release, or ship a migration adapter that
translates the experimental config into the promoted standard's config. If
neither is possible, the alias MUST produce an error with a migration path
instead of silently dropping enforcement.

---

## 9. Migration from legacy split configuration

The prior project model used `apss.toml` for activation and allowed some
per-standard configuration under `.apss/config.toml`. CF01 keeps `APSS.yaml`
as the user-owned manifest and forbids user-authored configuration under
`.apss/`.

### 9.1 Scope of the change

| Concern | Before | After |
|---------|--------|-------|
| File location | `apss.toml` and `.apss/config.toml` | `APSS.yaml` |
| Serialisation | TOML | YAML |
| Schema identifier | `apss.project/v1` | `apss.project/v1` |
| Configuration in `.apss/` | Allowed (EXP-V1-0004) | Forbidden, dotdir is for generated artifacts only |
| Per-standard configuration | Split across files | `standards.<slug>.config` mapping |
| Substandard configuration | Implicit, varied | `substandards` list plus standard-owned config keys |

### 9.2 Top-level mapping

The current single-file YAML shape is:

```yaml
schema: apss.project/v1

project:
  name: my-service
  apss_version: v1

standards:
  code-topology:
    id: APS-V1-0001
    version: ">=1.0.0, <2.0.0"
    enabled: true
    substandards: ["RS01", "CI01"]
    config:
      output_dir: .topology
      languages: ["rust"]

workspace:
  members: ["packages/*"]
  exclude: ["packages/legacy-*"]

tool:
  bin_dir: .apss/bin
  registry: https://crates.io
  offline: false
  log_level: warn
  hooks:
    pre_commit: true
```

Every previously valid `apss.toml` activation field moves into `APSS.yaml`;
only configuration stored under `.apss/` moves into the manifest.

### 9.2.1 Tool hook settings

`tool.hooks` controls APSS-managed local enforcement hooks.

| Key | Type | Default | Purpose |
| --- | --- | --- | --- |
| `pre_commit` | bool | `true` | Installs or updates the managed pre-commit hook during `apss install`. |

`pre_commit: false` is an explicit local escape hatch. Installers MUST emit
a clear warning when it disables hook installation because commit-time APSS
validation will not run until the setting is re-enabled or hooks are installed
another way.

### 9.3 Transition behavior

Tooling MUST behave as follows during the transition window:

- If `APSS.yaml` exists: load it normally.
- If only `.apss/config.toml` exists: emit `CF_LEGACY_APSS_CONFIG_TOML` at
  error severity with the same hint. Tooling MUST NOT read configuration
  from `.apss/`.
- If both `APSS.yaml` and `.apss/config.toml` exist: load `APSS.yaml` and
  emit `CF_LEGACY_APSS_CONFIG_TOML` for the generated-directory config.
- A future major version of APSS MAY remove the legacy diagnostics and
  refuse to start when a legacy file is present. The diagnostics are the
  one-shot warning that lets the operator notice the rename.

### 9.4 Manual conversion

The recommended manual conversion is:

1. Keep `APSS.yaml` as the project manifest.
2. Keep or set `schema: apss.project/v1`.
3. Move keys from `.apss/config.toml` into the appropriate
   `standards.<slug>.config` mapping in `APSS.yaml`.
4. Delete `.apss/config.toml`.
5. Re-run the unified installer to refresh `apss.lock` and the composed
   binary.

DI01 and per-standard install contracts MAY ship an automated converter as
a fast-follow; the converter is out of scope for this spec.

### 9.5 Spec-internal compatibility

The authoritative project configuration file for this PR is `APSS.yaml`.

---

## 10. QA Checks

The meta validator MUST enforce the manifest, registry, contribution
schema, delegation, and substandard nesting rules across both `standards/`
and `standards-experimental/`. The normative check list lives in the
sibling spec `07_qa_checks.md`. The checks driven directly from Section 2
of this document (schema identifier, core sections, cascade) MUST be part
of the meta validator's manifest-parse stage; the registry, schema, and
delegation checks MUST be part of the corresponding stages defined in their
sibling specs.

---

## 11. Error Codes

Error codes for the slug registry, contribution schema, validation
delegation, substandard nesting, install seam, and QA checks live in their
respective sibling specs. The codes below are CF01-owned for the manifest
parsing and core sections.

### 11.1 Manifest parsing and core sections (this document)

| Code | Severity | Rule |
|------|----------|------|
| `CF_APSS_VERSION_MISMATCH` | Error | Child `project.apss_version` differs from root. |
| `CF_DUPLICATE_STANDARD_ID` | Error | Two slugs reference the same standard ID. |
| `CF_EMPTY_STANDARDS` | Warning | No standards declared in the manifest. |
| `CF_EMPTY_WORKSPACE_GLOB` | Warning | A workspace member glob matched no directory. |
| `CF_INVALID_APSS_VERSION` | Error | `project.apss_version` is not a supported value. |
| `CF_INVALID_STANDARD_ID` | Error | Entry `id` does not match `APS-V1-\d{4}` or `EXP-V1-\d{4}`. |
| `CF_INVALID_SUBSTANDARD_CODE` | Error | Substandard code does not match `[A-Z]{2}\d{2}`. |
| `CF_INVALID_VERSION_REQ` | Error | `version` is not a valid SemVer requirement. |
| `CF_LEGACY_APSS_CONFIG_TOML` | Error | Legacy `.apss/config.toml` present; see Section 9. |
| `CF_LEGACY_APSS_TOML` | Error | Legacy `apss.toml` present; see Section 9. |
| `CF_MISSING_PROJECT_NAME` | Error | `project.name` missing or empty. |
| `CF_MISSING_SCHEMA` | Error | Top-level `schema` not equal to `apss.project/v1`. |
| `CF_MISSING_STANDARD_ID` | Error | Standards entry without `id`. |
| `CF_MISSING_VERSION_REQ` | Error | Standards entry without `version`. |
| `CF_SCHEMA_MISMATCH` | Error | Cascade child uses a different `schema` than the root. |
| `CF_UNKNOWN_TOP_LEVEL_KEY` | Error | Top-level key is neither a core key nor a registered slug. |
| `CF_VERSION_RANGE_CONFLICT` | Error | Empty intersection between root and child version ranges. |
| `CF_WORKSPACE_IN_CHILD` | Error | `workspace` key present in a child manifest. |
| `CF_EXPERIMENT_DECLARED` | Warning | Manifest declares an experimental standard; enforcement is enabled by explicit opt-in. |
| `CF_EXPERIMENT_PROMOTED` | Warning | Manifest declares an experiment that DI01 resolved through a promoted-standard alias. |

<!-- Codes for the slug registry, contribution schema, validation
delegation, substandard nesting, install seam, and QA checks are owned
by their sibling specs. -->
