//! Configuration types for TODO/FIXME tracker.
//!
//! This module defines the configuration schema and provides
//! defaults for the tracker.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration schema identifier
pub const CONFIG_SCHEMA: &str = "aps.todo-tracker-config/v1";

/// Default tags to scan for
pub const DEFAULT_TAGS: &[&str] = &["TODO", "FIXME"];

/// Default enforcement level
pub const DEFAULT_ENFORCEMENT: &str = "warn";

/// Default cache TTL in hours
pub const DEFAULT_CACHE_TTL: u32 = 24;

/// Default max file size in MB
pub const DEFAULT_MAX_FILE_SIZE: u32 = 10;

/// Tracker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerConfig {
    /// Schema identifier
    #[serde(default = "default_schema")]
    pub schema: String,

    /// Tracker settings
    #[serde(default)]
    pub tracker: TrackerSettings,

    /// Enforcement settings
    #[serde(default)]
    pub enforcement: EnforcementSettings,

    /// GitHub settings
    #[serde(default)]
    pub github: GitHubSettings,

    /// Scan settings
    #[serde(default)]
    pub scan: ScanSettings,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            schema: CONFIG_SCHEMA.to_string(),
            tracker: TrackerSettings::default(),
            enforcement: EnforcementSettings::default(),
            github: GitHubSettings::default(),
            scan: ScanSettings::default(),
        }
    }
}

impl TrackerConfig {
    /// Load configuration from a TOML file
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        toml::from_str(&content).map_err(|e| ConfigError::Parse {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Try to load configuration from default location (.todo-tracker.toml)
    /// Returns default config if file doesn't exist
    pub fn load_or_default(repo_root: &Path) -> Self {
        let config_path = repo_root.join(".todo-tracker.toml");
        if config_path.exists() {
            Self::from_file(&config_path).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Check if a tag requires an issue reference
    pub fn requires_issue(&self, tag: &str) -> bool {
        self.tracker.require_issue_tags.contains(&tag.to_string())
    }
}

fn default_schema() -> String {
    CONFIG_SCHEMA.to_string()
}

/// Tracker-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerSettings {
    /// Tags to scan for (e.g., TODO, FIXME, HACK)
    #[serde(default = "default_tags")]
    pub tags: Vec<String>,

    /// Which tags require issue references
    #[serde(default = "default_require_issue_tags")]
    pub require_issue_tags: Vec<String>,
}

impl Default for TrackerSettings {
    fn default() -> Self {
        Self {
            tags: default_tags(),
            require_issue_tags: default_require_issue_tags(),
        }
    }
}

fn default_tags() -> Vec<String> {
    DEFAULT_TAGS.iter().map(|s| s.to_string()).collect()
}

fn default_require_issue_tags() -> Vec<String> {
    DEFAULT_TAGS.iter().map(|s| s.to_string()).collect()
}

/// Enforcement level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EnforcementLevel {
    /// No enforcement, informational only
    Off,
    /// Report violations as warnings, exit code 0
    #[default]
    Warn,
    /// Report violations as errors, exit code 1
    Error,
}

impl std::fmt::Display for EnforcementLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnforcementLevel::Off => write!(f, "off"),
            EnforcementLevel::Warn => write!(f, "warn"),
            EnforcementLevel::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for EnforcementLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(EnforcementLevel::Off),
            "warn" => Ok(EnforcementLevel::Warn),
            "error" => Ok(EnforcementLevel::Error),
            _ => Err(format!("Invalid enforcement level: {s}")),
        }
    }
}

/// Enforcement settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementSettings {
    /// How to handle missing issue references
    #[serde(default = "default_enforcement")]
    pub missing_issue: EnforcementLevel,

    /// How to handle malformed format
    #[serde(default = "default_enforcement")]
    pub invalid_format: EnforcementLevel,

    /// How to handle closed issues (requires GitHub validation)
    #[serde(default)]
    pub closed_issue: EnforcementLevel,
}

