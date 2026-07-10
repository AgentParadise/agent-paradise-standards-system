//! Distribution & Installation (APS-V1-0000.DI01)
//!
//! Defines how APS standards are packaged, distributed, installed, and
//! composed into project-local CLI binaries.
//!
//! ## Key Concepts
//!
//! - **Standard crates**  -  each standard publishes as an independent Rust crate
//! - **Bootstrap binary**  -  lightweight `apss` CLI for init/install
//! - **Composed binary**  -  project-local binary with only declared standards
//! - **Lockfile**  -  `apss.lock` pins exact versions for reproducibility
//!
//! ## Quick Start
//!
//! ```bash
//! cargo install apss              # install bootstrap
//! apss init --standard code-topology   # create apss.yaml
//! apss install                    # build composed binary
//! apss run topology analyze .     # use it
//! ```

pub mod codegen;

use crate::{Diagnostic, Diagnostics};
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

    /// Standard crate doesn't depend on `apss-core`.
    pub const DI_MISSING_APSS_CORE_DEP: &str = "DI_MISSING_APSS_CORE_DEP";

    /// Cargo.toml is missing from the crate directory.
    pub const DI_MISSING_CARGO_TOML: &str = "DI_MISSING_CARGO_TOML";

    /// Cargo.toml failed to parse.
    pub const DI_CARGO_TOML_PARSE_ERROR: &str = "DI_CARGO_TOML_PARSE_ERROR";

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

    /// Crate is missing recommended discovery metadata for crates.io.
    pub const DI_MISSING_DISCOVERY_METADATA: &str = "DI_MISSING_DISCOVERY_METADATA";

    /// Crate uses `publish = false` but is expected to be publishable.
    pub const DI_PUBLISH_DISABLED: &str = "DI_PUBLISH_DISABLED";

    /// A standard's substandard codes and its Cargo.toml `[features]` keys
    /// (excluding `default`) are not in one-to-one correspondence.
    pub const DI_SUBSTANDARD_FEATURE_MISMATCH: &str = "DI_SUBSTANDARD_FEATURE_MISMATCH";
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
/// - Depends on `apss-core`
/// - Exports a `register()` function
pub fn validate_publishable_standard(crate_path: &Path) -> Diagnostics {
    let mut diags = Diagnostics::new();

    // Check Cargo.toml exists and has correct name pattern
    let cargo_path = crate_path.join("Cargo.toml");
    if !cargo_path.exists() {
        diags.push(
            Diagnostic::error(
                error_codes::DI_MISSING_CARGO_TOML,
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
                    error_codes::DI_MISSING_CARGO_TOML,
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
                    error_codes::DI_CARGO_TOML_PARSE_ERROR,
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
        if !crate::ecosystem::is_ecosystem_crate(name) && !is_valid_standard_crate_name(name) {
            diags.push(
                Diagnostic::error(
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

    // Check apss-core dependency
    let has_core_dep = cargo_toml
        .get("dependencies")
        .map(|deps| deps.get("apss-core").is_some())
        .unwrap_or(false);

    if !has_core_dep {
        diags.push(
            Diagnostic::error(
                error_codes::DI_MISSING_APSS_CORE_DEP,
                "Standard crate must depend on apss-core",
            )
            .with_path(&cargo_path)
            .with_hint("Add apss-core to [dependencies]"),
        );
    }

    // Check for register() function in lib.rs
    let lib_path = crate_path.join("src/lib.rs");
    if lib_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&lib_path) {
            if !content.contains("pub fn register") {
                diags.push(
                    Diagnostic::error(
                        error_codes::DI_MISSING_REGISTER_FN,
                        "Standard crate must export a `pub fn register(registry: &mut dyn StandardRegistry)` function",
                    )
                    .with_path(&lib_path)
                    .with_hint("Add a register() function for CLI composition"),
                );
            }
        }
    }

    diags
}

/// Whether a package at this path is published to crates.io.
///
/// Per ADR-0002, official standards and experiments publish to crates.io, but
/// the meta-standard (APS-V1-0000) and its internal substandards (CF01, DI01,
/// CL01, SS01) are never published. Discovery-metadata recommendations are
/// therefore only meaningful for the former.
fn publishes_to_crates_io(crate_path: &Path) -> bool {
    !crate_path
        .components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with("APS-V1-0000"))
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
        Err(e) => {
            diags.push(
                Diagnostic::error(
                    error_codes::DI_MISSING_CARGO_TOML,
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
                    error_codes::DI_CARGO_TOML_PARSE_ERROR,
                    format!("Failed to parse Cargo.toml: {e}"),
                )
                .with_path(&cargo_path),
            );
            return diags;
        }
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

    // Skip workspace-inherited versions  -  they're managed centrally
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
                crate::metadata::parse_standard_metadata(&crate_path.join("standard.toml"))
                    .ok()
                    .map(|m| m.standard.version)
            } else if crate_path.join("substandard.toml").exists() {
                crate::metadata::parse_substandard_metadata(&crate_path.join("substandard.toml"))
                    .ok()
                    .map(|m| m.substandard.version)
            } else if crate_path.join("experiment.toml").exists() {
                crate::metadata::parse_experiment_metadata(&crate_path.join("experiment.toml"))
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
                "Missing 'description' in Cargo.toml  -  required for crates.io publishing",
            )
            .with_path(&cargo_path),
        );
    }

    if !has_license && !has_license_ws {
        diags.push(
            Diagnostic::warning(
                error_codes::DI_MISSING_PUBLISH_METADATA,
                "Missing 'license' in Cargo.toml  -  required for crates.io publishing",
            )
            .with_path(&cargo_path),
        );
    }

    if !has_repository && !has_repo_ws {
        diags.push(
            Diagnostic::warning(
                error_codes::DI_MISSING_PUBLISH_METADATA,
                "Missing 'repository' in Cargo.toml  -  required for crates.io publishing",
            )
            .with_path(&cargo_path),
        );
    }

    // --- Discovery metadata ---
    // These fields are not required to publish (cargo publish succeeds without
    // them) but they make a crate discoverable and presentable on crates.io.
    // They are info-level recommendations, not warnings, so they do not fail
    // the distribution gate. The meta-standard and its internal substandards
    // (CF01/DI01/CL01/SS01) are never published to crates.io per ADR-0002, so
    // they are exempt entirely: nagging them for a crates.io landing page would
    // contradict the distribution model.
    if publishes_to_crates_io(crate_path) {
        let has_readme = package.get("readme").is_some();
        let has_keywords = package.get("keywords").is_some();
        let has_categories = package.get("categories").is_some();

        if !has_readme {
            diags.push(
                Diagnostic::info(
                    error_codes::DI_MISSING_DISCOVERY_METADATA,
                    "Missing 'readme' in Cargo.toml  -  recommended for the crates.io landing page",
                )
                .with_path(&cargo_path)
                .with_hint("Add 'readme = \"README.md\"' and a crate README"),
            );
        }

        if !has_keywords {
            diags.push(
                Diagnostic::info(
                    error_codes::DI_MISSING_DISCOVERY_METADATA,
                    "Missing 'keywords' in Cargo.toml  -  recommended for crates.io discovery",
                )
                .with_path(&cargo_path)
                .with_hint("Add 'keywords = [...]' (up to 5 terms, each 20 chars or fewer)"),
            );
        }

        if !has_categories {
            diags.push(
                Diagnostic::info(
                    error_codes::DI_MISSING_DISCOVERY_METADATA,
                    "Missing 'categories' in Cargo.toml  -  recommended for crates.io discovery",
                )
                .with_path(&cargo_path)
                .with_hint("Add 'categories = [...]' using crates.io category slugs"),
            );
        }
    }

    // --- Publish flag ---
    if let Some(publish) = package.get("publish").and_then(|v| v.as_bool()) {
        if !publish {
            diags.push(
                Diagnostic::warning(
                    error_codes::DI_PUBLISH_DISABLED,
                    "Crate has 'publish = false'  -  it won't be publishable to crates.io",
                )
                .with_path(&cargo_path)
                .with_hint("Remove 'publish = false' if this crate should be distributed"),
            );
        }
    }

    diags
}

