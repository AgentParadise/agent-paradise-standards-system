---
name: "Architecture Decision Records"
description: "What ADRs are, when to write one, lifecycle, and the naming convention this project follows"
---

# Architecture Decision Records (ADRs)

This directory holds the project's **architecture decision records**. An
ADR is a document that captures one important architecture decision along
with its context and the consequences that follow from it.

The conventions on this page summarise the canonical community resource
(Joel Parker Henderson,
<https://github.com/architecture-decision-record/architecture-decision-record>)
through the lens of the APSS documentation standard (EXP-V1-0004.ADR01).
If something here looks unfamiliar, read the canonical resource first.

## When to write one

Write an ADR when:

- You are about to lock in a design choice that addresses an
  architecturally significant requirement (ASR). If it has measurable
  architectural impact, capture it before you commit to it.
- You weighed two or more options. The rationale, the alternatives, and
  the trade-offs deserve to survive the commit.
- The decision is important or urgent enough that a future reader will
  ask "why did we do this" and the answer matters.
- An earlier ADR no longer reflects current reality. ADRs are immutable;
  write a new ADR that supersedes the older one.

If the decision has no architectural impact, it does not need an ADR.

## One decision per ADR

Each ADR captures exactly one architecture decision. If two decisions
surfaced together, split them into two ADRs and link them.

## Lifecycle (the `status` field)

Every ADR carries a `status` in its YAML frontmatter:

- `proposed` - under discussion, not yet adopted.
- `accepted` - the decision is in force.
- `deprecated` - discouraged, but still informative.
- `superseded` - replaced by a later ADR. The frontmatter MUST also
  include `superseded_by: ADR-XXX-<slug>` pointing at the replacement.

ADRs are never edited in place. If a decision changes, write a new ADR
that supersedes the old one and update the old one's status. Code that
backlinks the retired ADR will surface as a
`ADR01-superseded-reference` warning when the target is `superseded`
(the hint names the `superseded_by` target so the backlink can be
retargeted) or `ADR01-deprecated-reference` when the target is
`deprecated` (the warning suggests retarget or annotate the reference
as intentional, since deprecated ADRs are still informative).

## File naming

The project naming pattern is:

```
ADR-<NNN>-<imperative-slug>.md
```

- `<NNN>` is a zero-padded number 3 to 5 digits wide. Padding keeps the
  files sorted in the order they were written.
- `<imperative-slug>` is a present-tense verb phrase in lowercase with
  dashes (canonical guidance: "present tense imperative verb phrase").

Examples:

- `ADR-001-choose-database.md`
- `ADR-042-format-timestamps.md`
- `ADR-100-manage-passwords.md`

The pattern is configurable via `docs.adr.naming_pattern` in
`APSS.yaml`.

## Backlinking from code

Implementation files that exist to satisfy an ADR MUST contain a
backlink near the top, of the form `ADR-NNN-<slug>`. Example:

```rust
// Implements ADR-001-choose-database.
```

This is parent-level invariant (see EXP-V1-0004 Section 7), enforced by
the validator under `docs.backlinking`. Stale references to deleted or
superseded ADRs are surfaced automatically.

## Writing a new ADR

1. Copy `ADR-000-template.md.example` to a new file named
   `ADR-<NNN>-<imperative-slug>.md`.
2. Fill in the frontmatter (`name`, `description`, `status: proposed`).
3. Write `## Context`, `## Decision`, and `## Consequences`. The
   canonical guidance on what makes each section good is summarised in
   the substandard overview at
   `standards-experimental/v1/EXP-V1-0004-documentation/substandards/ADR01-architecture-decision-records/docs/00_overview.md`.
4. Commit. The pre-commit hook validates structure and refuses to land
   ADRs that are missing required sections or backlinks.

## Reading existing ADRs

The directory README's `## Index` table (managed by the parent docs
standard's index generator) is the entry point. Each row carries the
ADR's `name` and `description` from its frontmatter so the index is
enough to decide which ADR you actually need to open. This is the
"progressive disclosure" model the parent standard is built around: the
index tells you what is here; you read the body only when the
description tells you to.

## Source

The definitions on this page (ADR, AD, ADL, ASR) and the lifecycle and
file-naming guidance are summarised from
<https://github.com/architecture-decision-record/architecture-decision-record>
(version 3.2.0, 2025-05-29). The substandard overview at
`docs/00_overview.md` carries the full citation.
