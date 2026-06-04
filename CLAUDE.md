---
name: "agent-paradise-standards-system"
description: "Repo orientation for Claude Code: where standards live, how docs are governed, and the mandatory backlinking rule"
---

# Claude Code, start here

This repository is the **Agent Paradise Standards System** (APSS). It contains versioned, executable standards (Rust crates with validators), not just documents. This file gives you enough to orient yourself in one read; for the full operator workflow protocol see [`AGENTS.md`](AGENTS.md).

## Where things live

- **Standards source.** Official: [`standards/v1/`](standards/v1/). Experimental: [`standards-experimental/v1/`](standards-experimental/v1/).
- **Documentation governance.** This repo dogfoods the documentation standard [`EXP-V1-0004`](standards-experimental/v1/EXP-V1-0004-documentation/docs/00_overview.md). Normative rules: [`EXP-V1-0004/docs/01_spec.md`](standards-experimental/v1/EXP-V1-0004-documentation/docs/01_spec.md).
- **Architecture Decision Records.** ADRs live under `docs/adrs/` by default (per the ADR01 substandard). Any architectural decision you make MUST be captured there.
- **Meta-standard for how standards are themselves structured:** [`standards/v1/APS-V1-0000-meta/`](standards/v1/APS-V1-0000-meta/).
- **Project README:** [`README.md`](README.md) for the user-facing build, CLI, and package overview.

## Mandatory rules when working in this repo

1. **Code MUST backlink the governing doc.** Implementation files that satisfy an architectural decision MUST contain a token of the form `<DOC-TYPE-ID>-<NUMBER>-<NAME>` near the top, for example `// Implements ADR-001-security-architecture`. Backlinking is part of the standard, not an opt-in. See [`EXP-V1-0004/docs/01_spec.md` Section 7](standards-experimental/v1/EXP-V1-0004-documentation/docs/01_spec.md#7-backlinking-always-part-of-the-standard).
2. **New architectural decisions MUST land as ADRs** in `docs/adrs/`, following [`EXP-V1-0004.ADR01`](standards-experimental/v1/EXP-V1-0004-documentation/substandards/ADR01-architecture-decision-records/docs/01_spec.md).
3. **Diagnostic codes are human-readable** kebab strings (`ADR01-dir-not-found`, `index-stale`, `frontmatter-unclosed`). No numeric or opaque codes.
4. **No em dashes or en dashes** in prose, code, comments, or commit messages. Use regular hyphens or rephrase.

## Where to find more

- Documentation standard overview: [`standards-experimental/v1/EXP-V1-0004-documentation/docs/00_overview.md`](standards-experimental/v1/EXP-V1-0004-documentation/docs/00_overview.md).
- Doc type registry (active doc types and locations): [`EXP-V1-0004/docs/01_spec.md` Section 8](standards-experimental/v1/EXP-V1-0004-documentation/docs/01_spec.md#8-doc-type-registry).
- ADR substandard: [`standards-experimental/v1/EXP-V1-0004-documentation/substandards/ADR01-architecture-decision-records/`](standards-experimental/v1/EXP-V1-0004-documentation/substandards/ADR01-architecture-decision-records/).
- Operator workflow (RIPER-5 protocol) and the QA checkpoint: [`AGENTS.md`](AGENTS.md).

## Quick validation

```bash
aps run docs validate [path]     # CI-friendly report
aps run docs install [path]      # Install the pre-commit hook + default config (idempotent)
```

The pre-commit hook refuses commits with errors and re-stages auto-refreshed indexes. See [`EXP-V1-0004/docs/01_spec.md` Section 9](standards-experimental/v1/EXP-V1-0004-documentation/docs/01_spec.md#9-install-contract-hook--validator--index).
