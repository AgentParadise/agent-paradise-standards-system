//! Language grammar definitions.
//!
//! Each supported language implements the [`Grammar`] trait, which defines:
//! - File extensions to match
//! - Tree-sitter language and queries
//! - Complexity calculation rules
//!
//! ## Adding a New Language
//!
//! 1. Create a new module (e.g., `python.rs`)
//! 2. Implement the [`Grammar`] trait
//! 3. Register the grammar in your adapter's registry
//!
//! ```ignore
//! use code_topology::adapter::grammars::Grammar;
//!
//! pub struct PythonGrammar;
//!
//! impl Grammar for PythonGrammar {
//!     fn language_id(&self) -> &'static str { "python" }
//!     fn file_extensions(&self) -> &'static [&'static str] { &[".py", ".pyi"] }
//!     // ... other methods
//! }
//! ```

pub mod python;
pub mod rust;

pub use python::PythonGrammar;
pub use rust::RustGrammar;

use std::path::Path;

use tree_sitter::Language;

// ============================================================================
// Grammar Trait
// ============================================================================

/// A language grammar definition for tree-sitter parsing.
///
/// This trait defines everything needed to analyze a programming language:
/// - Parser configuration (language, extensions)
/// - Query patterns (functions, calls, imports)
/// - Complexity rules (decision nodes, nesting nodes)
pub trait Grammar: Send + Sync {
    /// Language identifier (e.g., "rust", "python", "typescript").
    ///
    /// This should be a short, lowercase string used for:
    /// - Registry lookups
    /// - Qualified name prefixes
    /// - CLI arguments
    fn language_id(&self) -> &'static str;

    /// File extensions this grammar handles.
    ///
    /// Must include the leading dot (e.g., `".rs"`, `".py"`).
    fn file_extensions(&self) -> &'static [&'static str];

    /// Get the tree-sitter Language for parsing.
    fn ts_language(&self) -> Language;

    /// Tree-sitter query pattern for extracting function definitions.
    ///
    /// The query should capture:
    /// - `@function.name` or `@name` - Function name identifier
    /// - `@function` - The entire function node
    /// - `@function.body` or `@body` - The function body (optional)
    /// - `@function.params` - Parameters (optional)
    ///
    /// Example (Rust):
    /// ```scheme
    /// (function_item
    ///   name: (identifier) @function.name
    ///   body: (block) @function.body) @function
    /// ```
    fn function_query(&self) -> &str;

    /// Tree-sitter query pattern for extracting function calls.
    ///
    /// The query should capture:
    /// - `@call.name` - Simple function name
    /// - `@call.method` - Method call name
    ///
    /// Example (Rust):
    /// ```scheme
    /// (call_expression
    ///   function: (identifier) @call.name)
    /// ```
    fn call_query(&self) -> &str;

    /// Tree-sitter query pattern for extracting imports.
    ///
    /// The query should capture:
    /// - `@import.path` - The import path
    /// - `@import.source` - The module being imported from
    ///
    /// Example (Rust):
    /// ```scheme
    /// (use_declaration
    ///   argument: (use_tree) @import.path)
    /// ```
    fn import_query(&self) -> &str;

    /// Node types that increase cyclomatic complexity.
    ///
    /// Each occurrence of these node types adds 1 to the cyclomatic complexity.
    /// Common examples:
    /// - `if_statement`, `else_clause`
    /// - `for_statement`, `while_statement`
    /// - `match_arm`, `switch_case`
    /// - `binary_expression` (for && and ||)
    fn decision_nodes(&self) -> &'static [&'static str];

    /// Node types that increase cognitive complexity nesting penalty.
    ///
    /// When inside these nodes, decision points get an additional penalty
    /// based on the nesting depth. Common examples:
    /// - `if_statement`, `for_statement`, `while_statement`
    /// - `closure_expression`, `lambda`
    /// - `try_statement`
    fn nesting_nodes(&self) -> &'static [&'static str];

    /// Node types to ignore for complexity calculation.
    ///
    /// These are language-specific exceptions that shouldn't count as
    /// decision points. Examples:
    /// - Rust: `try_expression` (? operator)
    /// - Python: `with_statement`
    /// - TypeScript: `optional_chain_expression` (?.)
    fn ignored_nodes(&self) -> &'static [&'static str] {
        &[]
    }

    /// Compute the module path from a file path.
    ///
    /// This converts a file system path to a logical module path.
    /// The implementation depends on the language's module system.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the source file
    /// * `root` - Root directory of the analysis
    ///
    /// # Examples
    ///
    /// - Rust: `src/auth/validator.rs` → `crate::auth::validator`
    /// - Python: `app/services/auth.py` → `app.services.auth`
    /// - TypeScript: `src/api/handlers.ts` → `src/api/handlers`
    fn compute_module_path(&self, file_path: &Path, root: &Path) -> String;

    /// Optional: Tree-sitter query for extracting type definitions.
    ///
    /// Used for calculating abstractness metrics.
    fn type_query(&self) -> &str {
        ""
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test grammar for verification
    struct DummyGrammar;

    impl Grammar for DummyGrammar {
        fn language_id(&self) -> &'static str {
            "dummy"
        }

        fn file_extensions(&self) -> &'static [&'static str] {
            &[".dummy"]
        }

        fn ts_language(&self) -> Language {
            panic!("DummyGrammar doesn't have a real language")
        }

        fn function_query(&self) -> &str {
            "(function_definition name: (identifier) @function.name) @function"
        }

        fn call_query(&self) -> &str {
            "(call_expression function: (identifier) @call.name)"
        }

        fn import_query(&self) -> &str {
            "(import_statement name: (identifier) @import.path)"
        }

        fn decision_nodes(&self) -> &'static [&'static str] {
            &["if_statement", "for_statement"]
        }

        fn nesting_nodes(&self) -> &'static [&'static str] {
            &["if_statement", "for_statement"]
        }

        fn compute_module_path(&self, file_path: &Path, _root: &Path) -> String {
            file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string()
        }
    }

    #[test]
    fn test_grammar_trait_methods() {
        let grammar = DummyGrammar;

        assert_eq!(grammar.language_id(), "dummy");
        assert_eq!(grammar.file_extensions(), &[".dummy"]);
        assert!(!grammar.function_query().is_empty());
        assert!(!grammar.call_query().is_empty());
        assert!(!grammar.import_query().is_empty());
        assert_eq!(grammar.decision_nodes().len(), 2);
        assert_eq!(grammar.nesting_nodes().len(), 2);
        assert!(grammar.ignored_nodes().is_empty()); // Default
        assert!(grammar.type_query().is_empty()); // Default
    }

    #[test]
    fn test_compute_module_path() {
        let grammar = DummyGrammar;
        let path = Path::new("src/foo/bar.dummy");
        let root = Path::new("src");

        let module = grammar.compute_module_path(path, root);
        assert_eq!(module, "bar");
    }
}
