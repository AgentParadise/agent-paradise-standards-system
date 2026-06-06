---
name: "Documentation Standard Specification"
description: "Normative rules for documentation structure, the doc type registry, indexing, and the install hook contract"
---

# EXP-V1-0004 - Documentation and Context Engineering (Canonical Specification)

**Version**: 0.1.0
**Status**: Experimental
**Category**: Governance

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

When this standard says a rule is "default-on, switchable-off", it means the rule is part of the standard, applied unconditionally unless a specific `disable` flag in `APSS.yaml` turns it off for that one project. Defaults are opinionated. Configuration is by exception, not by accumulation of optional flags.

---

## 1. Scope and Authority

This standard defines a generic frontmatter-driven indexing and
progressive disclosure mechanism for a project's technical documentation
directory, together with the install contract for the tooling that
enforces it.

The primary purpose is the generic mechanism. Every Markdown file under
the docs root carries YAML frontmatter; every directory README carries
a `## Index` table generated from that frontmatter (Section 4). An
agent or human reads the index first and descends only into the
documents whose `description` says they should. The docs tree becomes a
skill manifest, shallow on the outside, deep where it needs to be.

Concrete doc types are instances of the generic mechanism. Each is
defined as a substandard that lives under `substandards/`, inherits the
parent's frontmatter and indexing rules, and adds its own structure
rules and diagnostic codes. The shipped registry is in Section 8.
Adding a new doc type does not change this spec.

The unlocks layered on top of the generic mechanism are:

1. **Frontmatter driven indexing (Section 4).** Validated structure is
   the prerequisite for semantic lookups, progressive disclosure,
   vectorize any directory pipelines, and every other tool that wants
   to operate on docs as data.
2. **A configurable, growing doc type registry (Section 8).** Doc
   types are default on; a project disables one by flipping a single
   flag in `APSS.yaml`.
3. **Installable enforcement (Section 9).** Installing the standard
   installs a git pre-commit hook that auto-updates the doc index,
   validates structure against the config, and fails the commit when
   the structure is inaccurate or incomplete.

The documentation root defaults to `docs/`. The base parent standard
enforces:

- **DOC02**: README index, frontmatter, and per-directory AI context files.
- **DOC03**: Root-level context files so agents always find docs from a fresh start.

Doc-type specific rules (ADRs, the North Star, Retrospectives, future
types) live in substandards. The current registry is defined in
Section 8.

### 1.1 Relationship to APS-V1-0000 and the unified APSS config

This standard plugs into the unified APSS configuration model owned by the meta-standard APS-V1-0000 (via its CF01 substandard). Project-level configuration for every APSS standard lives in a single file at the repository root, `APSS.yaml`, whose top-level structure and slug registry are owned by CF01. Each standard registers a unique short slug and contributes a config-section schema; the meta-validator aggregates and delegates validation of each namespaced section to its owner.

This standard:

1. Registers the slug `docs`.
2. Contributes the schema for the `docs` section of `APSS.yaml` (Section 3 below).
3. Validates its own section: the parent validator validates the `docs` block and its core sub-blocks (`index`, `context_files`, `readme`, `root_context`, `backlinking`); each substandard validates its own nested key (`adr`, `north-star`, `retrospectives`).

Substandards do NOT register their own top-level slugs. They nest under the `docs` key as namespaced sub-sections (`docs.adr`, `docs.north-star`, `docs.retrospectives`); the nesting convention is normative and is owned by the meta-standard.

The `.apss/` dotdir, when it exists, holds GENERATED artifacts (such as cached indexes and validator state) only. It MUST NOT hold configuration. Earlier drafts of this standard placed configuration in the `.apss` tree; that layout is superseded by `APSS.yaml` at the repository root. Tooling MUST NOT continue to read legacy `.apss` project configuration.

This standard complements APS-V1-0000's requirement for a per-package `docs/01_spec.md` by enforcing broader documentation structure across the project's docs root, beyond each standard package's own spec file.

---

## 2. Core Definitions

- **Front matter**: A YAML block delimited by `---` at the top of a Markdown file. The opening and closing delimiters MUST each appear on their own line; horizontal rules (`---` followed by blank lines and prose, or `----` of any length) are not front matter.
- **Index**: An auto-generated `## Index` section in `README.md` listing the documents in that directory with selected frontmatter fields rendered as table columns.
- **Context file**: `CLAUDE.md` or `AGENTS.md`, one per directory, providing AI agents with lightweight orientation to that directory.
- **Docs root**: The project's technical documentation directory. Default `docs/`, configurable via `docs.root`.
- **Doc type**: A class of document with its own structure rules (ADR, North Star, Retrospective, ...). Each doc type is implemented as a substandard.
- **Doc type registry**: The set of `docs.<type>` keys in `APSS.yaml` that declare which doc types are active in a given project. See Section 8.
- **Backlink**: A reference from an implementation file to the governing doc (ADR, North Star, ...) that it implements. Backlinking is part of every doc type, not a per type opt in. See Section 7.

---

## 3. Configuration

### 3.1 Config Location

Project-level configuration MUST be located at `APSS.yaml` relative to the repository root. The file is owned by the meta-standard (APS-V1-0000.CF01); this standard registers and contributes the `docs` section.

