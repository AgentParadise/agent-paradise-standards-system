//! Consumer manifest types for APS adoption.
//!
//! Defines the schema for `.aps/manifest.toml` files that declare
//! which standards a consumer project adopts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Consumer manifest schema version.
pub const MANIFEST_SCHEMA: &str = "aps.manifest/v1";

/// Lock file schema version.
pub const LOCK_SCHEMA: &str = "aps.lock/v1";

/// Registry schema version.
pub const REGISTRY_SCHEMA: &str = "aps.registry/v1";

/// Root manifest structure (`.aps/manifest.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerManifest {
    /// Schema identifier (must be "aps.manifest/v1").
    pub schema: String,
    /// Project information.
    pub project: ProjectInfo,
    /// Standards this project adopts, keyed by slug.
    #[serde(default)]
    pub standards: HashMap<String, StandardRef>,
    /// Substandards to enable, keyed by "parent.profile".
    #[serde(default)]
    pub substandards: HashMap<String, SubstandardConfig>,
    /// Source configuration and overrides.
    #[serde(default)]
    pub sources: SourceConfig,
}

/// Project information in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Project name/identifier.
    pub name: String,
    /// APS major version (e.g., "v1").
    pub aps_version: String,
}

/// Reference to an adopted standard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardRef {
    /// Full standard ID (e.g., "EXP-V1-0001").
    pub id: String,
    /// Pinned version (semver).
    pub version: String,
    /// Source URI (e.g., "github:AgentParadise/agent-paradise-standards-system").
    pub source: String,
    /// Generated artifact paths (e.g., [".topology/"]).
    #[serde(default)]
    pub artifacts: Vec<String>,
}

/// Substandard configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstandardConfig {
    /// Whether this substandard is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Source configuration with optional overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceConfig {
    /// Source URI overrides for development/dogfooding.
    #[serde(default)]
    pub overrides: HashMap<String, SourceOverride>,
}

/// Source override for development or local paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SourceOverride {
    /// Git submodule in consumer repo.
    Submodule {
        /// Relative path to submodule.
        path: PathBuf,
    },
    /// GitHub release artifacts.
    Release {
        /// Optional URL override.
        url: Option<String>,
    },
    /// Local filesystem path.
    Local {
        /// Path to local standards repo.
        path: PathBuf,
    },
}

fn default_true() -> bool {
    true
}

// ============================================================================
// Lock File Types
// ============================================================================

/// Lock file for pinned package versions (`.aps/manifest.lock`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestLock {
    /// Schema identifier (must be "aps.lock/v1").
    pub schema: String,
    /// ISO8601 timestamp of generation.
    pub generated_at: String,
    /// Locked packages.
    #[serde(rename = "package", default)]
    pub packages: Vec<LockedPackage>,
}

/// A locked package entry with pinned version and checksum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    /// Package slug (e.g., "code-topology").
    pub slug: String,
    /// Full standard ID (e.g., "EXP-V1-0001").
    pub id: String,
    /// Pinned version.
    pub version: String,
    /// Source URI.
    pub source: String,
    /// SHA-256 checksum of the package tarball.
    pub checksum: String,
    /// Resolved download URL.
    pub resolved_url: String,
}

impl ManifestLock {
    /// Create a new empty lock file.
    pub fn new() -> Self {
        Self {
            schema: LOCK_SCHEMA.to_string(),
            generated_at: chrono_lite_now(),
            packages: Vec::new(),
        }
    }

    /// Find a locked package by slug.
    pub fn find_package(&self, slug: &str) -> Option<&LockedPackage> {
        self.packages.iter().find(|p| p.slug == slug)
    }

    /// Add or update a package in the lock file.
    pub fn upsert_package(&mut self, package: LockedPackage) {
        if let Some(existing) = self.packages.iter_mut().find(|p| p.slug == package.slug) {
            *existing = package;
        } else {
            self.packages.push(package);
        }
        self.generated_at = chrono_lite_now();
    }
}

impl Default for ManifestLock {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Registry Types (for GitHub releases)
// ============================================================================

/// Package registry published with GitHub releases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRegistry {
    /// Schema version.
    pub schema_version: String,
    /// ISO8601 timestamp of publication.
    pub published_at: String,
    /// Available standards.
    pub standards: Vec<RegistryStandard>,
}

/// A standard entry in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStandard {
    /// Package slug.
    pub slug: String,
    /// Full standard ID.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Available versions.
    pub versions: Vec<RegistryVersion>,
    /// Latest version.
    pub latest: String,
}

/// A version entry in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryVersion {
    /// Version string.
    pub version: String,
    /// Asset filename.
    pub asset: String,
    /// SHA-256 checksum.
    pub checksum: String,
    /// Size in bytes.
    pub size_bytes: u64,
    /// ISO8601 timestamp of publication.
    pub published_at: String,
}