/// Validate that a standard crate's `[features]` keys are exactly its
/// (merged) substandard codes.
///
/// This is the poka-yoke for the ADR-0002 / DI01 invariant: the cargo feature
/// name equals the substandard profile code (the id suffix after the last `.`).
/// Because codegen passes selected codes through verbatim as feature names, a
/// feature that is missing (or named differently) would silently drop a
/// substandard from composed builds. This check makes that impossible: it emits
/// an Error for each substandard code with no matching `[features]` entry, and
/// an Error for each non-`default` feature with no matching substandard.
///
/// Parity only applies to the feature-module distribution pattern: substandards
/// that ship inside the parent crate as feature-gated modules. Per ADR-0002,
/// those are exactly the substandards with no `Cargo.toml` of their own.
/// Substandards that publish as their own crates (each with a `Cargo.toml`) are
/// not feature-gated, so they are ignored here. A standard with no merged
/// substandards is a no-op.
///
/// The substandard codes are read from `substandards/*/substandard.toml` ids;
/// the feature keys are read from the standard crate's own `Cargo.toml`.
pub fn validate_substandard_feature_parity(crate_path: &Path) -> Diagnostics {
    use std::collections::BTreeSet;

    let mut diags = Diagnostics::new();

    let substandards_dir = crate_path.join("substandards");
    if !substandards_dir.is_dir() {
        // Standards without substandards have nothing to keep in parity.
        return diags;
    }

    // The feature-module distribution pattern is identified by the parent crate
    // declaring a `[features]` table. A standard with no `[features]` block is
    // not shipping substandards as feature modules (it uses the separate-crate
    // pattern, or has no substandards at all), so parity does not apply.
    let cargo_path = crate_path.join("Cargo.toml");
    let cargo_content = match std::fs::read_to_string(&cargo_path) {
        Ok(c) => c,
        Err(e) => {
            diags.push(
                Diagnostic::error(
                    error_codes::DI_MISSING_CARGO_TOML,
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
                    error_codes::DI_CARGO_TOML_PARSE_ERROR,
                    format!("Failed to parse Cargo.toml: {e}"),
                )
                .with_path(&cargo_path),
            );
            return diags;
        }
    };

    // Collect merged-substandard codes (the id suffix after the last '.').
    // Only feature-module substandards count: those without their own
    // Cargo.toml. Their PRESENCE, not the existence of a `[features]` table, is
    // what makes parity mandatory: a merged substandard with no `[features]`
    // table at all is the exact silent-mistake this check exists to catch.
    let mut codes: BTreeSet<String> = BTreeSet::new();
    let entries = match std::fs::read_dir(&substandards_dir) {
        Ok(entries) => entries,
        Err(e) => {
            diags.push(
                Diagnostic::error(
                    error_codes::DI_SUBSTANDARD_FEATURE_MISMATCH,
                    format!("Failed to read substandards directory: {e}"),
                )
                .with_path(&substandards_dir),
            );
            return diags;
        }
    };
    let mut has_separate_crate_substandard = false;
    for entry in entries.flatten() {
        let sub_dir = entry.path();
        let toml_path = sub_dir.join("substandard.toml");
        if !toml_path.is_file() {
            continue;
        }
        // Separate-crate substandards (own Cargo.toml) are not feature-gated.
        if sub_dir.join("Cargo.toml").is_file() {
            has_separate_crate_substandard = true;
            continue;
        }
        // Only genuine, parseable merged substandards participate in feature
        // parity. Metadata well-formedness is the structure validator's job;
        // an unparseable substandard.toml is skipped here so this check stays
        // focused on the feature-module invariant and does not double-report.
        if let Ok(meta) = crate::metadata::parse_substandard_metadata(&toml_path) {
            if let Some(code) = meta.substandard.id.rsplit('.').next() {
                codes.insert(code.to_string());
            }
        }
    }

    // No merged (feature-module) substandards: this is the separate-crate
    // pattern or has no substandards, so feature parity does not apply and a
    // `[features]` table (if any) is unrelated to substandards. No-op.
    if codes.is_empty() {
        let _ = has_separate_crate_substandard;
        return diags;
    }

    // There ARE merged substandards, so a matching `[features]` table is
    // required. A missing table is treated as an empty feature set, so every
    // code is reported as having no matching feature.
    let mut features: BTreeSet<String> = BTreeSet::new();
    if let Some(features_table) = cargo_toml.get("features").and_then(|f| f.as_table()) {
        for key in features_table.keys() {
            if key != "default" {
                features.insert(key.clone());
            }
        }
    }

    // A substandard code with no matching [features] entry.
    for code in codes.difference(&features) {
        diags.push(
            Diagnostic::error(
                error_codes::DI_SUBSTANDARD_FEATURE_MISMATCH,
                format!(
                    "Substandard code '{code}' has no matching '[features]' entry in Cargo.toml"
                ),
            )
            .with_path(&cargo_path)
            .with_hint(format!(
                "Add a '{code} = [...]' feature so the cargo feature name equals the substandard code"
            )),
        );
    }

    // A non-default feature with no matching substandard.
    for feature in features.difference(&codes) {
        diags.push(
            Diagnostic::error(
                error_codes::DI_SUBSTANDARD_FEATURE_MISMATCH,
                format!("Cargo.toml feature '{feature}' has no matching substandard"),
            )
            .with_path(&cargo_path)
            .with_hint(format!(
                "Remove the '{feature}' feature or add a substandard whose code is '{feature}'"
            )),
        );
    }

    diags
}

