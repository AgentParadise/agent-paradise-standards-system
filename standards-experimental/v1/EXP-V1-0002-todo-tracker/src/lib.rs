//! TODO/FIXME Tracker and Issue Validator
//!
//! This crate implements EXP-V1-0002, a standard for tracking TODO and FIXME
//! comments in source code with validation that they reference GitHub issues.
//!
//! # Overview
//!
//! The tracker scans source code for TODO/FIXME comments, validates they follow
//! the required format `TAG(#N): description`, and generates standardized artifacts
//! for AI agents and tooling.
//!
//! # Artifacts
//!
//! The tracker generates the following artifacts in `.todo-tracker/`:
//!
//! - `manifest.toml` - Scan metadata
//! - `items.json` - All TODO/FIXME items (core artifact)
//! - `summary.json` - Statistics
//!
//! # Example
//!
//! ```rust,no_run
//! use todo_tracker::config::TrackerConfig;
//! use std::path::Path;
//!
//! let config = TrackerConfig::default();
//! let repo_root = Path::new(".");
//!
//! // Scan will be implemented in scanner module
//! ```

pub mod artifact;
pub mod config;
pub mod languages;
pub mod scanner;

// Re-export commonly used types
pub use artifact::{
    CodeContext, GitHubConfig, IssueReference, ItemSummary, ScanMetadata, TodoItem, TodoItems,
    TrackerManifest,
};
pub use config::{
    ConfigError, EnforcementLevel, EnforcementSettings, GitHubSettings, ScanSettings,
    TrackerConfig, TrackerSettings,
};
pub use scanner::{ScanResult, Scanner, ScannerError};

/// Version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Errors that can occur during TODO tracking
#[derive(Debug, thiserror::Error)]
pub enum TrackerError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML serialization error
    #[error("TOML error: {0}")]
    Toml(#[from] toml::ser::Error),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Result type for tracker operations
pub type Result<T> = std::result::Result<T, TrackerError>;

/// Register this package with a composed APSS runner.
pub fn register(registry: &mut dyn apss_core::registry::StandardRegistry) {
    registry.register(
        apss_core::registry::RegisteredStandard {
            id: "EXP-V1-0002".to_string(),
            slug: "todo-tracker".to_string(),
            name: "TODO Tracker".to_string(),
            description: "TODO and FIXME tracking experiment".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            commands: Vec::new(),
        },
        Box::new(NoopCommandHandler),
    );
}

struct NoopCommandHandler;

impl apss_core::registry::CommandHandler for NoopCommandHandler {
    fn execute(&self, _command: &str, _args: &[String], _config: &toml::Value) -> i32 {
        eprintln!("No composed CLI commands are registered for todo-tracker yet.");
        5
    }

    fn commands(&self) -> Vec<apss_core::registry::CommandInfo> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        // VERSION is a const, so we just check it's defined
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }
}
