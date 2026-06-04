//! Project configuration parsing for `apss.toml`.
//!
//! This module provides types and functions for reading consumer project
//! configuration files. An `apss.toml` at the root of a project declares
//! which APS standards the project implements, their version requirements,
//! and standard-specific configuration.
//!
//! See `APS-V1-0000.CF01` for the normative specification.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Schema identifier for project configuration files.
pub const PROJECT_SCHEMA: &str = "apss.project/v1";

/// Default config filename.
pub const CONFIG_FILENAME: &str = "apss.toml";

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur when parsing project configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to read the configuration file.
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to parse the TOML content.
    #[error("failed to parse {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },

    /// Configuration file not found.
    #[error("no apss.toml found (searched from {start_dir})")]
    NotFound { start_dir: PathBuf },

    /// Included config file does not exist.
    #[error("included config file not found: {path}")]
    IncludeNotFound { path: PathBuf },

    /// Included config file failed to parse as TOML.
    #[error("failed to parse included config {path}: {source}")]
    IncludeParse {
        path: PathBuf,
        source: Box<toml::de::Error>,
    },
}

// ============================================================================
// Configuration Types
// ============================================================================

/// Parsed `apss.toml` project configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectConfig {
    /// Schema identifier. MUST be `"apss.project/v1"`.
    pub schema: String,

    /// Project identity.
    pub project: ProjectInfo,

    /// Declared standards. Keys are slugs used for CLI dispatch.
    #[serde(default)]
    pub standards: BTreeMap<String, StandardEntry>,

    /// Workspace configuration for monorepos.
    #[serde(default)]
    pub workspace: Option<WorkspaceConfig>,

    /// Tool configuration controlling APSS CLI behavior. This is the
    /// *raw* parse-time view: fields are `Option<T>` so downstream merge
    /// can distinguish "omitted" from "explicitly set to default". The
    /// resolved, defaulted form lives in `ResolvedProjectConfig::tool`
    /// as [`ToolConfig`].
    #[serde(default)]
    pub tool: Option<RawToolConfig>,
}

/// Project identity information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectInfo {
    /// Human-readable project name.
    pub name: String,

    /// APSS major version. Currently only `"v1"`.
    pub apss_version: String,
}

/// A declared standard with version requirement and configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StandardEntry {
    /// Standard ID (e.g., `"APS-V1-0001"`).
    pub id: String,

    /// Semver version requirement (Cargo-style, e.g., `">=1.0.0, <2.0.0"`).
    pub version: String,

    /// Whether this standard is enabled. Default: `true`.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Enabled substandard profile codes (e.g., `["RS01", "CI01"]`).
    /// If omitted, all substandards are enabled.
    #[serde(default)]
    pub substandards: Option<Vec<String>>,

    /// Standard-specific configuration. Opaque to CF01; validated by
    /// each standard's `StandardConfig` implementation.
    #[serde(default = "default_empty_table")]
    pub config: toml::Value,
}

/// Workspace configuration for monorepo support.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceConfig {
    /// Glob patterns for child package directories that may have their own `apss.toml`.
    pub members: Vec<String>,

    /// Glob patterns to exclude from workspace discovery.
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Tool configuration controlling APSS CLI behavior.
///
/// Resolved form with all fields populated. Produced from [`RawToolConfig`]
/// by [`RawToolConfig::into_resolved`] or its defaulted shape via
/// [`ToolConfig::default`].
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolConfig {
    /// Directory for the composed binary. Default: `".apss/bin"`.
    #[serde(default = "default_bin_dir")]
    pub bin_dir: String,

    /// Registry URL for fetching standards. Default: `"https://crates.io"`.
    #[serde(default = "default_registry")]
    pub registry: String,

    /// Whether to use only cached crates. Default: `false`.
    #[serde(default)]
    pub offline: bool,

    /// Log level for APSS operations. Default: `"warn"`.
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

/// Raw, parse-time view of `[tool]` config. Every field is `Option<T>` so
/// the cascading merge in `resolution` can distinguish "omitted by the user"
/// from "explicitly set to the default value" — a distinction required for
/// child configs that intentionally re-set a boolean back to `false`.
///
/// This type is only used at the parse boundary; resolution always produces
/// a fully-populated [`ToolConfig`] via [`Self::into_resolved`].
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RawToolConfig {
    pub bin_dir: Option<String>,
    pub registry: Option<String>,
    pub offline: Option<bool>,
    pub log_level: Option<String>,
}

impl RawToolConfig {
    /// Fill in any unset field from its default and return a [`ToolConfig`].
    pub fn into_resolved(self) -> ToolConfig {
        ToolConfig {
            bin_dir: self.bin_dir.unwrap_or_else(default_bin_dir),
            registry: self.registry.unwrap_or_else(default_registry),
            offline: self.offline.unwrap_or(false),
            log_level: self.log_level.unwrap_or_else(default_log_level),
        }
    }
}

// ============================================================================
// Defaults
// ============================================================================

fn default_true() -> bool {
    true
}

fn default_empty_table() -> toml::Value {
    toml::Value::Table(toml::map::Map::new())
}

fn default_bin_dir() -> String {
    ".apss/bin".to_string()
}

fn default_registry() -> String {
    "https://crates.io".to_string()
}

fn default_log_level() -> String {
    "warn".to_string()
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            bin_dir: default_bin_dir(),
            registry: default_registry(),
            offline: false,
            log_level: default_log_level(),
        }
    }
}

