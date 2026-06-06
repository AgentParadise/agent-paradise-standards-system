---
name: "Retrospectives Specification"
description: "Normative rules for retrospective documents, append-only history, and required structure"
---

# Retrospectives Specification

**Substandard:** EXP-V1-0004.RETRO01
**Parent:** EXP-V1-0004 (Documentation and Context Engineering)
**Version:** 0.1.0

Key words: MUST, MUST NOT, SHOULD, SHALL per [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## 1. Why this substandard exists

A retrospective is the project's institutional memory: what we tried, what worked, what did not, what we are going to do next time. Without enforcement, retros drift into arbitrary formats; without structure, agents and humans cannot pull patterns out of twenty old retros at once. RETRO01 keeps the retro directory uniform enough that someone three months from now can answer "what did we learn the last three times we shipped a launch?" by reading the index, not by skimming a forest.

The append-only rule is the key invariant: a retrospective is a record of what was true at a point in time. Editing a past retro rewrites history and destroys the very thing that makes retros useful.

## 2. Directory (RETRO01-dir-not-found)

A directory MUST exist at `<docs.root>/<docs.retrospectives.directory>`
(default: `<docs.root>/retrospectives/`, resolving to
`docs/retrospectives/` when `docs.root` carries its default).

The `directory` value is **docs-root-relative** per the parent spec
Section 3.3 path-resolution rule. RETRO01 and ADR01 follow the same
convention so a reader who customises `docs.root` does not have to
guess. A leading `/` (`RETRO01-absolute-directory`) and `..` escapes
(`RETRO01-directory-out-of-tree`) are hard errors.

Diagnostic: `RETRO01-dir-not-found` (error). Hint: "Create the
directory at `<resolved-directory>` or set
`docs.retrospectives.disable = true` in `APSS.yaml`."

The retrospective directory inherits the parent rules: it MUST contain a `README.md` with a `## Index` section (auto generated from frontmatter) and per directory `CLAUDE.md` and `AGENTS.md`.

## 3. Naming (RETRO01-naming-mismatch, RETRO01-invalid-naming-regex)

Every `.md` file in the retrospective directory (excluding `README.md`, `CLAUDE.md`, `AGENTS.md`) MUST match the configured naming pattern.

**Default pattern:** `RETRO-\d{3,5}-[a-zA-Z0-9-]+\.md`

**Examples:**

- `RETRO-001-q1-launch.md`
- `RETRO-042-deployment-failure-postmortem.md`
- `RETRO-10000-five-year-retro.md`

The numeric component MUST support 3 to 5 digits so the project can grow past 1000 retros without changing the standard. (Mirrors the ADR01 rule for consistency.) The file prefix `RETRO-` matches the substandard ID the same way ADR01 substandard uses an `ADR-` file prefix; agents looking at a directory listing should be able to tell the doc type from the filename alone.

A retrospective file MUST be reachable from the directory's `## Index` section once committed.

Diagnostics:
- `RETRO01-naming-mismatch` (error): a file does not match the regex.
- `RETRO01-invalid-naming-regex` (error): the configured regex itself is not valid. Hint: "Check `docs.retrospectives.naming_pattern` in `APSS.yaml`."

## 4. Frontmatter (RETRO01-frontmatter-missing, RETRO01-frontmatter-field-missing)

Every retro MUST start with YAML frontmatter containing:

| Field | Required | Description |
|-------|----------|-------------|
| `name` | YES | Human readable retro title. |
| `description` | YES | One line summary used in the directory index. |
| `date` | YES | ISO 8601 date the retro was written (`YYYY-MM-DD`). |
| `status` | YES | Lifecycle status (Section 6). |
| `superseded_by` | conditional | Required when `status == superseded`. |

Frontmatter parsing rules are inherited from the parent spec, Section 4.1.

## 5. Required sections (RETRO01-missing-section)

Every retro SHOULD contain the following H2 headings, in this order:

1. `## Context`: what we were doing and why.
2. `## What Went Well`: keep the wins explicit so the project notices them.
3. `## What Did Not`: the hard data the retro exists to record.
4. `## Followups`: concrete changes proposed as a result. Each followup SHOULD reference an issue, ADR, or other tracked artifact.

Heading matching is case insensitive and tolerates trailing whitespace. Severity is warning, not error: a retro that omits a section is still better than no retro. Additional sections are permitted.

Diagnostic: `RETRO01-missing-section` (warning) with the missing section name in the hint.

## 6. Lifecycle status (RETRO01-invalid-status)

`status` MUST be one of `proposed`, `active`, `deprecated`, `superseded`. RETRO01 chooses `active` as the "in force" term per the parent spec Section 8.1 per-doc-type table; ADR01 uses `accepted`, PV01 and RETRO01 use `active`. The slash in the parent's `accepted / active` notation means "every doc type picks one"; an adopter who writes `accepted` here gets `RETRO01-invalid-status` (error) with a hint pointing at the per-doc-type vocabulary table.

- `proposed`: draft retro, written but not yet socialized.
- `active`: published retro, kept as is.
- `deprecated`: lessons later proved wrong or replaced; kept for historical context.
- `superseded`: replaced by a later retro that re-litigated the same ground. Frontmatter MUST include `superseded_by`.

Most retros are `active` forever; status exists to handle rare cases honestly.

Diagnostic: `RETRO01-invalid-status` (error).

## 7. Append-only history (RETRO01-history-modified)

The validator MUST, when invoked in `Changed` scope (Section 9.2 of the parent spec), inspect every retro file that appears in the staged change set and emit `RETRO01-history-modified` (error) when an existing retro's content has been modified outside an appended footnote region.

**Definition of an allowed modification:**

- New content added strictly after the last existing line of the file.
- Edits inside an explicit append region delimited by:
  ```
  <!-- RETRO01:append-only -->
  ```
  followed by any content. Multiple append regions are permitted.

**Disallowed:**

- Editing any prior heading, body paragraph, or list item.
- Reformatting (whitespace, list markers) above the append region.

The check MUST use `git diff --cached <retro-path>` (or equivalent staged diff) and reject the commit if any hunk modifies content outside the append region.

When `Full` scope is used (no staged diff context), the append-only check MUST be skipped; the rule is only meaningful relative to a change set.

Configuration: `docs.retrospectives.append_only = true` (default). Setting it to `false` disables the check entirely.

## 8. Configuration

```yaml
docs:
  retrospectives:
    directory:      retrospectives   # docs-root-relative; lands at <docs.root>/retrospectives
    naming_pattern: "RETRO-\\d{3,5}-[a-zA-Z0-9-]+\\.md"
    append_only:    true
```

## 9. Error Codes

| Code | Severity | Description |
|------|----------|-------------|
| `RETRO01-dir-not-found` | error | Retrospective directory does not exist. |
| `RETRO01-naming-mismatch` | error | File does not match the naming pattern. |
| `RETRO01-invalid-naming-regex` | error | Configured naming regex is invalid. |
| `RETRO01-frontmatter-missing` | error | Retro file has no frontmatter. |
| `RETRO01-frontmatter-field-missing` | error | Required frontmatter field absent. |
| `RETRO01-invalid-status` | error | `status` is not one of the allowed values. |
| `RETRO01-missing-section` | warning | Retro missing a required section heading. |
| `RETRO01-history-modified` | error | An existing retro was modified outside an append region. |

## 10. Template (informative)

```markdown
---
name: "<Retro title>"
description: "<One-line summary>"
date: 2026-04-12
status: active
---

# <Retro title>

## Context

We ...

## What Went Well

- ...

## What Did Not

- ...

## Followups

- ...

<!-- RETRO01:append-only -->
```

The substandard MAY ship this template under `examples/retro-template.md` in a follow up PR.

## 11. Implementation status

This substandard is scaffolded in this PR. The validator implementation and tests will land in a follow up. The contract above is what the implementation MUST satisfy.