Configuration MUST NOT be placed under `.apss/`. The `.apss/` dotdir is reserved for GENERATED artifacts (cached indexes, validator state) only.

Monorepo cascade: a nested `APSS.yaml` inside a sub-package layers over the root file using the meta-standard's cascade rules (nearer file overrides root values). Cascade resolution is owned by CF01; this standard inherits whatever the meta-validator produces and validates the merged `docs` block.

### 3.2 Default Behavior (absence equals enabled)

The standard follows an absence-equals-enabled convention modelled on
environment variables. If `APSS.yaml` does not exist, or it exists but
contains no `docs` key, or a `docs` block exists but a given
sub-section is absent, the validator MUST apply the documented
defaults for the absent surface. The validator MUST NOT error on a
missing config file, a missing `docs` section, or a missing
sub-section. Zero-config works; every flag defaults to the recommended
setting and every feature defaults to enabled.

A key only appears in `APSS.yaml` to do one of two things:

1. Opt OUT of a default-on rule with `disable: true`.
2. Override a non-`disable` default value (for example, change
   `docs.adr.directory` from `adrs` to `architecture/decisions`).

A `disable: false` line is therefore never the right thing to write
into a real or example config: it is the default that the validator
already applies for absence. Tooling and documentation MUST NOT
generate `disable: false` boilerplate, and operators reading examples
MUST be shown the empty-section happy path, not a `disable: false`
crutch. The smallest valid `APSS.yaml` for a project that adopts every
default of this standard is:

```yaml
docs: {}
```

or simply no `APSS.yaml` at all (CF01 owns whether the file is
required for other reasons).

### 3.3 Schema

The schema is normative. **Scalar fields not listed here under a KNOWN nested section MUST be rejected with `unknown-config-field`.** Unknown nested keys under `docs` (for example `docs.<some-future-slug>`) are tolerated per Section 3.4 forward-compatibility. The schema below shows the `docs` block as it appears inside `APSS.yaml`. Every line below is a default that the validator applies for absence; per Section 3.2 a project only writes a key to opt out (`disable: true`) or to override a non-`disable` value. The surrounding top-level structure (schema declaration, project identity, standard activation) is owned by CF01.

**Path resolution (normative).** Every doc-type location key in this
schema is **docs-root-relative**: the validator resolves it as
`<docs.root>/<location-value>` (or `<docs.root>/<directory-value>`).
The schema example below shows the resolved-when-defaults-apply value
inline as a comment so an adopter scanning the file sees both the
literal value and where it lands. A doc-type config value that
starts with `/` is a hard error (`docs-absolute-location`); a value
that escapes the docs root via `..` segments is also rejected
(`docs-location-out-of-tree`). Substandards inherit this convention
and MUST NOT define their own path semantics.

