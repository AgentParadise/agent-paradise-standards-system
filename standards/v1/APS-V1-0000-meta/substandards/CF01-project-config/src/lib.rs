//! Project Configuration (APS-V1-0000.CF01)
//!
//! Validates `apss.toml` project configuration files and ensures standards
//! define typed configuration surfaces via the `StandardConfig` trait.
//!
//! ## Dual Validation Role
//!
//! CF01 validates two things:
//!
//! 1. **Consumer `apss.toml` files** — schema, field types, version requirements,
//!    cascading consistency, and standard-specific config blocks.
//!
//! 2. **Standard config compliance** — ensures every standard in the APS repo
//!    exports a `StandardConfig` implementation (or `NoConfig`).
//!
//! ## Quick Start
//!
//! ```ignore
//! use apss_project_config::validate_project_config;
//! use std::path::Path;
//!
//! let diags = validate_project_config(Path::new("apss.toml"));
//! if diags.has_errors() {
//!     eprintln!("{diags}");
//!     std::process::exit(1);
//! }
//! ```

use aps_core::config::{self, CONFIG_FILENAME, PROJECT_SCHEMA, ProjectConfig};
use aps_core::{Diagnostic, Diagnostics};
use std::path::Path;

// ============================================================================
// Error Codes
// ============================================================================

/// Error codes for CF01 validation.
pub mod error_codes {
    // --- Consumer apss.toml validation ---

    /// `schema` field missing or not `"apss.project/v1"`.
    pub const CF_MISSING_SCHEMA: &str = "CF_MISSING_SCHEMA";

    /// `project.name` missing or empty.
    pub const CF_MISSING_PROJECT_NAME: &str = "CF_MISSING_PROJECT_NAME";

    /// `project.apss_version` is not a supported version.
    pub const CF_INVALID_APSS_VERSION: &str = "CF_INVALID_APSS_VERSION";

    /// A standard entry is missing the `id` field.
    pub const CF_MISSING_STANDARD_ID: &str = "CF_MISSING_STANDARD_ID";

    /// A standard ID doesn't match the `APS-V1-\d{4}` pattern.
    pub const CF_INVALID_STANDARD_ID: &str = "CF_INVALID_STANDARD_ID";

    /// A standard entry is missing the `version` field.
    pub const CF_MISSING_VERSION_REQ: &str = "CF_MISSING_VERSION_REQ";

    /// A version requirement string can't be parsed as semver.
    pub const CF_INVALID_VERSION_REQ: &str = "CF_INVALID_VERSION_REQ";

    /// Two different slugs map to the same standard ID.
    pub const CF_DUPLICATE_STANDARD_ID: &str = "CF_DUPLICATE_STANDARD_ID";

    /// A substandard code doesn't match `[A-Z]{2}\d{2}`.
    pub const CF_INVALID_SUBSTANDARD_CODE: &str = "CF_INVALID_SUBSTANDARD_CODE";

    /// A child `apss.toml` contains a `[workspace]` section.
    pub const CF_WORKSPACE_IN_CHILD: &str = "CF_WORKSPACE_IN_CHILD";

    /// Child and root `apss_version` values differ.
    pub const CF_APSS_VERSION_MISMATCH: &str = "CF_APSS_VERSION_MISMATCH";

    /// Child and root version ranges have an empty intersection.
    pub const CF_VERSION_RANGE_CONFLICT: &str = "CF_VERSION_RANGE_CONFLICT";

    /// Config block fails to deserialize into the standard's config type.
    pub const CF_INVALID_CONFIG_VALUE: &str = "CF_INVALID_CONFIG_VALUE";

    /// Config deserializes but `StandardConfig::validate()` returns errors.
    pub const CF_CONFIG_VALIDATION_FAILED: &str = "CF_CONFIG_VALIDATION_FAILED";

    /// `[standards]` section exists but is empty.
    pub const CF_EMPTY_STANDARDS: &str = "CF_EMPTY_STANDARDS";

    /// `apss.toml` exists but no `apss.lock` found.
    pub const CF_NO_LOCKFILE: &str = "CF_NO_LOCKFILE";

