//! Tree-sitter based language adapter framework.
//!
//! This module provides a unified approach to analyzing source code across
//! multiple programming languages using tree-sitter parsers.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                    TreeSitterAdapter                                │
//! │  ┌─────────────────────────────────────────────────────────────┐   │
//! │  │              Shared Complexity Engine                        │   │
//! │  │  • compute_cyclomatic(tree, rules) → u32                    │   │
//! │  │  • compute_cognitive(tree, rules) → u32                     │   │
//! │  │  • compute_halstead(tree) → Metrics                         │   │
//! │  └─────────────────────────────────────────────────────────────┘   │
//! │                              ▲                                      │
//! │         ┌────────────────────┼────────────────────┐                │
//! │   ┌─────┴─────┐       ┌─────┴─────┐       ┌─────┴─────┐           │
//! │   │   Rust    │       │  Python   │       │    ...    │           │
//! │   │  Grammar  │       │  Grammar  │       │  Grammar  │           │
//! │   └───────────┘       └───────────┘       └───────────┘           │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use code_topology::adapter::{TreeSitterAdapter, GrammarRegistry};
//! use code_topology::adapter::grammars::RustGrammar;
//!
//! let mut registry = GrammarRegistry::new();
//! registry.register(Box::new(RustGrammar::new()));
//!
//! let adapter = TreeSitterAdapter::new(registry);
//! let functions = adapter.extract_functions(source, path)?;
//! ```

pub mod complexity;
pub mod grammars;
pub mod queries;

use std::collections::HashMap;
use std::path::Path;

use thiserror::Error;
use tree_sitter::{Language, Parser, Tree};

use crate::{
    AdapterError, CallInfo, FunctionInfo, FunctionMetrics, ImportInfo, LanguageAdapter, TypeInfo,
};

use self::complexity::ComplexityCalculator;
use self::grammars::Grammar;

// ============================================================================
// Errors
// ============================================================================

/// Errors from the tree-sitter adapter framework.
#[derive(Debug, Error)]
pub enum FrameworkError {
    #[error("No grammar found for file extension: {0}")]
    NoGrammarForExtension(String),

    #[error("No grammar found for language: {0}")]
    NoGrammarForLanguage(String),

    #[error("Failed to set parser language: {0}")]
    ParserSetup(String),

    #[error("Failed to parse source code")]
    ParseFailed,

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<FrameworkError> for AdapterError {
    fn from(err: FrameworkError) -> Self {
        AdapterError {
            code: "FRAMEWORK_ERROR".to_string(),
            message: err.to_string(),
            file: None,
            line: None,
        }
    }
}

// ============================================================================
// Grammar Registry
// ============================================================================

/// Registry of available language grammars.
///
/// The registry maps file extensions and language IDs to their grammar implementations.
pub struct GrammarRegistry {
    /// Extension -> Grammar ID mapping (e.g., ".rs" -> "rust")
    extension_map: HashMap<String, String>,
    /// Language ID -> Grammar mapping
    grammars: HashMap<String, Box<dyn Grammar>>,
}

impl GrammarRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            extension_map: HashMap::new(),
            grammars: HashMap::new(),
        }
    }

    /// Register a grammar.
    ///
    /// This adds the grammar to the registry and maps all its file extensions.
    pub fn register(&mut self, grammar: Box<dyn Grammar>) {
        let id = grammar.language_id().to_string();

        // Map all extensions to this grammar
        for ext in grammar.file_extensions() {
            self.extension_map.insert(ext.to_string(), id.clone());
        }

        self.grammars.insert(id, grammar);
    }

    /// Get a grammar by language ID.
    pub fn get(&self, language_id: &str) -> Option<&dyn Grammar> {
        self.grammars.get(language_id).map(|g| g.as_ref())
    }

    /// Get a grammar for a file path based on its extension.
    pub fn get_for_path(&self, path: &Path) -> Option<&dyn Grammar> {
        let ext = path.extension()?.to_str()?;
        let ext_with_dot = format!(".{ext}");
        let language_id = self.extension_map.get(&ext_with_dot)?;
        self.get(language_id)
    }

    /// List all registered language IDs.
    pub fn languages(&self) -> Vec<&str> {
        self.grammars.keys().map(|s| s.as_str()).collect()
    }

    /// List all supported file extensions.
    pub fn extensions(&self) -> Vec<&str> {
        self.extension_map.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a file extension is supported.
    pub fn supports_extension(&self, ext: &str) -> bool {
        let ext_with_dot = if ext.starts_with('.') {
            ext.to_string()
        } else {
            format!(".{ext}")
        };
        self.extension_map.contains_key(&ext_with_dot)
    }
}

impl Default for GrammarRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tree-Sitter Adapter
// ============================================================================

