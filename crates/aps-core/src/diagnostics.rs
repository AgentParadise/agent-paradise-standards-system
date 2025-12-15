//! Structured diagnostics for APS validation.
//!
//! Provides types for reporting errors, warnings, and informational messages
//! from validation operations.

use std::path::PathBuf;

/// Severity level for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational message, does not affect validation outcome.
    Info,
    /// Warning that should be addressed but doesn't fail validation.
    Warning,
    /// Error that causes validation to fail.
    Error,
}

/// Location within a file or package.
#[derive(Debug, Clone, Default)]
pub struct Location {
    /// Path to the file, if applicable.
    pub path: Option<PathBuf>,
    /// Line number (1-indexed), if applicable.
    pub line: Option<usize>,
    /// Column number (1-indexed), if applicable.
    pub column: Option<usize>,
}

/// A single diagnostic message from validation.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity of this diagnostic.
    pub severity: Severity,
    /// Unique error code (e.g., "APS-0001").
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Location where the issue was found.
    pub location: Location,
    /// Optional hint for how to fix the issue.
    pub fix_hint: Option<String>,
}

impl Diagnostic {
    /// Create a new error diagnostic.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            message: message.into(),
            location: Location::default(),
            fix_hint: None,
        }
    }

    /// Create a new warning diagnostic.
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            code: code.into(),
            message: message.into(),
            location: Location::default(),
            fix_hint: None,
        }
    }

    /// Create a new info diagnostic.
    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Info,
            code: code.into(),
            message: message.into(),
            location: Location::default(),
            fix_hint: None,
        }
    }

    /// Add a location to this diagnostic.
    pub fn with_location(mut self, location: Location) -> Self {
        self.location = location;
        self
    }

    /// Add a path to this diagnostic's location.
    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.location.path = Some(path.into());
        self
    }

    /// Add a fix hint to this diagnostic.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.fix_hint = Some(hint.into());
        self
    }
}

/// A collection of diagnostics from a validation operation.
#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
}

impl Diagnostics {
    /// Create an empty diagnostics collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a diagnostic to the collection.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.items.push(diagnostic);
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|d| d.severity == Severity::Error)
    }

    /// Check if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        self.items.iter().any(|d| d.severity == Severity::Warning)
    }

    /// Get the number of diagnostics.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Iterate over all diagnostics.
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.items.iter()
    }

    /// Get all errors.
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.items.iter().filter(|d| d.severity == Severity::Error)
    }

    /// Get all warnings.
    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.items
            .iter()
            .filter(|d| d.severity == Severity::Warning)
    }

    /// Merge another diagnostics collection into this one.
    pub fn merge(&mut self, other: Diagnostics) {
        self.items.extend(other.items);
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::error("APS-0001", "Missing required directory")
            .with_path("standards/v1/APS-V1-0000-meta")
            .with_hint("Create the 'examples' directory");

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.code, "APS-0001");
        assert!(diag.fix_hint.is_some());
    }

    #[test]
    fn test_diagnostics_collection() {
        let mut diags = Diagnostics::new();
        diags.push(Diagnostic::error("APS-0001", "Error 1"));
        diags.push(Diagnostic::warning("APS-0002", "Warning 1"));

        assert!(diags.has_errors());
        assert!(diags.has_warnings());
        assert_eq!(diags.len(), 2);
    }
}
