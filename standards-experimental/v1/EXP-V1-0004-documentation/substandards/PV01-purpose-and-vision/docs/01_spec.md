---
name: "North Star Specification"
description: "Normative rules for the project's single North Star document covering Mission, Vision, and Position"
---

# North Star Specification

**Substandard:** EXP-V1-0004.PV01
**Parent:** EXP-V1-0004 (Documentation and Context Engineering)
**Version:** 0.1.0

Key words: MUST, MUST NOT, SHOULD, SHALL per [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## 1. Why this substandard exists

Plans drift. Designs drift. Implementations drift the most. The
cheapest correction is to keep one short, authoritative document that
names what the project exists to do, the future state it is heading
for, and where it sits relative to its peers. Agents read it on a
fresh start; reviewers compare proposals against it; product changes
refer back to it. A project without a North Star document has no
shared answer to "is this still the thing we are trying to build?"

This substandard makes sure that document exists, is parseable, and is
findable from a fresh start. It is the canonical "single-document
doc type" example of the parent standard: one file, structured
frontmatter, required H2 sections, surfaced in the docs root index by
its `description`.

## 2. Document location (PV01-document-missing)

A file MUST exist at `<docs.root>/<docs.north-star.location>`
(default: `<docs.root>/north-star.md`, which resolves to
`docs/north-star.md` when `docs.root` carries its default).

The `location` value is **docs-root-relative** per the parent spec
Section 3.3 normative path-resolution paragraph; the parent standard
unifies this convention across every doc type so a reader who
customises `docs.root` does not have to guess. A leading `/` in
`location` is treated as a hard error
(`PV01-absolute-location`); embed `..` segments to escape the docs
root are also rejected (`PV01-location-out-of-tree`).

Diagnostic: `PV01-document-missing` (error). Hint: "Create the file at
`<resolved-location>` or set `docs.north-star.disable = true` in
`APSS.yaml`."

## 3. Frontmatter (PV01-frontmatter-missing, PV01-frontmatter-field-missing)

The document MUST start with a YAML frontmatter block containing:

| Field | Required | Description |
|-------|----------|-------------|
| `name` | YES | Human readable title (typically `"<Project Name> North Star"`). |
| `description` | YES | One line summary of what this project exists to do. |
| `status` | YES | Lifecycle status (Section 6). |
| `superseded_by` | conditional | Required when `status == superseded`. Value is the relative path to the replacement doc. |

Diagnostics: `PV01-frontmatter-missing` (error), `PV01-frontmatter-field-missing` (error).

Frontmatter parsing rules are inherited from the parent spec,
Section 4.1.

## 4. Required sections (PV01-missing-mission-section, PV01-missing-vision-section, PV01-missing-position-section)

The document body MUST contain three top level (`##`) headings, in
this order:

1. `## Mission` (PV01-missing-mission-section, error).
   The reason the project exists. Present tense, one to three
   paragraphs. Names the problem and the audience, not the solution.

2. `## Vision` (PV01-missing-vision-section, error).
   The intended state of the world at a specific future point
   (typically one to three years). Concrete enough that a reviewer can
   ask "are we on track" and get a non-vacuous answer.

3. `## Position` (PV01-missing-position-section, error).
   Where the project sits relative to its alternatives. One paragraph
   naming the closest peers and what this project does that they do
   not. The smallest of the three sections; the operator's escape
   hatch when "Mission" and "Vision" alone could be mistaken for any
   other project's.

Heading matching is case insensitive and tolerates trailing
whitespace. Additional sections are permitted (for example, a
`## Non-Goals` section that scopes ambition); only the three above
are enforced.

All three sections are required (severity `error`). Earlier drafts of
this substandard treated the third section as a warning; the rewrite
in this revision treats Mission, Vision, and Position as equal
pillars, so missing any one is an error. See
[`docs/proposals/north-star-shape-options.md`](../../../../../../../docs/proposals/north-star-shape-options.md)
in the repo root for the reasoning behind the choice.

## 5. Backlinking from root context files (parent rule)

The root `CLAUDE.md` and `AGENTS.md` MUST reference this document so
agents find it on a fresh start. This is checked by the parent
standard's `root-self-reference-missing` rule (DOC03-self-reference);
this substandard does not duplicate the check.

Implementation code files that are governed by this document MAY
backlink it using the `PV01-<NUMBER>-<NAME>` token form described in
Section 7 of the parent spec. The default project has a single North
Star document, so backlinks are not required, but they MUST be
honoured when present.

## 6. Lifecycle status (PV01-invalid-status, PV01-superseded-without-pointer)

`status` MUST be one of: `proposed`, `active`, `deprecated`,
`superseded`. PV01 chooses `active` as the "in force" term per the
parent spec Section 8.1 per-doc-type table: the parent's
`accepted / active` slash means "every doc type picks one"; ADR01
uses `accepted` (consistent with the Nygard tradition) and PV01 /
RETRO01 use `active`. Adopters who write `accepted` on a North Star
get `PV01-invalid-status` (error) with a hint pointing them at the
per-doc-type vocabulary table.

- `proposed`: under discussion. Validators MUST NOT block a project
  for having `status: proposed`; this is the normal state during
  project bootstrap.
- `active`: current source of truth.
- `deprecated`: discouraged but kept for historical context. The
  validator MUST emit `PV01-deprecated-active` (warning) if the
  document is the only North Star document and is `deprecated`; a
  project with no active North Star is a smell.
- `superseded`: replaced by another North Star document. Frontmatter
  MUST include `superseded_by: <relative-path-to-new-doc>`. Diagnostic
  when missing: `PV01-superseded-without-pointer` (error).

Diagnostic for an unrecognized value: `PV01-invalid-status` (error).

## 7. Configuration

Per the parent standard's absence-equals-enabled convention
(Section 3.2 of the parent spec), a project adopting every default for
this substandard writes nothing under `docs.north-star`. The substandard
is opt-out, not opt-in.

To override the default location:

```yaml
docs:
  north-star:
    location: docs/00_north_star.md
```

A project that legitimately has no North Star document (rare) sets
`disable: true`. The default for `docs.north-star` is the section being
absent.
Customizing `location` is supported (for example `docs/00_north_star.md` or
`NORTH-STAR.md` at the repo root) but the default is recommended so
agents looking for the file find it in one place.

## 8. Error Codes

| Code | Severity | Description |
|------|----------|-------------|
| `PV01-document-missing` | error | The configured `location` does not exist. |
| `PV01-frontmatter-missing` | error | Document has no frontmatter block. |
| `PV01-frontmatter-field-missing` | error | Required frontmatter field absent. |
| `PV01-missing-mission-section` | error | Document missing `## Mission` heading. |
| `PV01-missing-vision-section` | error | Document missing `## Vision` heading. |
| `PV01-missing-position-section` | error | Document missing `## Position` heading. |
| `PV01-invalid-status` | error | `status` is not one of `proposed`, `active`, `deprecated`, `superseded`. |
| `PV01-superseded-without-pointer` | error | `status: superseded` without `superseded_by` frontmatter field. |
| `PV01-deprecated-active` | warning | The only North Star document is `deprecated`. |

## 9. Template (informative)

```markdown
---
name: "<Project> North Star"
description: "Mission, Vision, and Position for this project in one read"
status: proposed
---

# North Star

## Mission

We exist to ...

## Vision

In three years, ...

## Position

We are the project that ...
```

The substandard MAY ship this template under
`templates/docs/north-star.md` in a follow up PR so the unified
installer materialises it on first install. This spec does not require
the template ship in this PR; the contract above is what the validator
enforces.

## 10. Implementation status

This substandard is scaffolded in this PR. The validator implementation
(`src/lib.rs`) and tests will land in a follow up. The contract above
is what the implementation MUST satisfy. The diagnostic code constants
exposed by the crate today reflect the rewrite to Mission, Vision, and
Position; downstream code referencing the older
`PV01-missing-purpose-section` and `PV01-missing-non-goals-section`
constants MUST be updated.
