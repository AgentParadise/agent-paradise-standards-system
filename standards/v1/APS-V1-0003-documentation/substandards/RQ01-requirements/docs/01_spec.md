---
name: "Requirements Specification"
description: "Normative rules for an optional per-system requirements document and the requirements gate agents apply before building"
---

# Requirements Specification

**Substandard:** APS-V1-0003.RQ01
**Parent:** APS-V1-0003 (Documentation and Context Engineering)
**Version:** 0.1.0

Key words: MUST, MUST NOT, SHOULD, SHALL, MAY, OPTIONAL per [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## 1. Why this substandard exists

The North Star (PV01) says why a project exists and where it is going.
It does not say what a given system must be and do, so it cannot settle
"should we build this feature?" A requirements document can. It turns
the North Star into numbered, measurable requirements with an owner, and
it gives agents a gate: a feature the requirements do not cover is not
built until the requirements owner amends them.

The document is OPTIONAL. Small projects and early prototypes do not need
one, and a validator that demanded one would teach adopters to write
empty documents. When a system does keep one, this substandard keeps it
parseable, measurable, and versioned.

## 2. Optional, opt-in, and location

### 2.1 Optional by default (RQ01-document-missing)

RQ01 follows the parent's absence-equals-enabled convention (parent spec
Section 3.2): the rule is on unless `docs.requirements.disable: true`.
What is optional is the document, not the rule:

- `docs.requirements.locations` absent: the validator looks at the
  default location `<docs.root>/requirements.md`. If a file is there it
  MUST conform to this spec. If no file is there the project passes.
- `docs.requirements.locations` present: declaring a location is the
  opt-in. Every listed document MUST exist and conform. A missing one is
  `RQ01-document-missing` (error).

There is no `required: true` flag; the parent spec forbids per-feature
optional flags (Section 3.4). Writing a location is the declaration.

### 2.2 Location (RQ01-absolute-location, RQ01-location-out-of-tree)

Locations are docs-root-relative, per the parent spec Section 3.3
path-resolution rule, exactly like PV01's `location`. The default is
`requirements.md`, which resolves to `docs/requirements.md` beside the
North Star at `docs/north-star.md`. A value starting with `/` is
`RQ01-absolute-location` (error); a value with `..` segments is
`RQ01-location-out-of-tree` (error).

One project often holds several systems. A list of locations gives each
system its own document without a new key:

```yaml
docs:
  requirements:
    locations:
      - requirements.md                  # the product as a whole
      - systems/billing/requirements.md  # one system
```

## 3. Frontmatter (RQ01-frontmatter-missing, RQ01-frontmatter-field-missing)

The document MUST start with a YAML frontmatter block containing:

| Field | Required | Description |
|-------|----------|-------------|
| `name` | YES | Human readable title (parent index column). |
| `description` | YES | One line summary (parent index column). |
| `owner` | YES | The requirements owner: the person or role who amends the requirements and answers the requirements gate (Section 9). |
| `status` | YES | Lifecycle status (Section 3.1). |
| `version` | YES | `MAJOR.MINOR.PATCH` of the requirements themselves (Section 8). Diagnostic: `RQ01-invalid-version`. |
| `last_reviewed` | YES | `YYYY-MM-DD` date the owner last reviewed the whole document. Diagnostic: `RQ01-invalid-date`. |
| `serves` | YES | Relative path, from this file, to the North Star (PV01) document the requirements serve. It MUST resolve to a file. Diagnostic: `RQ01-serves-unresolved`. |
| `superseded_by` | conditional | Required when `status == superseded`. |

### 3.1 Status (RQ01-invalid-status, RQ01-superseded-without-pointer)

`status` uses the parent's shared vocabulary (parent spec Section 8.1):
`proposed`, `accepted`, `deprecated`, `superseded`. Requirements pick
`accepted` for "in force", like ADRs, because the owner ratifies them at
a moment. A draft is `proposed`. Any other value is `RQ01-invalid-status`
(error). `superseded` without `superseded_by` is
`RQ01-superseded-without-pointer` (error).

## 4. Required sections (RQ01-missing-section)

The body MUST contain these `##` headings (case insensitive). Additional
sections are permitted.

1. `## Vision`: one short paragraph, the system's part of the North Star.
2. `## Roles`: the user roles of the platform (Section 6).
3. `## Requirements`: pillars and numbered requirements (Section 5).
4. `## User Stories`: stories per role (Section 6).
5. `## Later and Out of Scope`: work that is deliberately not built now (Section 7).
6. `## Change Log`: dated, versioned requirement changes (Section 8).

