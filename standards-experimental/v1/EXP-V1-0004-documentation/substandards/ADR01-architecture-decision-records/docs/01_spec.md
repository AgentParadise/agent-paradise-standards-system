---
name: "ADR Enforcement Specification"
description: "Normative rules for Architecture Decision Record validation"
---

# ADR Enforcement Specification

**Substandard:** EXP-V1-0004.ADR01  
**Parent:** EXP-V1-0004 (Documentation and Context Engineering)  
**Version:** 0.1.0  

Key words: MUST, MUST NOT, SHOULD, SHALL per [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

This substandard's vocabulary, lifecycle, naming guidance, and shipped template default to the canonical community resource at <https://github.com/architecture-decision-record/architecture-decision-record>. The Michael Nygard template referenced in section 7 of this spec is the one cited by that resource. See ADR01 overview (`docs/00_overview.md`) for the cited definitions of ADR, AD, ADL, ASR, AKM and the four characteristics of a good ADR.

## 1. ADR Directory (ADR01-dir-not-found)

The docs root (default `docs/`) MUST contain an ADR directory (default `adrs/`).

- Configurable via `docs.adr.directory` in `apss.yaml`
- Full path: `<docs.root>/<docs.adr.directory>/` (e.g., `docs/adrs/`)
- The ADR directory SHOULD contain a `README.md` with a `## Index` section listing all ADRs (managed by the parent standard's index generation)

## 2. Naming Convention (ADR01-invalid-naming)

Every `.md` file in the ADR directory (excluding `README.md`, `CLAUDE.md`, `AGENTS.md`) MUST match the configured naming pattern.

**Default pattern:** `ADR-\d{3,5}-[a-zA-Z0-9-]+\.md`

**Examples:**
- `ADR-001-initial-architecture.md`
- `ADR-042-security.md`
- `ADR-100-api-versioning-strategy.md`

**Format:** `ADR-<number>-<kebab-case-name>.md`

- The number MUST be zero-padded to at least 3 digits (maximum 5 digits)
- The name MUST use kebab-case (lowercase letters, digits, hyphens)
- Configurable via `docs.adr.naming_pattern` in `apss.yaml`

## 3. Front Matter (ADR01-missing-frontmatter, ADR01-invalid-status)

Every ADR file MUST contain a YAML front matter block with:

| Field | Required | Description |
|-------|----------|-------------|
| `name` | YES | Human-readable ADR title |
| `description` | YES | One-line summary of the decision |
| `status` | YES | Lifecycle status: `proposed`, `accepted`, `deprecated`, or `superseded` |

ADRs should not be revised - they should be superseded by a new ADR. The `status` field enables agents and tooling to skip superseded decisions and focus on active ones.

**Example:**

```yaml
---
name: "Security Architecture"
description: "Defines authentication, authorization, and data protection patterns"
status: accepted
---
```

## 4. Required ADR Keywords (ADR01-missing-required-keyword)

Projects MAY configure required ADR topics via `docs.adr.required_adr_keywords`:

```yaml
docs:
  adr:
    required_adr_keywords: ["security", "testing", "deployment"]
```

For each keyword, at least one file whose stem matches the configured
`docs.adr.naming_pattern` and ends in `-<keyword>` MUST exist. The
keyword matcher is derived from `naming_pattern`, so a project that
customises the prefix (e.g., `DEC-...`) gets a keyword check that follows
its convention. The number prefix is flexible; only the keyword suffix is
enforced.

**Example:** With `required_adr_keywords: ["security"]`, any of these satisfy the requirement:
- `ADR-001-security.md`
- `ADR-042-security.md`

This mechanism lets the standard grow a base set of required architectural topics as the project matures.

## 5. Backlinking (ADR01-missing-backlink, reserved)

Backlinking is an always-on part of the standard: implementation files reference
the ADR they implement so context is never lost across plan, design, and impl.
The disable flag exists only as a per-project escape hatch; it is not the
intended default.

When `docs.backlinking.disable` is `false`, implementation files that are governed by an ADR SHOULD contain a reference to the ADR identifier.

**ADR identifier format:** `ADR-XXX-<name>` (the filename without `.md`)

**Example:** A source file implementing the security ADR should contain:

```rust
// Implements ADR-001-security
```

This enables bidirectional discovery:
- From code → ADR: developers and agents find the governing decision
- From ADR → code: `grep ADR-001-security` finds all implementing files

## 6. Dead ADR Reference Detection (ADR01-dead-reference, ADR01-superseded-reference)

When backlinking is enabled, the validator scans source files across the
repository for ADR identifiers and checks that each referenced ADR actually
exists in the ADR directory. The reference-extraction regex is derived from
the configured `docs.adr.naming_pattern`, so projects that customise the
prefix (e.g., `DEC-...`) still get their references scanned.

This catches stale references to ADRs that have been renamed, renumbered, or
deleted. Live references to an ADR whose status is `superseded` or
`deprecated` are flagged separately as `ADR01-superseded-reference`.

**Scanned file types:** `.rs`, `.py`, `.ts`, `.tsx`, `.js`, `.jsx`, `.go`, `.java`, `.kt`, `.rb`, `.sh`, `.yaml`, `.yml`, `.toml`, `.json`, `.md`

**Excluded from scanning:** Hidden directories, configured `exclude_dirs`, and files inside the ADR directory itself.

**Severity:** warning (ADR01-dead-reference, ADR01-superseded-reference)

**Controlled by:** `docs.backlinking.disable` in `apss.yaml`

## 7. Required ADR Headers (ADR01-missing-header)

Every ADR file SHOULD contain the standard ADR section headers:

- `## Context` - the forces at play
- `## Decision` - the change being proposed or made
- `## Consequences` - what happens as a result

Header matching is case-insensitive and tolerates extra whitespace.

**Severity:** warning (ADR01-missing-header)

## 8. ADR Context Files (ADR01-missing-context-file, ADR01-context-missing-guidance)

The ADR directory MUST contain `CLAUDE.md` and `AGENTS.md` files that provide guidance on how ADRs should be referenced in implementation code.

These context files serve a specific purpose beyond the parent standard's generic context file requirement: they instruct agents and developers to add a comment block at the top of files that implement an ADR, keeping the governing decision in context when making updates.

**Required content:** Each file SHOULD mention how to reference ADRs in code. The validator checks for keywords like `ADR-`, `backlink`, `reference`, or `comment block`.

**Example CLAUDE.md / AGENTS.md for the ADR directory:**

```markdown
# Architecture Decision Records

Files that implement an ADR should reference it in a comment block at the top of the file.

Example:
```
// Implements ADR-001-security
```

This keeps agents and developers in context when making updates to the codebase.
Use `grep ADR-001-security` to find all files implementing a given ADR.
```

**Severity:**
- Missing file: error (ADR01-missing-context-file)
- File exists but lacks referencing guidance: warning (ADR01-context-missing-guidance)

Guidance detection is case-insensitive, so casing variants like `Reference`
or `BACKLINK` do not produce false `ADR01-context-missing-guidance` warnings.

---

## 9. Configuration

```yaml
docs:
  adr:
    disable:               false                    # Default-on; set true to skip ADR validation
    directory:             "adrs"                   # ADR directory name under docs root
    naming_pattern:        "ADR-\\d{3,5}-[a-zA-Z0-9-]+\\.md" # File naming regex
    required_adr_keywords: []                       # Required topic keywords
```

## 10. ADR Template

The standard provides a default ADR template with the required front matter fields and section headers. Tooling MAY use this template to scaffold new ADRs:

```bash
aps run docs new adr <name>     # Scaffold a new ADR from template (future CLI)
```

The template includes:
- Front matter with `name`, `description`, and `status: proposed`
- Required sections: `## Context`, `## Decision`, `## Consequences`

Projects MAY customize the template. An example template is provided in the standard's `examples/adr-template.md`.

---

## 11. Error Codes

Codes follow the form `ADR01-<verb-phrase>`. The substandard prefix keeps the
rule domain visible; the suffix reads as plain English in CLI output. This
matches the operator invariant for human-readable codes.

| Code | Severity | Description |
|------|----------|-------------|
| ADR01-dir-not-found | error | ADR directory does not exist |
| ADR01-invalid-naming | error | File does not match naming pattern |
| ADR01-missing-frontmatter | error | Missing or incomplete front matter |
| ADR01-missing-required-keyword | error | Required ADR keyword not satisfied |
| ADR01-missing-backlink | - | Reserved (forward backlink enforcement not feasible) |
| ADR01-invalid-naming-regex | error | Invalid naming regex in configuration |
| ADR01-missing-context-file | error | ADR directory missing CLAUDE.md or AGENTS.md |
| ADR01-context-missing-guidance | warning | ADR context file lacks ADR referencing guidance |
| ADR01-dead-reference | warning | Source file references non-existent ADR (dead reference) |
| ADR01-missing-header | warning | ADR file missing required section header |
| ADR01-invalid-status | error | ADR missing or invalid `status` field |
| ADR01-superseded-reference | warning | Source file references a superseded or deprecated ADR |