    /// `apss.toml` was modified more recently than `apss.lock`.
    pub const CF_LOCKFILE_STALE: &str = "CF_LOCKFILE_STALE";

    /// Failed to parse the apss.toml file.
    pub const CF_PARSE_ERROR: &str = "CF_PARSE_ERROR";

    /// The apss.toml file was not found.
    pub const CF_FILE_NOT_FOUND: &str = "CF_FILE_NOT_FOUND";

    // --- Standard config surface validation (APS repo CI) ---

    /// Standard crate doesn't export a `StandardConfig` type.
    pub const CF_MISSING_CONFIG_TYPE: &str = "CF_MISSING_CONFIG_TYPE";

    /// Config type doesn't implement `Default`.
    pub const CF_NO_CONFIG_DEFAULTS: &str = "CF_NO_CONFIG_DEFAULTS";

    /// `config.schema.json` doesn't match generated output.
    pub const CF_CONFIG_SCHEMA_STALE: &str = "CF_CONFIG_SCHEMA_STALE";

    /// `validate()` is a no-op.
    pub const CF_NO_CONFIG_VALIDATION: &str = "CF_NO_CONFIG_VALIDATION";
}

// ============================================================================
// Validation Functions
// ============================================================================

/// Validate a project configuration file at the given path.
///
/// This is the primary entry point for CF01 validation. It checks:
/// - File existence and parsability
/// - Schema identifier
/// - Project identity fields
/// - Standard entries (IDs, versions, substandard codes)
/// - Lockfile presence and staleness
pub fn validate_project_config(path: &Path) -> Diagnostics {
    let mut diags = Diagnostics::new();

    // Parse the config file
    let config = match config::parse_project_config(path) {
        Ok(c) => c,
        Err(config::ConfigError::Io { .. }) => {
            diags.push(
                Diagnostic::error(
                    error_codes::CF_FILE_NOT_FOUND,
                    format!("Configuration file not found: {}", path.display()),
                )
                .with_path(path)
                .with_hint(format!(
                    "Create an {CONFIG_FILENAME} file or run 'apss init'"
                )),
            );
            return diags;
        }
        Err(config::ConfigError::Parse { source, .. }) => {
            diags.push(
                Diagnostic::error(
                    error_codes::CF_PARSE_ERROR,
                    format!("Failed to parse configuration: {source}"),
                )
                .with_path(path),
            );
            return diags;
        }
        Err(e) => {
            diags.push(
                Diagnostic::error(error_codes::CF_PARSE_ERROR, e.to_string()).with_path(path),
            );
            return diags;
        }
    };

    // Validate schema
    validate_schema(&config, path, &mut diags);

    // Validate project info
    validate_project_info(&config, path, &mut diags);

    // Validate standard entries
    validate_standards(&config, path, &mut diags);

    // Check lockfile
    validate_lockfile(path, &mut diags);

    diags
}

/// Validate a child `apss.toml` in a workspace context.
pub fn validate_child_config(child_path: &Path, root_config: &ProjectConfig) -> Diagnostics {
    let mut diags = validate_project_config(child_path);

    let child_config = match config::parse_project_config(child_path) {
        Ok(c) => c,
        Err(_) => return diags, // Already reported by validate_project_config
    };

    // Child must not have [workspace]
    if child_config.workspace.is_some() {
        diags.push(
            Diagnostic::error(
                error_codes::CF_WORKSPACE_IN_CHILD,
                "Child configuration must not contain a [workspace] section",
            )
            .with_path(child_path)
            .with_hint("Only the root apss.toml may define workspace members"),
        );
    }

    // APSS version must match root
    if child_config.project.apss_version != root_config.project.apss_version {
        diags.push(
            Diagnostic::error(
                error_codes::CF_APSS_VERSION_MISMATCH,
                format!(
                    "apss_version mismatch: root='{}', child='{}'",
                    root_config.project.apss_version, child_config.project.apss_version
                ),
            )
            .with_path(child_path)
            .with_hint("All workspace members must use the same apss_version as the root"),
        );
    }

    diags
}

