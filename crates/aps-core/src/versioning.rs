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
        &format!("version = \"{}\"", old_version),
        &format!("version = \"{}\"", new_version),
    );
    fs::write(&metadata_file, updated)?;

    // Also update Cargo.toml if it exists
    let cargo_toml = pkg.path.join("Cargo.toml");
    if cargo_toml.exists() {
        let content = fs::read_to_string(&cargo_toml)?;
        let updated = content.replace(
            &format!("version = \"{}\"", old_version),
            &format!("version = \"{}\"", new_version),
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

    Ok(format!("{}.{}.{}", new_major, new_minor, new_patch))
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
}
