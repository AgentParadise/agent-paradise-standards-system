---
name: "EXP-V1-0004.RETRO01 (Retrospectives)"
description: "Enforces an append-only retrospective directory with consistent naming and required sections"
---

# Retrospectives Substandard (EXP-V1-0004.RETRO01)

A retrospective is the project's institutional memory. RETRO01 keeps that memory append only, consistently named, and structured the same way every time so that agents and humans can scan twenty retros and pull patterns out of them.

## What It Enforces

- A retrospective directory exists at the configured location (default `retrospectives`, resolved relative to `docs.root` per the parent spec Section 3.3 path-resolution rule; the default `docs.root: docs` makes it `docs/retrospectives/`).
- Every retro file matches the naming pattern (default `RETRO-\d{3,5}-<slug>.md`).
- Every retro has frontmatter with `name`, `description`, `date`, and `status`.
- Every retro contains `## Context`, `## What Went Well`, `## What Did Not`, and `## Followups`.
- Retros are append only: an existing retro in the staged change set MUST NOT have content modifications outside an appended footnote section. (`Changed` validator scope only.)
- The retro directory has a `README.md` with the auto generated `## Index`, and the parent standard's `CLAUDE.md` and `AGENTS.md` rules apply.

## Quick Start

```bash
aps run docs validate .
```

A minimal valid `docs/retrospectives/RETRO-001-q1-launch.md`:

```markdown
---
name: "Q1 launch retrospective"
description: "What we learned from shipping the Q1 launch"
date: 2026-04-12
status: active
---

# Q1 launch retrospective

## Context

We shipped ...

## What Went Well

- ...

## What Did Not

- ...

## Followups

- ...
```

## Configuration

In `APSS.yaml` (owned by APS-V1-0000.CF01); this substandard registers the key `retrospectives` under the parent `docs` slug. Per the parent standard's absence-equals-enabled convention (Section 3.2 of the parent spec), a project adopting every default for this substandard writes nothing under `docs.retrospectives`. The substandard is opt-out, not opt-in.

To override the default directory or naming pattern:

```yaml
docs:
  retrospectives:
    directory: docs/learnings
    naming_pattern: "RETRO-\\d{3,5}-[a-zA-Z0-9-]+\\.md"
```

To disable the substandard entirely, set `disable: true`. Absence is
the default. `append_only: false` disables the append-only check; it
defaults to `true` because retros are a historical record, not a working
document.

## Error Codes

| Code | Severity | Description |
|------|----------|-------------|
| `RETRO01-dir-not-found` | error | Retrospective directory does not exist. |
| `RETRO01-naming-mismatch` | error | File in the retro directory does not match the naming pattern. |
| `RETRO01-frontmatter-missing` | error | Retro file has no frontmatter. |
| `RETRO01-frontmatter-field-missing` | error | Retro frontmatter missing a required field. |
| `RETRO01-invalid-status` | error | `status` is not one of `proposed`, `active`, `deprecated`, `superseded`. |
| `RETRO01-missing-section` | warning | Retro missing a required section heading. |
| `RETRO01-history-modified` | error | An existing retro was modified outside the allowed append region. |
| `RETRO01-invalid-naming-regex` | error | Configured naming regex is invalid. |