// ============================================================================
// Parsing Functions
// ============================================================================

/// Parse a project configuration from a file path.
///
/// After TOML deserialization, resolves `config = { include = "path" }` directives
/// by reading the referenced file and replacing the config value with its contents.
pub fn parse_project_config(path: &Path) -> Result<ProjectConfig, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.to_path_buf(),
        source: e,
    })?;

    let mut config: ProjectConfig = toml::from_str(&content).map_err(|e| ConfigError::Parse {
        path: path.to_path_buf(),
        source: e,
    })?;

    let config_dir = path.parent().unwrap_or(Path::new("."));
    resolve_includes(&mut config, config_dir)?;

    Ok(config)
}

/// Resolve `config = { include = "path" }` directives in standard entries.
///
/// A config value is treated as an include directive only when it is a table
/// with exactly one key `"include"` whose value is a string. This prevents
/// false positives when a standard's actual config has an `include` field
/// among other keys.
fn resolve_includes(config: &mut ProjectConfig, config_dir: &Path) -> Result<(), ConfigError> {
    for entry in config.standards.values_mut() {
        if let toml::Value::Table(ref table) = entry.config {
            if table.len() == 1 {
                if let Some(toml::Value::String(include_path)) = table.get("include") {
                    let resolved_path = config_dir.join(include_path);
                    let content = std::fs::read_to_string(&resolved_path).map_err(|e| {
                        if e.kind() == std::io::ErrorKind::NotFound {
                            ConfigError::IncludeNotFound {
                                path: resolved_path.clone(),
                            }
                        } else {
                            ConfigError::Io {
                                path: resolved_path.clone(),
                                source: e,
                            }
                        }
                    })?;
                    let parsed: toml::Value =
                        toml::from_str(&content).map_err(|e| ConfigError::IncludeParse {
                            path: resolved_path,
                            source: Box::new(e),
                        })?;
                    entry.config = parsed;
                }
            }
        }
    }
    Ok(())
}

