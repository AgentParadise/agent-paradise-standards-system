//! APS-V1-0000 Meta-Standard
//!
//! Defines the structure and validation rules for all APS V1 standards,
//! substandards, and experiments.
//!
//! This crate implements the `Standard` trait and provides validation rules
//! that all V1 packages must satisfy.

use aps_core::discovery::{DiscoveredPackage, discover_v1_packages};
use aps_core::metadata::parse_standard_metadata;
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
    pub const MISSING_LIB_RS: &str = "MISSING_LIB_RS";

    // Metadata validation errors
    pub const INVALID_METADATA: &str = "INVALID_METADATA";
    pub const INVALID_STANDARD_ID: &str = "INVALID_STANDARD_ID";
    pub const INVALID_VERSION: &str = "INVALID_VERSION";

    // Repository layout errors
    pub const MISSING_STANDARDS_DIR: &str = "MISSING_STANDARDS_DIR";
    pub const MISSING_EXPERIMENTAL_DIR: &str = "MISSING_EXPERIMENTAL_DIR";

    // Package validation summary
    pub const PACKAGE_VALIDATION_FAILED: &str = "PACKAGE_VALIDATION_FAILED";
}

/// Required directories for all standard packages.
pub const REQUIRED_PACKAGE_DIRS: &[&str] = &["docs", "examples", "tests", "agents/skills", "src"];

/// Metadata file options (one must exist).
pub const METADATA_FILES: &[&str] = &["standard.toml", "substandard.toml", "experiment.toml"];

/// Standard ID regex pattern.
pub const STANDARD_ID_PATTERN: &str = r"^APS-V1-\d{4}$";

/// Experiment ID regex pattern.
pub const EXPERIMENT_ID_PATTERN: &str = r"^EXP-V1-\d{4}$";

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

    /// Validate the structure of a package (directories, files).
    fn validate_structure(&self, path: &Path, diagnostics: &mut Diagnostics) {
        use error_codes::*;

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

        // Check for metadata file
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

        // Check for Cargo.toml
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

        // Check for src/lib.rs
        if !path.join("src/lib.rs").exists() {
            diagnostics.push(
                Diagnostic::error(
                    MISSING_LIB_RS,
                    "Missing src/lib.rs: standards must implement the Standard trait",
                )
                .with_path(path.join("src/lib.rs"))
                .with_hint("Create src/lib.rs with the Standard trait implementation"),
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
    }

    /// Validate the metadata content of a standard package.
    fn validate_standard_metadata(&self, path: &Path, diagnostics: &mut Diagnostics) {
        use error_codes::*;

        let metadata_path = path.join("standard.toml");
        if !metadata_path.exists() {
            return; // Already reported as MISSING_METADATA_FILE
        }

        match parse_standard_metadata(&metadata_path) {
            Ok(metadata) => {
                // Validate ID format
                if !is_valid_standard_id(&metadata.standard.id) {
                    diagnostics.push(
                        Diagnostic::error(
                            INVALID_STANDARD_ID,
                            format!(
                                "Invalid standard ID '{}': must match pattern APS-V1-XXXX",
                                metadata.standard.id
                            ),
                        )
                        .with_path(&metadata_path)
                        .with_hint("Use format: APS-V1-0001, APS-V1-0002, etc."),
                    );
                }

                // Validate version is semver-like
                if !is_valid_semver(&metadata.standard.version) {
                    diagnostics.push(
                        Diagnostic::warning(
                            INVALID_VERSION,
                            format!(
                                "Version '{}' may not be valid SemVer",
                                metadata.standard.version
                            ),
                        )
                        .with_path(&metadata_path)
                        .with_hint("Use SemVer format: MAJOR.MINOR.PATCH (e.g., 1.0.0)"),
                    );
                }
            }
            Err(e) => {
                diagnostics.push(
                    Diagnostic::error(INVALID_METADATA, format!("Failed to parse metadata: {e}"))
                        .with_path(&metadata_path)
                        .with_hint("Check the TOML syntax and required fields"),
                );
            }
        }
    }

    /// Validate a single discovered package.
    fn validate_discovered_package(
        &self,
        package: &DiscoveredPackage,
        diagnostics: &mut Diagnostics,
    ) {
        let pkg_diagnostics = self.validate_package(&package.path);

        if pkg_diagnostics.has_errors() {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::PACKAGE_VALIDATION_FAILED,
                    format!(
                        "Package {:?} has {} error(s)",
                        package.path.file_name().unwrap_or_default(),
                        pkg_diagnostics.error_count()
                    ),
                )
                .with_path(&package.path),
            );
        }

        diagnostics.merge(pkg_diagnostics);
    }
}

impl Default for MetaStandard {
    fn default() -> Self {
        Self::new()
    }
}

