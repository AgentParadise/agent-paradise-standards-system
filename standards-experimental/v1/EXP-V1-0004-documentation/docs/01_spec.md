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

This standard defines a configurable structure for a project's technical documentation directory. The big unlock is **frontmatter-driven indexing**: every Markdown file carries YAML front matter, and the standard auto-generates `## Index` tables from that metadata. This makes docs structured, searchable, and machine-readable — enabling validators, generators, vectorization, and fast navigation for both humans and AI agents.

The documentation root defaults to `docs/` but is configurable. The standard enforces:

- **DOC02** — README indexes, frontmatter, and AI context files per directory
- **DOC03** — Root-level context files so agents always find docs from a fresh start

Domain-specific rules (e.g., ADR enforcement) are handled by **substandards** that live inside the docs directory and inherit the parent's index format. See [Substandards](#7-substandards).

### 1.1 Relationship to APS-V1-0000

This standard complements APS-V1-0000's requirement for `docs/01_spec.md` by enforcing broader documentation structure beyond the standard package itself.

---

## 2. Core Definitions

- **Front matter**: A YAML block delimited by `---` at the top of a Markdown file, containing structured metadata. This is the foundation of the standard — front matter fields drive index generation, validation, and search.
- **Index**: An auto-generated `## Index` section in README.md that lists documents in a directory with their front matter metadata.
- **Context file**: `CLAUDE.md` or `AGENTS.md` — lightweight files providing AI agents with quick orientation to a directory's purpose and contents.
- **Docs root**: The project's technical documentation directory (default: `docs/`), configurable via `docs.root`.

---

## 3. Configuration

### 3.1 Config Location

Project-level configuration MUST be located at `.apss/config.toml` relative to the repository root.

### 3.2 Default Behavior

If `.apss/config.toml` does not exist, all defaults SHALL apply. The validator MUST NOT error on a missing config file. Zero-config works — all defaults are sensible.

### 3.3 Schema

```toml
schema = "apss.config/v1"

[docs]
enabled = true                    # Master switch for all doc validation
root = "docs"                     # Documentation root directory

[docs.index]
enabled = true                    # Enforce ## Index in README.md
auto_generate = true              # Allow index auto-generation via CLI
frontmatter_fields = ["name", "description"]  # Fields rendered in index tables

[docs.context_files]
require_claude_md = true          # Warn on missing CLAUDE.md per directory
require_agents_md = true          # Warn on missing AGENTS.md per directory

[docs.readme]
enabled = true
max_depth = -1                    # -1 = unlimited depth traversal
exclude_dirs = ["node_modules", ".git", "target", "vendor", ".topology"]

[docs.root_context]
enabled = true
docs_reference_pattern = "docs/"  # Pattern to check in root CLAUDE.md
```

Substandards may define additional config sections under `[docs.*]`. For example, the ADR substandard uses `[docs.adr]` — see the [ADR01 substandard spec](../substandards/ADR01-architecture-decision-records/docs/01_spec.md#9-configuration) for its schema.

---

## 4. Frontmatter and Indexing

### 4.1 Frontmatter Requirement

Every `.md` file under the docs root SHOULD contain a YAML front matter block with at least the fields listed in `docs.index.frontmatter_fields` (default: `name` and `description`).

```yaml
---
name: "API Authentication Guide"
description: "How authentication works across all service boundaries"
---
```

Front matter makes each document self-describing. Tooling reads these fields to generate indexes, power search, and provide agents with structured context about what each document covers without reading the full content.

### 4.2 Index Generation

When `docs.index.enabled` is `true`, every `README.md` SHOULD contain a `## Index` section. The index is a Markdown table auto-generated from the front matter of `.md` files in the same directory:

```markdown
## Index

| Document | Description |
|----------|-------------|
| [API Authentication Guide](api-auth.md) | How authentication works across all service boundaries |
| [Deployment Runbook](deployment.md) | Step-by-step production deployment procedure |
```

The columns rendered in the table are controlled by `docs.index.frontmatter_fields`. The first field becomes the link text; remaining fields become additional columns.

**Error codes**: `MISSING_INDEX`, `STALE_INDEX`

### 4.3 Index Auto-Generation

When `docs.index.auto_generate` is `true`, the CLI can write indexes directly into README.md files:

```bash
aps run docs index [path]          # Preview generated indexes (dry run)
aps run docs index [path] --write  # Write indexes into README.md files
```

The generator inserts or replaces the `## Index` section. Content outside this section is preserved.

---

## 5. README and Context Files (DOC02)

### 5.1 DOC02-001: README.md Per Directory

Every directory under the docs root (respecting `max_depth` and `exclude_dirs`) MUST contain a `README.md` file.

**Error code**: `MISSING_README`

### 5.2 DOC02-002: Context Files

Directories SHOULD contain:

- `CLAUDE.md` — AI context pointer
- `AGENTS.md` — Agent operational context pointer

Both serve as lightweight pointers that give agents immediate orientation:

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

Root `CLAUDE.md` SHOULD reference the documentation location (matching `docs.root_context.docs_reference_pattern`) so agents can find docs from a fresh start.

**Severity**: Warning
**Error code**: `MISSING_DOCS_REFERENCE`

---

## 7. Substandards

Domain-specific documentation rules are implemented as **substandards** that live inside the docs directory. Substandards inherit the parent's frontmatter format and index generation — they add domain-specific validation on top.

### 7.1 ADR Enforcement (EXP-V1-0004.ADR01)

Architecture Decision Records are enforced by the [ADR01 substandard](../substandards/ADR01-architecture-decision-records/docs/01_spec.md). It validates naming conventions, required front matter (including lifecycle status), keyword-based required topics, and dead reference detection. Configuration lives under `[docs.adr]` in `.apss/config.toml`.

---

## 8. CLI Interface

### 8.1 Validation

```bash
aps run docs validate [path]       # Validate all documentation rules
aps run docs validate [path] --json # JSON output for CI
```

### 8.2 Index Generation

```bash
aps run docs index [path]          # Preview generated indexes (dry run)
aps run docs index [path] --write  # Write indexes into README.md files
```

---

## 9. Error Codes

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

Substandard error codes are defined in their respective specs. ADR codes (`ADR01-001` through `ADR01-012`) are in the [ADR01 spec](../substandards/ADR01-architecture-decision-records/docs/01_spec.md#10-error-codes).

---

## Appendix A: Validation Checklist

- [ ] `.apss/config.toml` valid (or absent for defaults)
- [ ] Every docs directory has `README.md`
- [ ] `.md` files have front matter with configured fields
- [ ] README.md files have `## Index` section matching actual contents
- [ ] `CLAUDE.md` and `AGENTS.md` present per directory
- [ ] Root `CLAUDE.md` and `AGENTS.md` exist
- [ ] Root `CLAUDE.md` references documentation location