impl Default for EnforcementSettings {
    fn default() -> Self {
        Self {
            missing_issue: EnforcementLevel::Warn,
            invalid_format: EnforcementLevel::Warn,
            closed_issue: EnforcementLevel::Off,
        }
    }
}

fn default_enforcement() -> EnforcementLevel {
    EnforcementLevel::Warn
}

/// GitHub API settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubSettings {
    /// Enable GitHub API validation
    #[serde(default)]
    pub enabled: bool,

    /// Repository (auto-detected from .git/config if not specified)
    pub repo: Option<String>,

    /// Environment variable for token
    #[serde(default = "default_token_env")]
    pub token_env: String,

    /// Cache validation results (hours)
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u32,
}

impl Default for GitHubSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            repo: None,
            token_env: "GITHUB_TOKEN".to_string(),
            cache_ttl: DEFAULT_CACHE_TTL,
        }
    }
}

fn default_token_env() -> String {
    "GITHUB_TOKEN".to_string()
}

fn default_cache_ttl() -> u32 {
    DEFAULT_CACHE_TTL
}

/// Scan settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSettings {
    /// Paths to include (glob patterns)
    #[serde(default = "default_include")]
    pub include: Vec<String>,

    /// Paths to exclude (glob patterns)
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,

    /// File extensions to scan (empty = all text files)
    #[serde(default)]
    pub extensions: Vec<String>,

    /// Maximum file size to scan (MB)
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u32,
}

impl Default for ScanSettings {
    fn default() -> Self {
        Self {
            include: default_include(),
            exclude: default_exclude(),
            extensions: vec![],
            max_file_size: DEFAULT_MAX_FILE_SIZE,
        }
    }
}

fn default_include() -> Vec<String> {
    vec!["**/*".to_string()]
}

fn default_exclude() -> Vec<String> {
    vec![
        "target/**".to_string(),
        "node_modules/**".to_string(),
        ".git/**".to_string(),
        "dist/**".to_string(),
        "build/**".to_string(),
        ".todo-tracker/**".to_string(),
    ]
}

fn default_max_file_size() -> u32 {
    DEFAULT_MAX_FILE_SIZE
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// IO error reading config file
    #[error("Failed to read config file {path:?}: {source}")]
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    /// TOML parse error
    #[error("Failed to parse config file {path:?}: {source}")]
    Parse {
        path: std::path::PathBuf,
        source: toml::de::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TrackerConfig::default();

        assert_eq!(config.schema, CONFIG_SCHEMA);
        assert_eq!(config.tracker.tags, vec!["TODO", "FIXME"]);
        assert_eq!(config.enforcement.missing_issue, EnforcementLevel::Warn);
        assert!(!config.github.enabled);
        assert_eq!(config.scan.max_file_size, 10);
    }

    #[test]
    fn test_requires_issue() {
        let config = TrackerConfig::default();

        assert!(config.requires_issue("TODO"));
        assert!(config.requires_issue("FIXME"));
        assert!(!config.requires_issue("HACK"));
    }

    #[test]
    fn test_enforcement_level_from_str() {
        assert_eq!(
            "off".parse::<EnforcementLevel>().unwrap(),
            EnforcementLevel::Off
        );
        assert_eq!(
            "warn".parse::<EnforcementLevel>().unwrap(),
            EnforcementLevel::Warn
        );
        assert_eq!(
            "error".parse::<EnforcementLevel>().unwrap(),
            EnforcementLevel::Error
        );
        assert_eq!(
            "OFF".parse::<EnforcementLevel>().unwrap(),
            EnforcementLevel::Off
        );
        assert!("invalid".parse::<EnforcementLevel>().is_err());
    }

    #[test]
    fn test_enforcement_level_display() {
        assert_eq!(EnforcementLevel::Off.to_string(), "off");
        assert_eq!(EnforcementLevel::Warn.to_string(), "warn");
        assert_eq!(EnforcementLevel::Error.to_string(), "error");
    }

    #[test]
    fn test_config_serialization() {
        let config = TrackerConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: TrackerConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.schema, config.schema);
        assert_eq!(parsed.tracker.tags, config.tracker.tags);
    }
}
