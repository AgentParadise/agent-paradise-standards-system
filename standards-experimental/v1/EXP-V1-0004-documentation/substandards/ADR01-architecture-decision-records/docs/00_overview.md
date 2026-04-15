---
name: "EXP-V1-0004.ADR01 — Architecture Decision Records"
description: "Enforces ADR naming, front matter, required keywords, and backlinking"
---

# ADR Enforcement Substandard (EXP-V1-0004.ADR01)

Validates Architecture Decision Records within the documentation structure
defined by the parent standard (EXP-V1-0004).

## What It Enforces

- **Naming convention**: `ADR-XXX-<adr-name>.md` (configurable regex)
- **Front matter**: Every ADR must have `name` and `description` in YAML front matter
- **Required keywords**: Ensures ADRs exist for configured topic keywords (e.g., security, testing)
- **Backlinking**: Implementation files reference their governing ADR identifier

## Quick Start

```bash
# Validate ADRs (runs as part of docs validate)
aps run docs validate .

# Configure required ADR keywords
# .apss/config.toml
[docs.adr]
required_adr_keywords = ["security", "testing", "deployment"]
```

## Error Codes

| Code | Description |
|------|-------------|
| ADR01-001 | ADR directory not found |
| ADR01-002 | ADR file does not match naming pattern |
| ADR01-003 | ADR file missing required front matter |
| ADR01-004 | Required ADR keyword not satisfied |
| ADR01-005 | Implementation file missing ADR backlink |
| ADR01-006 | Invalid naming regex in config |
| ADR01-007 | ADR directory missing CLAUDE.md or AGENTS.md |
| ADR01-008 | ADR context file lacks ADR referencing guidance |
| ADR01-009 | Source file references non-existent ADR |
| ADR01-010 | ADR file missing required section header |
