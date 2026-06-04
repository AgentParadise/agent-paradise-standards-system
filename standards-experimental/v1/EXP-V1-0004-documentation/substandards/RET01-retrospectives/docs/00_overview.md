---
name: "EXP-V1-0004.RET01 (Retrospectives)"
description: "Enforces an append-only retrospective directory with consistent naming and required sections"
---

# Retrospectives Substandard (EXP-V1-0004.RET01)

A retrospective is the project's institutional memory. RET01 keeps that memory append only, consistently named, and structured the same way every time so that agents and humans can scan twenty retros and pull patterns out of them.

## What It Enforces

- A retrospective directory exists at the configured location (default `docs/retrospectives/`).
- Every retro file matches the naming pattern (default `RET-\d{3,5}-<slug>.md`).
- Every retro has frontmatter with `name`, `description`, `date`, and `status`.
- Every retro contains `## Context`, `## What Went Well`, `## What Did Not`, and `## Followups`.
- Retros are append only: an existing retro in the staged change set MUST NOT have content modifications outside an appended footnote section. (`Changed` validator scope only.)
- The retro directory has a `README.md` with the auto generated `## Index`, and the parent standard's `CLAUDE.md` and `AGENTS.md` rules apply.

## Quick Start

```bash
aps run docs validate .
```

A minimal valid `docs/retrospectives/RET-001-q1-launch.md`:

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

```toml
[docs.retrospectives]
disable        = false
directory      = "docs/retrospectives"
naming_pattern = "RET-\\d{3,5}-[a-zA-Z0-9-]+\\.md"
append_only    = true
```

`append_only = false` disables the append-only check. It is on by default because retros are a historical record, not a working document.

## Error Codes

| Code | Severity | Description |
|------|----------|-------------|
| `RET01-dir-not-found` | error | Retrospective directory does not exist. |
| `RET01-naming-mismatch` | error | File in the retro directory does not match the naming pattern. |
| `RET01-frontmatter-missing` | error | Retro file has no frontmatter. |
| `RET01-frontmatter-field-missing` | error | Retro frontmatter missing a required field. |
| `RET01-invalid-status` | error | `status` is not one of `proposed`, `active`, `deprecated`, `superseded`. |
| `RET01-missing-section` | warning | Retro missing a required section heading. |
| `RET01-history-modified` | error | An existing retro was modified outside the allowed append region. |
| `RET01-invalid-naming-regex` | error | Configured naming regex is invalid. |
