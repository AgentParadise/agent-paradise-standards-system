//! Python language grammar for tree-sitter based analysis.
//!
//! This grammar defines the queries and complexity rules for analyzing
//! Python source code using tree-sitter.
//!
//! ## Complexity Rules (from spec §7.2)
//!
//! **Increases CC:**
//! - if/elif/else branches
//! - for/while loops
//! - try/except (each except +1)
//! - list comprehensions with conditions
//! - assert statements
//! - logical operators (and, or)
//!
//! **Does NOT increase CC:**
//! - with statements (context managers)
//! - finally clause
//! - raise statements

use std::path::Path;

use tree_sitter::Language;

use super::Grammar;

// ============================================================================
// Python Grammar
// ============================================================================

/// Python language grammar implementation.
pub struct PythonGrammar;

impl PythonGrammar {
    /// Create a new Python grammar.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PythonGrammar {
    fn default() -> Self {
        Self::new()
    }
}

impl Grammar for PythonGrammar {
    fn language_id(&self) -> &'static str {
        "python"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &[".py", ".pyi"]
    }

    fn ts_language(&self) -> Language {
        tree_sitter_python::LANGUAGE.into()
    }

    fn function_query(&self) -> &str {
        PYTHON_FUNCTION_QUERY
    }

    fn call_query(&self) -> &str {
        PYTHON_CALL_QUERY
    }

    fn import_query(&self) -> &str {
        PYTHON_IMPORT_QUERY
    }

    fn type_query(&self) -> &str {
        PYTHON_TYPE_QUERY
    }

    fn decision_nodes(&self) -> &'static [&'static str] {
        &[
            "if_statement",
            "elif_clause",
            "else_clause",
            "for_statement",
            "while_statement",
            "except_clause",
            "list_comprehension", // With conditions
            "conditional_expression",
            "boolean_operator", // and, or
            "assert_statement",
        ]
    }

    fn nesting_nodes(&self) -> &'static [&'static str] {
        &[
            "if_statement",
            "for_statement",
            "while_statement",
            "try_statement",
            "with_statement",
            "function_definition",
            "class_definition",
            "lambda",
        ]
    }

    fn ignored_nodes(&self) -> &'static [&'static str] {
        // Note: with_statement is in nesting_nodes (adds nesting depth)
        // but NOT in decision_nodes (doesn't add to CC)
        &[
            "finally_clause",  // Finally doesn't add CC
            "raise_statement", // Raise doesn't add CC
        ]
    }

    fn compute_module_path(&self, file_path: &Path, root: &Path) -> String {
        // Get relative path from root
        let relative = file_path.strip_prefix(root).unwrap_or(file_path);

        // Remove extension and convert to Python module path
        let path_str = relative
            .with_extension("")
            .to_string_lossy()
            .replace(['/', '\\'], ".");

        // Handle __init__.py files
        if path_str.ends_with(".__init__") {
            return path_str.trim_end_matches(".__init__").to_string();
        }

        // Handle __main__.py files
        if path_str.ends_with(".__main__") {
            return path_str.trim_end_matches(".__main__").to_string();
        }

        path_str
    }
}

// ============================================================================
// Query Definitions
// ============================================================================

/// Tree-sitter query for extracting Python function definitions.
///
/// Captures:
/// - `@function.name` - Function name identifier
/// - `@function` - The entire function definition
/// - `@function.body` - The function body block
/// - `@method.name` - Method name (in class)
/// - `@method` - The entire method
const PYTHON_FUNCTION_QUERY: &str = r#"
; Function definitions
(function_definition
  name: (identifier) @function.name
  body: (block) @function.body) @function

; Async function definitions
(function_definition
  "async"
  name: (identifier) @function.name
  body: (block) @function.body) @async_function

; Methods in classes
(class_definition
  body: (block
    (function_definition
      name: (identifier) @method.name
      body: (block) @method.body) @method))
"#;

/// Tree-sitter query for extracting Python function calls.
///
/// Captures:
/// - `@call.name` - Simple function call identifier
/// - `@call.method` - Method call identifier
const PYTHON_CALL_QUERY: &str = r#"
; Simple function calls: foo()
(call
  function: (identifier) @call.name)

; Method calls: obj.method()
(call
  function: (attribute
    attribute: (identifier) @call.method))
"#;

/// Tree-sitter query for extracting Python imports.
///
/// Captures:
/// - `@import.path` - The import module path
/// - `@import.source` - The module imported from (in from...import)
const PYTHON_IMPORT_QUERY: &str = r#"
; import foo, import foo.bar
(import_statement
  name: (dotted_name) @import.path)

; from foo import bar
(import_from_statement
  module_name: (dotted_name) @import.source)

; from foo import bar (aliased or multiple)
(import_from_statement
  module_name: (relative_import
    (dotted_name) @import.source))
"#;

