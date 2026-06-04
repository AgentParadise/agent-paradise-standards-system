---
name: "Documentation and Context Engineering"
description: "Validated documentation structure that turns docs into machine-readable data for indexes, search, vectorization, generation, and agent context"
---

# EXP-V1-0004 - Documentation and Context Engineering

A configurable, growing doc type registry plus an installable git hook that keeps a project's documentation structure provably correct on every commit. The standard is opinionated by default, configurable by exception, and never an unconditional hard-break.

The point is not "consistent docs". The point is that once structure is guaranteed by a hook, the docs become reliable data that downstream tooling can stand on.

## Why this matters

Validated structure is the prerequisite for every interesting thing you want to do over documentation:

- **Automated validation.** A pre-commit hook refuses commits whose docs drift out of structure, so "the docs are mostly correct" stops being a hope.
- **Generation tooling.** Index tables, navigation, doc-type templates, and per-directory context files can be generated from frontmatter because the frontmatter is known to be present and well-formed.
- **Vectorize any directory.** With frontmatter and stable per-file metadata guaranteed, a docs tree can be embedded directly. No per-repo schema discovery, no preprocessing.
- **Semantic lookups and progressive disclosure.** Agents and humans can ask "what's the current Purpose and Vision?" or "list ADRs that supersede ADR-007" without scraping prose, because doc identifiers and statuses are first-class fields.
- **Context engineering for AI agents.** Per-directory `CLAUDE.md` and `AGENTS.md` files plus a normative root self-reference give a fresh-context agent enough to orient itself in one read.
- **Backlinking across plan, design, and implementation.** Implementation files are required to backlink the governing doc (ADR, Purpose and Vision, ...) so context is never lost across phases.

Consistency and process are by-products. The unlock is treating docs as structured data with a hook-enforced contract.

## What the standard provides

1. **A configurable doc type registry.** Each doc type (ADR, Purpose and Vision, Retrospectives, and future additions) is implemented as a substandard with its own frontmatter and validation rules. Adding a new doc type does not require changing the parent spec. The registry is enumerated in [01_spec.md](01_spec.md#8-doc-type-registry).
2. **A single shared config file** at `apss.yaml`, owned by the meta-standard (APS-V1-0000.CF01). This standard registers the slug `docs` and contributes the `docs:` section schema. Every rule is default on. A project switches one off by setting `disable: true` in the smallest scope that contains it. There are no scattered per-feature `optional` flags.
3. **Frontmatter-driven indexing.** Every `.md` file under the docs root carries YAML frontmatter. Every directory `README.md` gets an auto-generated `## Index` table built from that frontmatter. The dry-run output and the written output are byte-identical for the same input.
4. **An installable hook contract.** Installing the standard installs a git pre-commit hook that auto-refreshes indexes, runs the validator against staged docs, and blocks the commit on errors. The hook and the standalone CLI call the same validator, so behavior is identical. The contract is specified in [01_spec.md Section 9](01_spec.md#9-install-contract-hook--validator--index).
5. **A backlinking rule that applies across every doc type.** Code files that implement a governed doc MUST reference it by identifier. The validator flags missing and dead references. Backlinking is part of the standard, not a per-doc-type opt-in.
6. **Human-readable diagnostic codes.** All codes are kebab strings such as `index-stale`, `frontmatter-unclosed`, `ADR01-dir-not-found`, `PV01-missing-vision-section`. Numeric codes are not used.

## Shipped doc types

| Doc type | Substandard | Default location | Why it exists |
|----------|-------------|------------------|---------------|
| Architecture Decision Records | [`EXP-V1-0004.ADR01`](../substandards/ADR01-architecture-decision-records/docs/00_overview.md) | `docs/adrs/` | Append-only record of architectural decisions with lifecycle status. |
| Purpose and Vision | [`EXP-V1-0004.PV01`](../substandards/PV01-purpose-and-vision/docs/01_spec.md) | `docs/vision.md` | North Star document agents read during plan and design to stay aligned with the project's intent. |
| Retrospectives | [`EXP-V1-0004.RETRO01`](../substandards/RETRO01-retrospectives/docs/01_spec.md) | `docs/retrospectives/` | Append-only record of what was learned, by period or by milestone. |


Doc types are activated by their `docs.<slug>` key in `apss.yaml` (kebab-case slugs match each substandard's `substandard.toml`). Default on, switchable off.

## Configuration

All settings live in the `docs:` section of the project's root `apss.yaml` (owned by APS-V1-0000.CF01). Zero-config works; defaults are documented in [01_spec.md Section 3](01_spec.md#3-configuration). A complete example is in [examples/apss.yaml](../examples/apss.yaml).

## CLI

```bash
aps run docs install [path]          # Install hook + default config (idempotent)
aps run docs uninstall [path]        # Remove hook (config preserved)
aps run docs validate [path]         # Validate documentation structure
aps run docs index [path]            # Preview auto-generated indexes (dry run)
aps run docs index [path] --write    # Write indexes into README.md files
aps run docs hook --staged           # Hook entry point used by pre-commit
```

The install contract, the validator contract, the index generator contract, and the per-doc-type definition of "valid structure" are all normative and live in [01_spec.md](01_spec.md).

## Category

Governance. Inputs: a project's documentation tree. Outputs: a validated, indexable, vector-ready docs tree plus a hook that keeps it that way.
