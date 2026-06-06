---
name: "EXP-V1-0004.ADR01 - Architecture Decision Records"
description: "Enforces ADR naming, front matter, required keywords, and backlinking, aligned with the canonical ADR community guidance"
---

# ADR Enforcement Substandard (EXP-V1-0004.ADR01)

Validates Architecture Decision Records within the documentation structure
defined by the parent standard (EXP-V1-0004).

This substandard's vocabulary, lifecycle, and template default to the
canonical community resource Joel Parker Henderson maintains at
<https://github.com/architecture-decision-record/architecture-decision-record>
(version 3.2.0, 2025-05-29). Citations below point back to that resource so
projects can extend their ADR practice from a known starting point.

## What an ADR is, and when to write one

An **architecture decision record** (ADR) is a document that captures an
important architectural decision made along with its context and
consequences. An **architecture decision** (AD) is a software design choice
that addresses a significant requirement. The collection of every ADR a
project keeps is its **architecture decision log** (ADL). A requirement is
**architecturally significant** when it has a measurable effect on the
system's architecture (an ASR). All of the above sit inside the broader
practice of **architecture knowledge management** (AKM).
(Source: canonical README, "What is an architecture decision record?")

Write an ADR when:

- You are about to lock in a design choice that addresses an ASR. If the
  decision has measurable architectural impact, capture it before you
  commit to it.
- You have weighed two or more options and want the rationale, the
  pros/cons you considered, and any cost/benefit tradeoffs preserved for
  the next person to read.
- The decision is urgent or important enough that a future reader will ask
  "why did we do it this way" and the answer matters. A lightweight
  decision-identification practice (a decision todo list alongside the
  product todo list) is how the canonical guidance suggests teams find
  these moments.
- An earlier ADR no longer reflects current reality. ADRs are immutable;
  the supported move is to write a new ADR that supersedes the older one,
  rather than editing the old text. See the lifecycle below.

If a decision does not have architectural impact, it does not need an ADR.
ADRs work best when each one captures exactly one decision; multiple
decisions per ADR is an anti-pattern.

## What a good ADR looks like

The canonical guidance lists four characteristics every ADR should have:

- **Rationale.** Explain the reasons. The Context and the Consequences
  sections of the Nygard template are where this lands.
- **Specific.** One AD per ADR. If two decisions surfaced, split into two
  ADRs and link them.
- **Timestamped.** Identify when each item in the ADR was written. The
  parent standard's frontmatter contract carries the file's authoring
  date; large amendments should be timestamped inline.
- **Immutable.** Do not alter the body of an existing ADR. Amend by
  appending a dated note, or supersede with a new ADR that links back.

A good **Context** section explains the organisation's situation and
business priorities, the team's social and skills makeup as it bears on
the decision, and the pros and cons relevant to the project's needs and
goals.

A good **Consequences** section explains what becomes easier or harder,
follow-up work the decision triggers, expected after-action review (the
canonical guidance notes that a one-month re-read of each ADR against
what actually happened is a healthy practice), and any subsequent ADRs
the decision will force.

## ADR lifecycle (status field)

Every ADR carries a `status` in its frontmatter. The allowed values match
the canonical Nygard lifecycle:

- `proposed` - under discussion, not yet adopted.
- `accepted` - the decision is in force.
- `deprecated` - discouraged but still informative.
- `superseded` - replaced by a later ADR; the frontmatter MUST point at
  the superseding ADR via `superseded_by`.

ADRs are never revised. A change of direction is recorded by writing a
new ADR and marking the prior ADR `superseded`. Backlinking in code,
described below, is what keeps live references from pointing at retired
decisions.

## File naming convention

The default naming pattern is `ADR-\d{3,5}-[a-zA-Z0-9-]+\.md`, configurable
via `docs.adr.naming_pattern` in `APSS.yaml`. The canonical guidance prefers
present-tense imperative verb phrases in the name, lowercase, dashes for
word separation, and the `.md` extension. Examples:

- `ADR-001-choose-database.md`
- `ADR-042-format-timestamps.md`
- `ADR-100-manage-passwords.md`

The `ADR-` prefix and the digit width are this substandard's project
convention; the lowercase-with-dashes name body matches the canonical
guidance.

## Templates this substandard ships

When the docs install runs in a repo that lacks them, the installer
creates the following files (create-if-missing, never overwrite):

- `docs/adrs/README.md` - the ADR directory README, summarising what an
  ADR is, when to write one, the lifecycle, and the naming convention.
