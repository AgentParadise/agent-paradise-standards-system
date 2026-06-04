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

When this standard says a rule is "default-on, switchable-off", it means the rule is part of the standard, applied unconditionally unless a specific `disable` flag in `apss.yaml` turns it off for that one project. Defaults are opinionated. Configuration is by exception, not by accumulation of optional flags.

---

## 1. Scope and Authority

This standard defines the structure of a project's technical documentation directory and the install contract for the tooling that enforces it. The unlocks are:

1. **Frontmatter driven indexing.** Every Markdown file under the docs root carries YAML front matter, and the standard auto-generates `## Index` tables from that metadata. Validated structure is the prerequisite for semantic lookups, progressive disclosure, vectorize any directory pipelines, and any other tooling that wants to operate on docs as data.
2. **A configurable, growing doc type registry.** Each doc type (ADR, Purpose and Vision, Retrospectives, and future additions) is defined as a substandard that lives under `substandards/` and inherits the parent's frontmatter and index format. Doc types are default on; a project disables a doc type by flipping one flag in `apss.yaml`.
3. **Installable enforcement.** Installing the standard into a project installs git commit hooks that auto-update the doc index, validate structure against the config, and fail the commit when the structure is inaccurate or incomplete. The contract for that hook is normative and is specified in Section 9.

The documentation root defaults to `docs/`. The base parent standard enforces:

- **DOC02**: README index, frontmatter, and per-directory AI context files.
- **DOC03**: Root-level context files so agents always find docs from a fresh start.

Doc-type specific rules (ADR, Purpose and Vision, Retrospectives, future types) live in substandards. The current registry is defined in Section 8.

### 1.1 Relationship to APS-V1-0000 and the unified APSS config

This standard plugs into the unified APSS configuration model owned by the meta-standard APS-V1-0000 (via its CF01 substandard). Project-level configuration for every APSS standard lives in a single file at the repository root, `apss.yaml`, whose top-level structure and slug registry are owned by CF01. Each standard registers a unique short slug and contributes a config-section schema; the meta-validator aggregates and delegates validation of each namespaced section to its owner.

This standard:

1. Registers the slug `docs`.
2. Contributes the schema for the `docs` section of `apss.yaml` (Section 3 below).
3. Validates its own section: the parent validator validates the `docs` block and its core sub-blocks (`index`, `context_files`, `readme`, `root_context`, `backlinking`); each substandard validates its own nested key (`adr`, `purpose-and-vision`, `retrospectives`).

Substandards do NOT register their own top-level slugs. They nest under the `docs` key as namespaced sub-sections (`docs.adr`, `docs.purpose-and-vision`, `docs.retrospectives`); the nesting convention is normative and is owned by the meta-standard.

The `.apss/` dotdir, when it exists, holds GENERATED artifacts (such as cached indexes and validator state) only. It MUST NOT hold configuration. Earlier drafts of this standard placed configuration at `.apss/config.toml`; that location is superseded by `apss.yaml` at the repository root. Tooling MUST NOT continue to read `.apss/config.toml`; presence of that file is not a deprecation alias.

This standard complements APS-V1-0000's requirement for a per-package `docs/01_spec.md` by enforcing broader documentation structure across the project's docs root, beyond each standard package's own spec file.

---

## 2. Core Definitions

- **Front matter**: A YAML block delimited by `---` at the top of a Markdown file. The opening and closing delimiters MUST each appear on their own line; horizontal rules (`---` followed by blank lines and prose, or `----` of any length) are not front matter.
- **Index**: An auto-generated `## Index` section in `README.md` listing the documents in that directory with selected frontmatter fields rendered as table columns.
- **Context file**: `CLAUDE.md` or `AGENTS.md`, one per directory, providing AI agents with lightweight orientation to that directory.
- **Docs root**: The project's technical documentation directory. Default `docs/`, configurable via `docs.root`.
- **Doc type**: A class of document with its own structure rules (ADR, Purpose and Vision, Retrospective, ...). Each doc type is implemented as a substandard.
- **Doc type registry**: The set of `docs.<type>` keys in `apss.yaml` that declare which doc types are active in a given project. See Section 8.
- **Backlink**: A reference from an implementation file to the governing doc (ADR, Purpose and Vision, ...) that it implements. Backlinking is part of every doc type, not a per type opt in. See Section 7.

---

## 3. Configuration

### 3.1 Config Location

Project-level configuration MUST be located at `apss.yaml` relative to the repository root. The file is owned by the meta-standard (APS-V1-0000.CF01); this standard registers and contributes the `docs` section.

