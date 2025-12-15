//! Derived views generator.
//!
//! Generates registry files and other derived views from the canonical
//! filesystem structure. These are NOT authoritative - the filesystem is
//! the source of truth.

use crate::discovery::{PackageType, discover_v1_packages};
use crate::metadata::{
    parse_experiment_metadata, parse_standard_metadata, parse_substandard_metadata,
};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// A registry entry for a V1 package.
#[derive(Debug, Clone, Serialize)]
pub struct RegistryEntry {
    /// Package ID.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Filesystem slug.
    pub slug: String,
    /// SemVer version.
    pub version: String,
    /// Category.
    pub category: String,
    /// Package type: standard, substandard, or experiment.
    pub package_type: String,
    /// Status: active, deprecated, or experimental.
    pub status: String,
    /// Relative path from repo root.
    pub path: String,
    /// Maintainers list.
    pub maintainers: Vec<String>,
}

/// The complete V1 registry.
#[derive(Debug, Clone, Serialize)]
pub struct Registry {
    /// Schema version for this registry format.
    pub schema: String,
    /// Generation timestamp.
    pub generated_at: String,
    /// All packages in the registry.
    pub packages: Vec<RegistryEntry>,
    /// Summary statistics.
    pub summary: RegistrySummary,
}

/// Summary statistics for the registry.
#[derive(Debug, Clone, Serialize)]
pub struct RegistrySummary {
    /// Total number of packages.
    pub total: usize,
    /// Number of official standards.
    pub standards: usize,
    /// Number of substandards.
    pub substandards: usize,
    /// Number of experiments.
    pub experiments: usize,
}

/// Errors that can occur during view generation.
#[derive(Debug, thiserror::Error)]
pub enum ViewsError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Metadata parsing error.
    #[error("metadata error: {0}")]
    Metadata(String),
}

/// Generate the V1 registry from the filesystem.
pub fn generate_registry(repo_root: &Path) -> Result<Registry, ViewsError> {
    let packages = discover_v1_packages(repo_root);
    let mut entries = Vec::new();

    for pkg in &packages {
        let entry = match pkg.package_type {
            PackageType::Standard => {
                let metadata_path = pkg.path.join("standard.toml");
                if !metadata_path.exists() {
                    continue;
                }
                let metadata = parse_standard_metadata(&metadata_path)
                    .map_err(|e| ViewsError::Metadata(e.to_string()))?;

                RegistryEntry {
                    id: metadata.standard.id,
                    name: metadata.standard.name,
                    slug: metadata.standard.slug,
                    version: metadata.standard.version,
                    category: metadata.standard.category,
                    package_type: "standard".to_string(),
                    status: metadata.standard.status,
                    path: relative_path(repo_root, &pkg.path),
                    maintainers: metadata.ownership.maintainers,
                }
            }
            PackageType::Substandard => {
                let metadata_path = pkg.path.join("substandard.toml");
                if !metadata_path.exists() {
                    continue;
                }
                let metadata = parse_substandard_metadata(&metadata_path)
                    .map_err(|e| ViewsError::Metadata(e.to_string()))?;

                RegistryEntry {
                    id: metadata.substandard.id,
                    name: metadata.substandard.name,
                    slug: metadata.substandard.slug,
                    version: metadata.substandard.version,
                    category: String::new(), // Substandards inherit from parent
                    package_type: "substandard".to_string(),
                    status: "active".to_string(),
                    path: relative_path(repo_root, &pkg.path),
                    maintainers: metadata.ownership.maintainers,
                }
            }
            PackageType::Experiment => {
                let metadata_path = pkg.path.join("experiment.toml");
                if !metadata_path.exists() {
                    continue;
                }
                let metadata = parse_experiment_metadata(&metadata_path)
                    .map_err(|e| ViewsError::Metadata(e.to_string()))?;

                RegistryEntry {
                    id: metadata.experiment.id,
                    name: metadata.experiment.name,
                    slug: metadata.experiment.slug,
                    version: metadata.experiment.version,
                    category: metadata.experiment.category,
                    package_type: "experiment".to_string(),
                    status: "experimental".to_string(),
                    path: relative_path(repo_root, &pkg.path),
                    maintainers: metadata.ownership.maintainers,
                }
            }
        };

        entries.push(entry);
    }

    // Sort by ID for deterministic output
    entries.sort_by(|a, b| a.id.cmp(&b.id));

    let standards = entries
        .iter()
        .filter(|e| e.package_type == "standard")
        .count();
    let substandards = entries
        .iter()
        .filter(|e| e.package_type == "substandard")
        .count();
    let experiments = entries
        .iter()
        .filter(|e| e.package_type == "experiment")
        .count();

    Ok(Registry {
        schema: "aps.registry/v1".to_string(),
        generated_at: crate::promotion::chrono_lite_date(),
        packages: entries.clone(),
        summary: RegistrySummary {
            total: entries.len(),
            standards,
            substandards,
            experiments,
        },
    })
}