// ============================================================================
// Internal Validators
// ============================================================================

fn validate_schema(config: &ProjectConfig, path: &Path, diags: &mut Diagnostics) {
    if config.schema != PROJECT_SCHEMA {
        diags.push(
            Diagnostic::error(
                error_codes::CF_MISSING_SCHEMA,
                format!(
                    "Invalid schema: expected '{}', got '{}'",
                    PROJECT_SCHEMA, config.schema
                ),
            )
            .with_path(path)
            .with_hint(format!("Set schema = \"{PROJECT_SCHEMA}\"")),
        );
    }
}

fn validate_project_info(config: &ProjectConfig, path: &Path, diags: &mut Diagnostics) {
    if config.project.name.trim().is_empty() {
        diags.push(
            Diagnostic::error(
                error_codes::CF_MISSING_PROJECT_NAME,
                "project.name is required and must not be empty",
            )
            .with_path(path),
        );
    }

    if config.project.apss_version != "v1" {
        diags.push(
            Diagnostic::error(
                error_codes::CF_INVALID_APSS_VERSION,
                format!(
                    "Unsupported apss_version '{}'. Currently only 'v1' is supported",
                    config.project.apss_version
                ),
            )
            .with_path(path),
        );
    }
}

fn validate_standards(config: &ProjectConfig, path: &Path, diags: &mut Diagnostics) {
    if config.standards.is_empty() {
        diags.push(
            Diagnostic::warning(
                error_codes::CF_EMPTY_STANDARDS,
                "No standards declared. Add standards under [standards.<slug>]",
            )
            .with_path(path),
        );
        return;
    }

    // Track IDs to detect duplicates
    let mut seen_ids: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();

    for (slug, entry) in &config.standards {
        // Validate standard ID format
        if entry.id.is_empty() {
            diags.push(
                Diagnostic::error(
                    error_codes::CF_MISSING_STANDARD_ID,
                    format!("standards.{slug}.id is required"),
                )
                .with_path(path),
            );
        } else if !is_valid_standard_id(&entry.id) {
            diags.push(
                Diagnostic::error(
                    error_codes::CF_INVALID_STANDARD_ID,
                    format!(
                        "Invalid standard ID '{}' for slug '{slug}'. Must match APS-V1-XXXX",
                        entry.id
                    ),
                )
                .with_path(path),
            );
        }

        // Check for duplicate IDs
        if let Some(prev_slug) = seen_ids.insert(&entry.id, slug.as_str()) {
            diags.push(
                Diagnostic::error(
                    error_codes::CF_DUPLICATE_STANDARD_ID,
                    format!(
                        "Standard ID '{}' is declared under both '{}' and '{}'",
                        entry.id, prev_slug, slug
                    ),
                )
                .with_path(path)
                .with_hint("Each standard ID must map to exactly one slug"),
            );
        }

        // Validate version requirement
        if entry.version.is_empty() {
            diags.push(
                Diagnostic::error(
                    error_codes::CF_MISSING_VERSION_REQ,
                    format!("standards.{slug}.version is required"),
                )
                .with_path(path),
            );
        } else if semver::VersionReq::parse(&entry.version).is_err() {
            diags.push(
                Diagnostic::error(
                    error_codes::CF_INVALID_VERSION_REQ,
                    format!(
                        "Invalid version requirement '{}' for slug '{slug}'",
                        entry.version
                    ),
                )
                .with_path(path)
                .with_hint("Use Cargo-style semver (e.g., \">=1.0.0, <2.0.0\")"),
            );
        }

        // Validate substandard codes
        if let Some(subs) = &entry.substandards {
            for code in subs {
                if !is_valid_substandard_code(code) {
                    diags.push(
                        Diagnostic::error(
                            error_codes::CF_INVALID_SUBSTANDARD_CODE,
                            format!(
                                "Invalid substandard code '{code}' for slug '{slug}'. Must match [A-Z]{{2}}\\d{{2}} (e.g., RS01)"
                            ),
                        )
                        .with_path(path),
                    );
                }
            }
        }
    }
}

