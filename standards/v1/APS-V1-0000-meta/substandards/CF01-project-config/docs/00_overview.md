# APS-V1-0000.CF01 — Project Configuration

## Overview

CF01 defines how consumer projects declare their adoption of APS standards via `apss.toml`.

## Problem

Without a standard configuration format, there's no way for:
- A project to declare which APS standards it implements
- Standards to receive project-specific configuration
- CI/QA to validate a project's standards compliance
- Monorepos to manage cascading standard configurations

## Solution

CF01 specifies:

1. **`apss.toml` schema** — a TOML configuration file at the project root declaring standards, versions, substandards, and per-standard config
2. **Cascading configs** — monorepo support with root + child `apss.toml` files and deterministic merge rules
3. **`StandardConfig` trait** — a typed configuration contract that each standard implements, enabling compile-time config validation
4. **Dual validation** — CF01 validates both consumer config files (runtime) and standard config surfaces (CI)

## Related

- **APS-V1-0000.DI01** — Distribution & Installation (uses `apss.toml` for dependency resolution)
- **APS-V1-0000.CL01** — CLI Contract (extended with `configure()` method)
- **Meta-standard §8.3** — StandardConfig trait specification
