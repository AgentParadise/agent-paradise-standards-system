//! Metadata parsing for APS packages.
//!
//! Provides types and parsing for `standard.toml`, `substandard.toml`,
//! and `experiment.toml` files.

use serde::Deserialize;
use std::path::Path;

/// Default value for backwards_compat field (true = no breaking changes).
fn default_backwards_compat() -> bool {
    true
}

/// Standard metadata from `standard.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct StandardMetadata {
    /// Schema version (e.g., "aps.standard/v1").
    pub schema: String,
    /// Standard-specific fields.
    pub standard: StandardFields,
    /// APS ecosystem fields.
    pub aps: ApsFields,
    /// Ownership information.
    pub ownership: OwnershipFields,
}

/// Core fields for a standard.
#[derive(Debug, Clone, Deserialize)]
pub struct StandardFields {
    /// Unique identifier (e.g., "APS-V1-0000").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Filesystem-safe slug.
    pub slug: String,
    /// SemVer version.
    pub version: String,
    /// Category: governance, technical, design, process, security.
    pub category: String,
    /// Status: active, deprecated, experimental.
    pub status: String,
    /// Whether this release maintains backward compatibility with the previous version.
    /// Set to `false` when introducing breaking changes (requires MAJOR version bump).
    #[serde(default = "default_backwards_compat")]
    pub backwards_compat: bool,
}

/// Substandard metadata from `substandard.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct SubstandardMetadata {
    /// Schema version.
    pub schema: String,
    /// Substandard-specific fields.
    pub substandard: SubstandardFields,
    /// Ownership information.
    pub ownership: OwnershipFields,
}

/// Core fields for a substandard.
#[derive(Debug, Clone, Deserialize)]
pub struct SubstandardFields {
    /// Qualified identifier (e.g., "APS-V1-0002.GH01").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Filesystem-safe slug.
    pub slug: String,
    /// SemVer version.
    pub version: String,
    /// Parent standard ID.
    pub parent_id: String,
    /// Parent major version alignment.
    pub parent_major: String,
    /// Whether this release maintains backward compatibility with the previous version.
    /// Set to `false` when introducing breaking changes (requires MAJOR version bump).
    #[serde(default = "default_backwards_compat")]
    pub backwards_compat: bool,
}

/// Experiment metadata from `experiment.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct ExperimentMetadata {
    /// Schema version.
    pub schema: String,
    /// Experiment-specific fields.
    pub experiment: ExperimentFields,
    /// APS ecosystem fields.
    pub aps: ApsFields,
    /// Ownership information.
    pub ownership: OwnershipFields,
    /// Promotion information (added after promotion).
    pub promotion: Option<PromotionFields>,
}

/// Core fields for an experiment.
#[derive(Debug, Clone, Deserialize)]
pub struct ExperimentFields {
    /// Unique identifier (e.g., "EXP-V1-0001").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Filesystem-safe slug.
    pub slug: String,
    /// SemVer version.
    pub version: String,
    /// Category.
    pub category: String,
    /// Whether this release maintains backward compatibility with the previous version.
    /// Set to `false` when introducing breaking changes.
    /// Note: Experiments (0.x.x) are exempt from MAJOR version requirements.
    #[serde(default = "default_backwards_compat")]
    pub backwards_compat: bool,
}

/// APS ecosystem fields.
#[derive(Debug, Clone, Deserialize)]
pub struct ApsFields {
    /// APS major version (e.g., "v1").
    pub aps_major: String,
    /// Whether backward compatibility is required within the major version.
    /// Standards with this set to `true` must not introduce breaking changes within V1.
    pub backwards_compatible_major_required: Option<bool>,
}

/// Ownership information.
#[derive(Debug, Clone, Deserialize)]
pub struct OwnershipFields {
    /// List of maintainers.
    pub maintainers: Vec<String>,
}

/// Promotion information for experiments.
#[derive(Debug, Clone, Deserialize)]
pub struct PromotionFields {
    /// Official standard ID after promotion.
    pub promoted_to: String,
    /// Date of promotion.
    pub promoted_at: String,
    /// Optional notes.
    pub notes: Option<String>,
}

/// Parse a `standard.toml` file.
pub fn parse_standard_metadata(path: &Path) -> Result<StandardMetadata, MetadataError> {
    let content = std::fs::read_to_string(path).map_err(|e| MetadataError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    toml::from_str(&content).map_err(|e| MetadataError::Parse {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Parse a `substandard.toml` file.
pub fn parse_substandard_metadata(path: &Path) -> Result<SubstandardMetadata, MetadataError> {
    let content = std::fs::read_to_string(path).map_err(|e| MetadataError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    toml::from_str(&content).map_err(|e| MetadataError::Parse {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Parse an `experiment.toml` file.
pub fn parse_experiment_metadata(path: &Path) -> Result<ExperimentMetadata, MetadataError> {
    let content = std::fs::read_to_string(path).map_err(|e| MetadataError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    toml::from_str(&content).map_err(|e| MetadataError::Parse {
        path: path.to_path_buf(),
        source: e,
    })
}

/// Errors that can occur when parsing metadata.
#[derive(Debug, thiserror::Error)]
pub enum MetadataError {
    /// IO error reading the file.
    #[error("failed to read {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    /// TOML parsing error.
    #[error("failed to parse {path}: {source}")]
    Parse {
        path: std::path::PathBuf,
        source: toml::de::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_toml() {
        let toml_content = r#"
schema = "aps.standard/v1"

[standard]
id = "APS-V1-0000"
name = "APS Meta-Standard"
slug = "meta"
version = "1.0.0"
category = "governance"
status = "active"

[aps]
aps_major = "v1"

[ownership]
maintainers = ["AgentParadise"]
"#;

        let metadata: StandardMetadata = toml::from_str(toml_content).unwrap();
        assert_eq!(metadata.standard.id, "APS-V1-0000");
        assert_eq!(metadata.standard.category, "governance");
    }
}
