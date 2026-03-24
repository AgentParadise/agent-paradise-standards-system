//! Rust language grammar for tree-sitter based analysis.
//!
//! This grammar defines the queries and complexity rules for analyzing
//! Rust source code using tree-sitter.

use std::path::Path;

use tree_sitter::Language;

use super::Grammar;

// ============================================================================
// Rust Grammar
// ============================================================================

/// Rust language grammar implementation.
pub struct RustGrammar;

impl RustGrammar {
    /// Create a new Rust grammar.
    pub fn new() -> Self {
        Self
    }
}

impl Default for RustGrammar {
    fn default() -> Self {
        Self::new()
    }
}

impl Grammar for RustGrammar {
    fn language_id(&self) -> &'static str {
        "rust"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &[".rs"]
    }

    fn ts_language(&self) -> Language {
        tree_sitter_rust::LANGUAGE.into()
    }

    fn function_query(&self) -> &str {
        RUST_FUNCTION_QUERY
    }

    fn call_query(&self) -> &str {
        RUST_CALL_QUERY
    }

    fn import_query(&self) -> &str {
        RUST_IMPORT_QUERY
    }

    fn decision_nodes(&self) -> &'static [&'static str] {
        &[
            "if_expression",
            "else_clause",
            "match_arm",
            "while_expression",
            "loop_expression",
            "for_expression",
            "binary_expression", // For && and || operators
        ]
    }

    fn nesting_nodes(&self) -> &'static [&'static str] {
        &[
            "if_expression",
            "match_expression",
            "while_expression",
            "loop_expression",
            "for_expression",
            "closure_expression",
            "async_block",
        ]
    }

    fn ignored_nodes(&self) -> &'static [&'static str] {
        &[
            "try_expression", // ? operator - idiomatic, no CC penalty
        ]
    }

    fn compute_module_path(&self, file_path: &Path, root: &Path) -> String {
        // Get relative path from root
        let relative = file_path.strip_prefix(root).unwrap_or(file_path);

        // Remove extension and convert to module path
        let path_str = relative
            .with_extension("")
            .to_string_lossy()
            .replace(['/', '\\'], "::");

        // Handle special files
        let file_name = relative.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        if file_name == "lib" || file_name == "main" {
            // Root module - use parent path as module ID.
            // Strip the filename, then strip trailing "::src" segment.
            let without_file = path_str
                .rsplit_once("::")
                .map(|(prefix, _)| prefix)
                .unwrap_or("");

            if without_file.is_empty() || without_file == "src" {
                return "crate".to_string();
            }

            // Strip trailing ::src since it's just Rust convention, not a meaningful boundary
            let without_src = without_file.strip_suffix("::src").unwrap_or(without_file);

            return without_src.replace('-', "_");
        }

        if file_name == "mod" {
            // Module file - strip ::mod, then remove any src:: segments
            let without_mod = path_str.trim_end_matches("::mod");
            return remove_src_segments(without_mod);
        }

        // Regular file - remove src:: segments from path
        remove_src_segments(&path_str)
    }
}

/// Remove `src` segments from module paths.
/// e.g., `crates::aps_cli::src::parser` → `crates::aps_cli::parser`
fn remove_src_segments(path: &str) -> String {
    path.split("::")
        .filter(|seg| *seg != "src")
        .collect::<Vec<_>>()
        .join("::")
}

// ============================================================================
// Query Definitions
// ============================================================================

/// Tree-sitter query for extracting Rust function definitions.
///
/// Captures:
/// - `@function.name` - Function identifier
/// - `@function` - The entire function item
/// - `@function.body` - The function body block
/// - `@method.name` - Method identifier (in impl blocks)
/// - `@method` - The entire method
/// - `@visibility` - Visibility modifier (pub, pub(crate), etc.)
const RUST_FUNCTION_QUERY: &str = r#"
; Top-level functions
(function_item
  (visibility_modifier)? @visibility
  name: (identifier) @function.name
  body: (block) @function.body) @function

; Methods in impl blocks
(impl_item
  body: (declaration_list
    (function_item
      (visibility_modifier)? @visibility
      name: (identifier) @method.name
      body: (block) @method.body) @method))

; Trait method declarations (without body)
(trait_item
  body: (declaration_list
    (function_signature_item
      name: (identifier) @function.name) @function))
"#;

/// Tree-sitter query for extracting Rust function calls.
///
/// Captures:
/// - `@call.name` - Simple function call identifier
/// - `@call.method` - Method call identifier
const RUST_CALL_QUERY: &str = r#"
; Simple function calls: foo()
(call_expression
  function: (identifier) @call.name)

