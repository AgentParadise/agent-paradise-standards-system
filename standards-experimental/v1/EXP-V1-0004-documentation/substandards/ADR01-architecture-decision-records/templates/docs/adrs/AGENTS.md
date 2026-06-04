---
name: "Architecture Decision Records (agent context)"
description: "ADR location, when to write one, and the backlink rule for every agent reading this directory"
---

# Agents reading `docs/adrs/`, start here

This directory holds the project's architecture decision records (ADRs).
ADRs are the persistent memory of why this codebase is shaped the way it
is. When you read code that feels surprising, the ADR is usually the
answer.

This file is the canonical agent-context file for this directory.
`CLAUDE.md` in the same directory is a symlink to this file so Claude
Code follows the symlink and reads the same content. Gemini reads
`AGENTS.md` natively, so this directory does not ship a `GEMINI.md`.
The APSS documentation standard scaffolds this `AGENTS.md` and its
`CLAUDE.md` symlink on first install and never overwrites them on
subsequent installs (see `02_install_contract.md` Section 1.5 of the
parent standard); the content below is a starting point that a
project is free to edit.

## Where ADRs live

- Active and historical ADRs live in this directory (`docs/adrs/`).
- Each file is named `ADR-<NNN>-<imperative-slug>.md` (zero-padded
  numbers 3 to 5 digits wide; canonical lowercase-with-dashes slug).
- The directory `README.md` summarises what ADRs are, when to write
  one, the lifecycle, and the naming convention. Read it before
  writing your first ADR in this project.
- The reusable starting point is `ADR-000-template.md`.

## When to use an ADR

Reach for an ADR when you (or your operator) are about to lock in a
software design choice that addresses an architecturally significant
requirement. The canonical guidance treats every ADR as one decision,
captured with its context and its consequences. If you find yourself
weighing more than one option, recording the rationale, or expecting a
future reader to ask "why this way", you are in ADR territory.

Lightweight rule of thumb: if the decision is reversible in a single
PR with no downstream churn, you do not need an ADR. If reversing it
would touch many files or other decisions, you do.

## The backlink rule (load-bearing)

Implementation files that exist to satisfy an ADR MUST contain a
backlink to it near the top of the file. The token form is
`ADR-<NNN>-<slug>`, matching the filename without `.md`. Example:

```rust
// Implements ADR-001-choose-database.
```

```python
# Implements ADR-042-format-timestamps.
```

This is enforced parent-side under `docs.backlinking` in the EXP-V1-0004
documentation standard. The pre-commit hook surfaces:

- `ADR01-dead-reference` when code points at an ADR that does not exist
  (renamed, renumbered, or deleted).
- `ADR01-superseded-reference` when code points at an ADR whose
  `status` is `deprecated` or `superseded`. Retarget the backlink to
  the current ADR.

Skip the backlink for code that is genuinely not tied to a specific
decision. Backlinking exists so a fresh-context agent can recover the
"why" without scraping prose.

## How to add a new ADR

1. Copy `ADR-000-template.md` to
   `ADR-<NNN>-<imperative-slug>.md`. Pick the next free number.
2. Fill in the YAML frontmatter (`name`, `description`,
   `status: proposed`).
3. Write `## Context`, `## Decision`, `## Consequences`. Keep one
   decision per file.
4. If the new ADR replaces an older one, update the older ADR's
   `status` to `superseded` and add `superseded_by: ADR-<NNN>-<slug>`
   to its frontmatter, pointing at the new file.
5. Commit. The pre-commit hook validates structure.

## How to find the right ADR fast

Open `README.md` in this directory. The `## Index` table is generated
from each ADR's frontmatter and carries each ADR's `description`. Read
the index first, then open only the ADRs the descriptions tell you to.
This is the progressive disclosure pattern the parent documentation
standard is built around.

## Source

The "what is an ADR" definitions, the lifecycle vocabulary, and the
naming guidance come from the canonical community resource by Joel
Parker Henderson,
<https://github.com/architecture-decision-record/architecture-decision-record>
(version 3.2.0, 2025-05-29). The substandard overview at
`docs/00_overview.md` of EXP-V1-0004.ADR01 in this repo carries the
full citation and lists where this project's conventions diverge.
