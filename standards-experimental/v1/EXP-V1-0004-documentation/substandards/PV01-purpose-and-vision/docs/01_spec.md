---
name: "Purpose and Vision Specification"
description: "Normative rules for the project's Purpose and Vision document"
---

# Purpose and Vision Specification

**Substandard:** EXP-V1-0004.PV01
**Parent:** EXP-V1-0004 (Documentation and Context Engineering)
**Version:** 0.1.0

Key words: MUST, MUST NOT, SHOULD, SHALL per [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## 1. Why this substandard exists

Plans drift. Designs drift. Implementations drift the most. The cheapest correction is to keep a single authoritative document that names what the project is for and what it refuses to be. Agents read it on a fresh start; reviewers compare proposals to it; product changes refer to it. A project without a Purpose and Vision document has no shared answer to "is this still the thing we are trying to build?"

This substandard makes sure that document exists, is parseable, and is findable from a fresh start.

## 2. Document location (PV01-document-missing)

A file MUST exist at `docs.purpose-and-vision.location` (default: `docs/vision.md`).

Diagnostic: `PV01-document-missing` (error). Hint: "Create the file at `<location>` or set `docs.purpose-and-vision.disable = true` in `apss.yaml`."

## 3. Frontmatter (PV01-frontmatter-missing, PV01-frontmatter-field-missing)

The document MUST start with a YAML frontmatter block containing:

| Field | Required | Description |
|-------|----------|-------------|
| `name` | YES | Human readable title (typically `"<Project Name> Purpose and Vision"`). |
| `description` | YES | One line summary of what this project exists to do. |
| `status` | YES | Lifecycle status (Section 6). |
| `superseded_by` | conditional | Required when `status == superseded`. Value is the relative path to the replacement doc. |

Diagnostics: `PV01-frontmatter-missing` (error), `PV01-frontmatter-field-missing` (error).

Frontmatter parsing rules are inherited from the parent spec, Section 4.1.

## 4. Required sections (PV01-missing-purpose-section, PV01-missing-vision-section, PV01-missing-non-goals-section)

The document body MUST contain three top level (`##`) headings, in this order:

1. `## Purpose` (PV01-missing-purpose-section, error).
   The reason the project exists. Present tense, one to three paragraphs. Names the problem and the audience, not the solution.

2. `## Vision` (PV01-missing-vision-section, error).
   The intended state of the world at a specific future point (typically one to three years). Concrete enough that a reviewer can ask "are we on track?" and get a non vacuous answer.

3. `## Non-Goals` (PV01-missing-non-goals-section, warning).
   What the project will not do, especially adjacent problems it could plausibly tackle. The most important section for keeping scope honest.

Heading matching is case insensitive and tolerates trailing whitespace. Additional sections are permitted; only the three above are enforced.

## 5. Backlinking from root context files (parent rule)

The root `CLAUDE.md` and `AGENTS.md` MUST reference this document so agents find it on a fresh start. This is checked by the parent standard's `root-self-reference-missing` rule (DOC03-self-reference); this substandard does not duplicate the check.

Implementation code files that are governed by this document MAY backlink it using the `PV01-<NUMBER>-<NAME>` token form described in Section 7 of the parent spec. The default project has a single Purpose and Vision document, so backlinks are not required, but they MUST be honoured when present.

## 6. Lifecycle status (PV01-invalid-status, PV01-superseded-without-pointer)

`status` MUST be one of: `proposed`, `active`, `deprecated`, `superseded`.

- `proposed`: under discussion. Validators MUST NOT block a project for having `status: proposed`; this is the normal state during project bootstrap.
- `active`: current source of truth.
- `deprecated`: discouraged but kept for historical context. The validator MUST emit `PV01-deprecated-active` (warning) if the document is the only Purpose and Vision document and is `deprecated`; a project with no active Purpose and Vision is a smell.
- `superseded`: replaced by another Purpose and Vision document. Frontmatter MUST include `superseded_by: <relative-path-to-new-doc>`. Diagnostic when missing: `PV01-superseded-without-pointer` (error).

Diagnostic for an unrecognized value: `PV01-invalid-status` (error).

## 7. Configuration

```yaml
docs:
  purpose-and-vision:
    disable:  false
    location: docs/vision.md
```

A project that legitimately has no Purpose and Vision document (rare) sets `disable: true`. Customizing `location` is supported (for example `docs/00_purpose.md` or `VISION.md` at the repo root) but the default is recommended.

## 8. Error Codes

| Code | Severity | Description |
|------|----------|-------------|
| `PV01-document-missing` | error | The configured `location` does not exist. |
| `PV01-frontmatter-missing` | error | Document has no frontmatter block. |
| `PV01-frontmatter-field-missing` | error | Required frontmatter field absent. |
| `PV01-missing-purpose-section` | error | Document missing `## Purpose` heading. |
| `PV01-missing-vision-section` | error | Document missing `## Vision` heading. |
| `PV01-missing-non-goals-section` | warning | Document missing `## Non-Goals` heading. |
| `PV01-invalid-status` | error | `status` is not one of `proposed`, `active`, `deprecated`, `superseded`. |
| `PV01-superseded-without-pointer` | error | `status: superseded` without `superseded_by` frontmatter field. |
| `PV01-deprecated-active` | warning | The only Purpose and Vision document is `deprecated`. |

## 9. Template (informative)

```markdown
---
name: "<Project> Purpose and Vision"
description: "<One-line summary>"
status: proposed
---

# Purpose and Vision

## Purpose

We exist to ...

## Vision

In three years, ...

## Non-Goals

We will not ...
```

The substandard MAY ship this template under `examples/purpose-template.md` in a follow up PR; this spec does not require it.

## 10. Implementation status

This substandard is scaffolded in this PR. The validator implementation (`src/lib.rs`) and tests will land in a follow up. The contract above is what the implementation MUST satisfy.
