# LANG01: Rust Language Adapter

**Profile**: `LANG` (Language Adapter)  
**Parent**: EXP-V1-0001 Code Topology  
**Status**: Experimental

## Purpose

Analyzes Rust codebases and generates `.topology/` artifacts containing:

- **Complexity metrics**: Cyclomatic, Cognitive, Halstead per function
- **Call graphs**: Function-to-function relationships
- **Module metrics**: Martin's Ca/Ce/I/A/D per module
- **Coupling matrix**: Module-to-module coupling strength

## Usage

```rust
use code_topology_rust_adapter::RustAdapter;
use code_topology::LanguageAdapter;

let adapter = RustAdapter::new();
let topology = adapter.analyze(Path::new("my-rust-project"))?;

// Write artifacts
topology.write_to(".topology/")?;
```

## Complexity Rules

| Construct | Cyclomatic | Cognitive |
|-----------|------------|-----------|
| `if` / `else if` | +1 | +1, +nesting |
| `match` arm (non-wildcard) | +1 | +1, +nesting |
| `while` / `loop` / `for` | +1 | +1, +nesting |
| `?` (try operator) | +1 | +0 (linear flow) |
| `&&` / `||` | +1 | +1 |
| Recursion | +0 | +1 (fundamental) |
| Closure | +0 | +1 (context switch) |

## Output

Generates standard `.topology/` directory structure as defined by EXP-V1-0001.

