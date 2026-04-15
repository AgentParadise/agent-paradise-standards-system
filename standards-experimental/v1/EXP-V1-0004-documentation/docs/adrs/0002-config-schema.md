---
name: "Project-Level Configuration via .apss/config.toml"
description: "Decision to use .apss/config.toml for project-level documentation standard configuration"
---

# ADR-0002: Project-Level Configuration via .apss/config.toml

**Status:** Accepted
**Date:** 2026-04-15
**Deciders:** Initial design session

## Context

Projects adopting the documentation standard need a way to configure:

- Documentation root directory
- ADR directory and naming pattern
- Required ADRs
- Index generation settings
- Which enforcement rules to enable/disable

This configuration must be discoverable, project-scoped, and not conflict with other tooling configs.

## Decision

Use `.apss/config.toml` at the repository root as the project-level configuration file.

## Rationale

- **Namespace isolation**: The `.apss/` directory is the natural home for all APSS project-level configuration, similar to `.vscode/` for VS Code or `.github/` for GitHub. Other future standards can add their own sections to the same file.
- **TOML consistency**: The entire APSS ecosystem uses TOML for metadata (`standard.toml`, `experiment.toml`, `fitness.toml`). Using TOML for project config maintains consistency.
- **VS Code mental model**: The user specifically referenced the VS Code settings model — a single config file with sections for different tools/standards.
- **Zero-config default**: If the file doesn't exist, all defaults apply. This means adoption has zero friction — the validator works immediately with sensible defaults.
- **Single file**: Having one config file (with `[docs]`, and eventually `[topology]`, `[fitness]`, etc.) prevents config file proliferation.

## Consequences

- New config location (`.apss/`) to document and communicate
- Other standards can adopt sections in the same file (e.g., `[fitness]`, `[topology]`)
- Config schema versioning needed via `schema = "apss.config/v1"` field
- All fields must have serde defaults for zero-config support

## Alternatives Rejected

- **`docs.toml` at root**: Would clutter root with per-standard config files as more standards are adopted
- **In `standard.toml`**: That's for APSS package metadata, not consumer project configuration
- **`pyproject.toml`-style**: Not TOML-ecosystem-consistent and overloaded with non-APSS concerns