/// Unified tree-sitter based language adapter.
///
/// This adapter uses tree-sitter grammars to parse source code and extract
/// topology data. All complexity calculations use shared logic to ensure
/// consistency across languages.
pub struct TreeSitterAdapter {
    registry: GrammarRegistry,
}

impl TreeSitterAdapter {
    /// Create a new adapter with the given grammar registry.
    pub fn new(registry: GrammarRegistry) -> Self {
        Self { registry }
    }

    /// Create a parser configured for the given language.
    fn create_parser(&self, language: Language) -> Result<Parser, FrameworkError> {
        let mut parser = Parser::new();
        parser
            .set_language(&language)
            .map_err(|e| FrameworkError::ParserSetup(e.to_string()))?;
        Ok(parser)
    }

    /// Parse source code with the appropriate grammar.
    pub fn parse(&self, source: &str, path: &Path) -> Result<(Tree, &dyn Grammar), FrameworkError> {
        let grammar = self.registry.get_for_path(path).ok_or_else(|| {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown");
            FrameworkError::NoGrammarForExtension(ext.to_string())
        })?;

        let mut parser = self.create_parser(grammar.ts_language())?;
        let tree = parser
            .parse(source, None)
            .ok_or(FrameworkError::ParseFailed)?;

        Ok((tree, grammar))
    }

    /// Parse source code with a specific language.
    pub fn parse_with_language(
        &self,
        source: &str,
        language_id: &str,
    ) -> Result<(Tree, &dyn Grammar), FrameworkError> {
        let grammar = self
            .registry
            .get(language_id)
            .ok_or_else(|| FrameworkError::NoGrammarForLanguage(language_id.to_string()))?;

        let mut parser = self.create_parser(grammar.ts_language())?;
        let tree = parser
            .parse(source, None)
            .ok_or(FrameworkError::ParseFailed)?;

        Ok((tree, grammar))
    }

    /// Get the grammar registry.
    pub fn registry(&self) -> &GrammarRegistry {
        &self.registry
    }

    /// Compute metrics for a function using the shared complexity engine.
    pub fn compute_function_metrics(
        &self,
        source: &str,
        grammar: &dyn Grammar,
        start_line: u32,
        end_line: u32,
    ) -> Result<FunctionMetrics, FrameworkError> {
        let mut parser = self.create_parser(grammar.ts_language())?;
        let tree = parser
            .parse(source, None)
            .ok_or(FrameworkError::ParseFailed)?;

        let calculator = ComplexityCalculator::new(grammar);
        Ok(calculator.compute_metrics(&tree, source.as_bytes(), start_line, end_line))
    }
}

impl LanguageAdapter for TreeSitterAdapter {
    fn language_id(&self) -> &'static str {
        // This adapter handles multiple languages
        "multi"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        // Return empty - use registry.extensions() instead
        &[]
    }

    fn extract_functions(
        &self,
        source: &str,
        file_path: &Path,
    ) -> Result<Vec<FunctionInfo>, AdapterError> {
        let (tree, grammar) = self.parse(source, file_path)?;
        queries::extract_functions(&tree, source, file_path, grammar)
    }

    fn extract_calls(&self, source: &str, file_path: &Path) -> Result<Vec<CallInfo>, AdapterError> {
        let (tree, grammar) = self.parse(source, file_path)?;
        queries::extract_calls(&tree, source, file_path, grammar)
    }

    fn extract_imports(
        &self,
        source: &str,
        file_path: &Path,
    ) -> Result<Vec<ImportInfo>, AdapterError> {
        let (tree, grammar) = self.parse(source, file_path)?;
        queries::extract_imports(&tree, source, file_path, grammar)
    }

    fn compute_metrics(
        &self,
        source: &str,
        function: &FunctionInfo,
    ) -> Result<FunctionMetrics, AdapterError> {
        let grammar = self
            .registry
            .get_for_path(&function.file_path)
            .ok_or_else(|| AdapterError {
                code: "NO_GRAMMAR".to_string(),
                message: format!("No grammar for {:?}", function.file_path),
                file: Some(function.file_path.clone()),
                line: None,
            })?;

        self.compute_function_metrics(source, grammar, function.start_line, function.end_line)
            .map_err(|e| e.into())
    }

    fn extract_types(&self, source: &str, file_path: &Path) -> Result<Vec<TypeInfo>, AdapterError> {
        let (tree, grammar) = self.parse(source, file_path)?;
        queries::extract_types(&tree, source, file_path, grammar)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = GrammarRegistry::new();
        assert!(registry.languages().is_empty());
        assert!(registry.extensions().is_empty());
    }

    #[test]
    fn test_adapter_new() {
        let registry = GrammarRegistry::new();
        let adapter = TreeSitterAdapter::new(registry);
        assert!(adapter.registry().languages().is_empty());
    }
}
