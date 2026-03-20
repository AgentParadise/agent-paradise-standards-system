//! TypeScript and TSX language grammars for tree-sitter based analysis.
//!
//! Both `TypeScriptGrammar` (`.ts`) and `TsxGrammar` (`.tsx`) share identical
//! queries and complexity rules — TSX is TypeScript plus JSX syntax, and JSX
//! nodes do not affect cyclomatic complexity.
//!
//! ## Complexity Rules
//!
//! **Increases CC:**
//! - if/else branches
//! - for/for-in/for-of loops
//! - while/do-while loops
//! - switch cases
//! - catch clauses
//! - ternary expressions
//! - logical operators (&& and ||)
//!
//! **Does NOT increase CC:**
//! - optional chaining (`?.`) — idiomatic null-safety, no penalty

use std::path::Path;

use tree_sitter::Language;

use super::Grammar;

// ============================================================================
// TypeScript Grammar
// ============================================================================

/// TypeScript language grammar implementation (`.ts` files).
pub struct TypeScriptGrammar;

impl TypeScriptGrammar {
    /// Create a new TypeScript grammar.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypeScriptGrammar {
    fn default() -> Self {
        Self::new()
    }
}

impl Grammar for TypeScriptGrammar {
    fn language_id(&self) -> &'static str {
        "typescript"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &[".ts"]
    }

    fn ts_language(&self) -> Language {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
    }

    fn function_query(&self) -> &str {
        TS_FUNCTION_QUERY
    }

    fn call_query(&self) -> &str {
        TS_CALL_QUERY
    }

    fn import_query(&self) -> &str {
        TS_IMPORT_QUERY
    }

    fn type_query(&self) -> &str {
        TS_TYPE_QUERY
    }

    fn decision_nodes(&self) -> &'static [&'static str] {
        TS_DECISION_NODES
    }

    fn nesting_nodes(&self) -> &'static [&'static str] {
        TS_NESTING_NODES
    }

    fn ignored_nodes(&self) -> &'static [&'static str] {
        TS_IGNORED_NODES
    }

    fn compute_module_path(&self, file_path: &Path, root: &Path) -> String {
        compute_ts_module_path(file_path, root)
    }
}

// ============================================================================
// TSX Grammar
// ============================================================================

/// TSX language grammar implementation (`.tsx` files).
pub struct TsxGrammar;

impl TsxGrammar {
    /// Create a new TSX grammar.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TsxGrammar {
    fn default() -> Self {
        Self::new()
    }
}

impl Grammar for TsxGrammar {
    fn language_id(&self) -> &'static str {
        "tsx"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &[".tsx"]
    }

    fn ts_language(&self) -> Language {
        tree_sitter_typescript::LANGUAGE_TSX.into()
    }

    fn function_query(&self) -> &str {
        TS_FUNCTION_QUERY
    }

    fn call_query(&self) -> &str {
        TS_CALL_QUERY
    }

    fn import_query(&self) -> &str {
        TS_IMPORT_QUERY
    }

    fn type_query(&self) -> &str {
        TS_TYPE_QUERY
    }

    fn decision_nodes(&self) -> &'static [&'static str] {
        TS_DECISION_NODES
    }

    fn nesting_nodes(&self) -> &'static [&'static str] {
        TS_NESTING_NODES
    }

    fn ignored_nodes(&self) -> &'static [&'static str] {
        TS_IGNORED_NODES
    }

    fn compute_module_path(&self, file_path: &Path, root: &Path) -> String {
        compute_ts_module_path(file_path, root)
    }
}

// ============================================================================
// Shared complexity rules
// ============================================================================

const TS_DECISION_NODES: &[&str] = &[
    "if_statement",
    "else_clause",
    "for_statement",
    "for_in_statement",
    "while_statement",
    "do_statement",
    "switch_case",
    "catch_clause",
    "ternary_expression",
    "binary_expression", // && and ||
];

const TS_NESTING_NODES: &[&str] = &[
    "if_statement",
    "for_statement",
    "for_in_statement",
    "while_statement",
    "do_statement",
    "switch_statement",
    "try_statement",
    "arrow_function",
    "function",
    "function_declaration",
    "function_expression",
    "method_definition",
];

