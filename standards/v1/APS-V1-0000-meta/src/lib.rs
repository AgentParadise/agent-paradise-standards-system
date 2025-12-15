//! APS-V1-0000 Meta-Standard
//!
//! Defines the structure and validation rules for all APS V1 standards,
//! substandards, and experiments.
//!
//! This crate implements the `Standard` trait and provides validation rules
//! that all V1 packages must satisfy.

use aps_core::{Diagnostic, Diagnostics};
use std::path::Path;

/// Error codes for meta-standard validation.
///
/// These are used as the `code` field in diagnostics for programmatic matching.
/// The const name IS the error code - human-readable and grep-able.
pub mod error_codes {
    // Package structure errors
    pub const MISSING_REQUIRED_DIR: &str = "MISSING_REQUIRED_DIR";
    pub const MISSING_METADATA_FILE: &str = "MISSING_METADATA_FILE";
    pub const MISSING_CARGO_TOML: &str = "MISSING_CARGO_TOML";
    pub const MISSING_SPEC_DOC: &str = "MISSING_SPEC_DOC";

    // Repository layout errors
    pub const MISSING_STANDARDS_DIR: &str = "MISSING_STANDARDS_DIR";
    pub const MISSING_EXPERIMENTAL_DIR: &str = "MISSING_EXPERIMENTAL_DIR";
}

/// Required directories for all standard packages.
pub const REQUIRED_PACKAGE_DIRS: &[&str] = &["docs", "examples", "tests", "agents/skills", "src"];

/// Metadata file options (one must exist).
pub const METADATA_FILES: &[&str] = &["standard.toml", "substandard.toml", "experiment.toml"];

/// The Standard trait that all APS standards implement.
///
/// This trait defines the core interface for validation and is implemented
/// by each standard crate.
pub trait Standard {
    /// Validate a package against this standard's rules.
    ///
    /// Returns diagnostics containing any errors, warnings, or info messages.
    fn validate_package(&self, path: &Path) -> Diagnostics;

    /// Validate an entire repository against this standard's rules.
    ///
    /// This checks repository-level layout and all contained packages.
    fn validate_repo(&self, path: &Path) -> Diagnostics;
}

/// The APS-V1-0000 Meta-Standard implementation.
///
/// This standard defines the rules for all V1 standards, substandards,
/// and experiments.
pub struct MetaStandard;

impl MetaStandard {
    /// Create a new MetaStandard instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MetaStandard {
    fn default() -> Self {
        Self::new()
    }
}

impl Standard for MetaStandard {
    fn validate_package(&self, path: &Path) -> Diagnostics {
        use error_codes::*;

        let mut diagnostics = Diagnostics::new();

        // Check required directories
        for dir in REQUIRED_PACKAGE_DIRS {
            let dir_path = path.join(dir);
            if !dir_path.exists() {
                diagnostics.push(
                    Diagnostic::error(
                        MISSING_REQUIRED_DIR,
                        format!("Missing required directory: {dir}"),
                    )
                    .with_path(&dir_path)
                    .with_hint(format!("Create the '{dir}' directory")),
                );
            }
        }

        // Check for metadata file (one of the allowed options must exist)
        let has_metadata = METADATA_FILES.iter().any(|file| path.join(file).exists());

        if !has_metadata {
            diagnostics.push(
                Diagnostic::error(
                    MISSING_METADATA_FILE,
                    "Missing metadata file: expected standard.toml, substandard.toml, or experiment.toml",
                )
                .with_path(path)
                .with_hint("Create a metadata TOML file at the package root"),
            );
        }

        // Check for Cargo.toml (Rust crate requirement)
        if !path.join("Cargo.toml").exists() {
            diagnostics.push(
                Diagnostic::error(
                    MISSING_CARGO_TOML,
                    "Missing Cargo.toml: standards must be Rust crates",
                )
                .with_path(path)
                .with_hint("Create a Cargo.toml for this standard crate"),
            );
        }

        // Check for spec document
        let spec_path = path.join("docs/01_spec.md");
        if !spec_path.exists() {
            diagnostics.push(
                Diagnostic::error(MISSING_SPEC_DOC, "Missing normative spec: docs/01_spec.md")
                    .with_path(&spec_path)
                    .with_hint("Create docs/01_spec.md with the normative specification"),
            );
        }

        diagnostics
    }

    fn validate_repo(&self, path: &Path) -> Diagnostics {
        use error_codes::*;

        let mut diagnostics = Diagnostics::new();

        // Check repository-level layout
        let standards_dir = path.join("standards/v1");
        if !standards_dir.exists() {
            diagnostics.push(
                Diagnostic::error(
                    MISSING_STANDARDS_DIR,
                    "Missing standards directory: standards/v1/",
                )
                .with_path(&standards_dir)
                .with_hint("Create the standards/v1/ directory for official standards"),
            );
        }

        let experimental_dir = path.join("standards-experimental/v1");
        if !experimental_dir.exists() {
            diagnostics.push(
                Diagnostic::warning(
                    MISSING_EXPERIMENTAL_DIR,
                    "Missing experimental directory: standards-experimental/v1/",
                )
                .with_path(&experimental_dir)
                .with_hint("Create standards-experimental/v1/ for experimental standards"),
            );
        }

        // TODO: Implement in M5 - walk standards/v1/ and validate each package

        diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_standard_creation() {
        let meta = MetaStandard::new();
        let default_meta = MetaStandard::default();

        // Both should work
        let _ = meta;
        let _ = default_meta;
    }

    #[test]
    fn test_validate_missing_directories() {
        let temp_dir = tempfile::tempdir().unwrap();
        let meta = MetaStandard::new();

        let diagnostics = meta.validate_package(temp_dir.path());

        assert!(diagnostics.has_errors());
        assert!(diagnostics.errors().count() >= 4);
    }

    #[test]
    fn test_validate_repo_layout() {
        let temp_dir = tempfile::tempdir().unwrap();
        let meta = MetaStandard::new();

        let diagnostics = meta.validate_repo(temp_dir.path());

        assert!(diagnostics.has_errors());
    }

    #[test]
    fn test_error_codes_are_readable() {
        use error_codes::*;

        // Error codes should be human-readable, not cryptic
        assert!(MISSING_REQUIRED_DIR.contains("MISSING"));
        assert!(MISSING_METADATA_FILE.contains("METADATA"));
        assert!(MISSING_STANDARDS_DIR.contains("STANDARDS"));
    }
}