Configuration MUST NOT be placed at `.apss/config.toml`. The `.apss/` dotdir is reserved for GENERATED artifacts (cached indexes, validator state) only.

Monorepo cascade: a nested `apss.yaml` inside a sub-package layers over the root file using the meta-standard's cascade rules (nearer file overrides root values). Cascade resolution is owned by CF01; this standard inherits whatever the meta-validator produces and validates the merged `docs` block.

### 3.2 Default Behavior

If `apss.yaml` does not exist, or it exists but contains no `docs` key, the validator MUST apply the documented defaults. The validator MUST NOT error on a missing config file or a missing `docs` section. Zero-config works; every flag defaults to the recommended setting. All features are default on and can only be disabled via explicit configuration.

### 3.3 Schema

The schema is normative. Keys not listed here under the `docs` section MUST be rejected with `unknown-config-field`. The schema below shows the `docs` block as it appears inside `apss.yaml`; the surrounding top-level structure (schema declaration, project identity, standard activation) is owned by CF01.

```yaml
docs:
  disable: false                  # Master kill switch for the whole doc standard
  root: docs                      # Documentation root directory

  index:
    disable: false                # Stop enforcing `## Index` in README.md files
    auto_generate: true           # Allow the CLI / hook to (re)write indexes
    frontmatter_fields:           # Columns rendered in index tables
      - name
      - description

  context_files:
    require_claude_md: true       # Require CLAUDE.md per docs directory
    require_agents_md: true       # Require AGENTS.md per docs directory

  readme:
    disable: false
    max_depth: -1                 # -1 means unlimited depth
    exclude_dirs:
      - node_modules
      - .git
      - target
      - vendor
      - .topology

  root_context:
    disable: false
    docs_reference_pattern: docs/ # Pattern checked in root CLAUDE.md / AGENTS.md

  backlinking:
    disable: false                # Backlinking applies to every doc type when not disabled
    file_types:
      - rs
      - py
      - ts
      - tsx
      - js
      - jsx
      - go
      - java
      - kt
      - rb
      - sh
      - yaml
      - yml
      - toml
      - json
      - md

  # Doc type registry (substandards). Each `docs.<slug>` key opts that doc type into
  # validation. Default on. Substandard specs own the keys below the `disable` line.
  # Substandard keys use the substandard's kebab-case slug (matches `substandard.toml`).

  adr:
    disable: false
    directory: adrs
    naming_pattern: "ADR-\\d{3,5}-[a-zA-Z0-9-]+\\.md"
    required_adr_keywords: []

  purpose-and-vision:
    disable: false
    location: docs/vision.md      # Default file path. See PV01.

  retrospectives:
    disable: false
    directory: docs/retrospectives
    naming_pattern: "RETRO-\\d{3,5}-[a-zA-Z0-9-]+\\.md"
```

### 3.4 Configurability rules

- Every rule listed in this spec is on by default. A project disables one rule by setting `disable: true` in the smallest scope that contains it (a single nested key under `docs`, or the top-level `docs.disable` to disable all doc validation).
- There MUST NOT be per feature `optional` flags scattered through the spec. The shape is always: a `disable` flag at the top of the relevant section, plus that section's content.
- Adding a new doc type does not require changing this spec. A new substandard MAY claim its own `docs.<slug>` key; the parent standard MUST tolerate unknown `docs.<slug>` keys for forward compatibility, even though it MUST reject unknown scalar fields inside known sections.
- Substandard keys use the substandard's kebab-case slug (for example `purpose-and-vision`, not `purpose_and_vision`). Scalar field names inside each section remain snake_case to match the Rust struct contract.

### 3.5 Loading and validation of the config file itself

The CLI and hook MUST emit a single human-readable diagnostic, never a panic, when the config file is malformed:

- `invalid-apss-yaml`: `apss.yaml` is not valid YAML. Severity: error.
- `unknown-config-field`: a known section under `docs` contains an unknown scalar field. Severity: error.

Both diagnostics MUST include the file path, the offending field or token, and a one-line hint.

The parent meta-validator (CF01) is responsible for top-level structural diagnostics (missing required core sections, unknown top-level sections, slug registry violations); this standard's validator owns diagnostics scoped to the `docs` section.

---

## 4. Frontmatter and Indexing

### 4.1 Frontmatter Requirement

Every `.md` file under the docs root MUST contain a YAML front matter block with at least the fields listed in `docs.index.frontmatter_fields` (default: `name` and `description`).

```yaml
---
name: "API Authentication Guide"
description: "How authentication works across all service boundaries"
---
```

Front matter makes each document self-describing. Tooling reads these fields to generate indexes, power search, provide agents with structured context, and feed vectorization pipelines that need stable per-file metadata.

Parsing rules:

- The opening delimiter MUST be a line equal to `---` (followed by `\n` or `\r\n`).
- The closing delimiter MUST be a line equal to `---`.
- A line equal to `----` or longer is a horizontal rule, not a front matter delimiter.
- CRLF and LF line endings MUST both be accepted.

### 4.2 Index Generation

When `docs.index.disable` is `false`, every directory `README.md` under the docs root MUST contain a `## Index` section. The index is a Markdown table auto-generated from the front matter of `.md` files in the same directory:

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

