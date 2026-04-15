//! Distribution & Installation (APS-V1-0000.DI01)
//!
//! Defines how APS standards are packaged, distributed, installed, and
//! composed into project-local CLI binaries.
//!
//! ## Key Concepts
//!
//! - **Standard crates** — each standard publishes as an independent Rust crate
//! - **Bootstrap binary** — lightweight `apss` CLI for init/install
//! - **Composed binary** — project-local binary with only declared standards
//! - **Lockfile** — `apss.lock` pins exact versions for reproducibility
//!
//! ## Quick Start
//!
//! ```bash
//! cargo install apss              # install bootstrap
//! apss init --standard topology   # create apss.toml
//! apss install                    # build composed binary
//! apss run topology analyze .     # use it
//! ```

pub mod codegen;

use aps_core::{Diagnostic, Diagnostics};
use std::path::Path;

// ============================================================================
// Error Codes
// ============================================================================

/// Error codes for DI01 validation.
pub mod error_codes {
    /// Publishable standard crate doesn't export `register()`.
    pub const DI_MISSING_REGISTER_FN: &str = "DI_MISSING_REGISTER_FN";

    /// Crate name doesn't follow `apss-v1-NNNN-slug` pattern.
    pub const DI_INVALID_CRATE_NAME: &str = "DI_INVALID_CRATE_NAME";

    /// Standard crate doesn't depend on `aps-core`.
    pub const DI_MISSING_APS_CORE_DEP: &str = "DI_MISSING_APS_CORE_DEP";

    /// Checksum in `apss.lock` doesn't match crate tarball.
    pub const DI_LOCKFILE_INTEGRITY: &str = "DI_LOCKFILE_INTEGRITY";

    /// `apss.lock` fails to parse.
    pub const DI_LOCKFILE_PARSE_ERROR: &str = "DI_LOCKFILE_PARSE_ERROR";

    /// `.apss/build/` directory missing when binary expected.
    pub const DI_BUILD_DIR_MISSING: &str = "DI_BUILD_DIR_MISSING";

    /// Binary older than lockfile.
    pub const DI_BINARY_STALE: &str = "DI_BINARY_STALE";

    /// Lockfile exists but `.apss/bin/apss` doesn't.
    pub const DI_BINARY_MISSING: &str = "DI_BINARY_MISSING";

    /// Cargo.toml version doesn't match standard/substandard/experiment.toml version.
    pub const DI_VERSION_MISMATCH: &str = "DI_VERSION_MISMATCH";

    /// Crate is missing required metadata for publishing.
    pub const DI_MISSING_PUBLISH_METADATA: &str = "DI_MISSING_PUBLISH_METADATA";

    /// Crate uses `publish = false` but is expected to be publishable.
    pub const DI_PUBLISH_DISABLED: &str = "DI_PUBLISH_DISABLED";
}

// ============================================================================
// Constants
// ============================================================================

/// Standard crate name prefix.
pub const CRATE_PREFIX: &str = "apss-v1-";

/// Build directory relative to project root.
pub const BUILD_DIR: &str = ".apss/build";

/// Binary directory relative to project root.
pub const BIN_DIR: &str = ".apss/bin";

/// Binary name.
pub const BIN_NAME: &str = "apss";

// ============================================================================
// Validation Functions
// ============================================================================

