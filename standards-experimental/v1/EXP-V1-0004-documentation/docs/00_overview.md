---
name: "Documentation and Context Engineering"
description: "Generic frontmatter-driven indexing and progressive disclosure for any docs directory, plus a pluggable doc-type registry layered on top"
---

# EXP-V1-0004 - Documentation and Context Engineering

The primary purpose of this standard is small and sharp:

**A generic frontmatter-driven index and progressive disclosure mechanism
for any docs directory.** Every `.md` file under the docs root carries
YAML frontmatter with at minimum a `name` and a `description`. The
parent indexer reads that frontmatter and writes a `## Index` table
into every directory `README.md`. Agents and humans read the index
first, then descend only into the documents whose `description` says
they should. A docs directory becomes a skill manifest: shallow on the
outside, deep where it needs to be.

Everything else this standard ships is layered on top of that one
mechanism. The doc-type registry, the substandards (ADRs, the North
Star, Retrospectives), and the install contract are instances and
infrastructure for the generic primitive, not the point of it.

## What the generic mechanism guarantees

1. **Frontmatter is required.** Every `.md` file under the docs root
   has YAML frontmatter with the fields named in
   `docs.index.frontmatter_fields` (default: `name`, `description`).
   The validator emits `frontmatter-missing`, `frontmatter-unclosed`,
   and `frontmatter-field-missing` so the surface is provably
   structured.
2. **Indexes are generated from frontmatter.** Every directory
   `README.md` carries a `## Index` table built from the frontmatter of
   its sibling `.md` files. The dry run output and the written output
   are byte-identical for the same input, so `index-stale` is the
   single diagnostic that decides whether an index needs a rewrite.
3. **Progressive disclosure is the read model.** The index carries each
   document's `description`. An agent reads the index, picks the rows
   whose descriptions match the question at hand, and opens only those
   files. A long docs tree fits in one context window through the
   `description` column alone.
4. **A pre-commit hook keeps it true.** Installing the standard
   installs a hook that refreshes indexes for any directory whose
   contents changed and refuses commits whose docs drift out of
   structure. The hook and the standalone validator call the same
   entry point, so behavior is identical at commit time and in CI.

The point is not consistent docs. The point is that once the
frontmatter + index contract is hook-enforced, every downstream
operation can stand on it: semantic search, vectorisation, agent
context loading, generation tooling, doc-as-data pipelines. None of
them need to discover schemas per project.

## What lives on top of the generic mechanism

The generic mechanism is the substrate. Concrete doc types plug into
it through the doc-type registry, each as a substandard with its own
frontmatter, structure rules, and diagnostic codes. The substandards
shipped today are listed below; new substandards register their own
nested config key under `docs` without changing the parent spec.

| Doc type | Substandard | Default location | Why it exists |
|----------|-------------|------------------|---------------|
| Architecture Decision Records | [`EXP-V1-0004.ADR01`](../substandards/ADR01-architecture-decision-records/docs/00_overview.md) | `docs/adrs/` | Append-only record of architectural decisions with lifecycle status. |
| North Star (Mission, Vision, Position) | [`EXP-V1-0004.PV01`](../substandards/PV01-purpose-and-vision/docs/01_spec.md) | `docs/north-star.md` | Single document agents read during plan and design to stay aligned with the project's intent. |
| Retrospectives | [`EXP-V1-0004.RETRO01`](../substandards/RETRO01-retrospectives/docs/01_spec.md) | `docs/retrospectives/` | Append-only record of what was learned, by period or by milestone. |

Doc types are activated by their `docs.<slug>` key in `APSS.yaml`
(kebab-case slugs match each substandard's `substandard.toml`). Every
doc type is default on, switchable off.

Each substandard MAY ship starter template files (directory READMEs,
agent-context blocks, document templates). The installer materialises
them on first install and skips them on subsequent runs to preserve
the operator's edits. See
[`02_install_contract.md`](02_install_contract.md) Section 1.4 for the
shipped inventory.

## What else this standard provides

Beyond the substrate and the registry, the parent contributes a small
amount of project-wide infrastructure:

1. **A single shared config file** at `APSS.yaml`, owned by the
   meta-standard (APS-V1-0000.CF01). This standard registers the slug
   `docs` and contributes the `docs:` section schema. Every rule is
   default on. A project switches one off by setting `disable: true`
   in the smallest scope that contains it. There are no scattered
   per-feature `optional` flags. Absence of a key equals the default
   value: a project that adopts every default writes nothing into
   `APSS.yaml`, and `disable: false` is the default the validator
   applies for absence and MUST NOT appear in real configs or examples.
2. **An installable hook contract.** Installing the standard installs
   a git pre-commit hook that auto-refreshes indexes, runs the
   validator against staged docs, and blocks the commit on errors.
   The contract is specified in
   [`01_spec.md` Section 9](01_spec.md#9-install-contract-hook--validator--index)
   and reproduced in full in
   [`02_install_contract.md`](02_install_contract.md).
3. **Per-directory and per-repo context files.** `AGENTS.md` is the
   canonical agent context file; `CLAUDE.md` ships alongside it as a
   symlink (Gemini reads `AGENTS.md` natively, so the standard ships
   no `GEMINI.md`). Both are required at the docs root, at the
   repository root, and in every docs subdirectory. The root files MUST
   reference APSS, the docs root, and every active doc type's location
   so a fresh-context agent can orient itself in one read. For the
   docs-area context files the substandard ships templates and the
   installer creates them when absent without ever overwriting an
   existing file (see [`02_install_contract.md`](02_install_contract.md)
   Section 1.5).
4. **A backlinking rule that applies across every doc type.** Code
   files that implement a governed doc MUST reference it by identifier.
   The validator flags missing and dead references. Backlinking is part
   of the standard, not a per-doc-type opt-in.
5. **Human-readable diagnostic codes.** All codes are kebab strings
   such as `index-stale`, `frontmatter-unclosed`, `ADR01-dir-not-found`,
   `PV01-missing-mission-section`. Numeric codes are not used.

## Configuration

All settings live in the `docs:` section of the project's root
`APSS.yaml` (owned by APS-V1-0000.CF01). Zero-config works; defaults
are documented in
[`01_spec.md` Section 3](01_spec.md#3-configuration). A complete
example is in [`examples/APSS.yaml`](../examples/APSS.yaml).

## CLI

```bash
aps run docs install [path]          # Install hook + default config (idempotent)
aps run docs uninstall [path]        # Remove hook (config preserved)
aps run docs validate [path]         # Validate documentation structure
aps run docs index [path]            # Preview auto-generated indexes (dry run)
aps run docs index [path] --write    # Write indexes into README.md files
aps run docs hook --staged           # Hook entry point used by pre-commit
```

The install contract, the validator contract, the index generator
contract, and the per-doc-type definition of "valid structure" are
all normative and live in [`01_spec.md`](01_spec.md).

## Category

Governance. Inputs: a project's documentation tree. Outputs: a
validated, indexable, vector-ready docs tree plus a hook that keeps it
that way.
