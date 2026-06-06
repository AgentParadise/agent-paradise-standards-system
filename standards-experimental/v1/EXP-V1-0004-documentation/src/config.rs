//! Configuration deserialization for the `docs` section of `apss.yaml`.
//!
//! Per the unified-config brief (2026-06-04), APSS configuration lives in a
//! single `apss.yaml` at the project root owned by the meta-standard (CF01).
//! EXP-V1-0004 registers the `docs` section; this module deserialises that
//! section into [`DocsConfig`] and exposes the loader that reads
//! `apss.yaml`, picks the `docs` key, and applies defaults for missing
//! fields. Other top-level sections (`fitness`, `topology`, ...) belong to
//! other standards and are tolerated here; the meta-standard's validator
//! enforces uniqueness, registry membership, and unknown-section errors.
//!
//! All fields use `#[serde(default)]` so a missing `apss.yaml` or a missing
//! `docs:` block produces sensible defaults (zero-config works out of the
//! box).

use apss_core::config as project_config;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Default config file name at the project root.
///
/// Single source of truth so the CLI, the validator, and the hint strings all
/// agree. Changing this path is a CF01 contract change; do not flip without
/// updating the meta-standard.
pub const CONFIG_FILENAME: &str = project_config::CONFIG_FILENAME;

/// Top-level APSS configuration file.
///
/// Only the `docs` section is consumed here; other standards' sections are
/// permissive at deserialisation time so a docs-only validator does not
/// reject a fitness or topology block.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ApssConfig {
    #[serde(default)]
    pub docs: DocsConfig,
}

/// The `docs` section of `apss.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct DocsConfig {
    #[serde(default = "default_false")]
    pub disable: bool,
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
    #[serde(default)]
    pub backlinking: BacklinkingConfig,
    #[serde(default, rename = "north-star")]
    pub north_star: NorthStarConfig,
    #[serde(default)]
    pub retrospectives: RetrospectivesConfig,
}

impl Default for DocsConfig {
    fn default() -> Self {
        Self {
            disable: false,
            root: default_docs_root(),
            index: IndexConfig::default(),
            context_files: ContextFilesConfig::default(),
            adr: AdrConfig::default(),
            readme: ReadmeConfig::default(),
            root_context: RootContextConfig::default(),
            backlinking: BacklinkingConfig::default(),
            north_star: NorthStarConfig::default(),
            retrospectives: RetrospectivesConfig::default(),
        }
    }
}

/// The `docs.backlinking` section of `apss.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct BacklinkingConfig {
    #[serde(default = "default_false")]
    pub disable: bool,
    #[serde(default = "default_backlinking_file_types")]
    pub file_types: Vec<String>,
    #[serde(rename = "scan", default, alias = "include_globs")]
    pub scan: Option<Vec<String>>,
}

impl Default for BacklinkingConfig {
    fn default() -> Self {
        Self {
            disable: false,
            file_types: default_backlinking_file_types(),
            scan: None,
        }
    }
}

/// The `docs.north-star` section of `apss.yaml`.
///
/// The YAML key is `north-star` (kebab); the Rust field on
/// [`DocsConfig`] is `north_star` (snake) bridged by `#[serde(rename)]`.
#[derive(Debug, Clone, Deserialize)]
pub struct NorthStarConfig {
    #[serde(default = "default_false")]
    pub disable: bool,
    #[serde(default = "default_north_star_location")]
    pub location: String,
}

impl Default for NorthStarConfig {
    fn default() -> Self {
        Self {
            disable: false,
            location: default_north_star_location(),
        }
    }
}

/// The `[docs.retrospectives]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct RetrospectivesConfig {
    #[serde(default = "default_false")]
    pub disable: bool,
    #[serde(default = "default_retrospectives_directory")]
    pub directory: String,
    #[serde(default = "default_retrospectives_naming_pattern")]
    pub naming_pattern: String,
}

impl Default for RetrospectivesConfig {
    fn default() -> Self {
        Self {
            disable: false,
            directory: default_retrospectives_directory(),
            naming_pattern: default_retrospectives_naming_pattern(),
        }
    }
}

// ─── ADR pattern constants ────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct IndexConfig {
    #[serde(default = "default_false")]
    pub disable: bool,
    #[serde(default = "default_true")]
    pub auto_generate: bool,
    #[serde(default = "default_frontmatter_fields")]
    pub frontmatter_fields: Vec<String>,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            disable: false,
            auto_generate: true,
            frontmatter_fields: default_frontmatter_fields(),
        }
    }
}