fn validate_lockfile(config_path: &Path, diags: &mut Diagnostics) {
    let lockfile_path = config_path
        .parent()
        .unwrap_or(Path::new("."))
        .join(aps_core::lockfile::LOCKFILE_FILENAME);

    if !lockfile_path.exists() {
        diags.push(
            Diagnostic::warning(
                error_codes::CF_NO_LOCKFILE,
                "No apss.lock found. Run 'apss install' to generate one",
            )
            .with_path(config_path),
        );
        return;
    }

    // Check staleness
    if let (Ok(config_meta), Ok(lock_meta)) = (
        std::fs::metadata(config_path),
        std::fs::metadata(&lockfile_path),
    ) {
        if let (Ok(config_mod), Ok(lock_mod)) = (config_meta.modified(), lock_meta.modified()) {
            if config_mod > lock_mod {
                diags.push(
                    Diagnostic::warning(
                        error_codes::CF_LOCKFILE_STALE,
                        "apss.toml is newer than apss.lock. Run 'apss install' to update",
                    )
                    .with_path(config_path),
                );
            }
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Check if a string matches the `APS-V1-XXXX` pattern.
fn is_valid_standard_id(id: &str) -> bool {
    if !id.starts_with("APS-V1-") {
        return false;
    }
    let suffix = &id[7..];
    suffix.len() == 4 && suffix.chars().all(|c| c.is_ascii_digit())
}

/// Check if a string matches the `[A-Z]{2}\d{2}` substandard code pattern.
fn is_valid_substandard_code(code: &str) -> bool {
    if code.len() != 4 {
        return false;
    }
    let bytes = code.as_bytes();
    bytes[0].is_ascii_uppercase()
        && bytes[1].is_ascii_uppercase()
        && bytes[2].is_ascii_digit()
        && bytes[3].is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_standard_id() {
        assert!(is_valid_standard_id("APS-V1-0000"));
        assert!(is_valid_standard_id("APS-V1-0001"));
        assert!(is_valid_standard_id("APS-V1-9999"));
        assert!(!is_valid_standard_id("APS-V1-000"));
        assert!(!is_valid_standard_id("APS-V2-0001"));
        assert!(!is_valid_standard_id("EXP-V1-0001"));
        assert!(!is_valid_standard_id(""));
    }

    #[test]
    fn test_is_valid_substandard_code() {
        assert!(is_valid_substandard_code("RS01"));
        assert!(is_valid_substandard_code("CI01"));
        assert!(is_valid_substandard_code("VZ99"));
        assert!(!is_valid_substandard_code("rs01")); // lowercase
        assert!(!is_valid_substandard_code("R01")); // too short
        assert!(!is_valid_substandard_code("RST01")); // too long
        assert!(!is_valid_substandard_code("1234")); // no letters
    }

    #[test]
    fn test_validate_valid_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("apss.toml");
        std::fs::write(
            &config_path,
            r#"
schema = "apss.project/v1"

[project]
name = "test-project"
apss_version = "v1"

[standards.topology]
id = "APS-V1-0001"
version = ">=1.0.0, <2.0.0"
substandards = ["RS01", "CI01"]

[standards.topology.config]
output_dir = ".topology"
"#,
        )
        .unwrap();

        let diags = validate_project_config(&config_path);
        assert!(!diags.has_errors(), "Unexpected errors: {diags}");
    }

    #[test]
    fn test_validate_bad_schema() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("apss.toml");
        std::fs::write(
            &config_path,
            r#"
schema = "wrong/v1"
[project]
name = "test"
apss_version = "v1"
"#,
        )
        .unwrap();

        let diags = validate_project_config(&config_path);
        assert!(diags.has_errors());
        assert!(
            diags
                .iter()
                .any(|d| d.code == error_codes::CF_MISSING_SCHEMA)
        );
    }

    #[test]
    fn test_validate_bad_standard_id() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("apss.toml");
        std::fs::write(
            &config_path,
            r#"
schema = "apss.project/v1"
[project]
name = "test"
apss_version = "v1"

[standards.topology]
id = "INVALID"
version = ">=1.0.0"
"#,
        )
        .unwrap();

        let diags = validate_project_config(&config_path);
        assert!(diags.has_errors());
        assert!(
            diags
                .iter()
                .any(|d| d.code == error_codes::CF_INVALID_STANDARD_ID)
        );
    }

    #[test]
    fn test_validate_bad_version_req() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("apss.toml");
        std::fs::write(
            &config_path,
            r#"
schema = "apss.project/v1"
[project]
name = "test"
apss_version = "v1"

[standards.topology]
id = "APS-V1-0001"
version = "not-semver!!"
"#,
        )
        .unwrap();

        let diags = validate_project_config(&config_path);
        assert!(diags.has_errors());
        assert!(
            diags
                .iter()
                .any(|d| d.code == error_codes::CF_INVALID_VERSION_REQ)
        );
    }

    #[test]
    fn test_validate_duplicate_ids() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("apss.toml");
        std::fs::write(
            &config_path,
            r#"
schema = "apss.project/v1"
[project]
name = "test"
apss_version = "v1"

[standards.topology]
id = "APS-V1-0001"
version = ">=1.0.0"

[standards.topo]
id = "APS-V1-0001"
version = ">=1.0.0"
"#,
        )
        .unwrap();

        let diags = validate_project_config(&config_path);
        assert!(diags.has_errors());
        assert!(
            diags
                .iter()
                .any(|d| d.code == error_codes::CF_DUPLICATE_STANDARD_ID)
        );
    }

    #[test]
    fn test_validate_bad_substandard_code() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("apss.toml");
        std::fs::write(
            &config_path,
            r#"
schema = "apss.project/v1"
[project]
name = "test"
apss_version = "v1"

[standards.topology]
id = "APS-V1-0001"
version = ">=1.0.0"
substandards = ["rs01", "TOOLONG01"]
"#,
        )
        .unwrap();

        let diags = validate_project_config(&config_path);
        assert!(diags.has_errors());
        assert!(
            diags
                .iter()
                .any(|d| d.code == error_codes::CF_INVALID_SUBSTANDARD_CODE)
        );
    }

    #[test]
    fn test_validate_empty_standards_warning() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("apss.toml");
        std::fs::write(
            &config_path,
            r#"
schema = "apss.project/v1"
[project]
name = "test"
apss_version = "v1"
"#,
        )
        .unwrap();

        let diags = validate_project_config(&config_path);
        assert!(!diags.has_errors());
        assert!(diags.has_warnings());
        assert!(
            diags
                .iter()
                .any(|d| d.code == error_codes::CF_EMPTY_STANDARDS)
        );
    }

    #[test]
    fn test_validate_missing_file() {
        let diags = validate_project_config(Path::new("/nonexistent/apss.toml"));
        assert!(diags.has_errors());
        assert!(
            diags
                .iter()
                .any(|d| d.code == error_codes::CF_FILE_NOT_FOUND)
        );
    }

    #[test]
    fn test_validate_child_workspace_forbidden() {
        let temp = tempfile::tempdir().unwrap();

        let root_path = temp.path().join("apss.toml");
        std::fs::write(
            &root_path,
            r#"
schema = "apss.project/v1"
[project]
name = "root"
apss_version = "v1"
[workspace]
members = ["packages/*"]
"#,
        )
        .unwrap();

        let child_dir = temp.path().join("packages/a");
        std::fs::create_dir_all(&child_dir).unwrap();
        let child_path = child_dir.join("apss.toml");
        std::fs::write(
            &child_path,
            r#"
schema = "apss.project/v1"
[project]
name = "child"
apss_version = "v1"
[workspace]
members = ["sub/*"]
"#,
        )
        .unwrap();

        let root_config = config::parse_project_config(&root_path).unwrap();
        let diags = validate_child_config(&child_path, &root_config);
        assert!(diags.has_errors());
        assert!(
            diags
                .iter()
                .any(|d| d.code == error_codes::CF_WORKSPACE_IN_CHILD)
        );
    }
}