/// Validate a standard crate's readiness for publishing.
///
/// Checks that the crate follows DI01's packaging requirements:
/// - Correct crate naming convention
/// - Depends on `aps-core`
/// - Exports a `register()` function
pub fn validate_publishable_standard(crate_path: &Path) -> Diagnostics {
    let mut diags = Diagnostics::new();

    // Check Cargo.toml exists and has correct name pattern
    let cargo_path = crate_path.join("Cargo.toml");
    if !cargo_path.exists() {
        diags.push(
            Diagnostic::error(
                error_codes::DI_MISSING_APS_CORE_DEP,
                "No Cargo.toml found in standard crate",
            )
            .with_path(crate_path),
        );
        return diags;
    }

    let cargo_content = match std::fs::read_to_string(&cargo_path) {
        Ok(c) => c,
        Err(e) => {
            diags.push(
                Diagnostic::error(
                    error_codes::DI_MISSING_APS_CORE_DEP,
                    format!("Failed to read Cargo.toml: {e}"),
                )
                .with_path(&cargo_path),
            );
            return diags;
        }
    };

    let cargo_toml: toml::Value = match cargo_content.parse() {
        Ok(v) => v,
        Err(e) => {
            diags.push(
                Diagnostic::error(
                    error_codes::DI_MISSING_APS_CORE_DEP,
                    format!("Failed to parse Cargo.toml: {e}"),
                )
                .with_path(&cargo_path),
            );
            return diags;
        }
    };

    // Validate crate name follows convention
    if let Some(name) = cargo_toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
    {
        if !name.starts_with(CRATE_PREFIX) && name != "aps-core" && name != "apss" {
            diags.push(
                Diagnostic::warning(
                    error_codes::DI_INVALID_CRATE_NAME,
                    format!(
                        "Crate name '{name}' doesn't follow the '{CRATE_PREFIX}NNNN-slug' convention"
                    ),
                )
                .with_path(&cargo_path)
                .with_hint(format!("Rename to '{CRATE_PREFIX}NNNN-your-slug'")),
            );
        }
    }

    // Check aps-core dependency
    let has_core_dep = cargo_toml
        .get("dependencies")
        .map(|deps| deps.get("aps-core").is_some())
        .unwrap_or(false);

    if !has_core_dep {
        diags.push(
            Diagnostic::error(
                error_codes::DI_MISSING_APS_CORE_DEP,
                "Standard crate must depend on aps-core",
            )
            .with_path(&cargo_path)
            .with_hint("Add aps-core to [dependencies]"),
        );
    }

    // Check for register() function in lib.rs
    let lib_path = crate_path.join("src/lib.rs");
    if lib_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&lib_path) {
            if !content.contains("pub fn register") {
                diags.push(
                    Diagnostic::warning(
                        error_codes::DI_MISSING_REGISTER_FN,
                        "Standard crate should export a `pub fn register(registry: &mut dyn StandardRegistry)` function",
                    )
                    .with_path(&lib_path)
                    .with_hint("Add a register() function for CLI composition"),
                );
            }
        }
    }

    diags
}

/// Validate version consistency and publish-readiness for a standard crate.
///
/// Checks:
/// - Cargo.toml version matches metadata (standard/substandard/experiment.toml)
/// - Required publish metadata fields are present (description, license, repository)
/// - Crate is not marked `publish = false`
pub fn validate_release_readiness(crate_path: &Path) -> Diagnostics {
    let mut diags = Diagnostics::new();

    let cargo_path = crate_path.join("Cargo.toml");
    let cargo_content = match std::fs::read_to_string(&cargo_path) {
        Ok(c) => c,
        Err(_) => return diags,
    };

    let cargo_toml: toml::Value = match cargo_content.parse() {
        Ok(v) => v,
        Err(_) => return diags,
    };

    let package = match cargo_toml.get("package").and_then(|p| p.as_table()) {
        Some(p) => p,
        None => return diags,
    };

    // --- Version consistency ---
    let cargo_version = package
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Skip workspace-inherited versions — they're managed centrally
    let is_workspace_version = package
        .get("version")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("workspace"))
        .and_then(|w| w.as_bool())
        .unwrap_or(false);

    if !is_workspace_version {
        if let Some(cargo_ver) = &cargo_version {
            // Find the metadata version
            let metadata_version = if crate_path.join("standard.toml").exists() {
                aps_core::metadata::parse_standard_metadata(&crate_path.join("standard.toml"))
                    .ok()
                    .map(|m| m.standard.version)
            } else if crate_path.join("substandard.toml").exists() {
                aps_core::metadata::parse_substandard_metadata(&crate_path.join("substandard.toml"))
                    .ok()
                    .map(|m| m.substandard.version)
            } else if crate_path.join("experiment.toml").exists() {
                aps_core::metadata::parse_experiment_metadata(&crate_path.join("experiment.toml"))
                    .ok()
                    .map(|m| m.experiment.version)
            } else {
                None
            };

            if let Some(meta_ver) = metadata_version {
                if *cargo_ver != meta_ver {
                    diags.push(
                        Diagnostic::error(
                            error_codes::DI_VERSION_MISMATCH,
                            format!(
                                "Cargo.toml version '{cargo_ver}' doesn't match metadata version '{meta_ver}'"
                            ),
                        )
                        .with_path(&cargo_path)
                        .with_hint("Keep Cargo.toml and standard/substandard/experiment.toml versions in sync"),
                    );
                }
            }
        }
    }

    // --- Publish metadata ---
    let has_description = package.get("description").is_some();
    let has_license = package.get("license").is_some();
    let has_repository = package.get("repository").is_some();

    // Check for workspace-inherited fields too
    let has_license_ws = package
        .get("license")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("workspace"))
        .is_some();
    let has_repo_ws = package
        .get("repository")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("workspace"))
        .is_some();

    if !has_description {
        diags.push(
            Diagnostic::warning(
                error_codes::DI_MISSING_PUBLISH_METADATA,
                "Missing 'description' in Cargo.toml — required for crates.io publishing",
            )
            .with_path(&cargo_path),
        );
    }

    if !has_license && !has_license_ws {
        diags.push(
            Diagnostic::warning(
                error_codes::DI_MISSING_PUBLISH_METADATA,
                "Missing 'license' in Cargo.toml — required for crates.io publishing",
            )
            .with_path(&cargo_path),
        );
    }

    if !has_repository && !has_repo_ws {
        diags.push(
            Diagnostic::warning(
                error_codes::DI_MISSING_PUBLISH_METADATA,
                "Missing 'repository' in Cargo.toml — required for crates.io publishing",
            )
            .with_path(&cargo_path),
        );
    }

    // --- Publish flag ---
    if let Some(publish) = package.get("publish").and_then(|v| v.as_bool()) {
        if !publish {
            diags.push(
                Diagnostic::warning(
                    error_codes::DI_PUBLISH_DISABLED,
                    "Crate has 'publish = false' — it won't be publishable to crates.io",
                )
                .with_path(&cargo_path)
                .with_hint("Remove 'publish = false' if this crate should be distributed"),
            );
        }
    }

    diags
}

