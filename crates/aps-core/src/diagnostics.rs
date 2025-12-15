//! Structured diagnostics for APS validation.
//!
//! Provides types for reporting errors, warnings, and informational messages
//! from validation operations.
//!
//! # Output Formats
//!
//! Diagnostics can be rendered in two formats:
//! - **Human-readable**: Colored terminal output with context
//! - **Machine-readable**: JSON for CI integration

use serde::Serialize;
use std::fmt;
use std::path::PathBuf;

/// Severity level for a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational message, does not affect validation outcome.
    Info,
    /// Warning that should be addressed but doesn't fail validation.
    Warning,
    /// Error that causes validation to fail.
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// Location within a file or package.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Location {
    /// Path to the file, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    /// Line number (1-indexed), if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Column number (1-indexed), if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

/// A single diagnostic message from validation.
#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    /// Severity of this diagnostic.
    pub severity: Severity,
    /// Unique error code (e.g., "MISSING_REQUIRED_DIR").
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Location where the issue was found.
    #[serde(skip_serializing_if = "Location::is_empty")]
    pub location: Location,
    /// Optional hint for how to fix the issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<String>,
}

impl Location {
    /// Check if the location has no meaningful data.
    pub fn is_empty(&self) -> bool {
        self.path.is_none() && self.line.is_none() && self.column.is_none()
    }
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
#[derive(Debug, Clone, Default, Serialize)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// OUTPUT FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// ANSI color codes for terminal output.
mod colors {
    pub const RED: &str = "\x1b[31m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const CYAN: &str = "\x1b[36m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const RESET: &str = "\x1b[0m";
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (color, label) = match self.severity {
            Severity::Error => (colors::RED, "error"),
            Severity::Warning => (colors::YELLOW, "warning"),
            Severity::Info => (colors::CYAN, "info"),
        };

        // First line: severity and code
        write!(
            f,
            "{}{}{}{}: {}",
            colors::BOLD,
            color,
            label,
            colors::RESET,
            self.code
        )?;

        // Location if present
        if let Some(path) = &self.location.path {
            write!(f, "\n  {} --> {}", colors::DIM, path.display())?;
            if let Some(line) = self.location.line {
                write!(f, ":{line}")?;
                if let Some(col) = self.location.column {
                    write!(f, ":{col}")?;
                }
            }
            write!(f, "{}", colors::RESET)?;
        }

        // Message
        write!(f, "\n  {}", self.message)?;

        // Hint if present
        if let Some(hint) = &self.fix_hint {
            write!(f, "\n  {}hint:{} {}", colors::CYAN, colors::RESET, hint)?;
        }

        Ok(())
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, diag) in self.items.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{diag}")?;
        }

        // Summary
        let error_count = self
            .items
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let warning_count = self
            .items
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();

        if !self.items.is_empty() {
            writeln!(f)?;
            writeln!(f)?;
            write!(
                f,
                "{}Summary:{} {} error(s), {} warning(s)",
                colors::BOLD,
                colors::RESET,
                error_count,
                warning_count
            )?;
        }

        Ok(())
    }
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

    /// Render as JSON for CI integration.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Get the appropriate exit code for CLI.
    ///
    /// - `0`: All checks passed (no errors or warnings)
    /// - `1`: Errors found
    /// - `2`: Warnings only (no errors)
    pub fn exit_code(&self) -> i32 {
        if self.has_errors() {
            1
        } else if self.has_warnings() {
            2
        } else {
            0
        }
    }

    /// Count errors.
    pub fn error_count(&self) -> usize {
        self.items
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    /// Count warnings.
    pub fn warning_count(&self) -> usize {
        self.items
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
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
        let diag = Diagnostic::error("MISSING_REQUIRED_DIR", "Missing required directory")
            .with_path("standards/v1/APS-V1-0000-meta")
            .with_hint("Create the 'examples' directory");

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.code, "MISSING_REQUIRED_DIR");
        assert!(diag.fix_hint.is_some());
    }

    #[test]
    fn test_diagnostics_collection() {
        let mut diags = Diagnostics::new();
        diags.push(Diagnostic::error("ERROR_ONE", "Error 1"));
        diags.push(Diagnostic::warning("WARNING_ONE", "Warning 1"));

        assert!(diags.has_errors());
        assert!(diags.has_warnings());
        assert_eq!(diags.len(), 2);
        assert_eq!(diags.error_count(), 1);
        assert_eq!(diags.warning_count(), 1);
    }

    #[test]
    fn test_exit_codes() {
        let empty = Diagnostics::new();
        assert_eq!(empty.exit_code(), 0);

        let mut warnings_only = Diagnostics::new();
        warnings_only.push(Diagnostic::warning("WARN", "A warning"));
        assert_eq!(warnings_only.exit_code(), 2);

        let mut with_errors = Diagnostics::new();
        with_errors.push(Diagnostic::error("ERR", "An error"));
        assert_eq!(with_errors.exit_code(), 1);
    }

    #[test]
    fn test_json_output() {
        let mut diags = Diagnostics::new();
        diags.push(
            Diagnostic::error("MISSING_REQUIRED_DIR", "Missing directory: examples/")
                .with_path("standards/v1/APS-V1-0000-meta")
                .with_hint("Create the directory"),
        );

        let json = diags.to_json().expect("JSON serialization failed");
        assert!(json.contains("MISSING_REQUIRED_DIR"));
        assert!(json.contains("error"));
    }

    #[test]
    fn test_display_formatting() {
        let diag = Diagnostic::error("MISSING_REQUIRED_DIR", "Missing directory: examples/")
            .with_path("standards/v1/APS-V1-0000-meta")
            .with_hint("Create the directory");

        let output = format!("{diag}");
        assert!(output.contains("error"));
        assert!(output.contains("MISSING_REQUIRED_DIR"));
        assert!(output.contains("Missing directory"));
        assert!(output.contains("hint"));
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Error), "error");
        assert_eq!(format!("{}", Severity::Warning), "warning");
        assert_eq!(format!("{}", Severity::Info), "info");
    }

    #[test]
    fn test_location_is_empty() {
        let empty = Location::default();
        assert!(empty.is_empty());

        let with_path = Location {
            path: Some(PathBuf::from("test")),
            ..Default::default()
        };
        assert!(!with_path.is_empty());
    }
}
