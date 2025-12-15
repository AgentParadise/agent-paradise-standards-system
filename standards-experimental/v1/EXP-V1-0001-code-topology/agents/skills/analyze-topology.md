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
2. **Load language adapters** for detected languages (TypeScript, Python, Rust, C++)
3. **Extract functions** from all source files using tree-sitter queries
4. **Compute metrics** for each function:
   - Cyclomatic complexity (decision points + 1)
   - Cognitive complexity (nesting-aware)
   - Halstead metrics (operators/operands)
5. **Build call graph** by analyzing function call sites
6. **Build dependency graph** by analyzing imports
7. **Compute coupling matrix** between modules
8. **Calculate Martin's metrics** (Ca, Ce, I, A, D) per module
9. **Run force-directed layout** to compute 3D positions
10. **Write artifacts** to `.topology/` directory

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