impl PackageRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            schema_version: REGISTRY_SCHEMA.to_string(),
            published_at: chrono_lite_now(),
            standards: Vec::new(),
        }
    }

    /// Find a standard by slug.
    pub fn find_standard(&self, slug: &str) -> Option<&RegistryStandard> {
        self.standards.iter().find(|s| s.slug == slug)
    }

    /// Find a specific version of a standard.
    pub fn find_version(&self, slug: &str, version: &str) -> Option<&RegistryVersion> {
        self.find_standard(slug)?
            .versions
            .iter()
            .find(|v| v.version == version)
    }
}

impl Default for PackageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Agent Index Types
// ============================================================================

/// Auto-generated index for agent consumption (`.aps/index.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIndex {
    /// Schema version for the index format.
    pub schema_version: String,
    /// ISO8601 timestamp of generation.
    pub generated_at: String,
    /// Project name from manifest.
    pub project: String,
    /// APS major version.
    pub aps_version: String,
    /// Adopted standards with details.
    pub standards: Vec<IndexedStandard>,
    /// Enabled substandards.
    pub substandards: Vec<IndexedSubstandard>,
    /// Sum of all standard token estimates.
    pub total_tokens: u64,
}

/// Standard entry in the agent index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedStandard {
    /// Full standard ID.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Slug identifier.
    pub slug: String,
    /// Version string.
    pub version: String,
    /// Generated artifact paths.
    pub artifacts: Vec<String>,
    /// Available agent skills.
    pub skills: Vec<String>,
    /// Estimated context tokens.
    pub tokens: u64,
}

/// Substandard entry in the agent index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedSubstandard {
    /// Full substandard ID (e.g., "EXP-V1-0001.LANG01").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Parent standard slug.
    pub parent: String,
}

impl AgentIndex {
    /// Current schema version for the index.
    pub const SCHEMA_VERSION: &'static str = "1.0.0";

    /// Create a new empty index.
    pub fn new(project: &str, aps_version: &str) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            generated_at: chrono_lite_now(),
            project: project.to_string(),
            aps_version: aps_version.to_string(),
            standards: Vec::new(),
            substandards: Vec::new(),
            total_tokens: 0,
        }
    }
}

