---
name: "EXP-V1-0004.PV01 (Purpose and Vision)"
description: "Enforces the project's North Star Purpose and Vision document so agents stay aligned during plan and design"
---

# Purpose and Vision Substandard (EXP-V1-0004.PV01)

A small but load-bearing substandard. Every project that adopts EXP-V1-0004 carries a single Purpose and Vision document. Agents read it during planning and design to stay aligned with the project's North Star instead of drifting toward whatever the immediate prompt suggests.

## What It Enforces

- The Purpose and Vision document exists at the configured location (default `docs/PURPOSE.md`).
- It carries frontmatter with `name`, `description`, and `status`.
- It contains a `## Purpose` section, a `## Vision` section, and a `## Non-Goals` section.
- It is backlinked from the root `CLAUDE.md` and `AGENTS.md` so agents can find it on a fresh start (handled by the parent standard's DOC03 self-reference check).
- `status` follows the shared lifecycle vocabulary defined in the parent spec (Section 8.1): `proposed`, `active`, `deprecated`, `superseded`.

## Quick Start

```bash
# Validate Purpose and Vision (runs as part of docs validate)
aps run docs validate .
```

A minimal valid `docs/PURPOSE.md`:

```markdown
---
name: "Project Purpose and Vision"
description: "What this project exists to do and the world it tries to create"
status: active
---

# Purpose and Vision

## Purpose

We exist to ...

## Vision

In three years, ...

## Non-Goals

We will not ...
```

## Configuration

```toml
[docs.purpose_and_vision]
disable  = false
location = "docs/PURPOSE.md"
```

Disabling: set `disable = true`. Backlinking from `CLAUDE.md` and `AGENTS.md` is enforced by the parent standard's DOC03-self-reference check, not by this substandard.

## Error Codes

| Code | Severity | Description |
|------|----------|-------------|
| `PV01-document-missing` | error | No file at `docs.purpose_and_vision.location`. |
| `PV01-frontmatter-missing` | error | Document lacks frontmatter. |
| `PV01-frontmatter-field-missing` | error | Document missing `name`, `description`, or `status`. |
| `PV01-missing-purpose-section` | error | Document missing `## Purpose` heading. |
| `PV01-missing-vision-section` | error | Document missing `## Vision` heading. |
| `PV01-missing-non-goals-section` | warning | Document missing `## Non-Goals` heading. |
| `PV01-invalid-status` | error | `status` is not one of the allowed values. |
| `PV01-superseded-without-pointer` | error | `status: superseded` requires `superseded_by` in frontmatter. |
