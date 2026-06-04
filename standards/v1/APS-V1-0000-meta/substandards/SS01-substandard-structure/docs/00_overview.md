# APS-V1-0000.SS01 — Substandard Structure

## Overview

This substandard defines the structural requirements for all APS substandards. It establishes the contract that substandards must follow while inheriting from their parent standard.

## What is a Substandard?

A **substandard** is a domain-specific extension of a parent standard. Substandards provide specialized implementations, profiles, or bindings while maintaining structural consistency with the APS ecosystem.

### Common Use Cases

- **Language Bindings**: Python, TypeScript, Go implementations of a standard
- **Platform Profiles**: GitHub Actions, GitLab CI, VS Code extensions
- **Domain Specializations**: Security profiles, performance variants
- **Integration Guides**: Framework-specific implementations (React, Django, etc.)

## Key Differences from Standards

| Aspect | Standard | Substandard |
|--------|----------|-------------|
| **ID Format** | `APS-V1-XXXX` | `APS-V1-XXXX.YY01` |
| **Location** | `standards/v1/` | `standards/v1/{parent}/substandards/` |
| **Backwards Compat** | MUST maintain within V1 | MAY break within parent major |
| **Metadata File** | `standard.toml` | `substandard.toml` |
| **Parent Reference** | N/A | MUST reference valid parent |

## Quick Reference

```bash
# Create a new substandard
aps v1 create substandard APS-V1-0001 --profile GH01 --name "GitHub Profile"

# Validate a substandard
aps v1 validate substandard APS-V1-0001.GH01

# Bump substandard version
aps v1 version bump APS-V1-0001.GH01 minor
```