/// Validate the installation state of a project.
///
/// Checks that the composed binary exists and is up-to-date.
pub fn validate_installation(project_root: &Path) -> Diagnostics {
    let mut diags = Diagnostics::new();

    let lockfile_path = project_root.join(aps_core::lockfile::LOCKFILE_FILENAME);
    let binary_path = project_root.join(BIN_DIR).join(BIN_NAME);
    let build_dir = project_root.join(BUILD_DIR);

    // If no lockfile, nothing to validate
    if !lockfile_path.exists() {
        return diags;
    }

    // Lockfile exists — check it parses
    if let Err(e) = aps_core::lockfile::parse_lockfile(&lockfile_path) {
        diags.push(
            Diagnostic::error(
                error_codes::DI_LOCKFILE_PARSE_ERROR,
                format!("Failed to parse lockfile: {e}"),
            )
            .with_path(&lockfile_path),
        );
        return diags;
    }

    // Check binary exists
    if !binary_path.exists() {
        diags.push(
            Diagnostic::warning(
                error_codes::DI_BINARY_MISSING,
                "Lockfile exists but composed binary not found. Run 'apss install'",
            )
            .with_path(&binary_path),
        );
        return diags;
    }

    // Check build dir exists
    if !build_dir.exists() {
        diags.push(
            Diagnostic::warning(
                error_codes::DI_BUILD_DIR_MISSING,
                "Build directory missing. Run 'apss install' to regenerate",
            )
            .with_path(&build_dir),
        );
    }

    // Check binary staleness
    if let (Ok(lock_meta), Ok(bin_meta)) = (
        std::fs::metadata(&lockfile_path),
        std::fs::metadata(&binary_path),
    ) {
        if let (Ok(lock_mod), Ok(bin_mod)) = (lock_meta.modified(), bin_meta.modified()) {
            if lock_mod > bin_mod {
                diags.push(
                    Diagnostic::warning(
                        error_codes::DI_BINARY_STALE,
                        "Composed binary is older than lockfile. Run 'apss install' to rebuild",
                    )
                    .with_path(&binary_path),
                );
            }
        }
    }

    diags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_installation_no_lockfile() {
        let temp = tempfile::tempdir().unwrap();
        let diags = validate_installation(temp.path());
        assert!(diags.is_empty());
    }

    #[test]
    fn test_validate_installation_missing_binary() {
        let temp = tempfile::tempdir().unwrap();

        // Create a valid lockfile
        let lockfile = aps_core::lockfile::Lockfile::new("0.1.0".to_string());
        aps_core::lockfile::write_lockfile(
            &temp.path().join(aps_core::lockfile::LOCKFILE_FILENAME),
            &lockfile,
        )
        .unwrap();

        let diags = validate_installation(temp.path());
        assert!(diags.has_warnings());
        assert!(
            diags
                .iter()
                .any(|d| d.code == error_codes::DI_BINARY_MISSING)
        );
    }

    #[test]
    fn test_validate_publishable_no_cargo() {
        let temp = tempfile::tempdir().unwrap();
        let diags = validate_publishable_standard(temp.path());
        assert!(diags.has_errors());
    }

    #[test]
    fn test_validate_publishable_valid() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();

        std::fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "apss-v1-0001-code-topology"
version = "1.0.0"

[dependencies]
aps-core = "0.1.0"
"#,
        )
        .unwrap();

        std::fs::write(
            src.join("lib.rs"),
            r#"
pub fn register(registry: &mut dyn aps_core::StandardRegistry) {
    // ...
}
"#,
        )
        .unwrap();

        let diags = validate_publishable_standard(temp.path());
        assert!(!diags.has_errors(), "Unexpected errors: {diags}");
    }
}
