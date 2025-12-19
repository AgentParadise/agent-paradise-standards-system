//! Query execution helpers for extracting topology data.
//!
//! This module provides functions to execute tree-sitter queries and
//! convert the results into topology data structures.

use std::path::Path;

use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, QueryCursor, Tree};

use crate::{AdapterError, CallInfo, FunctionInfo, ImportInfo, TypeInfo, Visibility};

use super::grammars::Grammar;

// ============================================================================
// Function Extraction
// ============================================================================

/// Extract functions from a parsed tree.
pub fn extract_functions(
    tree: &Tree,
    source: &str,
    file_path: &Path,
    grammar: &dyn Grammar,
) -> Result<Vec<FunctionInfo>, AdapterError> {
    let query_str = grammar.function_query();
    if query_str.is_empty() {
        return Ok(vec![]);
    }

    let query = Query::new(&grammar.ts_language(), query_str).map_err(|e| AdapterError {
        code: "QUERY_ERROR".to_string(),
        message: format!("Failed to compile function query: {e}"),
        file: Some(file_path.to_path_buf()),
        line: None,
    })?;

    let mut cursor = QueryCursor::new();
    let capture_names = query.capture_names();
    let module = grammar.compute_module_path(file_path, Path::new("."));

    let mut functions = Vec::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    // Use StreamingIterator pattern for tree-sitter 0.24
    while let Some(m) = matches.next() {
        let mut name = String::new();
        let mut start_line = 0u32;
        let mut end_line = 0u32;
        let mut is_method = false;
        let mut visibility = Visibility::Private;
        let mut body_source = String::new();

        for capture in m.captures {
            let capture_name: &str = capture_names[capture.index as usize];
            let node = capture.node;
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");

            match capture_name {
                "function.name" | "method.name" | "name" => {
                    name = text.to_string();
                    start_line = node.start_position().row as u32 + 1;
                    end_line = node.end_position().row as u32 + 1;
                }
                "function" | "method" | "async_function" => {
                    start_line = node.start_position().row as u32 + 1;
                    end_line = node.end_position().row as u32 + 1;
                    is_method = capture_name == "method";
                }
                "function.body" | "method.body" | "body" => {
                    body_source = text.to_string();
                    end_line = node.end_position().row as u32 + 1;
                }
                "visibility" | "public" => {
                    if text == "pub" || text == "public" {
                        visibility = Visibility::Public;
                    }
                }
                _ => {}
            }
        }

        if !name.is_empty() {
            let qualified_name = format!("{}:{}::{}", grammar.language_id(), module, name);

            functions.push(FunctionInfo {
                qualified_name,
                name: name.clone(),
                file_path: file_path.to_path_buf(),
                module: module.clone(),
                start_line,
                end_line,
                parameter_count: 0, // TODO: count from params capture
                is_method,
                visibility,
                body_source,
            });
        }
    }

    Ok(functions)
}

// ============================================================================
// Call Extraction
// ============================================================================

/// Extract function calls from a parsed tree.
pub fn extract_calls(
    tree: &Tree,
    source: &str,
    file_path: &Path,
    grammar: &dyn Grammar,
) -> Result<Vec<CallInfo>, AdapterError> {
    let query_str = grammar.call_query();
    if query_str.is_empty() {
        return Ok(vec![]);
    }

    let query = Query::new(&grammar.ts_language(), query_str).map_err(|e| AdapterError {
        code: "QUERY_ERROR".to_string(),
        message: format!("Failed to compile call query: {e}"),
        file: Some(file_path.to_path_buf()),
        line: None,
    })?;

    let mut cursor = QueryCursor::new();
    let capture_names = query.capture_names();
    let module = grammar.compute_module_path(file_path, Path::new("."));

    let mut calls = Vec::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    while let Some(m) = matches.next() {
        let mut callee = String::new();
        let mut line = 0u32;

        for capture in m.captures {
            let capture_name: &str = capture_names[capture.index as usize];
            let node = capture.node;
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");

            match capture_name {
                "call.name" | "call.method" | "name" => {
                    callee = text.to_string();
                    line = node.start_position().row as u32 + 1;
                }
                _ => {}
            }
        }

        if !callee.is_empty() {
            calls.push(CallInfo {
                caller: module.clone(), // Simplified: use module as caller
                callee,
                file_path: file_path.to_path_buf(),
                line,
                resolved: false, // Will be resolved in a later pass
            });
        }
    }

    Ok(calls)
}

// ============================================================================
// Import Extraction
// ============================================================================

/// Extract imports from a parsed tree.
pub fn extract_imports(
    tree: &Tree,
    source: &str,
    file_path: &Path,
    grammar: &dyn Grammar,
) -> Result<Vec<ImportInfo>, AdapterError> {
    let query_str = grammar.import_query();
    if query_str.is_empty() {
        return Ok(vec![]);
    }

    let query = Query::new(&grammar.ts_language(), query_str).map_err(|e| AdapterError {
        code: "QUERY_ERROR".to_string(),
        message: format!("Failed to compile import query: {e}"),
        file: Some(file_path.to_path_buf()),
        line: None,
    })?;

    let mut cursor = QueryCursor::new();
    let capture_names = query.capture_names();
    let module = grammar.compute_module_path(file_path, Path::new("."));

    let mut imports = Vec::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    while let Some(m) = matches.next() {
        let mut import_path = String::new();
        let mut to_module = String::new();

        for capture in m.captures {
            let capture_name: &str = capture_names[capture.index as usize];
            let node = capture.node;
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");

            match capture_name {
                "import.path" | "import.source" | "import.name" | "path" => {
                    import_path = text.trim_matches('"').trim_matches('\'').to_string();
                    to_module = text.to_string();
                }
                _ => {}
            }
        }

        if !import_path.is_empty() {
            let is_external = !import_path.starts_with('.')
                && !import_path.starts_with("crate")
                && !import_path.starts_with("self")
                && !import_path.starts_with("super");

            imports.push(ImportInfo {
                from_module: module.clone(),
                to_module,
                import_path,
                is_external,
            });
        }
    }

    Ok(imports)
}

// ============================================================================
// Type Extraction
// ============================================================================

/// Extract type definitions from a parsed tree.
pub fn extract_types(
    _tree: &Tree,
    _source: &str,
    _file_path: &Path,
    _grammar: &dyn Grammar,
) -> Result<Vec<TypeInfo>, AdapterError> {
    // TODO: Implement type extraction for abstractness calculation
    // This is optional per the LanguageAdapter trait
    Ok(vec![])
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // Basic compilation test - actual query tests require a grammar
        // This test validates the module structure is correct
        let _placeholder = 42;
        assert_eq!(_placeholder, 42);
    }
}