/// Tree-sitter query for extracting Python type definitions.
///
/// Captures:
/// - `@class.name` - Class name identifier
/// - `@class` - The entire class definition
/// - `@class.abstract` - Abstract classes (inheriting from ABC/Protocol)
/// - `@method.abstract` - Methods with @abstractmethod decorator
const PYTHON_TYPE_QUERY: &str = r#"
; All class definitions
(class_definition
  name: (identifier) @class.name) @class

; Classes inheriting from ABC
(class_definition
  name: (identifier) @class.name
  superclasses: (argument_list
    (identifier) @base (#match? @base "^(ABC|ABCMeta|Protocol)$"))) @class.abstract

; Classes inheriting from typing.Protocol or abc.ABC
(class_definition
  name: (identifier) @class.name
  superclasses: (argument_list
    (attribute
      attribute: (identifier) @base (#match? @base "^(ABC|ABCMeta|Protocol)$")))) @class.abstract

; Methods with @abstractmethod decorator
(decorated_definition
  (decorator
    (identifier) @decorator (#eq? @decorator "abstractmethod"))
  definition: (function_definition
    name: (identifier) @method.abstract.name)) @method.abstract

; Methods with @abc.abstractmethod decorator
(decorated_definition
  (decorator
    (attribute
      attribute: (identifier) @decorator (#eq? @decorator "abstractmethod")))
  definition: (function_definition
    name: (identifier) @method.abstract.name)) @method.abstract
"#;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_grammar_basics() {
        let grammar = PythonGrammar::new();

        assert_eq!(grammar.language_id(), "python");
        assert_eq!(grammar.file_extensions(), &[".py", ".pyi"]);
        assert!(!grammar.function_query().is_empty());
        assert!(!grammar.call_query().is_empty());
        assert!(!grammar.import_query().is_empty());
    }

    #[test]
    fn test_python_decision_nodes() {
        let grammar = PythonGrammar::new();
        let nodes = grammar.decision_nodes();

        assert!(nodes.contains(&"if_statement"));
        assert!(nodes.contains(&"for_statement"));
        assert!(nodes.contains(&"except_clause"));
        assert!(nodes.contains(&"boolean_operator"));
    }

    #[test]
    fn test_python_nesting_nodes() {
        let grammar = PythonGrammar::new();
        let nodes = grammar.nesting_nodes();

        assert!(nodes.contains(&"if_statement"));
        assert!(nodes.contains(&"function_definition"));
        assert!(nodes.contains(&"class_definition"));
        assert!(nodes.contains(&"lambda"));
    }

    #[test]
    fn test_python_ignored_nodes() {
        let grammar = PythonGrammar::new();
        let nodes = grammar.ignored_nodes();

        // with_statement is in nesting_nodes (adds nesting) but NOT in ignored_nodes
        assert!(!nodes.contains(&"with_statement"));
        assert!(nodes.contains(&"finally_clause"));
        assert!(nodes.contains(&"raise_statement"));
    }

    #[test]
    fn test_compute_module_path_simple() {
        let grammar = PythonGrammar::new();
        let path = Path::new("app/services/auth.py");
        let root = Path::new(".");

        let module = grammar.compute_module_path(path, root);
        assert_eq!(module, "app.services.auth");
    }

    #[test]
    fn test_compute_module_path_init() {
        let grammar = PythonGrammar::new();
        let path = Path::new("app/services/__init__.py");
        let root = Path::new(".");

        let module = grammar.compute_module_path(path, root);
        assert_eq!(module, "app.services");
    }

    #[test]
    fn test_compute_module_path_main() {
        let grammar = PythonGrammar::new();
        let path = Path::new("app/__main__.py");
        let root = Path::new(".");

        let module = grammar.compute_module_path(path, root);
        assert_eq!(module, "app");
    }

    #[test]
    fn test_ts_language_loads() {
        let grammar = PythonGrammar::new();
        let lang = grammar.ts_language();

        // Verify the language is valid by checking version
        assert!(lang.version() > 0);
    }

    #[test]
    fn test_function_query_parses() {
        let grammar = PythonGrammar::new();
        let lang = grammar.ts_language();

        let result = tree_sitter::Query::new(&lang, grammar.function_query());
        assert!(
            result.is_ok(),
            "Function query failed to parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_call_query_parses() {
        let grammar = PythonGrammar::new();
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
        let grammar = PythonGrammar::new();
        let lang = grammar.ts_language();

        let result = tree_sitter::Query::new(&lang, grammar.import_query());
        assert!(
            result.is_ok(),
            "Import query failed to parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_type_query_parses() {
        let grammar = PythonGrammar::new();
        let lang = grammar.ts_language();

        let result = tree_sitter::Query::new(&lang, grammar.type_query());
        assert!(
            result.is_ok(),
            "Type query failed to parse: {:?}",
            result.err()
        );
    }
}
