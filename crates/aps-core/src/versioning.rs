//! Version management for APS packages.
//!
//! Provides utilities for bumping and managing semantic versions.

use crate::discovery::{PackageType, discover_v1_packages};
use crate::metadata::{
    parse_experiment_metadata, parse_standard_metadata, parse_substandard_metadata,
};
use std::fs;
use std::path::Path;

/// Errors that can occur during versioning.
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    /// Package not found.
    #[error("package not found: {0}")]
    PackageNotFound(String),

    /// Invalid version format.
    #[error("invalid version format: {0}")]
    InvalidVersion(String),

    /// Backwards compatibility violation.
    #[error("backwards_compat: false requires MAJOR version > 0, but version is {0}")]
    BackwardsCompatViolation(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Metadata error.
    #[error("metadata error: {0}")]
    Metadata(String),
}

/// Which part of the version to bump.
#[derive(Debug, Clone, Copy)]
pub enum BumpPart {
    Major,
    Minor,
    Patch,
}

/// Result of a version bump operation.
#[derive(Debug, Clone)]
pub struct VersionBumpResult {
    /// Package ID.
    pub id: String,
    /// Previous version.
    pub old_version: String,
    /// New version.
    pub new_version: String,
}

/// Get the current version of a package.
pub fn get_version(repo_root: &Path, id: &str) -> Result<String, VersionError> {
    let packages = discover_v1_packages(repo_root);

    let pkg = packages
        .iter()
        .find(|p| {
            p.path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name.starts_with(id))
        })
        .ok_or_else(|| VersionError::PackageNotFound(id.to_string()))?;

    let version = match pkg.package_type {
        PackageType::Standard => {
            let metadata = parse_standard_metadata(&pkg.path.join("standard.toml"))
                .map_err(|e| VersionError::Metadata(e.to_string()))?;
            metadata.standard.version
        }
        PackageType::Substandard => {
            let metadata = parse_substandard_metadata(&pkg.path.join("substandard.toml"))
                .map_err(|e| VersionError::Metadata(e.to_string()))?;
            metadata.substandard.version
        }
        PackageType::Experiment => {
            let metadata = parse_experiment_metadata(&pkg.path.join("experiment.toml"))
                .map_err(|e| VersionError::Metadata(e.to_string()))?;
            metadata.experiment.version
        }
    };

    Ok(version)
}

/// Bump the version of a package.
pub fn bump_version(
    repo_root: &Path,
    id: &str,
    part: BumpPart,
) -> Result<VersionBumpResult, VersionError> {
    let packages = discover_v1_packages(repo_root);

    let pkg = packages
        .iter()
        .find(|p| {
            p.path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name.starts_with(id))
        })
        .ok_or_else(|| VersionError::PackageNotFound(id.to_string()))?;

    let (metadata_file, old_version) = match pkg.package_type {
        PackageType::Standard => {
            let path = pkg.path.join("standard.toml");
            let metadata = parse_standard_metadata(&path)
                .map_err(|e| VersionError::Metadata(e.to_string()))?;
            (path, metadata.standard.version)
        }
        PackageType::Substandard => {
            let path = pkg.path.join("substandard.toml");
            let metadata = parse_substandard_metadata(&path)
                .map_err(|e| VersionError::Metadata(e.to_string()))?;
            (path, metadata.substandard.version)
        }
        PackageType::Experiment => {
            let path = pkg.path.join("experiment.toml");
            let metadata = parse_experiment_metadata(&path)
                .map_err(|e| VersionError::Metadata(e.to_string()))?;
            (path, metadata.experiment.version)
        }
    };

    let new_version = bump_semver(&old_version, part)?;

    // Update the metadata file
    let content = fs::read_to_string(&metadata_file)?;
    let updated = content.replace(
        &format!("version = \"{old_version}\""),
        &format!("version = \"{new_version}\""),
    );
    fs::write(&metadata_file, updated)?;

    // Also update Cargo.toml if it exists
    let cargo_toml = pkg.path.join("Cargo.toml");
    if cargo_toml.exists() {
        let content = fs::read_to_string(&cargo_toml)?;
        let updated = content.replace(
            &format!("version = \"{old_version}\""),
            &format!("version = \"{new_version}\""),
        );
        fs::write(&cargo_toml, updated)?;
    }

    Ok(VersionBumpResult {
        id: id.to_string(),
        old_version,
        new_version,
    })
}