; Scoped function calls: module::foo()
(call_expression
  function: (scoped_identifier
    name: (identifier) @call.name))

; Method calls: obj.method()
(call_expression
  function: (field_expression
    field: (field_identifier) @call.method))
"#;

/// Tree-sitter query for extracting Rust imports.
///
/// Captures:
/// - `@import.path` - The import path identifier
/// - `@import.wildcard` - Wildcard imports (use foo::*)
/// - `@import.symbol` - Individual symbols in use lists
/// - `@import.list` - The entire use list for multi-imports
const RUST_IMPORT_QUERY: &str = r#"
; Scoped use: use foo::bar::baz (single import)
(use_declaration
  (scoped_identifier) @import.path)

; Simple use: use foo (module import)
(use_declaration
  (identifier) @import.path)

; Wildcard use: use foo::* 
(use_declaration
  (use_wildcard) @import.wildcard)

; Use with path in scoped use list: use foo::{bar, baz}
(use_declaration
  (scoped_use_list
    path: (identifier) @import.path
    list: (use_list) @import.list))

(use_declaration
  (scoped_use_list
    path: (scoped_identifier) @import.path
    list: (use_list) @import.list))

; Capture individual symbols in use lists
(use_list
  (identifier) @import.symbol)

(use_list
  (scoped_identifier) @import.symbol)
"#;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_grammar_basics() {
        let grammar = RustGrammar::new();

        assert_eq!(grammar.language_id(), "rust");
        assert_eq!(grammar.file_extensions(), &[".rs"]);
        assert!(!grammar.function_query().is_empty());
        assert!(!grammar.call_query().is_empty());
        assert!(!grammar.import_query().is_empty());
    }

    #[test]
    fn test_rust_decision_nodes() {
        let grammar = RustGrammar::new();
        let nodes = grammar.decision_nodes();

        assert!(nodes.contains(&"if_expression"));
        assert!(nodes.contains(&"match_arm"));
        assert!(nodes.contains(&"for_expression"));
    }

    #[test]
    fn test_rust_nesting_nodes() {
        let grammar = RustGrammar::new();
        let nodes = grammar.nesting_nodes();

        assert!(nodes.contains(&"if_expression"));
        assert!(nodes.contains(&"closure_expression"));
        assert!(nodes.contains(&"async_block"));
    }

    #[test]
    fn test_rust_ignored_nodes() {
        let grammar = RustGrammar::new();
        let nodes = grammar.ignored_nodes();

        assert!(nodes.contains(&"try_expression"));
    }

    #[test]
    fn test_compute_module_path_simple() {
        let grammar = RustGrammar::new();
        let path = Path::new("src/auth/validator.rs");
        let root = Path::new("src");

        let module = grammar.compute_module_path(path, root);
        assert_eq!(module, "auth::validator");
    }

    #[test]
    fn test_compute_module_path_lib() {
        let grammar = RustGrammar::new();
        let path = Path::new("src/lib.rs");
        let root = Path::new(".");

        let module = grammar.compute_module_path(path, root);
        // Should return "crate" for lib.rs
        assert_eq!(module, "crate");
    }

    #[test]
    fn test_compute_module_path_mod() {
        let grammar = RustGrammar::new();
        let path = Path::new("src/auth/mod.rs");
        let root = Path::new("src");

        let module = grammar.compute_module_path(path, root);
        assert_eq!(module, "auth");
    }

    #[test]
    fn test_ts_language_loads() {
        let grammar = RustGrammar::new();
        let lang = grammar.ts_language();

        // Verify the language is valid by checking version
        assert!(lang.version() > 0);
    }

    #[test]
    fn test_function_query_parses() {
        let grammar = RustGrammar::new();
        let lang = grammar.ts_language();

        // Verify the query compiles
        let result = tree_sitter::Query::new(&lang, grammar.function_query());
        assert!(
            result.is_ok(),
            "Function query failed to parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_call_query_parses() {
        let grammar = RustGrammar::new();
        let lang = grammar.ts_language();

        let result = tree_sitter::Query::new(&lang, grammar.call_query());
        assert!(
            result.is_ok(),
            "Call query failed to parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_import_query_parses() {
        let grammar = RustGrammar::new();
        let lang = grammar.ts_language();

        let result = tree_sitter::Query::new(&lang, grammar.import_query());
        assert!(
            result.is_ok(),
            "Import query failed to parse: {:?}",
            result.err()
        );
    }
}