- `docs/adrs/AGENTS.md` - the canonical agent-context block for the ADR
  directory, carrying the ADR location, when-to-use guidance, the
  parent-level backlink rule, and a reference back to this substandard
  spec. `AGENTS.md` is the canonical file; Gemini reads it natively, so
  this substandard ships no `GEMINI.md`.
- `docs/adrs/CLAUDE.md` - a symlink to the adjacent `AGENTS.md`, so
  Claude Code follows the symlink and reads the same content.
- `docs/adrs/ADR-000-template.md.example` - a Nygard-style ADR template with the
  required frontmatter and `## Context`, `## Decision`, `## Consequences`
  sections.

The source templates ship inside this substandard at
`templates/`. The install contract (parent standard,
`docs/02_install_contract.md` Section 1.5) is normative on the
create-if-missing, never-overwrite behaviour: an existing
`AGENTS.md` or `CLAUDE.md` in a project's docs-area directory is
never modified by the installer regardless of how it differs from
the shipped template; the validator checks only that the files exist
and have well-formed frontmatter.

## Quick Start

```bash
# Validate ADRs (runs as part of docs validate)
aps run docs validate .

# Configure required ADR keywords in APSS.yaml at the repo root
# (config is owned by APS-V1-0000.CF01; the docs standard contributes the docs: block)
docs:
  adr:
    required_adr_keywords:
      - security
      - testing
      - deployment
```

## What It Enforces

- **Naming convention**: `ADR-XXX-<adr-name>.md` (configurable regex).
- **Front matter**: every ADR has `name`, `description`, and `status` in
  YAML frontmatter (plus `superseded_by` when status is `superseded`).
- **Lifecycle status**: `proposed`, `accepted`, `deprecated`,
  `superseded`. ADRs are never revised, only superseded.
- **Required keywords**: ensures ADRs exist for configured topic keywords
  (for example, security, testing).
- **Backlinking**: implementation files reference their governing ADR
  identifier. Backlinking is enforced parent-side under
  `docs.backlinking`; the per-substandard contract is documented here.
- **Dead reference detection**: warns on code referencing non-existent or
  superseded ADRs.

## Error Codes

Codes are human-readable. The prefix names the substandard, the suffix
names the condition, so a single line of CLI output
(`ADR01-dir-not-found: ...`) is self-explaining.

| Code | Description |
|------|-------------|
| `ADR01-dir-not-found` | ADR directory not found |
| `ADR01-invalid-naming` | ADR file does not match naming pattern |
| `ADR01-missing-frontmatter` | ADR file missing required front matter |
| `ADR01-missing-required-keyword` | Required ADR keyword not satisfied |
| `ADR01-missing-backlink` | Reserved (not emitted) |
| `ADR01-invalid-naming-regex` | Invalid naming regex in config |
| `ADR01-missing-context-file` | ADR directory missing CLAUDE.md or AGENTS.md |
| `ADR01-context-missing-guidance` | ADR context file lacks ADR referencing guidance |
| `ADR01-unknown-reference` | Source file references an ADR that does not resolve to a real file in `docs.adr.directory` matching the configured naming pattern (error; replaces the earlier `ADR01-dead-reference` warning) |
| `ADR01-missing-header` | ADR file missing required section header |
| `ADR01-invalid-status` | ADR missing or invalid `status` field |
| `ADR01-superseded-reference` | Source file references an ADR whose `status` is `superseded` (warning; hint names the `superseded_by` target) |
| `ADR01-deprecated-reference` | Source file references an ADR whose `status` is `deprecated` (warning; hint suggests retarget or annotate as intentional) |

## Source

The "what an ADR is" definitions, the lifecycle, the writing
characteristics, and the file naming guidance above are summarised from
the canonical community resource:

- Joel Parker Henderson, *Architecture decision record (ADR)*. GitHub,
  <https://github.com/architecture-decision-record/architecture-decision-record>
  (version 3.2.0, 2025-05-29). Sections:
  "What is an architecture decision record?",
  "How to start using ADRs", "File name conventions for ADRs",
  "Suggestions for writing good ADRs",
  "Decision record template by Michael Nygard".

Where this substandard's conventions diverge from that resource (the
fixed `ADR-NNNNN-` prefix and the kebab-case slug, the parent
standard's frontmatter contract, the parent-level backlinking rule)
the divergence is called out inline above.
