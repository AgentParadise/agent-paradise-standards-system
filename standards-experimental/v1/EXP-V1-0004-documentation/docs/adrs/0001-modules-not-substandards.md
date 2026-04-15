---
name: "ADR and README as Modules, Not Substandards"
description: "Decision to implement ADR and README enforcement as crate modules rather than APS substandards"
---

# ADR-0001: ADR and README as Modules, Not Substandards

**Status:** Superseded by EXP-V1-0004.ADR01 substandard
**Date:** 2026-04-15
**Deciders:** Initial design session

## Context

The documentation standard covers two distinct enforcement domains:

1. **ADR enforcement** — naming conventions, front matter, required ADRs
2. **README/index enforcement** — directory README.md, auto-generated indexes, context files

APS-V1-0001 (Code Topology) uses substandards for its diverse concerns (LANG01-rust, VIZ01-dashboard, 3D01-force-directed, CI01-github-actions). Should this standard follow the same pattern?

## Decision

Implement both domains as **modules within a single crate**, not as substandards.

## Rationale

- **Shared infrastructure**: Both domains use the same config file (`.apss/config.toml`), the same front matter parser, and operate on the same filesystem tree. Splitting them would duplicate these dependencies.
- **Tight coupling**: The index generation module is used by both ADR validation (to index ADRs) and README enforcement (to validate indexes). This creates a natural dependency that would complicate substandard boundaries.
- **No independent versioning need**: Unlike Code Topology's substandards (which have independent artifact schemas and can evolve separately), ADR and README enforcement are tightly coupled and will evolve together.
- **Experiment simplicity**: At the experimental stage, minimizing structural overhead allows faster iteration. The meta-standard's substandard requirements add boilerplate that isn't justified yet.

## Consequences

- Single crate, single version, single Cargo.toml
- Both domains are always validated together (no opt-in per substandard)
- Configuration toggles (`docs.adr.enabled`, `docs.readme.enabled`) provide the equivalent of substandard opt-in
- If the domains diverge significantly upon promotion, they can be extracted into substandards at that point

## Alternatives Rejected

- **Substandards**: Too much structural overhead for tightly coupled concerns. Would duplicate config parsing and front matter logic.
