# APS-V1-0000 Templates

This directory contains scaffolding templates for creating new APS packages.

## Available Templates

| Template | Description | Output Location |
|----------|-------------|-----------------|
| `standard/` | New official standard | `standards/v1/APS-V1-XXXX-<slug>/` |
| `substandard/` | New substandard (profile) | `standards/v1/<parent>/substandards/<slug>/` |
| `experiment/` | New experimental standard | `standards-experimental/v1/EXP-V1-XXXX-<slug>/` |

## Template Structure

Each template contains:

```
<template>/
├── template.toml     # Template metadata and variable definitions
└── skeleton/         # Files to scaffold (with Handlebars placeholders)
```

## Usage

Templates are used by the CLI:

```bash
# Create a new standard
aps v1 create standard my-new-standard

# Create a new experiment
aps v1 create experiment my-experiment
```

## Template Variables

### Standard Template

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `id` | ✅ | - | Standard ID (e.g., APS-V1-0001) |
| `name` | ✅ | - | Human-readable name |
| `slug` | ✅ | - | Filesystem-safe slug (kebab-case) |
| `version` | ❌ | `1.0.0` | Initial SemVer version |
| `category` | ❌ | `governance` | Category |
| `maintainers` | ❌ | `["AgentParadise"]` | Maintainer list |

### Substandard Template

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `id` | ✅ | - | Substandard ID (e.g., APS-V1-0002.GH01) |
| `name` | ✅ | - | Human-readable name |
| `slug` | ✅ | - | Filesystem-safe slug (kebab-case) |
| `version` | ❌ | `1.0.0` | Initial SemVer version |
| `parent_id` | ✅ | - | Parent standard ID |
| `parent_major` | ❌ | `1` | Required parent major version |
| `maintainers` | ❌ | `["AgentParadise"]` | Maintainer list |

### Experiment Template

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `id` | ✅ | - | Experiment ID (e.g., EXP-V1-0001) |
| `name` | ✅ | - | Human-readable name |
| `slug` | ✅ | - | Filesystem-safe slug (kebab-case) |
| `version` | ❌ | `0.1.0` | Initial version (0.x for experiments) |
| `category` | ❌ | `technical` | Category |
| `maintainers` | ❌ | `["AgentParadise"]` | Maintainer list |

## Handlebars Syntax

Templates use [Handlebars](https://handlebarsjs.com/) for variable substitution:

```handlebars
# {{name}}

Version: {{version}}
Category: {{category}}

Maintainers:
{{#each maintainers}}
- {{this}}
{{/each}}
```

## Requirements

Per the meta-standard spec (§13):

- Templates MUST be deterministic
- Templates MUST produce packages that pass validation immediately
- Templates MUST be co-located with the standard they belong to