## 5. Requirements

### 5.1 Pillars

Requirements are grouped into pillars (or groups), one `###` heading per
pillar under `## Requirements`, for example `### P3: Reliability`. The
pillar heading is free text; its code (`P3`, the text before the colon) is
the prefix of the ids inside it. A requirement row under no pillar heading
is `RQ01-requirement-outside-pillar`; a row whose id prefix (the part before
the `.`) differs from its pillar's code is
`RQ01-requirement-pillar-mismatch` (errors).

Content inside fenced code blocks (CommonMark: backtick or tilde fences of
three or more, closed by the same character at least as long) is
illustration, not structure: no check
in this spec reads it, so a fenced table neither satisfies nor fails any
rule.

### 5.2 Requirement rows

Each requirement is one row of a Markdown table with four columns, in
this order:

```markdown
| ID | Requirement | Measure | Check |
|----|-------------|---------|-------|
| P3.5 | Exports finish within one minute for 10k items. | p95 export time in the nightly benchmark. | checked: `bench-export` |
```

- **ID** MUST match `^[A-Z][A-Z0-9]*\d\.\d+$` (for example `P3.5`,
  `SEC1.2`) and MUST be unique in the document. Ids are stable: a removed
  requirement's id is never reused. Diagnostics:
  `RQ01-requirement-id-invalid`, `RQ01-requirement-id-duplicate` (errors).
- **Requirement** is the requirement in plain words. Diagnostic when
  empty: `RQ01-requirement-text-missing` (error).
- **Measure** says how you know it is met. Diagnostic when empty:
  `RQ01-measure-missing` (error).
- **Check** is the check status, optionally followed by `: <check name>`.
  Diagnostics: `RQ01-check-status-missing`, `RQ01-invalid-check-status`
  (errors).

A literal `|` inside a cell is written `\|`. The table header row (first
cell `ID`) and separator row are skipped. A `## Requirements` section with
no rows is `RQ01-no-requirements` (error).

### 5.3 Check status

| Value | Meaning |
|-------|---------|
| `checked` | A machine check proves it: a test, gate, lint or benchmark. The check MUST be named (`checked: <name>`); otherwise `RQ01-check-unnamed` (error). |
| `partial` | A machine check covers part of it. SHOULD name the check. |
| `unchecked` | In force, but only people check it today. |
| `later` | In force; the check is planned and not built yet. |

## 6. Roles and user stories (RQ01-no-roles, RQ01-no-user-stories, RQ01-story-role-unknown)

`## Roles` lists every user role of the platform as a top-level bullet:
`- **<Role>**: <who they are>`. A section with no bullets is
`RQ01-no-roles` (error). Typical roles are an end user, a creator or
designer, an agent, and an operator; each project names its own.

`## User Stories` holds stories in the form:

> As a <role>, I want <goal>, so that <reason>.

Stories SHOULD be grouped by role under `###` headings. At least one story
MUST exist (`RQ01-no-user-stories`, error). A story whose role is not
listed under `## Roles` is `RQ01-story-role-unknown` (warning).

## 7. Later and out of scope

`## Later and Out of Scope` keeps deferred and rejected work apart from
the requirements so it is not built by accident. Items here are not
requirements: they carry no check status and the requirements gate
treats them as not covered. Moving an item into `## Requirements` is a
requirement change (Section 8). The section MAY be empty, but it MUST
exist, so that "we decided not to" has a home.

## 8. Change log and versioning (RQ01-changelog-empty, RQ01-changelog-entry-invalid, RQ01-changelog-version-mismatch)

Requirement changes are versioned and dated. `## Change Log` lists one
top-level bullet per change, newest first:

```markdown
- 1.2.0 (2026-11-01): Added P3.5 (export time). Moved P2.4 to Later.
```

- Every top-level bullet MUST match
  `- <MAJOR.MINOR.PATCH> (<YYYY-MM-DD>): <change>`
  (`RQ01-changelog-entry-invalid`, error).
- At least one entry MUST exist (`RQ01-changelog-empty`, error).
- The newest (first) entry's version SHOULD equal frontmatter `version`
  (`RQ01-changelog-version-mismatch`, warning).

Version bumps follow SemVer applied to the requirements: MAJOR when a
requirement is removed or weakened, MINOR when one is added, PATCH for
wording that does not change meaning.