/// The `[docs.context_files]` section - CLAUDE.md and AGENTS.md per directory.
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
    #[serde(default = "default_false")]
    pub disable: bool,
    #[serde(default = "default_adr_directory")]
    pub directory: String,
    #[serde(default = "default_adr_naming_pattern")]
    pub naming_pattern: String,
    /// Required ADR keyword names (e.g., `["security", "testing"]`).
    /// For each keyword, at least one file matching `ADR-\d{3,5}-<keyword>\.md` must exist.
    #[serde(default)]
    pub required_adr_keywords: Vec<String>,
}

impl Default for AdrConfig {
    fn default() -> Self {
        Self {
            disable: false,
            directory: default_adr_directory(),
            naming_pattern: default_adr_naming_pattern(),
            required_adr_keywords: Vec::new(),
        }
    }
}

/// The `[docs.readme]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct ReadmeConfig {
    #[serde(default = "default_false")]
    pub disable: bool,
    #[serde(default = "default_max_depth")]
    pub max_depth: i32,
    #[serde(default = "default_exclude_dirs")]
    pub exclude_dirs: Vec<String>,
}

impl Default for ReadmeConfig {
    fn default() -> Self {
        Self {
            disable: false,
            max_depth: default_max_depth(),
            exclude_dirs: default_exclude_dirs(),
        }
    }
}

/// The `[docs.root_context]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct RootContextConfig {
    #[serde(default = "default_false")]
    pub disable: bool,
    #[serde(default = "default_docs_reference_pattern")]
    pub docs_reference_pattern: String,
}

impl Default for RootContextConfig {
    fn default() -> Self {
        Self {
            disable: false,
            docs_reference_pattern: default_docs_reference_pattern(),
        }
    }
}

// ─── ADR pattern constants ────────────────────────────────────────────────

/// Single source of truth for the ADR identifier pattern (stem without `.md`).
///
/// Matches: `ADR-001-security`, `ADR-99999-long-name`
/// Rejects: `ADR-01-short` (too few digits), `ADR-123456-six` (too many)
pub const ADR_STEM_PATTERN: &str = r"ADR-\d{3,5}-[a-zA-Z0-9-]+";

/// Default filename pattern for ADR files (stem + `.md` extension).
/// Used as the default value for `docs.adr.naming_pattern` in config.
pub fn default_adr_filename_pattern() -> String {
    format!(r"{ADR_STEM_PATTERN}\.md")
}

/// Return whether `filename` matches the configured ADR naming pattern AND
/// has a stem ending in `-<keyword>`. Used by the
/// `ADR01-missing-required-keyword` check to honour `docs.adr.naming_pattern`
/// rather than silently assuming the default `ADR-` prefix.
///
/// The keyword is compared literally (no regex semantics, no escaping needed),
/// so values like `c++` work without surprises.
pub fn adr_filename_has_keyword(naming_re: &regex::Regex, filename: &str, keyword: &str) -> bool {
    if !naming_re.is_match(filename) {
        return false;
    }
    let Some(stem) = filename.strip_suffix(".md") else {
        return false;
    };
    let suffix = format!("-{keyword}");
    stem.ends_with(&suffix) || stem == keyword
}

/// Derive the bare ADR stem pattern (filename without trailing `\.md`) from a
/// configured `docs.adr.naming_pattern`. Used to scan source files for
/// ADR references (`ADR01-dead-reference`) so reference extraction stays
/// aligned with the configured filename convention.
///
/// Strips a trailing `\.md`, `\.md$`, or `$` so the result describes only the
/// stem; falls back to the input untouched when no such suffix is present.
pub fn adr_stem_pattern_from_naming(naming_pattern: &str) -> String {
    let mut s = naming_pattern.trim_start_matches('^').to_string();
    if let Some(stripped) = s.strip_suffix('$') {
        s = stripped.to_string();
    }
    if let Some(stripped) = s.strip_suffix(r"\.md") {
        s = stripped.to_string();
    }
    s
}

// ─── Default value functions ───────────────────────────────────────────────

fn default_false() -> bool {
    false
}

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
    default_adr_filename_pattern()
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

fn default_backlinking_file_types() -> Vec<String> {
    Vec::new()
}

pub fn default_backlinking_scan() -> Vec<String> {
    vec![
        "**/*.rs".to_string(),
        "**/*.py".to_string(),
        "**/*.ts".to_string(),
        "**/*.tsx".to_string(),
        "**/*.js".to_string(),
        "**/*.jsx".to_string(),
        "**/*.go".to_string(),
        "**/*.java".to_string(),
        "**/*.kt".to_string(),
        "**/*.rb".to_string(),
        "**/*.sh".to_string(),
        "**/*.yaml".to_string(),
        "**/*.yml".to_string(),
        "**/*.toml".to_string(),
        "**/*.json".to_string(),
        "**/*.md".to_string(),
    ]
}

