//! Filesystem discovery for APS packages.
//!
//! Provides utilities for walking directory trees and finding
//! standard/substandard/experiment packages.

use crate::metadata::{
    ExperimentMetadata, StandardMetadata, SubstandardMetadata, parse_experiment_metadata,
    parse_standard_metadata, parse_substandard_metadata,
};
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

impl PackageType {
    /// Get the expected metadata filename for this package type.
    pub fn metadata_filename(&self) -> &'static str {
        match self {
            PackageType::Standard => "standard.toml",
            PackageType::Substandard => "substandard.toml",
            PackageType::Experiment => "experiment.toml",
        }
    }
}

/// Parsed metadata from a package.
#[derive(Debug, Clone)]
pub enum PackageMetadata {
    Standard(StandardMetadata),
    Substandard(SubstandardMetadata),
    Experiment(ExperimentMetadata),
}

impl PackageMetadata {
    /// Get the package ID from metadata.
    pub fn id(&self) -> &str {
        match self {
            PackageMetadata::Standard(m) => &m.standard.id,
            PackageMetadata::Substandard(m) => &m.substandard.id,
            PackageMetadata::Experiment(m) => &m.experiment.id,
        }
    }

    /// Get the package name from metadata.
    pub fn name(&self) -> &str {
        match self {
            PackageMetadata::Standard(m) => &m.standard.name,
            PackageMetadata::Substandard(m) => &m.substandard.name,
            PackageMetadata::Experiment(m) => &m.experiment.name,
        }
    }

    /// Get the package version from metadata.
    pub fn version(&self) -> &str {
        match self {
            PackageMetadata::Standard(m) => &m.standard.version,
            PackageMetadata::Substandard(m) => &m.substandard.version,
            PackageMetadata::Experiment(m) => &m.experiment.version,
        }
    }
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
    /// Parsed metadata (lazily loaded).
    metadata: Option<PackageMetadata>,
}

impl DiscoveredPackage {
    /// Create a new discovered package.
    fn new(path: PathBuf, package_type: PackageType) -> Self {
        Self {
            path,
            package_type,
            metadata_file: package_type.metadata_filename().to_string(),
            metadata: None,
        }
    }

    /// Get the full path to the metadata file.
    pub fn metadata_path(&self) -> PathBuf {
        self.path.join(&self.metadata_file)
    }

    /// Load and cache the package metadata.
    pub fn load_metadata(&mut self) -> Result<&PackageMetadata, crate::metadata::MetadataError> {
        if self.metadata.is_some() {
            return Ok(self.metadata.as_ref().unwrap());
        }

        let metadata_path = self.metadata_path();
        let metadata = match self.package_type {
            PackageType::Standard => {
                PackageMetadata::Standard(parse_standard_metadata(&metadata_path)?)
            }
            PackageType::Substandard => {
                PackageMetadata::Substandard(parse_substandard_metadata(&metadata_path)?)
            }
            PackageType::Experiment => {
                PackageMetadata::Experiment(parse_experiment_metadata(&metadata_path)?)
            }
        };

        self.metadata = Some(metadata);
        Ok(self.metadata.as_ref().unwrap())
    }

    /// Get cached metadata if already loaded.
    pub fn metadata(&self) -> Option<&PackageMetadata> {
        self.metadata.as_ref()
    }

    /// Get the package ID (loads metadata if needed).
    pub fn id(&mut self) -> Result<String, crate::metadata::MetadataError> {
        Ok(self.load_metadata()?.id().to_string())
    }
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

        // Skip hidden directories
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with('.'))
        {
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
    let metadata_file = expected_type.metadata_filename();

    if path.join(metadata_file).exists() {
        return Some(DiscoveredPackage::new(path.to_path_buf(), expected_type));
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
        // Check if the directory name starts with the ID
        p.path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| name.starts_with(id))
    })
}

/// Find a package by ID with metadata validation.
///
/// This version actually parses the metadata to match by ID field.
pub fn find_package_by_id_exact(repo_root: &Path, id: &str) -> Option<DiscoveredPackage> {
    let mut packages = discover_v1_packages(repo_root);

    for package in &mut packages {
        if let Ok(pkg_id) = package.id() {
            if pkg_id == id {
                return Some(package.clone());
            }
        }
    }

    None
}

/// Count packages by type.
pub fn count_packages(repo_root: &Path) -> (usize, usize, usize) {
    let packages = discover_v1_packages(repo_root);

    let standards = packages
        .iter()
        .filter(|p| p.package_type == PackageType::Standard)
        .count();
    let substandards = packages
        .iter()
        .filter(|p| p.package_type == PackageType::Substandard)
        .count();
    let experiments = packages
        .iter()
        .filter(|p| p.package_type == PackageType::Experiment)
        .count();

    (standards, substandards, experiments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_package_type_equality() {
        assert_eq!(PackageType::Standard, PackageType::Standard);
        assert_ne!(PackageType::Standard, PackageType::Experiment);
    }

    #[test]
    fn test_package_type_metadata_filename() {
        assert_eq!(PackageType::Standard.metadata_filename(), "standard.toml");
        assert_eq!(
            PackageType::Substandard.metadata_filename(),
            "substandard.toml"
        );
        assert_eq!(
            PackageType::Experiment.metadata_filename(),
            "experiment.toml"
        );
    }

    #[test]
    fn test_discover_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let packages = discover_v1_packages(temp_dir.path());
        assert!(packages.is_empty());
    }

    #[test]
    fn test_discover_with_standards_dir() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create standards/v1/APS-V1-0001-test/
        let standard_dir = temp_dir.path().join("standards/v1/APS-V1-0001-test");
        fs::create_dir_all(&standard_dir).unwrap();

        // Create standard.toml
        let toml_content = r#"
schema = "aps.standard/v1"

[standard]
id = "APS-V1-0001"
name = "Test Standard"
slug = "test"
version = "1.0.0"
category = "governance"
status = "active"

[aps]
aps_major = "v1"

[ownership]
maintainers = ["Test"]
"#;
        fs::write(standard_dir.join("standard.toml"), toml_content).unwrap();

        let packages = discover_v1_packages(temp_dir.path());
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].package_type, PackageType::Standard);
    }

    #[test]
    fn test_skip_hidden_directories() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create standards/v1/.hidden/
        let hidden_dir = temp_dir.path().join("standards/v1/.hidden");
        fs::create_dir_all(&hidden_dir).unwrap();
        fs::write(hidden_dir.join("standard.toml"), "").unwrap();

        let packages = discover_v1_packages(temp_dir.path());
        assert!(packages.is_empty());
    }

    #[test]
    fn test_count_packages() {
        let temp_dir = tempfile::tempdir().unwrap();
        let (standards, substandards, experiments) = count_packages(temp_dir.path());
        assert_eq!(standards, 0);
        assert_eq!(substandards, 0);
        assert_eq!(experiments, 0);
    }
}