/// Walk up from `start_dir` to find the nearest `apss.toml`.
///
/// Returns the path to the found config file, or `None` if no config
/// file is found before reaching the filesystem root.
pub fn find_project_config(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();
    loop {
        let candidate = current.join(CONFIG_FILENAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Walk up from `start_dir` to find the workspace root `apss.toml`.
///
/// The workspace root is the first `apss.toml` that contains a `[workspace]`
/// section. If no workspace root is found, returns the nearest `apss.toml`.
pub fn find_workspace_root(start_dir: &Path) -> Option<PathBuf> {
    let mut nearest: Option<PathBuf> = None;
    let mut current = start_dir.to_path_buf();

    loop {
        let candidate = current.join(CONFIG_FILENAME);
        if candidate.is_file() {
            if nearest.is_none() {
                nearest = Some(candidate.clone());
            }
            // Check if this one has a workspace section
            if let Ok(config) = parse_project_config(&candidate) {
                if config.workspace.is_some() {
                    return Some(candidate);
                }
            }
        }
        if !current.pop() {
            break;
        }
    }

    nearest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let toml_str = r#"
schema = "apss.project/v1"

[project]
name = "test-project"
apss_version = "v1"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.schema, PROJECT_SCHEMA);
        assert_eq!(config.project.name, "test-project");
        assert_eq!(config.project.apss_version, "v1");
        assert!(config.standards.is_empty());
        assert!(config.workspace.is_none());
    }

    #[test]
    fn test_parse_full_config() {
        let toml_str = r#"
schema = "apss.project/v1"

[project]
name = "my-service"
apss_version = "v1"

[standards.topology]
id = "APS-V1-0001"
version = ">=1.0.0, <2.0.0"
substandards = ["RS01", "CI01"]

[standards.topology.config]
output_dir = ".topology"
languages = ["rust", "python"]

[standards.fitness]
id = "APS-V1-0003"
version = ">=1.0.0"
enabled = false

[workspace]
members = ["packages/*", "services/*"]
exclude = ["packages/deprecated-*"]

[tool]
bin_dir = ".apss/bin"
offline = true
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.standards.len(), 2);

        let topology = &config.standards["topology"];
        assert_eq!(topology.id, "APS-V1-0001");
        assert!(topology.enabled);
        assert_eq!(topology.substandards.as_ref().unwrap(), &["RS01", "CI01"]);

        let fitness = &config.standards["fitness"];
        assert!(!fitness.enabled);

        let ws = config.workspace.unwrap();
        assert_eq!(ws.members, vec!["packages/*", "services/*"]);
        assert_eq!(ws.exclude, vec!["packages/deprecated-*"]);

        let tool = config.tool.unwrap();
        assert_eq!(tool.offline, Some(true));
    }

    #[test]
    fn test_default_standard_entry_values() {
        let toml_str = r#"
schema = "apss.project/v1"

[project]
name = "test"
apss_version = "v1"

[standards.topology]
id = "APS-V1-0001"
version = ">=1.0.0"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        let entry = &config.standards["topology"];
        assert!(entry.enabled); // default true
        assert!(entry.substandards.is_none()); // default none = all
        assert!(entry.config.is_table()); // default empty table
    }

    #[test]
    fn test_tool_config_defaults() {
        let config = ToolConfig::default();
        assert_eq!(config.bin_dir, ".apss/bin");
        assert_eq!(config.registry, "https://crates.io");
        assert!(!config.offline);
        assert_eq!(config.log_level, "warn");
    }

    #[test]
    fn test_find_project_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILENAME);
        std::fs::write(
            &config_path,
            r#"schema = "apss.project/v1"
[project]
name = "test"
apss_version = "v1"
"#,
        )
        .unwrap();

        // Find from the same directory
        let found = find_project_config(temp.path());
        assert_eq!(found, Some(config_path.clone()));

        // Find from a subdirectory
        let sub = temp.path().join("src").join("nested");
        std::fs::create_dir_all(&sub).unwrap();
        let found = find_project_config(&sub);
        assert_eq!(found, Some(config_path));
    }

    #[test]
    fn test_config_include_resolves() {
        let temp = tempfile::tempdir().unwrap();

        // Write included config file
        let config_dir = temp.path().join(".apss/config");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("topology.toml"),
            "output_dir = \".topology\"\nlanguages = [\"rust\"]\n",
        )
        .unwrap();

        // Write main config referencing it
        let config_path = temp.path().join(CONFIG_FILENAME);
        std::fs::write(
            &config_path,
            r#"schema = "apss.project/v1"
[project]
name = "test"
apss_version = "v1"

[standards.topology]
id = "APS-V1-0001"
version = ">=1.0.0"
config = { include = ".apss/config/topology.toml" }
"#,
        )
        .unwrap();

        let config = parse_project_config(&config_path).unwrap();
        let topo = &config.standards["topology"];
        assert_eq!(topo.config["output_dir"].as_str().unwrap(), ".topology");
        assert_eq!(topo.config["languages"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_config_include_not_found() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILENAME);
        std::fs::write(
            &config_path,
            r#"schema = "apss.project/v1"
[project]
name = "test"
apss_version = "v1"

[standards.topology]
id = "APS-V1-0001"
version = ">=1.0.0"
config = { include = "nonexistent.toml" }
"#,
        )
        .unwrap();

        let err = parse_project_config(&config_path).unwrap_err();
        assert!(matches!(err, ConfigError::IncludeNotFound { .. }));
    }

    #[test]
    fn test_config_include_bad_toml() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("bad.toml"), "not valid { toml").unwrap();

        let config_path = temp.path().join(CONFIG_FILENAME);
        std::fs::write(
            &config_path,
            r#"schema = "apss.project/v1"
[project]
name = "test"
apss_version = "v1"

[standards.topology]
id = "APS-V1-0001"
version = ">=1.0.0"
config = { include = "bad.toml" }
"#,
        )
        .unwrap();

        let err = parse_project_config(&config_path).unwrap_err();
        assert!(matches!(err, ConfigError::IncludeParse { .. }));
    }

    #[test]
    fn test_config_multi_key_table_not_treated_as_include() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join(CONFIG_FILENAME);
        std::fs::write(
            &config_path,
            r#"schema = "apss.project/v1"
[project]
name = "test"
apss_version = "v1"

[standards.topology]
id = "APS-V1-0001"
version = ">=1.0.0"

[standards.topology.config]
include = "some-value"
other_key = "other-value"
"#,
        )
        .unwrap();

        // Should parse without trying to resolve as include
        let config = parse_project_config(&config_path).unwrap();
        let topo = &config.standards["topology"];
        assert_eq!(topo.config["include"].as_str().unwrap(), "some-value");
        assert_eq!(topo.config["other_key"].as_str().unwrap(), "other-value");
    }
}