/// Simple ISO8601 timestamp without external chrono dependency.
fn chrono_lite_now() -> String {
    use std::time::SystemTime;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Calculate date/time components (simplified, assumes UTC)
    let days = now / 86400;
    let time_of_day = now % 86400;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate year, month, day from days since epoch (1970-01-01)
    let (year, month, day) = days_to_ymd(days);

    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

/// Convert days since Unix epoch to year, month, day.
fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    // Simplified calculation for dates after 1970
    let mut remaining = days as i64;
    let mut year = 1970u32;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    let days_in_months: [i64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for &days_in_month in &days_in_months {
        if remaining < days_in_month {
            break;
        }
        remaining -= days_in_month;
        month += 1;
    }

    (year, month, remaining as u32 + 1)
}

/// Check if a year is a leap year.
fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let toml_str = r#"
schema = "aps.manifest/v1"

[project]
name = "test-project"
aps_version = "v1"

[standards.code-topology]
id = "EXP-V1-0001"
version = "0.1.0"
source = "github:AgentParadise/agent-paradise-standards-system"
artifacts = [".topology/"]

[substandards]
"code-topology.rust-adapter" = { enabled = true }
"#;

        let manifest: ConsumerManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.schema, MANIFEST_SCHEMA);
        assert_eq!(manifest.project.name, "test-project");
        assert_eq!(manifest.standards.len(), 1);
        assert!(manifest.standards.contains_key("code-topology"));

        let ct = &manifest.standards["code-topology"];
        assert_eq!(ct.id, "EXP-V1-0001");
        assert_eq!(ct.version, "0.1.0");
        assert_eq!(ct.artifacts, vec![".topology/"]);

        assert!(
            manifest
                .substandards
                .contains_key("code-topology.rust-adapter")
        );
        assert!(manifest.substandards["code-topology.rust-adapter"].enabled);
    }

    #[test]
    fn test_parse_manifest_with_overrides() {
        let toml_str = r#"
schema = "aps.manifest/v1"

[project]
name = "test-project"
aps_version = "v1"

[standards.code-topology]
id = "EXP-V1-0001"
version = "0.1.0"
source = "github:AgentParadise/agent-paradise-standards-system"

[sources.overrides]
"github:AgentParadise/agent-paradise-standards-system" = { type = "submodule", path = "lib/aps" }
"#;

        let manifest: ConsumerManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(manifest.sources.overrides.len(), 1);

        let key = "github:AgentParadise/agent-paradise-standards-system";
        let override_ = &manifest.sources.overrides[key];
        match override_ {
            SourceOverride::Submodule { path } => {
                assert_eq!(path, &PathBuf::from("lib/aps"));
            }
            _ => panic!("Expected Submodule override"),
        }
    }

    #[test]
    fn test_serialize_agent_index() {
        let index = AgentIndex {
            schema_version: "1.0.0".to_string(),
            generated_at: "2025-12-16T00:00:00Z".to_string(),
            project: "test".to_string(),
            aps_version: "v1".to_string(),
            standards: vec![IndexedStandard {
                id: "EXP-V1-0001".to_string(),
                name: "Code Topology".to_string(),
                slug: "code-topology".to_string(),
                version: "0.1.0".to_string(),
                artifacts: vec![".topology/".to_string()],
                skills: vec!["analyze-topology".to_string()],
                tokens: 2500,
            }],
            substandards: vec![],
            total_tokens: 2500,
        };

        let json = serde_json::to_string_pretty(&index).unwrap();
        assert!(json.contains("EXP-V1-0001"));
        assert!(json.contains("Code Topology"));
    }

    #[test]
    fn test_chrono_lite() {
        let timestamp = chrono_lite_now();
        // Should be in ISO8601 format: YYYY-MM-DDTHH:MM:SSZ
        assert!(timestamp.contains('T'));
        assert!(timestamp.ends_with('Z'));
        assert_eq!(timestamp.len(), 20);
    }

    #[test]
    fn test_days_to_ymd() {
        // 1970-01-01 = day 0
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
        // 2000-01-01 = 10957 days since epoch
        assert_eq!(days_to_ymd(10957), (2000, 1, 1));
        // 2025-12-16 = 20438 days since epoch
        assert_eq!(days_to_ymd(20438), (2025, 12, 16));
    }

    #[test]
    fn test_manifest_lock_creation() {
        let lock = ManifestLock::new();
        assert_eq!(lock.schema, LOCK_SCHEMA);
        assert!(lock.packages.is_empty());
    }

    #[test]
    fn test_manifest_lock_upsert() {
        let mut lock = ManifestLock::new();

        let pkg = LockedPackage {
            slug: "code-topology".to_string(),
            id: "EXP-V1-0001".to_string(),
            version: "0.1.0".to_string(),
            source: "github:AgentParadise/aps".to_string(),
            checksum: "sha256:abc123".to_string(),
            resolved_url: "https://example.com/pkg.tar.gz".to_string(),
        };

        lock.upsert_package(pkg.clone());
        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.find_package("code-topology").unwrap().version, "0.1.0");

        // Update version
        let mut updated = pkg;
        updated.version = "0.2.0".to_string();
        lock.upsert_package(updated);
        assert_eq!(lock.packages.len(), 1);
        assert_eq!(lock.find_package("code-topology").unwrap().version, "0.2.0");
    }

    #[test]
    fn test_parse_manifest_lock() {
        let toml_str = r#"
schema = "aps.lock/v1"
generated_at = "2025-12-17T00:00:00Z"

[[package]]
slug = "code-topology"
id = "EXP-V1-0001"
version = "0.1.0"
source = "github:AgentParadise/aps"
checksum = "sha256:abc123def456"
resolved_url = "https://github.com/.../aps-code-topology-0.1.0.tar.gz"
"#;

        let lock: ManifestLock = toml::from_str(toml_str).unwrap();
        assert_eq!(lock.schema, LOCK_SCHEMA);
        assert_eq!(lock.packages.len(), 1);

        let pkg = &lock.packages[0];
        assert_eq!(pkg.slug, "code-topology");
        assert_eq!(pkg.checksum, "sha256:abc123def456");
    }

    #[test]
    fn test_package_registry() {
        let mut registry = PackageRegistry::new();
        registry.standards.push(RegistryStandard {
            slug: "code-topology".to_string(),
            id: "EXP-V1-0001".to_string(),
            name: "Code Topology".to_string(),
            versions: vec![RegistryVersion {
                version: "0.1.0".to_string(),
                asset: "aps-code-topology-0.1.0.tar.gz".to_string(),
                checksum: "sha256:abc123".to_string(),
                size_bytes: 45678,
                published_at: "2025-12-15T00:00:00Z".to_string(),
            }],
            latest: "0.1.0".to_string(),
        });

        assert!(registry.find_standard("code-topology").is_some());
        assert!(registry.find_standard("unknown").is_none());
        assert!(registry.find_version("code-topology", "0.1.0").is_some());
        assert!(registry.find_version("code-topology", "0.2.0").is_none());
    }
}