impl Standard for MetaStandard {
    fn validate_package(&self, path: &Path) -> Diagnostics {
        let mut diagnostics = Diagnostics::new();

        // Validate structure
        self.validate_structure(path, &mut diagnostics);

        // Validate metadata if it's a standard
        if path.join("standard.toml").exists() {
            self.validate_standard_metadata(path, &mut diagnostics);
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

        // Discover and validate all packages
        let packages = discover_v1_packages(path);

        diagnostics.push(Diagnostic::info(
            "DISCOVERY_COMPLETE",
            format!("Found {} package(s) to validate", packages.len()),
        ));

        for package in &packages {
            self.validate_discovered_package(package, &mut diagnostics);
        }

        diagnostics
    }
}

/// Check if a string matches the standard ID format (APS-V1-XXXX).
fn is_valid_standard_id(id: &str) -> bool {
    if !id.starts_with("APS-V1-") {
        return false;
    }
    let suffix = &id[7..];
    suffix.len() == 4 && suffix.chars().all(|c| c.is_ascii_digit())
}

/// Check if a string looks like valid SemVer (basic check).
fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u32>().is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_meta_standard_creation() {
        let meta = MetaStandard::new();
        let default_meta = MetaStandard;

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
        // Should have errors for: 5 dirs + metadata + Cargo.toml + lib.rs + spec
        assert!(diagnostics.error_count() >= 5);
    }

    #[test]
    fn test_validate_repo_layout() {
        let temp_dir = tempfile::tempdir().unwrap();
        let meta = MetaStandard::new();

        let diagnostics = meta.validate_repo(temp_dir.path());

        assert!(diagnostics.has_errors());
    }

    #[test]
    fn test_valid_standard_id() {
        assert!(is_valid_standard_id("APS-V1-0000"));
        assert!(is_valid_standard_id("APS-V1-0001"));
        assert!(is_valid_standard_id("APS-V1-9999"));

        assert!(!is_valid_standard_id("APS-V2-0000")); // Wrong version
        assert!(!is_valid_standard_id("APS-V1-000")); // Too short
        assert!(!is_valid_standard_id("APS-V1-00000")); // Too long
        assert!(!is_valid_standard_id("EXP-V1-0000")); // Experiment, not standard
    }

    #[test]
    fn test_valid_semver() {
        assert!(is_valid_semver("1.0.0"));
        assert!(is_valid_semver("0.1.0"));
        assert!(is_valid_semver("10.20.30"));
        assert!(is_valid_semver("1.0")); // 2-part is valid

        assert!(!is_valid_semver("1")); // Too few parts
        assert!(!is_valid_semver("1.0.0.0")); // Too many parts
        assert!(!is_valid_semver("a.b.c")); // Not numbers
    }

    #[test]
    fn test_validate_repo_with_valid_package() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create minimal valid structure
        let pkg_dir = temp_dir.path().join("standards/v1/APS-V1-0001-test");
        fs::create_dir_all(pkg_dir.join("docs")).unwrap();
        fs::create_dir_all(pkg_dir.join("examples")).unwrap();
        fs::create_dir_all(pkg_dir.join("tests")).unwrap();
        fs::create_dir_all(pkg_dir.join("agents/skills")).unwrap();
        fs::create_dir_all(pkg_dir.join("src")).unwrap();

        fs::write(pkg_dir.join("docs/01_spec.md"), "# Spec").unwrap();
        fs::write(pkg_dir.join("src/lib.rs"), "// lib").unwrap();
        fs::write(pkg_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let standard_toml = r#"
schema = "aps.standard/v1"

[standard]
id = "APS-V1-0001"
name = "Test"
slug = "test"
version = "1.0.0"
category = "governance"
status = "active"

[aps]
aps_major = "v1"

[ownership]
maintainers = ["Test"]
"#;
        fs::write(pkg_dir.join("standard.toml"), standard_toml).unwrap();

        // Create experimental dir
        fs::create_dir_all(temp_dir.path().join("standards-experimental/v1")).unwrap();

        let meta = MetaStandard::new();
        let diagnostics = meta.validate_repo(temp_dir.path());

        // Should have no errors (only info messages)
        assert!(
            !diagnostics.has_errors(),
            "Unexpected errors: {:?}",
            diagnostics.errors().map(|d| &d.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_error_codes_are_readable() {
        use error_codes::*;

        // Error codes should be human-readable
        assert!(MISSING_REQUIRED_DIR.contains("MISSING"));
        assert!(MISSING_METADATA_FILE.contains("METADATA"));
        assert!(MISSING_STANDARDS_DIR.contains("STANDARDS"));
        assert!(INVALID_STANDARD_ID.contains("STANDARD"));
    }
}
