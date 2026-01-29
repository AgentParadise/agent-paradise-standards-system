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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        // VERSION is a const, so we just check it's defined
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }
}
