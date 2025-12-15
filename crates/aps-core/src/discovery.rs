//! Filesystem discovery for APS packages.
//!
//! Provides utilities for walking directory trees and finding
//! standard/substandard/experiment packages.

use std::path::{Path, PathBuf};

/// The type of APS package discovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageType {
    /// An official standard (has `standard.toml`).
    Standard,
    /// A substandard (has `substandard.toml`).
    Substandard,
    /// An experimental standard (has `experiment.toml`).
    Experiment,
}

/// A discovered APS package.
#[derive(Debug, Clone)]
pub struct DiscoveredPackage {
    /// Path to the package root directory.
    pub path: PathBuf,
    /// Type of package.
    pub package_type: PackageType,
    /// The metadata file name (e.g., "standard.toml").
    pub metadata_file: String,
}

/// Discover all APS V1 packages in a repository.
///
/// Walks the `standards/v1/` and `standards-experimental/v1/` directories
/// looking for packages with valid metadata files.
///
/// # Arguments
///
/// * `repo_root` - Path to the repository root
///
/// # Returns
///
/// A vector of discovered packages.
pub fn discover_v1_packages(repo_root: &Path) -> Vec<DiscoveredPackage> {
    let mut packages = Vec::new();

    // Discover official standards
    let standards_dir = repo_root.join("standards/v1");
    if standards_dir.exists() {
        packages.extend(discover_in_directory(&standards_dir, PackageType::Standard));
    }

    // Discover experimental standards
    let experimental_dir = repo_root.join("standards-experimental/v1");
    if experimental_dir.exists() {
        packages.extend(discover_in_directory(
            &experimental_dir,
            PackageType::Experiment,
        ));
    }

    packages
}

/// Discover packages in a specific directory.
fn discover_in_directory(dir: &Path, expected_type: PackageType) -> Vec<DiscoveredPackage> {
    let mut packages = Vec::new();

    let Ok(entries) = std::fs::read_dir(dir) else {
        return packages;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // Check for metadata files
        if let Some(package) = detect_package(&path, expected_type) {
            packages.push(package);

            // If this is a standard, also check for substandards
            if expected_type == PackageType::Standard {
                let substandards_dir = path.join("substandards");
                if substandards_dir.exists() {
                    packages.extend(discover_in_directory(
                        &substandards_dir,
                        PackageType::Substandard,
                    ));
                }
            }
        }
    }

    packages
}

/// Detect if a directory is an APS package and what type.
fn detect_package(path: &Path, expected_type: PackageType) -> Option<DiscoveredPackage> {
    let metadata_file = match expected_type {
        PackageType::Standard => "standard.toml",
        PackageType::Substandard => "substandard.toml",
        PackageType::Experiment => "experiment.toml",
    };

    if path.join(metadata_file).exists() {
        return Some(DiscoveredPackage {
            path: path.to_path_buf(),
            package_type: expected_type,
            metadata_file: metadata_file.to_string(),
        });
    }

    None
}

/// Find a specific package by ID.
///
/// # Arguments
///
/// * `repo_root` - Path to the repository root
/// * `id` - The package ID (e.g., "APS-V1-0000" or "EXP-V1-0001")
///
/// # Returns
///
/// The discovered package if found.
pub fn find_package_by_id(repo_root: &Path, id: &str) -> Option<DiscoveredPackage> {
    discover_v1_packages(repo_root).into_iter().find(|p| {
        // TODO: Parse metadata and check ID
        // For now, check if the directory name contains the ID
        p.path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| name.starts_with(id))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_type_equality() {
        assert_eq!(PackageType::Standard, PackageType::Standard);
        assert_ne!(PackageType::Standard, PackageType::Experiment);
    }
}
