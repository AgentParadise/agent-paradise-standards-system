//! CLI Contract (APS-V1-0000.CLI01)
//!
//! Defines the CLI contract for APS standards - command patterns,
//! output formats, and integration traits.
//!
//! ## Quick Start
//!
//! ```bash
//! # Run a standard's CLI
//! aps run topology analyze .
//! aps run topology validate .topology/
//!
//! # Discovery
//! aps run --list
//! ```
//!
//! ## Implementing StandardCli
//!
//! Standards that expose CLI commands implement the `StandardCli` trait:
//!
//! ```ignore
//! use aps_v1_0000_cli01_cli_contract::{StandardCli, CliResult, CliCommandInfo};
//!
//! struct TopologyCli;
//!
//! impl StandardCli for TopologyCli {
//!     fn slug(&self) -> &str { "topology" }
//!     fn id(&self) -> &str { "EXP-V1-0001" }
//!     fn aps_version(&self) -> &str { "v1" }
//!     
//!     fn commands(&self) -> Vec<CliCommandInfo> {
//!         vec![
//!             CliCommandInfo::required("analyze", "Generate topology artifacts"),
//!             CliCommandInfo::required("validate", "Validate existing artifacts"),
//!         ]
//!     }
//!     
//!     fn execute(&self, command: &str, args: &[String]) -> CliResult {
//!         // ...
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Exit Codes
// ============================================================================

/// Standard exit codes for CLI commands.
pub mod exit_codes {
    /// Success - no errors or warnings.
    pub const SUCCESS: i32 = 0;
    /// Error - blocking errors found.
    pub const ERROR: i32 = 1;
    /// Warning - warnings only, no blocking errors.
    pub const WARNING: i32 = 2;
    /// Invalid arguments.
    pub const INVALID_ARGS: i32 = 3;
    /// IO/system error.
    pub const IO_ERROR: i32 = 4;
    /// Command not implemented.
    pub const NOT_IMPLEMENTED: i32 = 5;
}

// ============================================================================
// CLI Status
// ============================================================================

/// Execution status for JSON output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CliStatus {
    /// Command succeeded with no issues.
    Success,
    /// Command completed with warnings.
    Warning,
    /// Command failed with errors.
    Error,
}

impl CliStatus {
    /// Convert status to exit code.
    pub fn exit_code(&self) -> i32 {
        match self {
            CliStatus::Success => exit_codes::SUCCESS,
            CliStatus::Warning => exit_codes::WARNING,
            CliStatus::Error => exit_codes::ERROR,
        }
    }
}

// ============================================================================
// Diagnostic
// ============================================================================

/// Severity level for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    /// Informational message.
    Info,
    /// Warning - non-blocking issue.
    Warning,
    /// Error - blocking issue.
    Error,
}

/// A diagnostic message from a CLI command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliDiagnostic {
    /// Severity level.
    pub severity: DiagnosticSeverity,
    /// Error/warning code (e.g., "MISSING_ARTIFACT").
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// File location, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    /// Line number, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

impl CliDiagnostic {
    /// Create an error diagnostic.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            code: code.into(),
            message: message.into(),
            file: None,
            line: None,
        }
    }

    /// Create a warning diagnostic.
    pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            code: code.into(),
            message: message.into(),
            file: None,
            line: None,
        }
    }

    /// Create an info diagnostic.
    pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Info,
            code: code.into(),
            message: message.into(),
            file: None,
            line: None,
        }
    }

    /// Add file location.
    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Add line number.
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }
}

// ============================================================================
// CLI Result
// ============================================================================

/// Result of a CLI command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliResult {
    /// Execution status.
    pub status: CliStatus,
    /// Command that was executed.
    pub command: String,
    /// Standard version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Timestamp of execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Structured output data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Diagnostic messages.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<CliDiagnostic>,
}

impl CliResult {
    /// Create a successful result.
    pub fn success(command: impl Into<String>) -> Self {
        Self {
            status: CliStatus::Success,
            command: command.into(),
            version: None,
            timestamp: None,
            data: None,
            diagnostics: Vec::new(),
        }
    }

    /// Create an error result.
    pub fn error(command: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: CliStatus::Error,
            command: command.into(),
            version: None,
            timestamp: None,
            data: None,
            diagnostics: vec![CliDiagnostic::error("ERROR", message)],
        }
    }

    /// Create a warning result.
    pub fn warning(command: impl Into<String>) -> Self {
        Self {
            status: CliStatus::Warning,
            command: command.into(),
            version: None,
            timestamp: None,
            data: None,
            diagnostics: Vec::new(),
        }
    }

    /// Add data to the result.
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Add a diagnostic.
    pub fn with_diagnostic(mut self, diagnostic: CliDiagnostic) -> Self {
        // Update status based on diagnostic severity
        if diagnostic.severity == DiagnosticSeverity::Error {
            self.status = CliStatus::Error;
        } else if diagnostic.severity == DiagnosticSeverity::Warning
            && self.status == CliStatus::Success
        {
            self.status = CliStatus::Warning;
        }
        self.diagnostics.push(diagnostic);
        self
    }

    /// Get exit code for this result.
    pub fn exit_code(&self) -> i32 {
        self.status.exit_code()
    }

    /// Check if result has errors.
    pub fn has_errors(&self) -> bool {
        self.status == CliStatus::Error
    }

    /// Check if result has warnings.
    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == DiagnosticSeverity::Warning)
    }
}