/// Bump a semver version string.
fn bump_semver(version: &str, part: BumpPart) -> Result<String, VersionError> {
    let parts: Vec<&str> = version.split('.').collect();

    if parts.len() < 2 || parts.len() > 3 {
        return Err(VersionError::InvalidVersion(version.to_string()));
    }

    let major: u32 = parts[0]
        .parse()
        .map_err(|_| VersionError::InvalidVersion(version.to_string()))?;
    let minor: u32 = parts[1]
        .parse()
        .map_err(|_| VersionError::InvalidVersion(version.to_string()))?;
    let patch: u32 = if parts.len() == 3 {
        parts[2]
            .parse()
            .map_err(|_| VersionError::InvalidVersion(version.to_string()))?
    } else {
        0
    };

    let (new_major, new_minor, new_patch) = match part {
        BumpPart::Major => (major + 1, 0, 0),
        BumpPart::Minor => (major, minor + 1, 0),
        BumpPart::Patch => (major, minor, patch + 1),
    };

    Ok(format!("{new_major}.{new_minor}.{new_patch}"))
}

// ============================================================================
// Version Validation
// ============================================================================

/// Parse a version string into (major, minor, patch) components.
///
/// Returns None if the version is not valid semver (must be MAJOR.MINOR.PATCH).
pub fn parse_semver(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = version.split('.').collect();

    if parts.len() != 3 {
        return None;
    }

    let major: u32 = parts[0].parse().ok()?;
    let minor: u32 = parts[1].parse().ok()?;
    let patch: u32 = parts[2].parse().ok()?;

    Some((major, minor, patch))
}

/// Check if a version string is valid semver format.
pub fn is_valid_semver(version: &str) -> bool {
    parse_semver(version).is_some()
}

/// Validate that backwards_compat flag aligns with version rules.
///
/// Rules:
/// - If `backwards_compat` is false, MAJOR version must be > 0 (for non-experiments)
/// - Experiments (0.x.x) are exempt from this rule
///
/// Returns Ok(()) if valid, Err with explanation if invalid.
pub fn validate_backwards_compat(
    version: &str,
    backwards_compat: bool,
    is_experiment: bool,
) -> Result<(), VersionError> {
    // If backwards_compat is true, no special requirements
    if backwards_compat {
        return Ok(());
    }

    // backwards_compat is false - check version rules
    let (major, _, _) =
        parse_semver(version).ok_or_else(|| VersionError::InvalidVersion(version.to_string()))?;

    // Experiments can use 0.x.x freely even with backwards_compat: false
    if is_experiment {
        return Ok(());
    }

    // For non-experiments, backwards_compat: false requires MAJOR > 0
    if major == 0 {
        return Err(VersionError::BackwardsCompatViolation(version.to_string()));
    }

    Ok(())
}

/// Result of version validation.
#[derive(Debug, Clone)]
pub struct VersionValidation {
    /// Whether the version format is valid.
    pub is_valid_format: bool,
    /// Parsed version components (if valid).
    pub components: Option<(u32, u32, u32)>,
    /// Whether this is a pre-1.0 (experimental) version.
    pub is_prerelease: bool,
    /// Validation errors, if any.
    pub errors: Vec<String>,
}