```yaml
docs:
  # Omit a field to use the default for that surface.
  root: docs                      # Documentation root directory

  index:
    auto_generate: true           # Allow the CLI / hook to (re)write indexes
    frontmatter_fields:           # Columns rendered in index tables
      - name
      - description

  context_files:
    require_claude_md: true       # Require CLAUDE.md per docs directory
    require_agents_md: true       # Require AGENTS.md per docs directory

  readme:
    max_depth: -1                 # -1 means unlimited depth
    exclude_dirs:
      - node_modules
      - .git
      - target
      - vendor
      - .topology

  root_context:
    docs_reference_pattern: docs/ # Pattern checked in root CLAUDE.md / AGENTS.md

  backlinking:
    # Backlinking applies to every doc type when not disabled.
    #
    # The `scan` key is the canonical way to control which files the
    # reference validator walks. It is a list of include-globs evaluated
    # from the repository root. When the key is absent the validator
    # MUST apply the defaults below (absence equals enabled). An
    # explicit list overrides the defaults completely; combine entries
    # to widen the scope.
    scan:
      - "**/*.rs"
      - "**/*.py"
      - "**/*.ts"
      - "**/*.tsx"
      - "**/*.js"
      - "**/*.jsx"
      - "**/*.go"
      - "**/*.java"
      - "**/*.kt"
      - "**/*.rb"
      - "**/*.sh"
      - "**/*.yaml"
      - "**/*.yml"
      - "**/*.toml"
      - "**/*.json"
      - "**/*.md"
    # `file_types` is a deprecated alias. When set, each entry `X` is
    # treated as the glob `**/*.X` and unioned with `scan`. New projects
    # MUST use `scan`; tooling MUST surface a `backlinking-file-types-deprecated`
    # warning when `file_types` is present.

  # Doc type registry (substandards). Each `docs.<slug>` key is default
  # on. To opt a single doc type out, write `docs.<slug>.disable: true`;
  # otherwise the empty section is the happy path. Substandard keys use
  # the substandard's kebab-case slug (matches `substandard.toml`).

  adr:
    directory: adrs               # docs-root-relative; resolves to <docs.root>/adrs
    naming_pattern: "ADR-\\d{3,5}-[a-zA-Z0-9-]+\\.md"
    required_adr_keywords: []

  north-star:
    # Disable with docs.north-star.disable: true
    location: north-star.md       # docs-root-relative; resolves to <docs.root>/north-star.md

  retrospectives:
    directory: retrospectives     # docs-root-relative; resolves to <docs.root>/retrospectives
    naming_pattern: "RETRO-\\d{3,5}-[a-zA-Z0-9-]+\\.md"
```

### 3.4 Configurability rules

- Every rule listed in this spec is on by default. A project disables one rule by setting `disable: true` in the smallest scope that contains it (a single nested key under `docs`, or the top-level `docs.disable` to disable all doc validation).
- `disable: false` is the default the validator applies for absence and MUST NOT be written into real or example configs. Examples in this spec and in `examples/APSS.yaml` MUST show empty sections (or no section at all) for surfaces a project does not override.
- There MUST NOT be per feature `optional` flags scattered through the spec. The shape is always: an implicit default-on with `disable: true` as the opt-out, plus that section's content.
- Adding a new doc type does not require changing this spec. A new substandard MAY claim its own `docs.<slug>` key; the parent standard MUST tolerate unknown `docs.<slug>` keys for forward compatibility, even though it MUST reject unknown scalar fields inside known sections.
- Substandard keys use the substandard's kebab-case slug (for example `north-star`, not `north_star`). Scalar field names inside each section remain snake_case to match the Rust struct contract.

### 3.5 Loading and validation of the config file itself

The CLI and hook MUST emit a single human-readable diagnostic, never a panic, when the config file is malformed:

- `invalid-apss-yaml`: `APSS.yaml` is not valid YAML. Severity: error.
- `unknown-config-field`: a known section under `docs` contains an unknown scalar field. Severity: error.

Both diagnostics MUST include the file path, the offending field or token, and a one-line hint.

The parent meta-validator (CF01) is responsible for top-level structural diagnostics (missing required core sections, unknown top-level sections, slug registry violations); this standard's validator owns diagnostics scoped to the `docs` section.

---

## 4. Frontmatter and Indexing

Section 4 specifies the standard's primary mechanism: a contract on
every Markdown file under the docs root and a deterministic index
generator that reads those files. The contract is what makes
progressive disclosure possible: the index carries each file's
`description`, so an agent reads the index once and opens only the
files whose descriptions tell it to.

### 4.1 Frontmatter Requirement

Every `.md` file under the docs root MUST contain a YAML front matter block with at least the fields listed in `docs.index.frontmatter_fields` (default: `name` and `description`).

```yaml
---
name: "API Authentication Guide"
description: "How authentication works across all service boundaries"
---
```

Front matter is the load-bearing contract for the rest of the standard.
The `description` field MUST be a single line that lets a reader decide
whether to open the file without opening it. Index generation, agent
context loading, search, vectorisation, and every other downstream
consumer reads these fields directly.

The validator MUST treat any `.md` file under the docs root that lacks
the required fields as a `frontmatter-field-missing` (warning) or
`frontmatter-missing` (warning) finding. A file with an opened but
unclosed frontmatter block MUST be reported as `frontmatter-unclosed`
(error). The diagnostic table is in Section 4.4.

Parsing rules:

- The opening delimiter MUST be a line equal to `---` (followed by `\n` or `\r\n`).
- The closing delimiter MUST be a line equal to `---`.
- A line equal to `----` or longer is a horizontal rule, not a front matter delimiter.
- CRLF and LF line endings MUST both be accepted.

### 4.2 Index Generation

When `docs.index.disable` is `false`, every directory `README.md` under
the docs root MUST contain a `## Index` section. The index is a
Markdown table auto-generated from the front matter of `.md` files in
the same directory. This is the row a reader reads first; opening the
underlying file is the second step in progressive disclosure.

```markdown
## Index

| Document | Description |
|----------|-------------|
| [API Authentication Guide](api-auth.md) | How authentication works across all service boundaries |
| [Deployment Runbook](deployment.md) | Step-by-step production deployment procedure |
```

Rendering rules:

- Columns are derived from `docs.index.frontmatter_fields`. The first field is rendered as the document link text. Every remaining field MUST become its own column, populated from that file's front matter. Empty cells MUST be rendered for missing fields but MUST NOT silently fall back to a different field.
- The `## Index` heading MUST be matched as a whole heading line. Substring matches like `## Indexing` MUST NOT be treated as the index section.
- Table rows MUST use a single leading pipe and a single trailing pipe. `|| ... |` is not standard Markdown table syntax and MUST NOT be emitted by the generator.
- The replacement region for an existing index runs from the `## Index` heading line up to (but not including) the next heading line of the same or higher level. When the generator rewrites the section, it MUST preserve at least one trailing newline so the following heading does not collide on the same line as the table.

### 4.3 Index Auto-Generation

When `docs.index.auto_generate` is `true`, the CLI and the install hook MAY write indexes directly into `README.md` files. The dry run and write paths MUST produce identical content for the same input directory:

```bash
aps run docs index [path]          # Dry run: print the indexes that would be written
aps run docs index [path] --write  # Write indexes into README.md files
```

The generator MUST traverse only file entries; a directory named `something.md` MUST NOT be treated as a document.

When a directory contains no indexable files, the generator MUST emit the same placeholder both in dry run and write mode (default placeholder: a `## Index` heading with the body `_No indexable documents in this directory yet._`).

### 4.4 Diagnostic codes for indexing

| Code | Severity | Description |
|------|----------|-------------|
| `index-missing` | warning | Directory README is missing the `## Index` section. |
| `index-stale` | warning | `## Index` content does not match what the generator would write. |
| `index-malformed-row` | warning | Row uses non-standard syntax (e.g., `\|\| ...`) or has the wrong column count. |
| `frontmatter-missing` | warning | A `.md` file lacks a frontmatter block. |
| `frontmatter-unclosed` | error | A frontmatter block has an opening `---` but no closing delimiter. |
| `frontmatter-field-missing` | warning | A required frontmatter field (per `docs.index.frontmatter_fields`) is absent. |

---

## 5. README and Context Files (DOC02)

### 5.1 DOC02-readme-required

Every directory under the docs root (respecting `max_depth` and `exclude_dirs`) MUST contain a `README.md` file.

Diagnostic: `readme-missing` (error).

### 5.2 DOC02-context-files

Directories under the docs root MUST contain `AGENTS.md` and an
adjacent `CLAUDE.md`. `AGENTS.md` is the canonical agent context file
and carries the orientation prose; `CLAUDE.md` is a symlink to the
adjacent `AGENTS.md` (on filesystems that do not support symlinks, a
verbatim copy of the `AGENTS.md` content). The standard ships no
`GEMINI.md`; Gemini reads `AGENTS.md` natively.

A minimal `AGENTS.md` for a docs subdirectory:

```markdown
---
name: "<directory name>"
description: "AI context for <directory name>"
---

See [README.md](README.md) for the index and overview of this directory.
```

The validator MUST check existence only. An `AGENTS.md` that already
exists with project-specific content passes validation as long as its
frontmatter is well-formed per Section 4; the validator MUST NOT
compare on-disk content against any shipped template. The installer's
template-conflict warning (see `02_install_contract.md` Section 1.5)
is the surface for content drift.

Substandards MAY ship a templated `AGENTS.md` for their own docs-area
directories (for example ADR01 ships `docs/adrs/AGENTS.md` with ADR
context and a `CLAUDE.md` symlink); the install contract's
create-if-missing, never-overwrite rule applies in full.

Diagnostics: `agents-md-missing` (warning), `claude-md-missing`
(warning).

---

## 6. Root Context Files (DOC03)

### 6.1 DOC03-root-claude-md

The repository root MUST contain `CLAUDE.md`. Diagnostic:
`root-claude-md-missing` (**warning**, downgraded from error).

The downgrade is deliberate. Per the operator's earlier Correction 2
the standard does not own the root context file's content (the
root file is project-specific and the installer MUST NOT scaffold it,
per `02_install_contract.md` Section 1.5). Emitting `error` on a
file the installer is not allowed to create produces a hard block on
the operator's first commit, which contradicts the install
contract. The standard records the missing file as a warning so
adopters see it in CI without being blocked, and the install step 5
banner names the missing root files explicitly so they are not
silently skipped.

### 6.2 DOC03-root-agents-md

The repository root MUST contain `AGENTS.md`. Diagnostic:
`root-agents-md-missing` (**warning**, downgraded from error, for
the same reason documented in Section 6.1).

### 6.3 DOC03-self-reference

The root `CLAUDE.md` and root `AGENTS.md` MUST reference:

1. The Agent Paradise Standards System and where the standard package lives in this repository.
2. The docs root and, for each active doc type, the directory or file that holds that doc type (for example, the ADR directory).
3. The rule that implementation code files MUST backlink the docs they implement (see Section 7).

Diagnostic: `root-self-reference-missing` (warning). The validator MUST check for the presence of:

- The literal token `APSS` or the phrase `Agent Paradise Standards System`.
- The docs root path (matching `docs.root_context.docs_reference_pattern`).
- Each active doc type's location (`docs.adr.directory`, `docs.north-star.location`, ...).

### 6.4 DOC03-skills-format

`CLAUDE.md`, `AGENTS.md`, and any `agents/skills/*/README.md` files SHOULD follow the Claude Code skills format documented at <https://code.claude.com/docs/en/skills.md>:

- Front matter at the top.
- A short, single paragraph "what this skill does" body.
- Links to the spec for any prose longer than a paragraph. Keep skill READMEs DRY; do not duplicate spec prose.

Diagnostic: `skills-format-violation` (warning).

---

## 7. Backlinking (always part of the standard)

Backlinking is a load-bearing invariant for every doc type, not an opt in feature. The motivation is context preservation across the plan, design, and implementation phases: when an agent or developer opens a source file, the governing decision must be immediately discoverable.

### 7.1 The backlinking rule

For every active doc type, an implementation file that is governed by a specific doc MUST contain a token of the form `<DOC-TYPE-ID>-<NUMBER>-<NAME>` somewhere in the file. The token MUST appear inside a comment in the file's source language so it never affects code semantics.

Examples:

```rust
// Implements ADR-001-security-architecture
```

```python
# Implements PV-001-product-purpose, RETRO-007-q1-launch
```

### 7.1.1 Placement guidance (normative)

The validator picks up backlink tokens regardless of where they appear in the file. Authors and agents SHOULD follow this placement convention so the backlink is discoverable by a reader skimming the file:

- **Top of file (PREFERRED).** When the file as a whole exists to satisfy one or more decisions, place a short header comment near the top giving the file's purpose in one line and listing the ADR (or other doc-type) identifiers it implements. This is the right shape for a module, a binary entry point, or a single-purpose source file.
- **Above a specific function or code block (ALSO ALLOWED).** When only a single function, struct, or contiguous block of code is governed by an ADR while the rest of the file is unrelated, place the backlink comment directly above the unit it scopes to. The validator treats this identically to a top-of-file backlink for the purpose of accuracy checking (Section 7.2).

Both placements satisfy the rule. A file MAY combine both: a top-of-file backlink for the file's overall purpose and additional per-function backlinks for ADRs that govern individual units. The validator MUST NOT prefer one placement over the other; it scans the whole file.

```rust
// Top-of-file placement: this whole file implements two ADRs.
// Implements ADR-001-security-architecture, ADR-014-token-storage.