/// Write the registry to a JSON file.
pub fn write_registry_json(registry: &Registry, output_path: &Path) -> Result<(), ViewsError> {
    let json = serde_json::to_string_pretty(registry)?;
    fs::write(output_path, json)?;
    Ok(())
}

/// Write a markdown index of all packages.
pub fn write_registry_markdown(registry: &Registry, output_path: &Path) -> Result<(), ViewsError> {
    let mut md = String::new();

    md.push_str("# APS V1 Standards Registry\n\n");
    md.push_str(&format!("_Generated: {}_\n\n", registry.generated_at));
    md.push_str("---\n\n");

    // Summary
    md.push_str("## Summary\n\n");
    md.push_str(&format!(
        "- **Total Packages**: {}\n",
        registry.summary.total
    ));
    md.push_str(&format!(
        "- **Standards**: {}\n",
        registry.summary.standards
    ));
    md.push_str(&format!(
        "- **Substandards**: {}\n",
        registry.summary.substandards
    ));
    md.push_str(&format!(
        "- **Experiments**: {}\n\n",
        registry.summary.experiments
    ));

    // Standards
    md.push_str("## Official Standards\n\n");
    md.push_str("| ID | Name | Version | Category | Status |\n");
    md.push_str("|----|------|---------|----------|--------|\n");

    for entry in registry
        .packages
        .iter()
        .filter(|e| e.package_type == "standard")
    {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            entry.id, entry.name, entry.version, entry.category, entry.status
        ));
    }

    md.push('\n');

    // Experiments
    if registry.summary.experiments > 0 {
        md.push_str("## Experimental Standards\n\n");
        md.push_str("| ID | Name | Version | Category |\n");
        md.push_str("|----|------|---------|----------|\n");

        for entry in registry
            .packages
            .iter()
            .filter(|e| e.package_type == "experiment")
        {
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                entry.id, entry.name, entry.version, entry.category
            ));
        }

        md.push('\n');
    }

    md.push_str("---\n\n");
    md.push_str("_This file is auto-generated. Do not edit directly._\n");

    fs::write(output_path, md)?;
    Ok(())
}

/// Generate all derived views.
pub fn generate_all_views(repo_root: &Path) -> Result<Vec<PathBuf>, ViewsError> {
    let registry = generate_registry(repo_root)?;
    let mut generated = Vec::new();

    // Create generated directory
    let generated_dir = repo_root.join("generated");
    fs::create_dir_all(&generated_dir)?;

    // Write registry.json
    let json_path = generated_dir.join("registry.json");
    write_registry_json(&registry, &json_path)?;
    generated.push(json_path);

    // Write INDEX.md
    let md_path = generated_dir.join("INDEX.md");
    write_registry_markdown(&registry, &md_path)?;
    generated.push(md_path);

    Ok(generated)
}

/// Get the relative path from repo root.
fn relative_path(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_registry_empty() {
        let temp_dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp_dir.path().join("standards/v1")).unwrap();
        fs::create_dir_all(temp_dir.path().join("standards-experimental/v1")).unwrap();

        let registry = generate_registry(temp_dir.path()).unwrap();

        assert_eq!(registry.packages.len(), 0);
        assert_eq!(registry.summary.total, 0);
    }

    #[test]
    fn test_registry_entry_serialization() {
        let entry = RegistryEntry {
            id: "APS-V1-0001".to_string(),
            name: "Test".to_string(),
            slug: "test".to_string(),
            version: "1.0.0".to_string(),
            category: "governance".to_string(),
            package_type: "standard".to_string(),
            status: "active".to_string(),
            path: "standards/v1/APS-V1-0001-test".to_string(),
            maintainers: vec!["Alice".to_string()],
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("APS-V1-0001"));
        assert!(json.contains("governance"));
    }
}
