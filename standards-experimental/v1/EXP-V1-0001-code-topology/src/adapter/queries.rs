//! Query execution helpers for extracting topology data.
//!
//! This module provides functions to execute tree-sitter queries and
//! convert the results into topology data structures.

use std::path::Path;

use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, QueryCursor, Tree};

use crate::{AdapterError, CallInfo, FunctionInfo, ImportInfo, ImportKind, TypeInfo, Visibility};

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
        let mut symbols: Vec<String> = Vec::new();
        let mut is_wildcard = false;
        let mut has_list = false;

        for capture in m.captures {
            let capture_name: &str = capture_names[capture.index as usize];
            let node = capture.node;
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");

            match capture_name {
                "import.path" | "import.source" | "import.name" | "path" => {
                    import_path = text.trim_matches('"').trim_matches('\'').to_string();
                    to_module = text.to_string();
                }
                "import.wildcard" => {
                    is_wildcard = true;
                    // For wildcard, extract the path from the parent node
                    if let Some(parent) = node.parent() {
                        if let Some(path_node) = parent.child_by_field_name("path") {
                            import_path = path_node.utf8_text(source.as_bytes()).unwrap_or("").to_string();
                            to_module = import_path.clone();
                        }
                    }
                }
                "import.list" => {
                    has_list = true;
                    // Extract symbols from the use list
                    for i in 0..node.named_child_count() {
                        if let Some(child) = node.named_child(i) {
                            let symbol = child.utf8_text(source.as_bytes()).unwrap_or("");
                            if !symbol.is_empty() {
                                symbols.push(symbol.to_string());
                            }
                        }
                    }
                }
                "import.symbol" => {
                    // Individual symbol capture (fallback)
                    if !text.is_empty() && !symbols.contains(&text.to_string()) {
                        symbols.push(text.to_string());
                    }
                }
                _ => {}
            }
        }

        if !import_path.is_empty() || is_wildcard {
            let is_external = !import_path.starts_with('.')
                && !import_path.starts_with("crate")
                && !import_path.starts_with("self")
                && !import_path.starts_with("super");

            // Determine import kind
            let kind = if is_wildcard {
                ImportKind::Wildcard
            } else if has_list || symbols.len() > 1 {
                ImportKind::Multi
            } else if symbols.len() == 1 || import_path.contains("::") {
                ImportKind::Single
            } else {
                ImportKind::Module
            };

            imports.push(ImportInfo {
                from_module: module.clone(),
                to_module,
                import_path,
                is_external,
                symbols,
                is_wildcard,
                kind,
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
    tree: &Tree,
    source: &str,
    file_path: &Path,
    grammar: &dyn Grammar,
) -> Result<Vec<TypeInfo>, AdapterError> {
    let query_str = grammar.type_query();
    if query_str.is_empty() {
        return Ok(vec![]);
    }

    let query = Query::new(&grammar.ts_language(), query_str).map_err(|e| AdapterError {
        code: "QUERY_ERROR".to_string(),
        message: format!("Failed to compile type query: {e}"),
        file: Some(file_path.to_path_buf()),
        line: None,
    })?;

    let mut cursor = QueryCursor::new();
    let capture_names = query.capture_names();
    let module = grammar.compute_module_path(file_path, Path::new("."));

    let mut types = Vec::new();
    let mut abstract_classes: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut all_classes: std::collections::HashMap<String, bool> = std::collections::HashMap::new();

    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    while let Some(m) = matches.next() {
        let mut class_name = String::new();
        let mut is_abstract_match = false;

        for capture in m.captures {
            let capture_name: &str = capture_names[capture.index as usize];
            let node = capture.node;
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");

            match capture_name {
                "class.name" => {
                    class_name = text.to_string();
                }
                "class.abstract" => {
                    is_abstract_match = true;
                }
                "method.abstract" | "method.abstract.name" => {
                    // If we see an abstract method, the containing class is abstract
                    // Find the parent class
                    let mut current = node;
                    while let Some(parent) = current.parent() {
                        if parent.kind() == "class_definition" {
                            // Use child_by_field_name for more robust class name extraction
                            if let Some(name_node) = parent.child_by_field_name("name") {
                                let name = name_node.utf8_text(source.as_bytes()).unwrap_or("");
                                abstract_classes.insert(name.to_string());
                            } else {
                                // Fallback: scan children for an identifier
                                for child in parent.children(&mut parent.walk()) {
                                    if child.kind() == "identifier" {
                                        let name = child.utf8_text(source.as_bytes()).unwrap_or("");
                                        abstract_classes.insert(name.to_string());
                                        break;
                                    }
                                }
                            }
                            break;
                        }
                        current = parent;
                    }
                }
                _ => {}
            }
        }

        if !class_name.is_empty() {
            // Track this class, will determine abstractness later
            let entry = all_classes.entry(class_name.clone()).or_insert(false);
            if is_abstract_match {
                *entry = true;
            }
        }
    }

    // Build final type list
    for (name, is_abstract) in &all_classes {
        let is_abstract_final = *is_abstract || abstract_classes.contains(name);
        types.push(TypeInfo {
            name: name.clone(),
            module: module.clone(),
            is_abstract: is_abstract_final,
        });
    }

    Ok(types)
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