Directories under the docs root MUST contain `CLAUDE.md` and `AGENTS.md`, each with front matter and a short pointer to the directory README:

```markdown
---
name: "<directory name>"
description: "AI context for <directory name>"
---

See [README.md](README.md) for the index and overview of this directory.
```

Diagnostics: `claude-md-missing`, `agents-md-missing` (warning).

---

## 6. Root Context Files (DOC03)

### 6.1 DOC03-root-claude-md

The repository root MUST contain `CLAUDE.md`. Diagnostic: `root-claude-md-missing` (error).

### 6.2 DOC03-root-agents-md

The repository root MUST contain `AGENTS.md`. Diagnostic: `root-agents-md-missing` (error).

### 6.3 DOC03-self-reference

The root `CLAUDE.md` and root `AGENTS.md` MUST reference:

1. The Agent Paradise Standards System and where the standard package lives in this repository.
2. The docs root and, for each active doc type, the directory or file that holds that doc type (for example, the ADR directory).
3. The rule that implementation code files MUST backlink the docs they implement (see Section 7).

Diagnostic: `root-self-reference-missing` (warning). The validator MUST check for the presence of:

- The literal token `APSS` or the phrase `Agent Paradise Standards System`.
- The docs root path (matching `docs.root_context.docs_reference_pattern`).
- Each active doc type's location (`docs.adr.directory`, `docs.purpose-and-vision.location`, ...).

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

For every active doc type, an implementation file that is governed by a specific doc MUST contain a token of the form `<DOC-TYPE-ID>-<NUMBER>-<NAME>` somewhere in the file (typically a single line comment near the top).

Examples:

```rust
// Implements ADR-001-security-architecture
```

```python
# Implements PV-001-product-purpose, RETRO-007-q1-launch
```

### 7.2 Dead-reference detection

The validator MUST scan source files in the repository (respecting `docs.backlinking.file_types`) and emit:

- `backlink-dead-reference` (warning) when a code file references a doc identifier that does not exist in the corresponding doc type directory.
- `backlink-superseded-reference` (warning) when a code file references a doc whose status is `deprecated` or `superseded`.

The reference extraction regex MUST be derived from each doc type's configured `naming_pattern` (so a project that customizes its ADR naming pattern still gets accurate dead reference detection).

### 7.3 Disabling

Backlinking is enabled by default. A project that needs to disable it sets `docs.backlinking.disable = true`. Per doc type backlinking toggles are not supported by design: backlinking is either on for the project or off for the project.

### 7.4 Generator side requirements

The standard does not require code files to be auto generated with backlinks. It requires that the validator emit a diagnostic when a backlink is missing or dead. Adding the backlink line is the implementer's responsibility (and a good fit for a planning agent's checklist).

---

## 8. Doc Type Registry

The parent standard defines the doc type registry. Each doc type is implemented as a substandard under `substandards/`. The shipped doc types are:

| Doc type | Substandard | Default location | Config key in `apss.yaml` |
|----------|-------------|------------------|---------------------------|
| Architecture Decision Records | `EXP-V1-0004.ADR01` | `docs/adrs/` | `docs.adr` |
| Purpose and Vision | `EXP-V1-0004.PV01` | `docs/vision.md` | `docs.purpose-and-vision` |
| Retrospectives | `EXP-V1-0004.RETRO01` | `docs/retrospectives/` | `docs.retrospectives` |

### 8.1 Lifecycle status (shared)

Doc types that have lifecycle status MUST use a shared vocabulary so tooling can be uniform across types:

- `proposed`: under discussion, not yet adopted.
- `accepted` / `active`: current source of truth.
- `deprecated`: discouraged but still informative.
- `superseded`: replaced by another doc of the same type; the front matter MUST include `superseded_by: <doc-id>`.

ADRs are never revised; they are superseded. Retrospectives are append only. Purpose-and-Vision documents follow the same status field but typically remain `active` for long stretches.

