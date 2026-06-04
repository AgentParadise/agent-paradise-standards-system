# APS-V1-0001.LANG01 — Rust Language Adapter (Specification)

**Version**: 0.1.0
**Status**: Promoted
**Parent**: APS-V1-0001 (Code Topology and Coupling Analysis)

---

## 1. Scope

This substandard specifies the **Rust Language Adapter** for analyzing Rust codebases and generating `.topology/` artifacts.

The adapter parses Rust source files using `syn` and produces:

- **Function-level complexity metrics** (cyclomatic, cognitive, Halstead)
- **Call graphs** (function-to-function relationships)
- **Module metrics** (Martin's Ca/Ce/I/A/D per module)
- **Coupling matrix** (module-to-module coupling strength)

## 2. Required Output Artifacts

| Artifact | Path | Description |
|----------|------|-------------|
| Module metrics | `metrics/modules.json` | Per-module complexity and Martin metrics |
| Function metrics | `metrics/functions.json` | Per-function complexity metrics |
| Call graph | `graphs/call-graph.json` | Function-level call relationships |
| Coupling matrix | `graphs/coupling-matrix.json` | Module-to-module coupling coefficients |

## 3. Complexity Rules

| Construct | Cyclomatic | Cognitive |
|-----------|------------|-----------|
| `if` / `else if` | +1 | +1, +nesting |
| `match` arm (non-wildcard) | +1 | +1, +nesting |
| `while` / `loop` / `for` | +1 | +1, +nesting |
| `?` (try operator) | +1 | +0 (linear flow) |
| `&&` / `||` | +1 | +1 |
| Recursion | +0 | +1 (fundamental) |
| Closure | +0 | +1 (context switch) |

## 4. Error Codes

| Code | Description |
|------|-------------|
| `PARSE_ERROR` | Failed to parse Rust source file |
| `NO_RUST_FILES` | No `.rs` files found in target directory |
| `OUTPUT_WRITE_FAILED` | Failed to write topology artifacts |