const TS_IGNORED_NODES: &[&str] = &[
    "optional_chain", // ?. operator — idiomatic null-safety, no CC penalty
];

// ============================================================================
// Shared module path logic
// ============================================================================

fn compute_ts_module_path(file_path: &Path, root: &Path) -> String {
    let relative = file_path.strip_prefix(root).unwrap_or(file_path);

    // Strip extension
    let without_ext = relative.with_extension("");
    let path_str = without_ext.to_string_lossy();

    // Normalize separators
    let normalized = path_str.replace('\\', "/");

    // Strip trailing /index (barrel exports resolve to parent directory)
    if normalized.ends_with("/index") {
        return normalized[..normalized.len() - "/index".len()].to_string();
    }

    normalized
}

// ============================================================================
// Query Definitions
// ============================================================================

/// Tree-sitter query for extracting TypeScript/TSX function definitions.
///
/// Captures named functions, arrow functions, methods, and function expressions.
/// Note: function expressions use `function_expression`, not `function`.
const TS_FUNCTION_QUERY: &str = r#"
; Named function declarations
(function_declaration
  name: (identifier) @function.name
  body: (statement_block) @function.body) @function

; Arrow functions assigned to variables
(variable_declarator
  name: (identifier) @function.name
  value: (arrow_function
    body: (_) @function.body) @function)

; Method definitions in classes
(method_definition
  name: (property_identifier) @method.name
  body: (statement_block) @method.body) @method

; Function expressions assigned to variables
(variable_declarator
  name: (identifier) @function.name
  value: (function_expression
    body: (statement_block) @function.body) @function)

; Exported function declarations
(export_statement
  declaration: (function_declaration
    name: (identifier) @function.name
    body: (statement_block) @function.body) @function)
"#;

/// Tree-sitter query for extracting TypeScript/TSX function calls.
const TS_CALL_QUERY: &str = r#"
; Simple function calls: foo()
(call_expression
  function: (identifier) @call.name)

; Method calls: obj.method()
(call_expression
  function: (member_expression
    property: (property_identifier) @call.method))
"#;

/// Tree-sitter query for extracting TypeScript/TSX imports.
const TS_IMPORT_QUERY: &str = r#"
; import foo from "module"
; import { foo } from "module"
; import * as foo from "module"
(import_statement
  source: (string) @import.source)
"#;

/// Tree-sitter query for extracting TypeScript/TSX type definitions.
///
/// Captures interfaces, type aliases, classes, and abstract classes.
const TS_TYPE_QUERY: &str = r#"
; Interface declarations
(interface_declaration
  name: (type_identifier) @type.name) @type

; Type alias declarations
(type_alias_declaration
  name: (type_identifier) @type.name) @type

; Class declarations
(class_declaration
  name: (type_identifier) @type.name) @type

; Abstract class declarations
(abstract_class_declaration
  name: (type_identifier) @type.name) @type
