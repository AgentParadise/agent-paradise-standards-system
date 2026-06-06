---
name: "EXP-V1-0004.PV01 (North Star: Mission, Vision, Position)"
description: "Enforces the project's single North Star document so agents stay aligned during plan and design"
---

# North Star Substandard (EXP-V1-0004.PV01)

A small but load-bearing substandard. Every project that adopts
EXP-V1-0004 carries a single North Star document made up of three
sections: Mission, Vision, and Position. Agents read this document
during planning and design to stay aligned with the project's intent
instead of drifting toward whatever the immediate prompt suggests.

This substandard is the canonical example of the parent standard's
"single-document doc type" pattern: one file, structured frontmatter,
required H2 sections, validated on every commit, surfaced in the docs
root index by `description`.

## What It Enforces

- The North Star document exists at the configured location (default
  `north-star.md`, resolved relative to `docs.root` per the parent
  spec Section 3.3 path-resolution rule; the default
  `docs.root: docs` makes it `docs/north-star.md`).
- It carries frontmatter with `name`, `description`, and `status`.
- It contains, in order, a `## Mission` section, a `## Vision` section,
  and a `## Position` section.
- It is backlinked from the root `CLAUDE.md` and `AGENTS.md` so agents
  can find it on a fresh start (handled by the parent standard's DOC03
  self-reference check).
- `status` follows the shared lifecycle vocabulary defined in the
  parent spec (Section 8.1): `proposed`, `active`, `deprecated`,
  `superseded`.

## Quick Start

```bash
# Validate the North Star document (runs as part of docs validate)
aps run docs validate .
```

A minimal valid `docs/north-star.md`:

```markdown
---
name: "Project North Star"
description: "Mission, Vision, and Position for this project in one read"
status: active
---

# North Star

## Mission

We exist to ...

## Vision

In three years, ...

## Position

We are the project that ...
```

## Configuration

In `APSS.yaml` (owned by APS-V1-0000.CF01); this substandard registers
the kebab-case key `north-star` under the parent `docs` slug. Per the
parent standard's absence-equals-enabled convention (Section 3.2 of
the parent spec), a project adopting every default writes nothing for
this substandard. To override the default location:

```yaml
docs:
  north-star:
    location: my-north-star.md  # docs-root-relative; lands at <docs.root>/my-north-star.md
```

Disabling: set `disable: true`. Absence is the default.
Backlinking from `CLAUDE.md` and `AGENTS.md` is enforced by
the parent standard's DOC03-self-reference check, not by this
substandard.

## Why three sections, not two and not four

The substandard's purpose is keeping plans, designs, and
implementations aligned with the project's intent. The three sections
are the smallest set that survives that load:

- **Mission** answers "what do we exist to do". Present tense, names
  the problem and the audience, not the solution.
- **Vision** answers "where are we going". A concrete future state
  measured against a date, so a reviewer can ask "are we on track"
  and get a non-vacuous answer.
- **Position** answers "where do we sit relative to the alternatives".
  The shortest of the three; one paragraph naming the peers and what
  this project does that they do not.

A North Star without one of those three sections is doing one of the
other two jobs badly. The validator therefore treats all three as
required.

## Error Codes

| Code | Severity | Description |
|------|----------|-------------|
| `PV01-document-missing` | error | No file at `docs.north-star.location`. |
| `PV01-frontmatter-missing` | error | Document lacks frontmatter. |
| `PV01-frontmatter-field-missing` | error | Document missing `name`, `description`, or `status`. |
| `PV01-missing-mission-section` | error | Document missing `## Mission` heading. |
| `PV01-missing-vision-section` | error | Document missing `## Vision` heading. |
| `PV01-missing-position-section` | error | Document missing `## Position` heading. |
| `PV01-invalid-status` | error | `status` is not one of the allowed values. |
| `PV01-superseded-without-pointer` | error | `status: superseded` requires `superseded_by` in frontmatter. |
| `PV01-deprecated-active` | warning | The only North Star document is `deprecated`. |
