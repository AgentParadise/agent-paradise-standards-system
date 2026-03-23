# Skill: Analyze Topology

## Overview

Generate `.topology/` artifacts from a codebase by running complexity analysis across all supported languages.

## When to Use

- When starting work on a new codebase to understand its architecture
- Before refactoring to identify complexity hotspots
- As part of CI to track complexity trends
- When asked to "analyze the codebase structure" or "find complex code"

## Inputs

| Input | Required | Description |
|-------|----------|-------------|
| `path` | Yes | Path to the codebase root |
| `languages` | No | Languages to analyze (default: auto-detect) |
| `exclude` | No | Glob patterns to exclude |

## Outputs

| Output | Description |
|--------|-------------|
| `.topology/manifest.toml` | Analysis metadata |
| `.topology/metrics/*.json` | Complexity metrics at function/file/module levels |
| `.topology/graphs/*.json` | Call graph, dependency graph, coupling matrix |

## Procedure

1. **Detect languages** in the codebase by scanning file extensions
2. **Load grammars** from the unified tree-sitter framework (Rust, Python, TypeScript, TSX)
3. **Walk directory** tree, filtering hidden dirs and build artifacts
4. **Extract functions** from all source files using tree-sitter queries
5. **Compute metrics** for each function using shared complexity engine:
   - Cyclomatic complexity (decision points + 1)
   - Cognitive complexity (nesting-aware)
   - Halstead metrics (operators/operands)
6. **Aggregate by module** to compute module-level metrics
7. **Build coupling matrix** between modules
8. **Write artifacts** to `.topology/` directory:
   - `manifest.toml` — Analysis metadata
   - `metrics/functions.json` — Per-function metrics
   - `metrics/modules.json` — Per-module aggregates
   - `graphs/coupling-matrix.json` — Module coupling

## Supported Languages

| Language | Extensions | Status |
|----------|------------|--------|
| Rust | `.rs` | ✅ Implemented |
| Python | `.py`, `.pyi` | ✅ Implemented |
| TypeScript | `.ts` | ✅ Implemented |
| TSX | `.tsx` | ✅ Implemented |
| JavaScript/JSX | `.js`, `.jsx`, `.mjs` | 📋 Planned |

## CLI Commands

```bash
# Analyze the current directory (auto-detect languages)
aps run topology analyze .

# Analyze only Python files
aps run topology analyze . --language python

# Analyze only Rust files
aps run topology analyze . --language rust

# Analyze with custom output location
aps run topology analyze . --output .topology/

# Validate existing artifacts
aps run topology validate .topology/

# Generate a report
aps run topology report .topology/

# Show supported languages
aps run topology --help
```

## Example Usage

```
User: Analyze this codebase and identify the most complex modules.

Agent: I'll analyze the codebase topology to identify complexity hotspots.

[Runs topology analysis]

Based on the analysis:

**Complexity Hotspots:**
1. `api/handlers.ts::processRequest` — CC=12, Cognitive=18 ⚠️
   - Consider extracting validation logic
2. `auth/validator.rs::validate_token` — CC=8, Cognitive=12
   - Multiple error paths could be simplified

**Coupling Analysis:**
- `auth` and `crypto` are tightly coupled (0.75)
- `utils` is in "Zone of Pain" — many modules depend on it

The full analysis is saved in `.topology/` for visualization.
```

## Error Handling

| Error | Action |
|-------|--------|
| Unsupported language | Skip files, log warning |
| Parse error | Skip file, include in manifest.errors |
| Permission denied | Fail with clear error message |

## Related Skills

- `query-coupling` — Query coupling relationships from existing artifacts
- `visualize-architecture` — Render topology as 3D visualization

