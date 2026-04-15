---
name: "Documentation Standard Specification"
description: "Normative rules for documentation structure, index generation, and context files"
---

# EXP-V1-0004 — Documentation and Context Engineering (Canonical Specification)

**Version**: 0.1.0
**Status**: Experimental
**Category**: Governance

---

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

---

## 1. Scope and Authority

This document defines normative rules for documentation consistency and context engineering in projects adopting the APSS documentation standard.

The standard covers two enforcement domains:

- **DOC02**: README indexes and AI context files
- **DOC03**: Root-level context files

ADR enforcement is handled by the **ADR01 substandard** ([EXP-V1-0004.ADR01](../substandards/ADR01-architecture-decision-records/docs/01_spec.md)).

### 1.1 Relationship to APS-V1-0000

This standard complements APS-V1-0000's requirement for `docs/01_spec.md` by enforcing broader documentation structure beyond the standard package itself.

---

## 2. Core Definitions

- **ADR**: Architecture Decision Record — a document capturing a significant architectural decision, its context, rationale, and consequences.
- **Front matter**: A YAML block delimited by `---` at the top of a Markdown file, containing structured metadata.
- **Index**: An auto-generated `## Index` section in README.md that lists documents in a directory with their front matter metadata.
- **Context file**: `CLAUDE.md` or `AGENTS.md` — lightweight files providing AI agents with quick orientation.

---

## 3. Configuration

### 3.1 Config Location

Project-level configuration MUST be located at `.apss/config.toml` relative to the repository root.

### 3.2 Default Behavior

If `.apss/config.toml` does not exist, all defaults SHALL apply. The validator MUST NOT error on a missing config file.

### 3.3 Schema

```toml
schema = "apss.config/v1"

[docs]
enabled = true                    # Master switch for all doc validation
root = "docs"                     # Documentation root directory

[docs.index]
enabled = true                    # Enforce ## Index in README.md
auto_generate = true              # Allow index auto-generation
frontmatter_fields = ["name", "description"]

[docs.context_files]
require_claude_md = true          # Warn on missing CLAUDE.md per directory
require_agents_md = true          # Warn on missing AGENTS.md per directory

[docs.adr]
enabled = true
directory = "adrs"                # Relative to docs root
naming_pattern = "ADR-\\d{3,}-[a-zA-Z0-9-]+\\.md"
required_adr_keywords = []        # e.g., ["security", "testing"]
backlinking = true

[docs.readme]
enabled = true
max_depth = -1                    # -1 = unlimited
exclude_dirs = ["node_modules", ".git", "target", "vendor", ".topology"]

[docs.root_context]
enabled = true
docs_reference_pattern = "docs/"
```

---

## 4. ADR Enforcement

ADR rules are defined and enforced by the **ADR01 substandard** ([EXP-V1-0004.ADR01](../substandards/ADR01-architecture-decision-records/docs/01_spec.md)).

The parent standard provides shared configuration (`docs.adr.*` in `.apss/config.toml`) and the front matter parser. The substandard implements all ADR-specific validation: naming conventions (`ADR-XXX-<name>.md`), front matter requirements, keyword-based required ADRs, and backlinking.

---

## 5. README/Index Enforcement (DOC02)

### 5.1 DOC02-001: README.md Per Directory

Every directory under the docs root (respecting `max_depth` and `exclude_dirs`) MUST contain a `README.md` file.

**Error code**: `MISSING_README`

### 5.2 DOC02-002: Index Section

README.md files SHOULD contain a `## Index` section auto-generated from front matter of `.md` files in the same directory.

The index is a Markdown table:

```markdown
## Index

| Document | Description |
|----------|-------------|
| [Initial Architecture](ADR-001-initial-architecture.md) | Defines system architecture |
```

**Error codes**: `MISSING_INDEX`, `STALE_INDEX`

### 5.3 DOC02-003: Context Files

Directories SHOULD contain:

- `CLAUDE.md` — AI context pointer
- `AGENTS.md` — Agent operational context pointer

Both serve as lightweight pointers:

```markdown
---
name: "<directory name>"
description: "AI context for <directory name>"
---

See [README.md](README.md) for full index and overview of this directory.
```

**Severity**: Warning
**Error codes**: `MISSING_CLAUDE_MD`, `MISSING_AGENTS_MD`

---

## 6. Root Context Files (DOC03)

### 6.1 DOC03-001: Root CLAUDE.md

The repository root MUST contain a `CLAUDE.md` file.

**Error code**: `MISSING_ROOT_CLAUDE_MD`

### 6.2 DOC03-002: Root AGENTS.md

The repository root MUST contain an `AGENTS.md` file.

**Error code**: `MISSING_ROOT_AGENTS_MD`

### 6.3 DOC03-003: Documentation Reference

Root `CLAUDE.md` SHOULD reference the documentation location so agents can find it from a fresh start.

**Severity**: Warning
**Error code**: `MISSING_DOCS_REFERENCE`

---

## 7. CLI Interface

### 7.1 Validation

```bash
aps run docs validate [path]       # Validate all documentation rules
aps run docs validate [path] --json # JSON output for CI
```

### 7.2 Index Generation

```bash
aps run docs index [path]          # Preview generated indexes (dry run)
aps run docs index [path] --write  # Write indexes into README.md files
```

---

## 8. Error Codes

| Code | Severity | Domain | Description |
|------|----------|--------|-------------|
| `MISSING_README` | Error | DOC02 | Directory missing README.md |
| `MISSING_CLAUDE_MD` | Warning | DOC02 | Directory missing CLAUDE.md |
| `MISSING_AGENTS_MD` | Warning | DOC02 | Directory missing AGENTS.md |
| `MISSING_INDEX` | Warning | DOC02 | README.md missing ## Index section |
| `STALE_INDEX` | Warning | DOC02 | README.md ## Index is out of date |
| `MISSING_ROOT_CLAUDE_MD` | Error | DOC03 | Root missing CLAUDE.md |
| `MISSING_ROOT_AGENTS_MD` | Error | DOC03 | Root missing AGENTS.md |
| `MISSING_DOCS_REFERENCE` | Warning | DOC03 | Root CLAUDE.md missing docs reference |
| `INVALID_CONFIG` | Error | Config | .apss/config.toml is invalid |

ADR error codes (`ADR01-001` through `ADR01-010`) are defined in the [ADR01 substandard spec](../substandards/ADR01-architecture-decision-records/docs/01_spec.md#10-error-codes).

---

## Appendix A: Validation Checklist

- [ ] `.apss/config.toml` valid (or absent for defaults)
- [ ] ADR directory exists at configured path
- [ ] All ADR files match naming convention
- [ ] All ADR files have `name` and `description` front matter
- [ ] All required ADRs present
- [ ] Every docs directory has `README.md`
- [ ] README.md files have `## Index` section
- [ ] `CLAUDE.md` and `AGENTS.md` present per directory
- [ ] Root `CLAUDE.md` and `AGENTS.md` exist
- [ ] Root `CLAUDE.md` references documentation location