## 9. Agent obligations: read first, then the requirements gate

This section is normative for agents working in a project that keeps a
requirements document. It is not machine-checked.

1. **Read before building.** Before building or changing a system, an
   agent MUST read that system's requirements document (and the North
   Star it serves).
2. **The requirements gate.** When a proposed feature or change is not
   covered by an `accepted` requirement, the agent MUST NOT just build
   it. It MUST raise the gap with the requirements owner (frontmatter
   `owner`), propose the amendment (new or changed requirement, measure
   and check), and build only after the owner amends the document. Items
   under `## Later and Out of Scope` count as not covered.
3. **Use the requirements to decide how.** The measure and check of the
   requirements a change touches are the agent's acceptance criteria. A
   change that makes a `checked` requirement's check fail is a regression.
4. **Keep it true.** When an agent lands a check for a requirement, it
   SHOULD update that requirement's check status and name the check, as a
   PATCH change in the change log.

Bug fixes that restore an existing requirement and changes that touch no
behaviour (refactors, docs, tooling) pass the gate without an amendment.

## 10. Configuration

Per the parent's absence-equals-enabled convention, a project writes
nothing under `docs.requirements` unless it declares locations or opts
out:

```yaml
docs:
  requirements:
    locations:                 # opt-in: each listed doc MUST exist
      - requirements.md        # docs-root-relative; lands at <docs.root>/requirements.md
```

`disable: true` turns RQ01 off. Absence is the default and validates
`<docs.root>/requirements.md` only if it exists.

## 11. Error Codes

| Code | Severity | Description |
|------|----------|-------------|
| `RQ01-document-missing` | error | A location listed in `docs.requirements.locations` has no file. |
| `RQ01-absolute-location` | error | A location starts with `/`. |
| `RQ01-location-out-of-tree` | error | A location escapes `docs.root` with `..`. |
| `RQ01-frontmatter-missing` | error | Document has no closed frontmatter block. |
| `RQ01-frontmatter-field-missing` | error | A required frontmatter field is absent or empty. |
| `RQ01-invalid-status` | error | `status` is not `proposed`, `accepted`, `deprecated` or `superseded`. |
| `RQ01-superseded-without-pointer` | error | `status: superseded` without `superseded_by`. |
| `RQ01-invalid-version` | error | `version` is not `MAJOR.MINOR.PATCH`. |
| `RQ01-invalid-date` | error | `last_reviewed` is not `YYYY-MM-DD`. |
| `RQ01-serves-unresolved` | error | `serves` does not resolve to a file. |
| `RQ01-missing-section` | error | A required `##` section is absent. |
| `RQ01-no-requirements` | error | `## Requirements` holds no requirement rows. |
| `RQ01-requirement-id-invalid` | error | Id is not of the form `<PILLAR><n>.<m>`. |
| `RQ01-requirement-id-duplicate` | error | Id is used more than once. |
| `RQ01-requirement-outside-pillar` | error | A requirement row is under no `### <PILLAR>: <name>` heading. |
| `RQ01-requirement-pillar-mismatch` | error | A requirement id's prefix differs from its pillar's code. |
| `RQ01-requirement-text-missing` | error | Requirement cell is empty. |
| `RQ01-measure-missing` | error | Measure cell is empty. |
| `RQ01-check-status-missing` | error | Check cell is empty. |
| `RQ01-invalid-check-status` | error | Check status is not `checked`, `partial`, `unchecked` or `later`. |
| `RQ01-check-unnamed` | error | `checked` without a named check. |
| `RQ01-no-roles` | error | `## Roles` lists no roles. |
| `RQ01-no-user-stories` | error | `## User Stories` holds no story. |
| `RQ01-story-role-unknown` | warning | A story's role is not listed under `## Roles`. |
| `RQ01-changelog-empty` | error | `## Change Log` has no entries. |
| `RQ01-changelog-entry-invalid` | error | A change log entry is not `- <version> (<date>): <change>`. |
| `RQ01-changelog-version-mismatch` | warning | Newest change log version differs from frontmatter `version`. |

## 12. Template (informative)

A complete, valid document is in the parent's
[`examples/requirements.md`](../../../examples/requirements.md). RQ01
ships no installer template on purpose: the parent install contract
materialises templates for every active doc type, and a requirements
document a project did not ask for would defeat the "optional" rule.