"#;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_grammar_basics() {
        let grammar = TypeScriptGrammar::new();

        assert_eq!(grammar.language_id(), "typescript");
        assert_eq!(grammar.file_extensions(), &[".ts"]);
        assert!(!grammar.function_query().is_empty());
        assert!(!grammar.call_query().is_empty());
        assert!(!grammar.import_query().is_empty());
        assert!(!grammar.type_query().is_empty());
    }

    #[test]
    fn test_tsx_grammar_basics() {
        let grammar = TsxGrammar::new();

        assert_eq!(grammar.language_id(), "tsx");
        assert_eq!(grammar.file_extensions(), &[".tsx"]);
        assert!(!grammar.function_query().is_empty());
        assert!(!grammar.call_query().is_empty());
        assert!(!grammar.import_query().is_empty());
        assert!(!grammar.type_query().is_empty());
    }

    #[test]
    fn test_decision_nodes() {
        let grammar = TypeScriptGrammar::new();
        let nodes = grammar.decision_nodes();

        assert!(nodes.contains(&"if_statement"));
        assert!(nodes.contains(&"else_clause"));
        assert!(nodes.contains(&"for_statement"));
        assert!(nodes.contains(&"for_in_statement"));
        assert!(nodes.contains(&"while_statement"));
        assert!(nodes.contains(&"do_statement"));
        assert!(nodes.contains(&"switch_case"));
        assert!(nodes.contains(&"catch_clause"));
        assert!(nodes.contains(&"ternary_expression"));
        assert!(nodes.contains(&"binary_expression"));
    }

    #[test]
    fn test_nesting_nodes() {
        let grammar = TypeScriptGrammar::new();
        let nodes = grammar.nesting_nodes();

        assert!(nodes.contains(&"if_statement"));
        assert!(nodes.contains(&"for_statement"));
        assert!(nodes.contains(&"arrow_function"));
        assert!(nodes.contains(&"method_definition"));
    }

    #[test]
    fn test_ignored_nodes() {
        let grammar = TypeScriptGrammar::new();
        let nodes = grammar.ignored_nodes();

        assert!(nodes.contains(&"optional_chain"));
    }

    #[test]
    fn test_typescript_language_loads() {
        let grammar = TypeScriptGrammar::new();
        let lang = grammar.ts_language();
        assert!(lang.version() > 0);
    }

    #[test]
    fn test_tsx_language_loads() {
        let grammar = TsxGrammar::new();
        let lang = grammar.ts_language();
        assert!(lang.version() > 0);
    }

    #[test]
    fn test_typescript_function_query_parses() {
        let grammar = TypeScriptGrammar::new();
        let lang = grammar.ts_language();
        let result = tree_sitter::Query::new(&lang, grammar.function_query());
        assert!(result.is_ok(), "Function query failed: {:?}", result.err());
    }

    #[test]
    fn test_tsx_function_query_parses() {
        let grammar = TsxGrammar::new();
        let lang = grammar.ts_language();
        let result = tree_sitter::Query::new(&lang, grammar.function_query());
        assert!(result.is_ok(), "Function query failed: {:?}", result.err());
    }

    #[test]
    fn test_typescript_call_query_parses() {
        let grammar = TypeScriptGrammar::new();
        let lang = grammar.ts_language();
        let result = tree_sitter::Query::new(&lang, grammar.call_query());
        assert!(result.is_ok(), "Call query failed: {:?}", result.err());
    }

    #[test]
    fn test_typescript_import_query_parses() {
        let grammar = TypeScriptGrammar::new();
        let lang = grammar.ts_language();
        let result = tree_sitter::Query::new(&lang, grammar.import_query());
        assert!(result.is_ok(), "Import query failed: {:?}", result.err());
    }

    #[test]
    fn test_typescript_type_query_parses() {
        let grammar = TypeScriptGrammar::new();
        let lang = grammar.ts_language();
        let result = tree_sitter::Query::new(&lang, grammar.type_query());
        assert!(result.is_ok(), "Type query failed: {:?}", result.err());
    }

    #[test]
    fn test_tsx_type_query_parses() {
        let grammar = TsxGrammar::new();
        let lang = grammar.ts_language();
        let result = tree_sitter::Query::new(&lang, grammar.type_query());
        assert!(result.is_ok(), "Type query failed: {:?}", result.err());
    }

    #[test]
    fn test_compute_module_path_simple() {
        let grammar = TypeScriptGrammar::new();
        let path = Path::new("src/api/handlers.ts");
        let root = Path::new(".");

        let module = grammar.compute_module_path(path, root);
        assert_eq!(module, "src/api/handlers");
    }

    #[test]
    fn test_compute_module_path_index() {
        let grammar = TypeScriptGrammar::new();
        let path = Path::new("src/components/Button/index.ts");
        let root = Path::new(".");

        let module = grammar.compute_module_path(path, root);
        assert_eq!(module, "src/components/Button");
    }

    #[test]
    fn test_compute_module_path_tsx_index() {
        let grammar = TsxGrammar::new();
        let path = Path::new("src/components/Button/index.tsx");
        let root = Path::new(".");

        let module = grammar.compute_module_path(path, root);
        assert_eq!(module, "src/components/Button");
    }
}
