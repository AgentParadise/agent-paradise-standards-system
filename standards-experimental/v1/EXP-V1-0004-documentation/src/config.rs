//! Configuration deserialization for `.apss/config.toml`.
//!
//! All fields use `#[serde(default)]` so a missing config file or partial
//! config produces sensible defaults (zero-config works out of the box).

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Top-level APSS configuration file.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ApssConfig {
    #[serde(default)]
    pub docs: DocsConfig,
}

/// The `[docs]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct DocsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_docs_root")]
    pub root: String,
    #[serde(default)]
    pub index: IndexConfig,
    #[serde(default)]
    pub context_files: ContextFilesConfig,
    #[serde(default)]
    pub adr: AdrConfig,
    #[serde(default)]
    pub readme: ReadmeConfig,
    #[serde(default)]
    pub root_context: RootContextConfig,
}

impl Default for DocsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            root: default_docs_root(),
            index: IndexConfig::default(),
            context_files: ContextFilesConfig::default(),
            adr: AdrConfig::default(),
            readme: ReadmeConfig::default(),
            root_context: RootContextConfig::default(),
        }
    }
}

/// The `[docs.index]` section — controls `## Index` generation in README.md files.
#[derive(Debug, Clone, Deserialize)]
pub struct IndexConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub auto_generate: bool,
    #[serde(default = "default_frontmatter_fields")]
    pub frontmatter_fields: Vec<String>,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_generate: true,
            frontmatter_fields: default_frontmatter_fields(),
        }
    }
}

/// The `[docs.context_files]` section — CLAUDE.md and AGENTS.md per directory.
#[derive(Debug, Clone, Deserialize)]
pub struct ContextFilesConfig {
    #[serde(default = "default_true")]
    pub require_claude_md: bool,
    #[serde(default = "default_true")]
    pub require_agents_md: bool,
}

impl Default for ContextFilesConfig {
    fn default() -> Self {
        Self {
            require_claude_md: true,
            require_agents_md: true,
        }
    }
}

/// The `[docs.adr]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct AdrConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_adr_directory")]
    pub directory: String,
    #[serde(default = "default_adr_naming_pattern")]
    pub naming_pattern: String,
    /// Required ADR keyword names (e.g., `["security", "testing"]`).
    /// For each keyword, at least one file matching `ADR-\d+-<keyword>\.md` must exist.
    #[serde(default)]
    pub required_adr_keywords: Vec<String>,
    #[serde(default = "default_true")]
    pub backlinking: bool,
}

impl Default for AdrConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            directory: default_adr_directory(),
            naming_pattern: default_adr_naming_pattern(),
            required_adr_keywords: Vec::new(),
            backlinking: true,
        }
    }
}

/// The `[docs.readme]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct ReadmeConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_max_depth")]
    pub max_depth: i32,
    #[serde(default = "default_exclude_dirs")]
    pub exclude_dirs: Vec<String>,
}

impl Default for ReadmeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_depth: default_max_depth(),
            exclude_dirs: default_exclude_dirs(),
        }
    }
}

/// The `[docs.root_context]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct RootContextConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_docs_reference_pattern")]
    pub docs_reference_pattern: String,
}

impl Default for RootContextConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            docs_reference_pattern: default_docs_reference_pattern(),
        }
    }
}

// ─── Default value functions ───────────────────────────────────────────────

fn default_true() -> bool {
    true
}

fn default_docs_root() -> String {
    "docs".to_string()
}

fn default_frontmatter_fields() -> Vec<String> {
    vec!["name".to_string(), "description".to_string()]
}

fn default_adr_directory() -> String {
    "adrs".to_string()
}

fn default_adr_naming_pattern() -> String {
    r"ADR-\d{3,}-[a-zA-Z0-9-]+\.md".to_string()
}

fn default_max_depth() -> i32 {
    -1
}

fn default_exclude_dirs() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        ".git".to_string(),
        "target".to_string(),
        "vendor".to_string(),
        ".topology".to_string(),
    ]
}

fn default_docs_reference_pattern() -> String {
    "docs/".to_string()
}

// ─── Loading ───────────────────────────────────────────────────────────────

/// Load the APSS config from `.apss/config.toml` relative to the given root.
/// Returns default config if the file does not exist.
pub fn load_config(repo_root: &Path) -> Result<ApssConfig, ConfigError> {
    let config_path = repo_root.join(".apss").join("config.toml");
    if !config_path.exists() {
        return Ok(ApssConfig::default());
    }
    let content =
        std::fs::read_to_string(&config_path).map_err(|e| ConfigError::ReadError {
            path: config_path.clone(),
            source: e,
        })?;
    let config: ApssConfig =
        toml::from_str(&content).map_err(|e| ConfigError::ParseError {
            path: config_path,
            source: e,
        })?;
    Ok(config)
}

/// Resolve the absolute ADR directory path from config + repo root.
pub fn resolve_adr_dir(repo_root: &Path, docs_config: &DocsConfig) -> PathBuf {
    repo_root
        .join(&docs_config.root)
        .join(&docs_config.adr.directory)
}

/// Resolve the absolute docs root path.
pub fn resolve_docs_root(repo_root: &Path, docs_config: &DocsConfig) -> PathBuf {
    repo_root.join(&docs_config.root)
}

// ─── Errors ────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config at {path}: {source}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse config at {path}: {source}")]
    ParseError {
        path: PathBuf,
        source: toml::de::Error,
    },
}