fn login(...) { ... }
fn logout(...) { ... }
```

```rust
// Per-function placement: only this one function is governed by the ADR;
// the rest of the file is unrelated background work.

fn unrelated_helper(...) { ... }

// Implements ADR-027-rate-limit-burst.
fn enforce_rate_limit(...) {
    ...
}
```

The agent-context template at `docs/adrs/AGENTS.md` (shipped by the ADR01 substandard) MUST repeat this guidance so the convention is on hand whenever an agent opens the ADR directory.

#### 7.1.2 Word-boundary rule (normative)

A backlink token MUST be surrounded by characters that are NOT
alphanumeric and NOT `-` (hyphen): whitespace, punctuation, line
starts and line ends all satisfy the rule. The reference validator
MUST treat the alphanumeric-or-hyphen class as a single greedy run
and MUST anchor token extraction with word-boundary equivalents
(`\b` in regex terms) on BOTH sides so:

- `BADR-001-foo` does NOT extract `ADR-001-foo` (left boundary
  prevents an embedded match).
- `ADR-001-foo-ADR-002-bar` extracts TWO tokens (`ADR-001-foo` and
  `ADR-002-bar`) rather than the merged `ADR-001-foo-ADR-002-bar`.
- `ADR-001-foo-` (trailing hyphen, common in markdown bullets)
  extracts `ADR-001-foo` (trailing `-` MUST be stripped before
  lookup, OR the slug pattern MUST end with `[a-zA-Z0-9]`).

The validator's emitted diagnostic for a non-resolving token MUST
quote the token verbatim so a reader can tell whether a false
positive came from a malformed boundary; the spec language above is
the contract the validator implements.

Authors writing prose that mentions an ADR identifier inside a
hyphenated sentence (for example "supersedes ADR-001-foo-and-also")
MUST surround the identifier with whitespace or punctuation; the
validator does NOT attempt to disambiguate prose hyphens from slug
hyphens.

### 7.2 Reference accuracy

The validator MUST scan source files in the repository (respecting `docs.backlinking.scan` and, for backward compatibility, the deprecated `docs.backlinking.file_types`) and validate every backlink token it finds against the corresponding doc type's on-disk state.

The `scan` key is a list of include-globs evaluated from the repository root. When the key is absent the validator MUST apply the defaults documented in Section 3.3 (absence equals enabled). When the key is set, the explicit list overrides the defaults entirely; a project widens the surface by combining entries. The validator MUST always exclude hidden directories and the project's `docs.readme.exclude_dirs`, regardless of how `scan` is configured. The configured doc type directories are also excluded from the scan so an ADR's own body or a retrospective's text can reference other ADRs without tripping the validator.

The deprecated `docs.backlinking.file_types` key MAY still appear in older configs; when present, each entry `X` MUST be treated as the glob `**/*.X` and unioned with `scan`. A project that sets `file_types` MUST receive a `backlinking-file-types-deprecated` warning so the operator migrates to `scan`. Tooling MUST NOT emit this warning when only `scan` is present.

The generic parent-level diagnostics are:

- `backlink-dead-reference` (warning) when a code file references a doc identifier that does not exist in the corresponding doc type directory and the doc type does not ship a stricter substandard-specific accuracy code.
- `backlink-superseded-reference` (warning) when a code file references a doc whose status is `deprecated` or `superseded`.

The reference extraction regex MUST be derived from each doc type's configured `naming_pattern` (so a project that customizes its ADR naming pattern still gets accurate dead reference detection).

#### 7.2.1 Substandard tightening (normative)

A substandard MAY tighten accuracy to error severity by emitting its own code. ADR01 (Section 6 of `substandards/ADR01-architecture-decision-records/docs/01_spec.md`) defines `ADR01-unknown-reference` (error) for any code-side `ADR-NNN-...` token (3 to 5 digit number) that does not resolve to a real ADR file in `docs.adr.directory` matching the configured `docs.adr.naming_pattern`. When a substandard's strict code applies to a finding, the validator MUST emit the substandard code and MUST NOT also emit the generic warning. Substandards that do not ship a strict accuracy code inherit the generic warning above.

#### 7.2.2 Diagnostic contents

Every accuracy diagnostic MUST include the source file path, the line number where the bad reference appears, and the offending token verbatim. A diagnostic without file and line is non-actionable and MUST NOT be emitted by the validator.

### 7.3 Disabling

Backlinking is enabled by default. A project that needs to disable it sets `docs.backlinking.disable = true`. Per doc type backlinking toggles are not supported by design: backlinking is either on for the project or off for the project. When backlinking is disabled, both the generic parent diagnostics and every substandard accuracy code MUST be suppressed.

### 7.4 Generator side requirements

The standard does not require code files to be auto generated with backlinks. It requires that the validator emit a diagnostic when a backlink is missing, dead, or unresolved. Adding the backlink line is the implementer's responsibility (and a good fit for a planning agent's checklist).

---

## 8. Doc Type Registry

The parent standard defines the doc type registry. Each doc type is implemented as a substandard under `substandards/`. The shipped doc types are:

| Doc type | Substandard | Default location (resolved with default `docs.root: docs`) | Config key in `APSS.yaml` |
|----------|-------------|------------------|---------------------------|
| Architecture Decision Records | `EXP-V1-0004.ADR01` | `docs/adrs/` (literal config value: `adrs`) | `docs.adr` |
| North Star (Mission, Vision, Position) | `EXP-V1-0004.PV01` | `docs/north-star.md` (literal config value: `north-star.md`) | `docs.north-star` |
| Retrospectives | `EXP-V1-0004.RETRO01` | `docs/retrospectives/` (literal config value: `retrospectives`) | `docs.retrospectives` |

All location values are docs-root-relative per Section 3.3 normative
path resolution. A literal value that starts with `/` or escapes
`docs.root` with `..` segments MUST be rejected.

### 8.1 Lifecycle status (shared, with per-doc-type table)

Doc types that have lifecycle status share the four-value
vocabulary so tooling can be uniform across types:

- `proposed`: under discussion, not yet adopted.
- `accepted` / `active`: current source of truth. The slash means
  **each doc type picks one** (see the per-doc-type table below);
  it does NOT mean both are interchangeable for the same doc type.
- `deprecated`: discouraged but still informative.
- `superseded`: replaced by another doc of the same type; the
  front matter MUST include `superseded_by: <doc-id>`.

Per-doc-type vocabulary (normative):

| Doc type | "Proposed" | "In force" | "Discouraged but informative" | "Replaced" |
|----------|-----------|-----------|-------------------------------|------------|
| ADR (`ADR01`)              | `proposed` | `accepted` | `deprecated` | `superseded` |
| North Star (`PV01`)         | `proposed` | `active`   | `deprecated` | `superseded` |
| Retrospectives (`RETRO01`)  | `proposed` | `active`   | `deprecated` | `superseded` |

ADRs adopt `accepted` to match the Nygard tradition the substandard
cites; PV01 and RETRO01 adopt `active` because the North Star and
retrospectives describe an ongoing reality rather than a discrete
decision moment. A substandard's validator MUST emit
`<SUBSTANDARD-ID>-invalid-status` (error) when a document uses the
wrong term for its type, with a hint pointing at this table.

ADRs are never revised; they are superseded. Retrospectives are
append only. North Star documents follow the same status field but
typically remain `active` for long stretches.

### 8.2 Adding a new doc type

A new doc type is added by:

1. Creating a substandard under `substandards/<ID>-<slug>/`.
2. Documenting its nested key under `docs` in `APSS.yaml`, using the substandard's kebab-case slug (so `docs.<slug>`). Per Section 3.2 the validator MUST treat the absence of `docs.<slug>` as default-on; the substandard's spec MUST NOT instruct projects to write `disable: false`. Any non-`disable` fields are owned by the substandard. Substandards do NOT register their own top-level slug in the meta-standard registry.
3. Registering the doc type in this section's table.
4. Defining the substandard's diagnostic codes using the human readable scheme described in Section 10.

The parent standard MUST NOT hard code the list of doc types in code paths that would break when a new doc type is added. Validators MUST iterate the registry, not enumerate types by name.

### 8.3 Substandard summaries

- **EXP-V1-0004.ADR01 (Architecture Decision Records).** Spec: [`substandards/ADR01-architecture-decision-records/docs/01_spec.md`](../substandards/ADR01-architecture-decision-records/docs/01_spec.md). Validates naming, frontmatter (including `status`), required topic keywords, header conventions, and per directory context files. Backlinking and dead reference detection use the shared rules in Section 7.
- **EXP-V1-0004.PV01 (North Star: Mission, Vision, Position).** Spec: [`substandards/PV01-purpose-and-vision/docs/01_spec.md`](../substandards/PV01-purpose-and-vision/docs/01_spec.md). Validates the presence and structure of the project's single North Star document, used by agents during plan and design to stay aligned with the project's intent.
- **EXP-V1-0004.RETRO01 (Retrospectives).** Spec: [`substandards/RETRO01-retrospectives/docs/01_spec.md`](../substandards/RETRO01-retrospectives/docs/01_spec.md). Validates the retrospective directory, append only history, naming, and required sections.

---

## 9. Install Contract (hook + validator + index)

This section is normative. Installing this standard into a project installs three coordinated pieces: an index updater, a validator, and a git pre-commit hook that drives both. The working installer ships as a follow up; this spec is what that installer MUST implement. For the full contract in one place, see [`docs/02_install_contract.md`](02_install_contract.md).

### 9.1 Install entry point

```bash
aps run docs install [<repo-root>]
aps run docs uninstall [<repo-root>]
```

Behavior:

- `install` MUST:
  1. If `APSS.yaml` does not exist, ask the meta-standard's installer (CF01) to create it with the project's selected standards. If `APSS.yaml` exists, MUST NOT overwrite it; only add a `docs:` block if missing, leaving every other section untouched. The added `docs:` block uses the documented defaults from Section 3.3.
  2. Install the git pre-commit hook described in Section 9.4. If a pre-commit hook already exists, MUST insert an `apss-docs-hook` block delimited by sentinel comments rather than replace the user's hook.
  3. Print the resolved doc type registry so the operator sees which doc types just became active.
- `uninstall` MUST remove only the `apss-docs-hook` block from the pre-commit hook and MUST leave `APSS.yaml` (and its `docs:` block) and the rest of the hook intact.
- Both commands MUST be idempotent.

### 9.2 Validator contract

The validator is the source of truth. The hook and the standalone CLI MUST call the same validator entry point so behavior is identical.

**Inputs**:

- `repo_root: Path`: absolute path to the repository root.
- `config: ApssConfig`: the merged `docs` block from `APSS.yaml` (after CF01 cascade resolution), with defaults applied for any missing fields.
- `scope: ValidationScope`: one of:
  - `Full`: walk the entire docs root and every doc type directory.
  - `Changed { staged_paths: Vec<PathBuf> }`: only inspect docs touched by the staged change set; the hook MUST use this scope.

**Outputs**: a `ValidationReport` with:

- `diagnostics: Vec<Diagnostic>`: every diagnostic has `code`, `severity`, `path`, `line` (optional), `message`, and `hint`.
- `summary: { errors: u32, warnings: u32 }`.
- `machine_readable: Json`: the same content rendered as stable JSON for CI consumers. JSON keys MUST be the human readable diagnostic codes.

**Exit behavior**:

- `aps run docs validate` MUST exit `0` only when `summary.errors == 0`.
- The hook MUST refuse the commit when `summary.errors > 0`. Warnings MUST be printed but MUST NOT block the commit.
- An internal failure (panic, IO error, regex compile failure on a built in pattern) MUST be reported as the synthetic diagnostic `validator-internal-error` with severity `error` and MUST block the commit. The validator MUST NOT swallow internal errors silently.

### 9.3 Index generator contract

The index generator and the validator MUST agree:

- The validator's `index-stale` diagnostic MUST be true if and only if running the generator over the same directory with the same config would change the file.
- The generator MUST be deterministic: same inputs, byte identical output.
- Dry run output MUST be byte identical to what `--write` would produce, including trailing newlines.

**Inputs**: `repo_root`, `config`, and a list of directories to refresh.

**Outputs**:

- `dry_run` mode: a list of `(path, new_content)` pairs printed to stdout.
- `write` mode: each `README.md` rewritten in place, returning the same list of pairs.

**Exit behavior**:

- `aps run docs index --write` MUST exit `0` after a successful write, even if it made no changes.
- Failure to write any individual file MUST emit `index-write-failed` and MUST exit non zero.

### 9.4 Git pre-commit hook contract

The installed hook is a small shell wrapper that calls into `aps run docs hook --staged`. The hook MUST:

1. Resolve the repository root and the staged file list (`git diff --cached --name-only --diff-filter=ACMR`).
2. Refresh indexes for any docs directory whose contents changed in the staged set, by calling the index generator in `--write` mode. The hook MUST re-stage rewritten `README.md` files (`git add`) so the commit is self consistent.
3. Run the validator with `scope = Changed { staged_paths }`.
4. Exit non zero (and print every error diagnostic) when the validator reports errors. The commit is blocked.
5. Print warnings but allow the commit.

**Inputs/outputs**:

- `STDIN`: none.
- `STDOUT`: human readable report (color when TTY).
- `STDERR`: diagnostics on failure.
- `Exit codes`: `0` on success, `1` on validation errors, `2` on internal hook errors (config load failure, missing `aps` binary, ...). The hook MUST NOT exit `0` after re-staging modified files unless the validator also passes.

**Escape hatch**: the operator's standard `git commit --no-verify` continues to work. The standard MUST NOT teach agents to use `--no-verify`; that flag is a human operator escape hatch, not a documented workflow.

**What "valid structure" means per doc type**:

- ADR (`EXP-V1-0004.ADR01`): the ADR directory exists, every non-`.example` file matches the naming pattern, every ADR has the required frontmatter and `status` (per the per-doc-type table in Section 8.1: ADRs use `accepted` for in-force), required topic keywords are satisfied, context files exist with referencing guidance, every `ADR-NNN-<slug>` token found in the file set defined by `docs.backlinking.scan` (defaults in Section 3.3; deprecated `file_types` honoured with a `backlinking-file-types-deprecated` warning) resolves to a real ADR file matching `docs.adr.naming_pattern` (`ADR01-unknown-reference`, error), and resolved references are split between `ADR01-superseded-reference` (warning) for `status: superseded` targets and `ADR01-deprecated-reference` (warning) for `status: deprecated` targets. See the ADR01 spec for the per rule diagnostic codes.
- North Star (`EXP-V1-0004.PV01`): a single `north-star.md` (or configured location) exists with frontmatter, a `## Mission` section, a `## Vision` section, a `## Position` section, and a current `status`. See PV01 spec.
- Retrospectives (`EXP-V1-0004.RETRO01`): the retrospective directory exists, each file matches the naming pattern, files are append only (no historical retros modified in the staged change set), and required sections are present. See RETRO01 spec.

### 9.5 Why the install contract matters

A validated, hook-enforced doc structure is what lets downstream tooling treat docs as structured data: semantic search over frontmatter, progressive disclosure of long specs, vectorize-any-directory pipelines, and AI agents that can rely on docs being syntactically correct at commit time. Without the hook, "the docs are mostly structured" is true until it isn't. With the hook, the structure is a guarantee.

---

## 10. Diagnostic Code Scheme

All diagnostic codes in this standard and its substandards MUST be human readable. Numeric or opaque codes MUST NOT be added.

Format:

- Parent standard: `<area>-<short-name>` (lowercase kebab). Examples: `readme-missing`, `frontmatter-unclosed`, `index-stale`.
- Substandards: `<substandard-id>-<short-name>`. Examples: `ADR01-dir-not-found`, `ADR01-naming-mismatch`, `PV01-missing-vision-section`, `RETRO01-history-modified`.

Existing numeric or composite codes (for example, `ADR01-001`) MAY be retained as aliases during the transition but MUST be supplemented by the human readable form in tool output. New codes MUST be human readable from the start.

### 10.1 Parent standard codes

| Code | Severity | Domain | Description |
|------|----------|--------|-------------|
| `invalid-apss-yaml` | error | Config | `APSS.yaml` is not valid YAML. |
| `unknown-config-field` | error | Config | A known section under `docs` contains an unknown scalar field. |
| `readme-missing` | error | DOC02 | Directory missing `README.md`. |
| `claude-md-missing` | warning | DOC02 | Directory missing `CLAUDE.md`. |
| `agents-md-missing` | warning | DOC02 | Directory missing `AGENTS.md`. |
| `index-missing` | warning | DOC02 | `README.md` missing `## Index` section. |
| `index-stale` | warning | DOC02 | `## Index` content does not match the generator. |
| `index-malformed-row` | warning | DOC02 | Index row uses non-standard syntax. |
| `index-write-failed` | error | DOC02 | Generator could not write `README.md`. |
| `frontmatter-missing` | warning | DOC02 | `.md` file lacks a frontmatter block. |
| `frontmatter-unclosed` | error | DOC02 | Frontmatter block has no closing delimiter. |
| `frontmatter-field-missing` | warning | DOC02 | Required frontmatter field absent. |
| `root-claude-md-missing` | warning | DOC03 | Root missing `CLAUDE.md`. Downgraded from error per Section 6.1: the standard cannot scaffold the root file (Install Contract Section 1.5), so it MUST NOT block the operator's first commit on it. |
| `root-agents-md-missing` | warning | DOC03 | Root missing `AGENTS.md`. Same reasoning as `root-claude-md-missing`. |
| `docs-absolute-location` | error | Config | A doc-type `location` or `directory` value starts with `/`; all doc-type paths are docs-root-relative per Section 3.3. |
| `docs-location-out-of-tree` | error | Config | A doc-type `location` or `directory` value escapes `docs.root` via `..` segments. |
| `root-self-reference-missing` | warning | DOC03 | Root context file missing required APSS, docs, or doc-type references. |
| `skills-format-violation` | warning | DOC03 | Skill README does not follow the Claude Code skills format. |
| `backlink-dead-reference` | warning | Backlink | Code references a doc identifier that does not exist; superseded by a substandard-specific accuracy code when one applies. |
| `backlink-superseded-reference` | warning | Backlink | Code references a `deprecated` or `superseded` doc. |
| `backlinking-file-types-deprecated` | warning | Backlink | `docs.backlinking.file_types` is the deprecated alias for `docs.backlinking.scan`; migrate to `scan`. |
| `validator-internal-error` | error | Tooling | Validator hit an internal error. |

Substandard codes are defined in their own specs.

---

## 11. CLI Interface

```bash
aps run docs install [<repo-root>]                 # Install hook + default config (idempotent)
aps run docs uninstall [<repo-root>]               # Remove hook (config preserved)
aps run docs validate [<path>] [--json]            # Run validator (CI-friendly)
aps run docs index [<path>] [--write]              # Run index generator
aps run docs hook --staged                         # Hook entry point (used by pre-commit)
```

Every command MUST emit the same diagnostics shape as the validator (Section 9.2).

---

## Appendix A: Validation Checklist

- [ ] `APSS.yaml` valid, or absent for defaults; the `docs:` block (if present) parses against Section 3.3.
- [ ] Every docs directory has `README.md` with a valid `## Index` section.
- [ ] `.md` files under the docs root have closed frontmatter with the configured fields.
- [ ] `CLAUDE.md` and `AGENTS.md` present per docs directory.
- [ ] Root `CLAUDE.md` and `AGENTS.md` exist and reference APSS, the docs root, and every active doc type's location.
- [ ] For every active doc type, the substandard's own checks pass.
- [ ] No code file references a missing, deprecated, or superseded doc identifier.
- [ ] The pre-commit hook is installed and refuses commits with errors.
