# APS-V1-0000 Templates

This directory contains scaffolding templates for creating new APS packages.

## Available Templates

| Template | Description |
|----------|-------------|
| `standard/` | Scaffold a new official standard (TODO: M6) |
| `substandard/` | Scaffold a new substandard under a parent (TODO: M6) |
| `experiment/` | Scaffold a new experimental standard (TODO: M6) |

## Template Structure

Each template contains:

```
<template>/
├── template.toml     # Template metadata and variables
└── skeleton/         # Files to copy/render
```

## Usage

Templates are used by the CLI:

```bash
# Create a new standard using the standard template
aps v1 create standard my-new-standard

# Create a substandard using the substandard template
aps v1 create substandard APS-V1-0002 GH01

# Create an experiment using the experiment template
aps v1 create experiment my-experiment
```

## Template Variables

Templates support variable substitution:

| Variable | Description | Example |
|----------|-------------|---------|
| `{{id}}` | Package ID | `APS-V1-0005` |
| `{{name}}` | Human-readable name | `My New Standard` |
| `{{slug}}` | Filesystem-safe slug | `my-new-standard` |
| `{{version}}` | Initial version | `1.0.0` |
| `{{category}}` | Standard category | `technical` |
| `{{maintainers}}` | Maintainer list | `["AgentParadise"]` |

## Requirements

Per the meta-standard spec (§13):

- Templates MUST be deterministic
- Templates MUST produce packages that pass validation immediately
- Templates MUST be co-located with the standard they belong to

