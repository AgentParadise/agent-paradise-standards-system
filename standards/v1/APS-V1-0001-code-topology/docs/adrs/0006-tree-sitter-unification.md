# ADR-0006: Tree-Sitter Unification for Language Adapters

## Status

**Accepted**

## Context

The Code Topology standard (EXP-V1-0001) needs to support multiple programming languages for complexity analysis. The initial implementation used `syn` for Rust parsing, which provided excellent Rust-specific semantics but created several challenges:

1. **Different parsing approaches per language** — Each language would need its own parsing library (syn for Rust, tree-sitter-python for Python, etc.)
2. **Duplicated complexity logic** — Cyclomatic, Cognitive, and Halstead calculations would need to be reimplemented for each parser
3. **Inconsistent behavior** — Different parsers might compute metrics differently
4. **High barrier to add new languages** — Each language requires significant implementation effort

Since this is an experimental standard with no downstream dependencies yet, we have the opportunity to standardize the approach before adoption.

## Decision

We will use **tree-sitter** as the unified parsing framework for all language adapters, including Rust. This involves:

1. **Creating a `Grammar` trait** that each language implements
2. **Sharing complexity calculation logic** across all languages
3. **Using tree-sitter queries** to extract functions, calls, and imports
4. **Defining language-specific rules** via configuration (decision nodes, nesting nodes, ignored nodes)

### Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    TreeSitterAdapter                                │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │              Shared Complexity Engine                        │   │
│  │  • compute_cyclomatic(tree, rules) → u32                    │   │
│  │  • compute_cognitive(tree, rules) → u32                     │   │
│  │  • compute_halstead(tree) → Metrics                         │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              ▲                                      │
│         ┌────────────────────┼────────────────────┐                │
│   ┌─────┴─────┐       ┌─────┴─────┐       ┌─────┴─────┐           │
│   │   Rust    │       │  Python   │       │    ...    │           │
│   │  Grammar  │       │  Grammar  │       │  Grammar  │           │
│   └───────────┘       └───────────┘       └───────────┘           │
└─────────────────────────────────────────────────────────────────────┘
```

### Grammar Trait

```rust
pub trait Grammar: Send + Sync {
    fn language_id(&self) -> &'static str;
    fn file_extensions(&self) -> &'static [&'static str];
    fn ts_language(&self) -> tree_sitter::Language;
    fn function_query(&self) -> &str;
    fn call_query(&self) -> &str;
    fn import_query(&self) -> &str;
    fn decision_nodes(&self) -> &'static [&'static str];
    fn nesting_nodes(&self) -> &'static [&'static str];
    fn ignored_nodes(&self) -> &'static [&'static str];
    fn compute_module_path(&self, file_path: &Path, root: &Path) -> String;
}
```

## Consequences

### Positive

- **Consistent metrics** — Same complexity calculation logic for all languages
- **Easy to add languages** — New language = ~200 lines of queries + rules
- **Single testing strategy** — Test the shared engine once
- **Better maintainability** — Bug fixes apply to all languages
- **Config-driven extension** — Future languages could be defined via TOML (per spec §8.3)

### Negative

- **Loss of syn's Rust-specific semantics** — Tree-sitter provides less semantic understanding than syn
- **Tree-sitter version management** — Grammar versions need to be pinned together
- **Query complexity** — Tree-sitter queries can be complex to write and debug

### Mitigations

1. **Version pinning** — All tree-sitter grammars are pinned in `Cargo.toml` workspace dependencies
2. **Query testing** — Each grammar includes tests that verify queries parse successfully
3. **Regression testing** — Rust output is compared against baseline to detect drift

## Alternatives Considered

### 1. Keep syn for Rust, tree-sitter for others

**Rejected** because:
- Duplicated complexity logic
- Different code paths for each language family
- Higher maintenance burden

### 2. Use Language Server Protocol (LSP)

**Rejected** because:
- Requires running language servers
- Heavy dependency for CLI tool
- Overkill for static analysis

### 3. Use AST-based libraries per language

**Rejected** because:
- Same problem as syn (different library per language)
- Less consistent than tree-sitter's unified approach

## Implementation

- **M1**: Framework core with Grammar trait and shared complexity engine
- **M2**: RustGrammar implementation with tree-sitter-rust
- **M3**: PythonGrammar implementation with tree-sitter-python
- **M4**: CLI integration with auto-detection
- **M5**: Documentation updates

## References

- [Tree-sitter documentation](https://tree-sitter.github.io/tree-sitter/)
- [Spec §6: Language Adapter Interface](../01_spec.md#6-language-adapter-interface)
- [Spec §7: Supported Languages](../01_spec.md#7-supported-languages)
- [Spec §8: Adding New Languages](../01_spec.md#8-adding-new-languages)