// ============================================================================
// Command Info
// ============================================================================

/// Information about a CLI command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommandInfo {
    /// Command name (e.g., "analyze").
    pub name: String,
    /// Short description.
    pub description: String,
    /// Usage pattern (e.g., "analyze <path> [--output <dir>]").
    pub usage: String,
    /// Whether this command is required by the CLI contract.
    pub required: bool,
}

impl CliCommandInfo {
    /// Create a required command.
    pub fn required(name: impl Into<String>, description: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            usage: format!("{name} <path>"),
            name,
            description: description.into(),
            required: true,
        }
    }

    /// Create an optional command.
    pub fn optional(name: impl Into<String>, description: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            usage: format!("{name} <path>"),
            name,
            description: description.into(),
            required: false,
        }
    }

    /// Set custom usage pattern.
    pub fn with_usage(mut self, usage: impl Into<String>) -> Self {
        self.usage = usage.into();
        self
    }
}

// ============================================================================
// Standard CLI Trait
// ============================================================================

/// Trait for standards that expose CLI commands.
///
/// Implement this trait to register a standard's CLI with the `aps run` dispatcher.
pub trait StandardCli: Send + Sync {
    /// Standard slug for command dispatch (e.g., "topology").
    fn slug(&self) -> &str;

    /// Standard ID (e.g., "EXP-V1-0001").
    fn id(&self) -> &str;

    /// APS version this standard uses (e.g., "v1").
    fn aps_version(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Short description.
    fn description(&self) -> &str;

    /// Current version of the standard.
    fn version(&self) -> &str;

    /// List supported commands.
    fn commands(&self) -> Vec<CliCommandInfo>;

    /// Execute a command.
    ///
    /// # Arguments
    ///
    /// * `command` - Command name (e.g., "analyze")
    /// * `args` - Command arguments
    ///
    /// # Returns
    ///
    /// CLI result with status, data, and diagnostics.
    fn execute(&self, command: &str, args: &[String]) -> CliResult;

    /// Get all slug aliases for this standard.
    fn aliases(&self) -> Vec<&str> {
        vec![self.slug()]
    }

    /// Receive project-specific configuration from `apss.toml`.
    ///
    /// Called before `execute()` when running in a project context.
    /// The `config` value corresponds to `[standards.<slug>.config]` from `apss.toml`,
    /// already validated against the standard's `StandardConfig` type.
    ///
    /// # Default
    ///
    /// The default implementation ignores configuration. Override this to
    /// accept project-specific settings.
    fn configure(&mut self, config: serde_json::Value) -> Result<(), String> {
        let _ = config;
        Ok(())
    }
}

// ============================================================================
// Standard Info
// ============================================================================

/// Information about a registered standard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardInfo {
    /// Standard ID.
    pub id: String,
    /// Primary slug.
    pub slug: String,
    /// APS version.
    pub aps_version: String,
    /// Human-readable name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Current version.
    pub version: String,
    /// Available commands.
    pub commands: Vec<CliCommandInfo>,
}

impl StandardInfo {
    /// Create from a StandardCli implementation.
    pub fn from_cli(cli: &dyn StandardCli) -> Self {
        Self {
            id: cli.id().to_string(),
            slug: cli.slug().to_string(),
            aps_version: cli.aps_version().to_string(),
            name: cli.name().to_string(),
            description: cli.description().to_string(),
            version: cli.version().to_string(),
            commands: cli.commands(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_result_success() {
        let result = CliResult::success("test");
        assert_eq!(result.status, CliStatus::Success);
        assert_eq!(result.exit_code(), 0);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_cli_result_error() {
        let result = CliResult::error("test", "Something went wrong");
        assert_eq!(result.status, CliStatus::Error);
        assert_eq!(result.exit_code(), 1);
        assert!(result.has_errors());
    }

    #[test]
    fn test_cli_result_with_diagnostic() {
        let result = CliResult::success("test")
            .with_diagnostic(CliDiagnostic::warning("WARN", "Be careful"));
        assert_eq!(result.status, CliStatus::Warning);
        assert_eq!(result.exit_code(), 2);
        assert!(result.has_warnings());
    }

    #[test]
    fn test_diagnostic_with_location() {
        let diag = CliDiagnostic::error("MISSING", "File not found")
            .with_file("src/main.rs")
            .with_line(42);
        assert_eq!(diag.file, Some(PathBuf::from("src/main.rs")));
        assert_eq!(diag.line, Some(42));
    }

    #[test]
    fn test_command_info() {
        let cmd = CliCommandInfo::required("analyze", "Analyze codebase")
            .with_usage("analyze <path> [--output <dir>]");
        assert!(cmd.required);
        assert!(cmd.usage.contains("--output"));
    }

    #[test]
    fn test_cli_status_exit_codes() {
        assert_eq!(CliStatus::Success.exit_code(), 0);
        assert_eq!(CliStatus::Error.exit_code(), 1);
        assert_eq!(CliStatus::Warning.exit_code(), 2);
    }
}
