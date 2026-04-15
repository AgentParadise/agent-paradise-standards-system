---
name: "ADR Enforcement Specification"
description: "Normative rules for Architecture Decision Record validation"
---

# ADR Enforcement Specification

**Substandard:** EXP-V1-0004.ADR01  
**Parent:** EXP-V1-0004 (Documentation and Context Engineering)  
**Version:** 0.1.0  

Key words: MUST, MUST NOT, SHOULD, SHALL per [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119).

## 1. ADR Directory (ADR01-001)

The docs root (default `docs/`) MUST contain an ADR directory (default `adrs/`).

- Configurable via `docs.adr.directory` in `.apss/config.toml`
- Full path: `<docs.root>/<docs.adr.directory>/` (e.g., `docs/adrs/`)
- The ADR directory SHOULD contain a `README.md` with a `## Index` section listing all ADRs (managed by the parent standard's index generation)

## 2. Naming Convention (ADR01-002)

Every `.md` file in the ADR directory (excluding `README.md`, `CLAUDE.md`, `AGENTS.md`) MUST match the configured naming pattern.

**Default pattern:** `ADR-\d{3,}-[a-zA-Z0-9-]+\.md`

**Examples:**
- `ADR-001-initial-architecture.md`
- `ADR-042-security.md`
- `ADR-100-api-versioning-strategy.md`

**Format:** `ADR-<number>-<kebab-case-name>.md`

- The number MUST be zero-padded to at least 3 digits
- The name MUST use kebab-case (lowercase letters, digits, hyphens)
- Configurable via `docs.adr.naming_pattern` in `.apss/config.toml`

## 3. Front Matter (ADR01-003)

Every ADR file MUST contain a YAML front matter block with:

| Field | Required | Description |
|-------|----------|-------------|
| `name` | YES | Human-readable ADR title |
| `description` | YES | One-line summary of the decision |

**Example:**

```yaml
---
name: "Security Architecture"
description: "Defines authentication, authorization, and data protection patterns"
---
```

## 4. Required ADR Keywords (ADR01-004)

Projects MAY configure required ADR topics via `docs.adr.required_adr_keywords`:

```toml
[docs.adr]
required_adr_keywords = ["security", "testing", "deployment"]
```

For each keyword, at least one file matching `ADR-\d+-<keyword>\.md` MUST exist. The number prefix is flexible — only the keyword suffix is enforced.

**Example:** With `required_adr_keywords = ["security"]`, any of these satisfy the requirement:
- `ADR-001-security.md`
- `ADR-042-security.md`

This mechanism lets the standard grow a base set of required architectural topics as the project matures.

## 5. Backlinking (ADR01-005)

When `docs.adr.backlinking` is enabled (default: `true`), implementation files that are governed by an ADR SHOULD contain a reference to the ADR identifier.

**ADR identifier format:** `ADR-XXX-<name>` (the filename without `.md`)

**Example:** A source file implementing the security ADR should contain:

```rust
// Implements ADR-001-security
```

This enables bidirectional discovery:
- From code → ADR: developers and agents find the governing decision
- From ADR → code: `grep ADR-001-security` finds all implementing files

## 6. Dead ADR Reference Detection (ADR01-009)

When backlinking is enabled, the validator scans source files across the repository for `ADR-XXX-<name>` patterns and checks that each referenced ADR actually exists in the ADR directory.

This catches stale references to ADRs that have been renamed, renumbered, or deleted.

**Scanned file types:** `.rs`, `.py`, `.ts`, `.tsx`, `.js`, `.jsx`, `.go`, `.java`, `.kt`, `.rb`, `.sh`, `.yaml`, `.yml`, `.toml`, `.json`, `.md`

**Excluded from scanning:** Hidden directories, configured `exclude_dirs`, and files inside the ADR directory itself.

**Severity:** warning (ADR01-009)

**Controlled by:** `docs.adr.backlinking` in `.apss/config.toml`

## 7. Required ADR Headers (ADR01-010)

Every ADR file SHOULD contain the standard ADR section headers:

- `## Context` — the forces at play
- `## Decision` — the change being proposed or made
- `## Consequences` — what happens as a result

Header matching is case-insensitive and tolerates extra whitespace.

**Severity:** warning (ADR01-010)

## 8. ADR Context Files (ADR01-007, ADR01-008)

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
- Missing file: error (ADR01-007)
- File exists but lacks referencing guidance: warning (ADR01-008)

---

## 9. Configuration

```toml
[docs.adr]
enabled = true                                    # Enable/disable ADR validation
directory = "adrs"                                # ADR directory name under docs root
naming_pattern = "ADR-\\d{3,}-[a-zA-Z0-9-]+\\.md" # File naming regex
required_adr_keywords = []                         # Required topic keywords
backlinking = true                                 # Enforce backlinks in implementation files
```

## 10. Error Codes

| Code | Severity | Description |
|------|----------|-------------|
| ADR01-001 | error | ADR directory does not exist |
| ADR01-002 | error | File does not match naming pattern |
| ADR01-003 | error | Missing or incomplete front matter |
| ADR01-004 | error | Required ADR keyword not satisfied |
| ADR01-005 | warning | Implementation file missing ADR backlink |
| ADR01-006 | error | Invalid naming regex in configuration |
| ADR01-007 | error | ADR directory missing CLAUDE.md or AGENTS.md |
| ADR01-008 | warning | ADR context file lacks ADR referencing guidance |
| ADR01-009 | warning | Source file references non-existent ADR (dead reference) |
| ADR01-010 | warning | ADR file missing required section header |
