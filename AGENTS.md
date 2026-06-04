---
name: "agent-paradise-standards-system"
description: "Repo orientation for AI agents: where standards live, how docs are governed, and the mandatory backlinking rule"
alwaysApply: true
---

# Agents reading this repo, start here

This repository is the **Agent Paradise Standards System** (APSS). It is the home of versioned, executable standards (Rust crates with validators), not just documents.

## Where things live

- **Standards source.** Official standards: [`standards/v1/`](standards/v1/). Experimental: [`standards-experimental/v1/`](standards-experimental/v1/).
- **Documentation governance.** This repo dogfoods its own documentation standard, [`EXP-V1-0004`](standards-experimental/v1/EXP-V1-0004-documentation/docs/00_overview.md). All normative rules for ADRs, indexing, frontmatter, the install hook, and backlinking live in [`EXP-V1-0004/docs/01_spec.md`](standards-experimental/v1/EXP-V1-0004-documentation/docs/01_spec.md).
- **Architecture Decision Records.** When the documentation standard is installed in a downstream project, ADRs live under `<project-root>/docs/adrs/` by default. For this repo, follow the same convention: any architectural decision made while working here MUST be captured as an ADR in `docs/adrs/` (per the ADR01 substandard).
- **Meta-standard.** Rules for how standards are themselves structured: [`standards/v1/APS-V1-0000-meta/`](standards/v1/APS-V1-0000-meta/).
- **Project README.** See [`README.md`](README.md) for the user-facing build, CLI, and package overview.

## Mandatory rules when working in this repo

These are loaded by the APSS documentation standard and are not optional:

1. **Code MUST backlink the governing doc.** Any implementation file that exists to satisfy an architectural decision MUST contain a token of the form `<DOC-TYPE-ID>-<NUMBER>-<NAME>` near the top (typically a single-line comment). Examples: `// Implements ADR-001-security-architecture`, `# Implements PV-001-product-purpose, RETRO-007-q1-launch`. See [`EXP-V1-0004/docs/01_spec.md` Section 7](standards-experimental/v1/EXP-V1-0004-documentation/docs/01_spec.md#7-backlinking-always-part-of-the-standard). This is a load-bearing invariant for context preservation across plan, design, and implementation phases.
2. **New architectural decisions MUST land as ADRs** in `docs/adrs/`, following [`EXP-V1-0004.ADR01`](standards-experimental/v1/EXP-V1-0004-documentation/substandards/ADR01-architecture-decision-records/docs/01_spec.md).
3. **Diagnostic codes are human-readable** kebab strings (`ADR01-dir-not-found`, `index-stale`, `frontmatter-unclosed`). Do not introduce numeric or opaque codes.
4. **No em dashes or en dashes** in prose, code, comments, or commit messages. Use regular hyphens or rephrase.

## Where to find more

- Documentation standard overview: [`standards-experimental/v1/EXP-V1-0004-documentation/docs/00_overview.md`](standards-experimental/v1/EXP-V1-0004-documentation/docs/00_overview.md).
- Doc type registry (which doc types are active, where they live): [`EXP-V1-0004/docs/01_spec.md` Section 8](standards-experimental/v1/EXP-V1-0004-documentation/docs/01_spec.md#8-doc-type-registry).
- ADR substandard: [`standards-experimental/v1/EXP-V1-0004-documentation/substandards/ADR01-architecture-decision-records/`](standards-experimental/v1/EXP-V1-0004-documentation/substandards/ADR01-architecture-decision-records/).
- Skills for this standard: [`standards-experimental/v1/EXP-V1-0004-documentation/agents/skills/README.md`](standards-experimental/v1/EXP-V1-0004-documentation/agents/skills/README.md).
- A second context file with the same orientation, formatted for Claude Code: [`CLAUDE.md`](CLAUDE.md).

The validator that enforces the rules above lives in `aps-cli` and the documentation standard crate; run `aps run docs validate [path]` for a CI-friendly report or install the pre-commit hook via `aps run docs install [path]`.

---

# RIPER-5 operational protocol (operator workflow)

The rest of this file is the operator's RIPER-5 mode protocol. It governs how agents step through research, planning, and execution in this repo. Read it before opening a non-trivial PR.

## Mode Transition Signals

Only transition modes when these exact signals are used:

```
ENTER RESEARCH MODE or ERM
ENTER INNOVATE MODE or EIM
ENTER PLAN MODE or EPM
ENTER EXECUTE MODE or EEM
ENTER REVIEW MODE or EQM
DIRECT EXECUTE MODE or DEM // bypass the plan and go straight to execute
```

## Meta-Instruction

Begin every response with your current mode in brackets: `[MODE: MODE_NAME]`.

## The five modes

### MODE 1: RESEARCH
- **Purpose:** information gathering only.
- **Permitted:** reading files, asking questions, understanding code.
- **Forbidden:** suggestions, planning, implementation.

### MODE 2: INNOVATE
- **Purpose:** brainstorming approaches.
- **Permitted:** discussing ideas, weighing trade-offs.
- **Forbidden:** concrete planning, code writing.

### MODE 3: PLAN
- **Purpose:** producing a technical specification.
- **Permitted:** detailed plans with file paths and changes.
- **Forbidden:** implementation or code writing.
- **Required:** a comprehensive `PROJECT-PLAN_YYYYMMDD_<TASK-NAME>.md` with milestones whose tasks have empty checkboxes. NEVER commit the `PROJECT-PLAN_*` file.
- **ADRs:** any architectural decision made here MUST be captured as an ADR (see the rules above).
- **TDD:** keep testing in mind; add tests first, then implement.

### MODE 4: EXECUTE
- **Purpose:** implementing the approved plan exactly.
- **Permitted:** implementing the plan, running the QA checkpoint.
- **Forbidden:** deviations or creative additions.
- **Required:** run the QA checkpoint after each milestone and commit before moving on.
- Use `TODO` comments for future improvements and `FIXME` for breaking issues.

### MODE 5: REVIEW
- **Purpose:** validate the implementation against the plan.
- **Required:** flag ANY deviation with `:warning: DEVIATION DETECTED: [description]`.

## QA Checkpoint

After each milestone in EXECUTE mode:

1. Run the linter with auto-format.
2. Run type checks.
3. Run tests.
4. Review changes.
5. Commit using conventional commit messages.

For Python projects in this org, use `uv` for package management (not `pip` directly). Composite check: `poetry run poe check-fix` or `python scripts/qa_checkpoint.py`.

## Commit format

Conventional commits: `type(scope): description`. Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`.

## Critical guidelines

- Never transition between modes without explicit permission.
- Always declare current mode at the start of every response.
- Follow the plan with 100% fidelity in EXECUTE mode.
- Flag even the smallest deviation in REVIEW mode.
- Return to PLAN mode if any implementation issue forces a deviation.
- Use conventional commit messages for all commits.
