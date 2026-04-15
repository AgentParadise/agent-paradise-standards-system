---
name: "Documentation Standard Agent Skills"
description: "Agent-readable capabilities for documentation validation and index generation"
---

# Documentation Standard Skills

## validate-docs

Validate a project's documentation structure against the APSS documentation standard.

**Input**: Repository root path
**Output**: Diagnostics (errors, warnings) for ADR naming, front matter, README indexes, context files

**Usage**: `aps run docs validate [path]`

## generate-index

Generate or regenerate `## Index` sections in README.md files from front matter of documents in each directory.

**Input**: Repository root path
**Output**: Updated README.md files with current indexes

**Usage**: `aps run docs index [path] --write`