### 8.2 Adding a new doc type

A new doc type is added by:

1. Creating a substandard under `substandards/<ID>-<slug>/`.
2. Defining its nested key under `docs` in `apss.yaml`, using the substandard's kebab-case slug (so `docs.<slug>`). The block MUST start with `disable: false`. Any further fields are owned by the substandard. Substandards do NOT register their own top-level slug in the meta-standard registry.
3. Registering the doc type in this section's table.
4. Defining the substandard's diagnostic codes using the human readable scheme described in Section 10.

The parent standard MUST NOT hard code the list of doc types in code paths that would break when a new doc type is added. Validators MUST iterate the registry, not enumerate types by name.

### 8.3 Substandard summaries

- **EXP-V1-0004.ADR01 (Architecture Decision Records).** Spec: [`substandards/ADR01-architecture-decision-records/docs/01_spec.md`](../substandards/ADR01-architecture-decision-records/docs/01_spec.md). Validates naming, frontmatter (including `status`), required topic keywords, header conventions, and per directory context files. Backlinking and dead reference detection use the shared rules in Section 7.
- **EXP-V1-0004.PV01 (Purpose and Vision).** Spec: [`substandards/PV01-purpose-and-vision/docs/01_spec.md`](../substandards/PV01-purpose-and-vision/docs/01_spec.md). Validates the presence and structure of the project's Purpose and Vision document, used by agents during plan and design to stay aligned with the project's North Star.
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
  1. If `apss.yaml` does not exist, ask the meta-standard's installer (CF01) to create it with the project's selected standards. If `apss.yaml` exists, MUST NOT overwrite it; only add a `docs:` block if missing, leaving every other section untouched. The added `docs:` block uses the documented defaults from Section 3.3.
  2. Install the git pre-commit hook described in Section 9.4. If a pre-commit hook already exists, MUST insert an `apss-docs-hook` block delimited by sentinel comments rather than replace the user's hook.
  3. Print the resolved doc type registry so the operator sees which doc types just became active.
- `uninstall` MUST remove only the `apss-docs-hook` block from the pre-commit hook and MUST leave `apss.yaml` (and its `docs:` block) and the rest of the hook intact.
- Both commands MUST be idempotent.

### 9.2 Validator contract

The validator is the source of truth. The hook and the standalone CLI MUST call the same validator entry point so behavior is identical.

**Inputs**:

- `repo_root: Path`: absolute path to the repository root.
- `config: ApssConfig`: the merged `docs` block from `apss.yaml` (after CF01 cascade resolution), with defaults applied for any missing fields.
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

- ADR (`EXP-V1-0004.ADR01`): the ADR directory exists, every file matches the naming pattern, every ADR has the required frontmatter and `status`, required topic keywords are satisfied, context files exist with referencing guidance, and there are no dead or superseded backlinks. See the ADR01 spec for the per rule diagnostic codes.
- Purpose and Vision (`EXP-V1-0004.PV01`): a single `vision.md` (or configured location) exists with frontmatter, a `## Purpose` section, a `## Vision` section, a `## Non-Goals` section, and a current `status`. See PV01 spec.
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
| `invalid-apss-yaml` | error | Config | `apss.yaml` is not valid YAML. |
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
| `root-claude-md-missing` | error | DOC03 | Root missing `CLAUDE.md`. |
| `root-agents-md-missing` | error | DOC03 | Root missing `AGENTS.md`. |
| `root-self-reference-missing` | warning | DOC03 | Root context file missing required APSS, docs, or doc-type references. |
| `skills-format-violation` | warning | DOC03 | Skill README does not follow the Claude Code skills format. |
| `backlink-dead-reference` | warning | Backlink | Code references a doc identifier that does not exist. |
| `backlink-superseded-reference` | warning | Backlink | Code references a `deprecated` or `superseded` doc. |
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

- [ ] `apss.yaml` valid, or absent for defaults; the `docs:` block (if present) parses against Section 3.3.
- [ ] Every docs directory has `README.md` with a valid `## Index` section.
- [ ] `.md` files under the docs root have closed frontmatter with the configured fields.
- [ ] `CLAUDE.md` and `AGENTS.md` present per docs directory.
- [ ] Root `CLAUDE.md` and `AGENTS.md` exist and reference APSS, the docs root, and every active doc type's location.
- [ ] For every active doc type, the substandard's own checks pass.
- [ ] No code file references a missing, deprecated, or superseded doc identifier.
- [ ] The pre-commit hook is installed and refuses commits with errors.
