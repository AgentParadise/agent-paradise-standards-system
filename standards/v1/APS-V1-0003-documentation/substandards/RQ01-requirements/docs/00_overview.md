---
name: "APS-V1-0003.RQ01 (Requirements)"
description: "Optional per-system requirements document agents read before building, plus the requirements gate"
---

# Requirements Substandard (APS-V1-0003.RQ01)

An optional doc type. A system MAY keep one requirements document that
states what the system must be and do. Agents that know this standard
read it whenever they build or change that system, and use it to decide
whether a feature should be built at all and how.

It sits one step below the North Star (PV01): the North Star says why the
project exists; the requirements say what each system must do to serve
it, and how you know.

## The requirements gate

When a proposed feature is not covered by the requirements, an agent does
not just build it. It raises the gap with the requirements owner (the
`owner` in the document's frontmatter), proposes the amendment, and builds
after the owner amends the requirements. See
[Section 9 of the spec](01_spec.md#9-agent-obligations-read-first-then-the-requirements-gate).

## What It Enforces

Only for documents that exist, or that a project declares:

- Frontmatter: `name`, `description`, `owner`, `status`, `version`,
  `last_reviewed`, and `serves` (a link to the North Star it serves).
- Sections: `## Vision`, `## Roles`, `## Requirements`,
  `## User Stories`, `## Later and Out of Scope`, `## Change Log`.
- Requirements: a table of rows with a stable, unique id (`P3.5`), the
  requirement in plain words, a measure, and a check status
  (`checked`, `partial`, `unchecked`, `later`), naming the check when it
  is `checked`.
- At least one user story ("As a <role>, I want <goal>, so that
  <reason>") and at least one listed role.
- A dated, versioned change log.

A project with no requirements document passes.

## Quick Start

```bash
apss run documentation validate .
```

A complete valid document is in
[`examples/requirements.md`](../../../examples/requirements.md).

## Configuration

Absent config: `<docs.root>/requirements.md` is validated if it exists.
To opt in to presence checks, or to keep one document per system, list
the locations (docs-root-relative):

```yaml
docs:
  requirements:
    locations:
      - requirements.md
      - systems/billing/requirements.md
```

Disabling: `disable: true`.

## Why a table, not prose

Each requirement needs four things a reviewer and an agent can find in
one glance: id, requirement, measure, check. A table row keeps them
together, diffs line by line, and parses without a Markdown AST. Pillars
stay as `###` headings above each table so the grouping reads naturally.

## Error Codes

See [Section 11 of the spec](01_spec.md#11-error-codes).