fn default_north_star_location() -> String {
    "docs/north-star.md".to_string()
}

fn default_retrospectives_directory() -> String {
    "docs/retrospectives".to_string()
}

fn default_retrospectives_naming_pattern() -> String {
    "RETRO-\\d{3,5}-[a-zA-Z0-9-]+\\.md".to_string()
}

// ─── Loading ───────────────────────────────────────────────────────────────

/// Load the APSS config from `apss.yaml` relative to the given root.
/// Returns default config if the file does not exist.
pub fn load_config(repo_root: &Path) -> Result<ApssConfig, ConfigError> {
    let config_path = repo_root.join(CONFIG_FILENAME);
    if !config_path.exists() {
        return Ok(ApssConfig::default());
    }
    let content = std::fs::read_to_string(&config_path).map_err(|e| ConfigError::ReadError {
        path: config_path.clone(),
        source: e,
    })?;
    let config: ApssConfig =
        serde_yaml::from_str(&content).map_err(|e| ConfigError::ParseError {
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
        source: serde_yaml::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stem_pattern_compiles() {
        regex::Regex::new(ADR_STEM_PATTERN).expect("ADR_STEM_PATTERN must be valid regex");
    }

    #[test]
    fn default_filename_pattern_matches_expected() {
        let re = regex::Regex::new(&format!("^{}$", default_adr_filename_pattern())).unwrap();
        assert!(re.is_match("ADR-001-security.md"));
        assert!(re.is_match("ADR-99999-long-name.md"));
        assert!(!re.is_match("ADR-01-short.md")); // too few digits
        assert!(!re.is_match("ADR-123456-six.md")); // too many digits
        assert!(!re.is_match("ADR-001-security")); // no .md
    }

    fn default_naming_re() -> regex::Regex {
        regex::Regex::new(&format!("^{}$", default_adr_filename_pattern())).unwrap()
    }

    #[test]
    fn filename_keyword_matches_expected() {
        let re = default_naming_re();
        assert!(adr_filename_has_keyword(
            &re,
            "ADR-001-security.md",
            "security"
        ));
        assert!(adr_filename_has_keyword(
            &re,
            "ADR-99999-security.md",
            "security"
        ));
        assert!(!adr_filename_has_keyword(
            &re,
            "ADR-001-testing.md",
            "security"
        )); // wrong keyword
        assert!(!adr_filename_has_keyword(
            &re,
            "ADR-01-security.md",
            "security"
        )); // too few digits
        assert!(!adr_filename_has_keyword(
            &re,
            "ADR-123456-security.md",
            "security"
        )); // too many digits
    }

    #[test]
    fn filename_keyword_handles_metacharacters_literally() {
        // The keyword is compared as a literal string, so regex metacharacters
        // like `+` do not need escaping.
        let custom = regex::Regex::new(r"^ADR-\d{3,5}-[a-zA-Z0-9+-]+\.md$").unwrap();
        assert!(adr_filename_has_keyword(&custom, "ADR-001-c++.md", "c++"));
        assert!(!adr_filename_has_keyword(&custom, "ADR-001-cpp.md", "c++"));
    }

    #[test]
    fn filename_keyword_follows_custom_naming() {
        // A project that customises naming_pattern (e.g., DEC- prefix) gets a
        // keyword check that matches its convention, not the silent default.
        let custom = regex::Regex::new(r"^DEC-\d{3,5}-[a-zA-Z0-9-]+\.md$").unwrap();
        assert!(adr_filename_has_keyword(
            &custom,
            "DEC-001-security.md",
            "security"
        ));
        assert!(!adr_filename_has_keyword(
            &custom,
            "ADR-001-security.md",
            "security"
        ));
    }

    #[test]
    fn stem_pattern_strips_md_suffix() {
        assert_eq!(
            adr_stem_pattern_from_naming(r"ADR-\d{3,5}-[a-zA-Z0-9-]+\.md"),
            r"ADR-\d{3,5}-[a-zA-Z0-9-]+",
        );
    }

    #[test]
    fn stem_pattern_strips_anchors_and_md() {
        assert_eq!(
            adr_stem_pattern_from_naming(r"^ADR-\d{3,5}-[a-zA-Z0-9-]+\.md$"),
            r"ADR-\d{3,5}-[a-zA-Z0-9-]+",
        );
    }

    #[test]
    fn stem_pattern_passthrough_when_no_md_suffix() {
        assert_eq!(
            adr_stem_pattern_from_naming(r"ADR-\d{3,5}-[a-zA-Z0-9-]+"),
            r"ADR-\d{3,5}-[a-zA-Z0-9-]+",
        );
    }
}