/// Validate a version string comprehensively.
pub fn validate_version(
    version: &str,
    backwards_compat: bool,
    is_experiment: bool,
) -> VersionValidation {
    let mut validation = VersionValidation {
        is_valid_format: false,
        components: None,
        is_prerelease: false,
        errors: Vec::new(),
    };

    // Check format
    match parse_semver(version) {
        Some((major, minor, patch)) => {
            validation.is_valid_format = true;
            validation.components = Some((major, minor, patch));
            validation.is_prerelease = major == 0;
        }
        None => {
            validation.errors.push(format!(
                "INVALID_VERSION_FORMAT: '{version}' is not valid semver (expected MAJOR.MINOR.PATCH)"
            ));
            return validation;
        }
    }

    // Check backwards_compat alignment
    if let Err(e) = validate_backwards_compat(version, backwards_compat, is_experiment) {
        validation
            .errors
            .push(format!("BACKWARDS_COMPAT_VIOLATION: {e}"));
    }

    validation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_semver_patch() {
        assert_eq!(bump_semver("1.0.0", BumpPart::Patch).unwrap(), "1.0.1");
        assert_eq!(bump_semver("1.2.3", BumpPart::Patch).unwrap(), "1.2.4");
    }

    #[test]
    fn test_bump_semver_minor() {
        assert_eq!(bump_semver("1.0.0", BumpPart::Minor).unwrap(), "1.1.0");
        assert_eq!(bump_semver("1.2.3", BumpPart::Minor).unwrap(), "1.3.0");
    }

    #[test]
    fn test_bump_semver_major() {
        assert_eq!(bump_semver("1.0.0", BumpPart::Major).unwrap(), "2.0.0");
        assert_eq!(bump_semver("1.2.3", BumpPart::Major).unwrap(), "2.0.0");
    }

    #[test]
    fn test_bump_semver_two_part() {
        assert_eq!(bump_semver("1.0", BumpPart::Patch).unwrap(), "1.0.1");
        assert_eq!(bump_semver("1.0", BumpPart::Minor).unwrap(), "1.1.0");
    }

    #[test]
    fn test_invalid_version() {
        assert!(bump_semver("invalid", BumpPart::Patch).is_err());
        assert!(bump_semver("1", BumpPart::Patch).is_err());
    }

    #[test]
    fn test_parse_semver() {
        assert_eq!(parse_semver("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_semver("0.1.0"), Some((0, 1, 0)));
        assert_eq!(parse_semver("10.20.30"), Some((10, 20, 30)));
        // Two-part versions are rejected (spec requires MAJOR.MINOR.PATCH)
        assert_eq!(parse_semver("1.0"), None);
        assert_eq!(parse_semver("invalid"), None);
        assert_eq!(parse_semver("1"), None);
        assert_eq!(parse_semver("1.2.3.4"), None);
    }

    #[test]
    fn test_is_valid_semver() {
        assert!(is_valid_semver("1.0.0"));
        assert!(is_valid_semver("0.1.0"));
        assert!(is_valid_semver("10.20.30"));
        assert!(!is_valid_semver("invalid"));
        assert!(!is_valid_semver("v1.0.0")); // No 'v' prefix
    }

    #[test]
    fn test_backwards_compat_true() {
        // backwards_compat: true is always valid
        assert!(validate_backwards_compat("0.1.0", true, false).is_ok());
        assert!(validate_backwards_compat("1.0.0", true, false).is_ok());
    }

    #[test]
    fn test_backwards_compat_false_experiment() {
        // Experiments can use 0.x.x with backwards_compat: false
        assert!(validate_backwards_compat("0.1.0", false, true).is_ok());
        assert!(validate_backwards_compat("0.2.0", false, true).is_ok());
    }

    #[test]
    fn test_backwards_compat_false_standard() {
        // Non-experiments need MAJOR > 0 with backwards_compat: false
        assert!(validate_backwards_compat("1.0.0", false, false).is_ok());
        assert!(validate_backwards_compat("2.0.0", false, false).is_ok());

        // 0.x.x with backwards_compat: false is invalid for non-experiments
        assert!(matches!(
            validate_backwards_compat("0.1.0", false, false),
            Err(VersionError::BackwardsCompatViolation(_))
        ));
    }

    #[test]
    fn test_validate_version_comprehensive() {
        // Valid version with backwards_compat: true
        let v = validate_version("1.2.3", true, false);
        assert!(v.is_valid_format);
        assert_eq!(v.components, Some((1, 2, 3)));
        assert!(!v.is_prerelease);
        assert!(v.errors.is_empty());

        // Pre-release version
        let v = validate_version("0.1.0", true, true);
        assert!(v.is_valid_format);
        assert!(v.is_prerelease);
        assert!(v.errors.is_empty());

        // Invalid format
        let v = validate_version("invalid", true, false);
        assert!(!v.is_valid_format);
        assert!(!v.errors.is_empty());

        // Backwards compat violation
        let v = validate_version("0.1.0", false, false);
        assert!(v.is_valid_format);
        assert!(!v.errors.is_empty());
        assert!(v.errors[0].contains("BACKWARDS_COMPAT_VIOLATION"));
    }
}