/// Validate the installation state of a project.
///
/// Checks that the composed binary exists and is up-to-date.
pub fn validate_installation(project_root: &Path) -> Diagnostics {
    let mut diags = Diagnostics::new();

    let lockfile_path = project_root.join(crate::lockfile::LOCKFILE_FILENAME);
    let binary_path = project_root.join(BIN_DIR).join(BIN_NAME);
    let build_dir = project_root.join(BUILD_DIR);

    // If no lockfile, nothing to validate
    if !lockfile_path.exists() {
        return diags;
    }

    // Lockfile exists  -  check it parses
    if let Err(e) = crate::lockfile::parse_lockfile(&lockfile_path) {
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
            Diagnostic::error(
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

// ============================================================================
// Helpers
// ============================================================================

/// Check if a crate name follows the `apss-v1-NNNN-slug` pattern.
fn is_valid_standard_crate_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix(CRATE_PREFIX) else {
        return false;
    };
    // Must have at least 4-digit ID + hyphen + slug
    if rest.len() < 6 {
        return false;
    }
    let (digits, after_digits) = rest.split_at(4);
    digits.chars().all(|c| c.is_ascii_digit())
        && after_digits.starts_with('-')
        && after_digits.len() > 1
}

/// Register DI01 with an APSS runtime registry.
pub fn register(registry: &mut dyn crate::registry::StandardRegistry) {
    registry.register(
        crate::registry::RegisteredStandard {
            id: "APS-V1-0000.DI01".to_string(),
            slug: "distribution".to_string(),
            name: "Distribution & Installation".to_string(),
            description: "Distribution and installation validation for APSS standards".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            commands: Vec::new(),
        },
        Box::new(NoopCommandHandler),
    );
}

struct NoopCommandHandler;

impl crate::registry::CommandHandler for NoopCommandHandler {
    fn execute(&self, _command: &str, _args: &[String], _config: &toml::Value) -> i32 {
        eprintln!("No composed CLI commands are registered for distribution yet.");
        5
    }

    fn commands(&self) -> Vec<crate::registry::CommandInfo> {
        Vec::new()
    }
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
        let lockfile = crate::lockfile::Lockfile::new("0.1.0".to_string());
        crate::lockfile::write_lockfile(
            &temp.path().join(crate::lockfile::LOCKFILE_FILENAME),
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
        assert!(
            diags
                .iter()
                .any(|d| d.code == error_codes::DI_MISSING_CARGO_TOML)
        );
    }

    #[test]
    fn test_is_valid_standard_crate_name() {
        assert!(is_valid_standard_crate_name("apss-v1-0001-code-topology"));
        assert!(is_valid_standard_crate_name("apss-v1-0003-fitness"));
        assert!(!is_valid_standard_crate_name("apss-v1-topology")); // no 4-digit ID
        assert!(!is_valid_standard_crate_name("apss-v1-0001")); // no slug
        assert!(!is_valid_standard_crate_name("apss-core")); // not a standard
    }

    #[test]
    fn test_ecosystem_allowlist_wired_through_apss_core() {
        // Regression guard: DI01 must read the ecosystem allowlist from apss-core
        // rather than maintaining its own list. A minimal spot-check here is
        // enough; the full matrix lives in crate::ecosystem's own tests.
        assert!(crate::ecosystem::is_ecosystem_crate("apss-core"));
        assert!(crate::ecosystem::is_ecosystem_crate(
            "apss-v1-0000-cf01-project-config"
        ));
        assert!(!crate::ecosystem::is_ecosystem_crate(
            "apss-v1-0001-code-topology"
        ));
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
apss-core = "0.1.0"
"#,
        )
        .unwrap();

        std::fs::write(
            src.join("lib.rs"),
            r#"
pub fn register(registry: &mut dyn crate::StandardRegistry) {
    // ...
}
"#,
        )
        .unwrap();

        let diags = validate_publishable_standard(temp.path());
        assert!(!diags.has_errors(), "Unexpected errors: {diags}");
    }

    #[test]
    fn test_validate_publishable_wrong_name_is_error() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        std::fs::create_dir_all(&src).unwrap();

        std::fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "bad-name"
version = "1.0.0"

[dependencies]
apss-core = "0.1.0"
"#,
        )
        .unwrap();
        std::fs::write(src.join("lib.rs"), "pub fn register() {}").unwrap();

        let diags = validate_publishable_standard(temp.path());
        let name_diag = diags
            .iter()
            .find(|d| d.code == error_codes::DI_INVALID_CRATE_NAME)
            .expect("expected DI_INVALID_CRATE_NAME diagnostic");
        assert!(
            name_diag.severity == crate::Severity::Error,
            "DI_INVALID_CRATE_NAME should be error severity"
        );
    }

    #[test]
    fn test_validate_publishable_no_register_is_error() {
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
apss-core = "0.1.0"
"#,
        )
        .unwrap();
        std::fs::write(src.join("lib.rs"), "// no register function").unwrap();

        let diags = validate_publishable_standard(temp.path());
        let reg_diag = diags
            .iter()
            .find(|d| d.code == error_codes::DI_MISSING_REGISTER_FN)
            .expect("expected DI_MISSING_REGISTER_FN diagnostic");
        assert!(
            reg_diag.severity == crate::Severity::Error,
            "DI_MISSING_REGISTER_FN should be error severity"
        );
    }

    #[test]
    fn test_validate_publishable_unreadable_cargo_uses_cargo_error() {
        let temp = tempfile::tempdir().unwrap();
        // No Cargo.toml at all → DI_MISSING_CARGO_TOML
        let diags = validate_publishable_standard(temp.path());
        let cargo_diag = diags
            .iter()
            .find(|d| d.code == error_codes::DI_MISSING_CARGO_TOML)
            .expect("expected DI_MISSING_CARGO_TOML diagnostic");
        assert!(
            cargo_diag.severity == crate::Severity::Error,
            "DI_MISSING_CARGO_TOML should be error severity"
        );
    }

    #[test]
    fn test_validate_publishable_bad_toml_uses_parse_error() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "not valid [[[ toml").unwrap();

        let diags = validate_publishable_standard(temp.path());
        assert!(
            diags
                .iter()
                .any(|d| d.code == error_codes::DI_CARGO_TOML_PARSE_ERROR),
            "expected DI_CARGO_TOML_PARSE_ERROR for malformed Cargo.toml"
        );
    }

    #[test]
    fn test_release_readiness_flags_missing_discovery_metadata_as_info() {
        let temp = tempfile::tempdir().unwrap();

        // A publishable standard path (not under APS-V1-0000-meta) with the
        // required publish metadata but no readme/keywords/categories.
        let crate_dir = temp.path().join("APS-V1-0001-code-topology");
        std::fs::create_dir(&crate_dir).unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "apss-v1-0001-code-topology"
version = "1.0.0"
description = "A standard"
license = "MIT"
repository = "https://example.com/repo"
"#,
        )
        .unwrap();

        let diags = validate_release_readiness(&crate_dir);

        // Discovery metadata is advisory: info, not warnings, so it never fails
        // the distribution gate.
        assert!(!diags.has_errors(), "Unexpected errors: {diags}");
        assert!(
            !diags.has_warnings(),
            "discovery metadata must not raise warnings: {diags}"
        );

        let discovery: Vec<_> = diags
            .iter()
            .filter(|d| d.code == error_codes::DI_MISSING_DISCOVERY_METADATA)
            .collect();
        assert_eq!(
            discovery.len(),
            3,
            "expected one info each for readme, keywords, categories"
        );
        for d in &discovery {
            assert_eq!(
                d.severity,
                crate::Severity::Info,
                "discovery metadata diagnostics must be info-level recommendations"
            );
        }

        let messages: Vec<&str> = discovery.iter().map(|d| d.message.as_str()).collect();
        assert!(messages.iter().any(|m| m.contains("readme")));
        assert!(messages.iter().any(|m| m.contains("keywords")));
        assert!(messages.iter().any(|m| m.contains("categories")));
    }

    #[test]
    fn test_release_readiness_skips_discovery_for_unpublished_meta() {
        let temp = tempfile::tempdir().unwrap();

        // The meta-standard and its substandards are never published, so they
        // get no discovery-metadata recommendations at all.
        let crate_dir = temp.path().join("APS-V1-0000-meta");
        std::fs::create_dir(&crate_dir).unwrap();
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "apss-v1-0000-meta"
version = "1.0.0"
description = "Meta standard"
license = "MIT"
repository = "https://example.com/repo"
"#,
        )
        .unwrap();

        let diags = validate_release_readiness(&crate_dir);
        assert!(
            !diags
                .iter()
                .any(|d| d.code == error_codes::DI_MISSING_DISCOVERY_METADATA),
            "unpublished meta crates must get no discovery-metadata diagnostics: {diags}"
        );
    }

    #[test]
    fn test_release_readiness_no_discovery_warning_when_present() {
        let temp = tempfile::tempdir().unwrap();

        std::fs::write(
            temp.path().join("Cargo.toml"),
            r#"
[package]
name = "apss-v1-0001-code-topology"
version = "1.0.0"
description = "A standard"
license = "MIT"
repository = "https://example.com/repo"
readme = "README.md"
keywords = ["topology"]
categories = ["development-tools"]
"#,
        )
        .unwrap();

        let diags = validate_release_readiness(temp.path());
        assert!(
            !diags
                .iter()
                .any(|d| d.code == error_codes::DI_MISSING_DISCOVERY_METADATA),
            "no discovery-metadata warnings expected when all fields present: {diags}"
        );
    }

    /// Build a standard crate dir with the given Cargo.toml [features] block and
    /// substandard codes (one substandard.toml per code).
    fn make_parity_fixture(features_block: &str, codes: &[&str]) -> tempfile::TempDir {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            format!(
                "[package]\nname = \"apss-v1-0001-code-topology\"\nversion = \"1.0.0\"\n\n{features_block}\n"
            ),
        )
        .unwrap();
        let subs = temp.path().join("substandards");
        for code in codes {
            let dir = subs.join(code);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("substandard.toml"),
                format!(
                    r#"schema = "aps.substandard/v1"

[substandard]
id = "APS-V1-0001.{code}"
name = "{code} Profile"
slug = "{code}"
version = "0.1.0"
parent_id = "APS-V1-0001"
parent_major = "0"

[ownership]
maintainers = ["AgentParadise"]
"#
                ),
            )
            .unwrap();
        }
        temp
    }

    #[test]
    fn test_feature_parity_passes_when_equal() {
        let temp = make_parity_fixture(
            "[features]\ndefault = [\"RS01\", \"FD01\"]\nRS01 = []\nFD01 = []",
            &["RS01", "FD01"],
        );
        let diags = validate_substandard_feature_parity(temp.path());
        assert!(!diags.has_errors(), "expected parity to pass: {diags}");
    }

    #[test]
    fn test_feature_parity_missing_feature_fails() {
        // Substandard FD01 has no matching [features] entry.
        let temp = make_parity_fixture(
            "[features]\ndefault = [\"RS01\"]\nRS01 = []",
            &["RS01", "FD01"],
        );
        let diags = validate_substandard_feature_parity(temp.path());
        let diag = diags
            .iter()
            .find(|d| d.code == error_codes::DI_SUBSTANDARD_FEATURE_MISMATCH)
            .expect("expected DI_SUBSTANDARD_FEATURE_MISMATCH");
        assert_eq!(diag.severity, crate::Severity::Error);
        assert!(
            diag.message.contains("FD01"),
            "diagnostic should name the missing code: {}",
            diag.message
        );
    }

    #[test]
    fn test_feature_parity_orphan_feature_fails() {
        // Feature ZZ99 has no matching substandard.
        let temp = make_parity_fixture(
            "[features]\ndefault = [\"RS01\", \"ZZ99\"]\nRS01 = []\nZZ99 = []",
            &["RS01"],
        );
        let diags = validate_substandard_feature_parity(temp.path());
        let diag = diags
            .iter()
            .find(|d| d.code == error_codes::DI_SUBSTANDARD_FEATURE_MISMATCH)
            .expect("expected DI_SUBSTANDARD_FEATURE_MISMATCH");
        assert_eq!(diag.severity, crate::Severity::Error);
        assert!(
            diag.message.contains("ZZ99"),
            "diagnostic should name the orphan feature: {}",
            diag.message
        );
    }

    #[test]
    fn test_feature_parity_skips_separate_crate_substandards() {
        // Separate-crate substandards (each with its own Cargo.toml) are not
        // feature-gated, so parity is a no-op for them regardless of whether the
        // parent has a [features] block.
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"apss-v1-0009-legacy\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        let sub = temp.path().join("substandards/OLD01-thing");
        std::fs::create_dir_all(&sub).unwrap();
        // Has its own Cargo.toml: the separate-crate pattern, not feature-module.
        std::fs::write(
            sub.join("Cargo.toml"),
            "[package]\nname = \"apss-old01-thing\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        std::fs::write(
            sub.join("substandard.toml"),
            "schema = \"aps.substandard/v1\"\n\n[substandard]\nid = \"APS-V1-0009.OL01\"\n",
        )
        .unwrap();
        let diags = validate_substandard_feature_parity(temp.path());
        assert!(
            diags.is_empty(),
            "separate-crate substandards should make parity a no-op: {diags}"
        );
    }

    #[test]
    fn test_feature_parity_merged_substandard_without_features_table_fails() {
        // The airtight case: a merged substandard (no Cargo.toml) exists but the
        // parent has no [features] table at all. This must fail, not silently
        // pass, because the substandard module would never be enabled.
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"apss-v1-0042-thing\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        let sub = temp.path().join("substandards/XX01-widget");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(
            sub.join("substandard.toml"),
            "schema = \"aps.substandard/v1\"\n\n[substandard]\nid = \"APS-V1-0042.XX01\"\nname = \"Widget\"\nslug = \"widget\"\nversion = \"0.1.0\"\nparent_id = \"APS-V1-0042\"\nparent_major = \"0\"\n\n[ownership]\nmaintainers = [\"test\"]\n",
        )
        .unwrap();
        let diags = validate_substandard_feature_parity(temp.path());
        assert!(
            diags
                .iter()
                .any(|d| d.code == error_codes::DI_SUBSTANDARD_FEATURE_MISMATCH
                    && d.message.contains("XX01")),
            "a merged substandard with no [features] table must fail: {diags}"
        );
    }

    #[test]
    fn test_feature_parity_skips_without_substandards_dir() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"apss-v1-0003-fitness\"\nversion = \"1.0.0\"\n",
        )
        .unwrap();
        let diags = validate_substandard_feature_parity(temp.path());
        assert!(diags.is_empty(), "no substandards dir should be a no-op");
    }

    #[test]
    fn test_validate_build_dir_missing_is_error() {
        let temp = tempfile::tempdir().unwrap();

        // Create lockfile + binary but no build dir
        let lockfile = crate::lockfile::Lockfile::new("0.1.0".to_string());
        crate::lockfile::write_lockfile(
            &temp.path().join(crate::lockfile::LOCKFILE_FILENAME),
            &lockfile,
        )
        .unwrap();

        let bin_dir = temp.path().join(BIN_DIR);
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(bin_dir.join(BIN_NAME), "fake binary").unwrap();

        let diags = validate_installation(temp.path());
        let build_diag = diags
            .iter()
            .find(|d| d.code == error_codes::DI_BUILD_DIR_MISSING)
            .expect("expected DI_BUILD_DIR_MISSING diagnostic");
        assert!(
            build_diag.severity == crate::Severity::Error,
            "DI_BUILD_DIR_MISSING should be error severity"
        );
    }
}
