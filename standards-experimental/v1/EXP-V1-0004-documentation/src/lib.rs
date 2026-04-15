//! EXP-V1-0004 — Documentation and Context Engineering
//!
//! Enforces documentation consistency across projects: ADR naming and front matter,
//! directory-level README indexes auto-generated from front matter, and AI context
//! files (CLAUDE.md, AGENTS.md) as lightweight pointers.
//!
//! This is the "consistency and context engineering" standard.

pub mod config;
pub mod context;
pub mod frontmatter;
pub mod index;
pub mod readme;

use config::{DocsConfig, load_config};
use std::path::{Path, PathBuf};

// ─── Error Codes ────────────────────────────────────────────────────────────

/// Error codes for documentation validation.
pub mod error_codes {
    // README / index sub-domain (DOC02)
    /// Directory is missing README.md.
    pub const MISSING_README: &str = "MISSING_README";
    /// Directory is missing CLAUDE.md.
    pub const MISSING_CLAUDE_MD: &str = "MISSING_CLAUDE_MD";
    /// Directory is missing AGENTS.md.
    pub const MISSING_AGENTS_MD: &str = "MISSING_AGENTS_MD";
    /// README.md is missing the ## Index section.
    pub const MISSING_INDEX: &str = "MISSING_INDEX";
    /// README.md ## Index section is out of date.
    pub const STALE_INDEX: &str = "STALE_INDEX";

    // Root context (DOC03)
    /// Repository root is missing CLAUDE.md.
    pub const MISSING_ROOT_CLAUDE_MD: &str = "MISSING_ROOT_CLAUDE_MD";
    /// Repository root is missing AGENTS.md.
    pub const MISSING_ROOT_AGENTS_MD: &str = "MISSING_ROOT_AGENTS_MD";
    /// Root CLAUDE.md does not reference documentation location.
    pub const MISSING_DOCS_REFERENCE: &str = "MISSING_DOCS_REFERENCE";

    // Config
    /// Config file is invalid.
    pub const INVALID_CONFIG: &str = "INVALID_CONFIG";
}

// ─── DocValidator ──────────────────────────────────────────────────────────

/// Main validator orchestrating all documentation checks.
pub struct DocValidator {
    docs_config: DocsConfig,
    repo_root: PathBuf,
}

/// Error loading the validator.
#[derive(Debug, thiserror::Error)]
pub enum DocError {
    #[error("configuration error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("index error: {0}")]
    Index(#[from] index::IndexError),
}

impl DocValidator {
    /// Load the validator, reading config from `.apss/config.toml`.
    ///
    /// If the config file does not exist, all defaults are used.
    pub fn load(repo_root: &Path) -> Result<Self, DocError> {
        let apss_config = load_config(repo_root)?;
        Ok(Self {
            docs_config: apss_config.docs,
            repo_root: repo_root.to_path_buf(),
        })
    }

    /// Load with a specific config (for testing or CLI override).
    pub fn with_config(repo_root: &Path, docs_config: DocsConfig) -> Self {
        Self {
            docs_config,
            repo_root: repo_root.to_path_buf(),
        }
    }

    /// Run all enabled documentation validations.
    pub fn validate(&self) -> aps_core::Diagnostics {
        let mut diagnostics = aps_core::Diagnostics::new();

        if !self.docs_config.enabled {
            return diagnostics;
        }

        // DOC02: README / index / context files
        readme::validate_readmes(&self.repo_root, &self.docs_config, &mut diagnostics);

        // DOC03: Root context files
        context::validate_root_context(&self.repo_root, &self.docs_config, &mut diagnostics);

        diagnostics
    }

    /// Generate indexes for all directories under the docs root.
    pub fn generate_indexes(&self) -> Result<Vec<index::GeneratedIndex>, DocError> {
        let docs_root = config::resolve_docs_root(&self.repo_root, &self.docs_config);
        if !docs_root.is_dir() {
            return Ok(Vec::new());
        }

        let mut indexes = Vec::new();
        collect_indexes_recursive(&docs_root, &self.docs_config, &mut indexes)?;
        Ok(indexes)
    }

    /// Write auto-generated indexes into README.md files.
    ///
    /// Returns the number of files updated. Respects `docs.index.auto_generate`
    /// — returns 0 without writing if auto-generation is disabled.
    pub fn write_indexes(&self) -> Result<usize, DocError> {
        if !self.docs_config.index.auto_generate {
            return Ok(0);
        }

        let docs_root = config::resolve_docs_root(&self.repo_root, &self.docs_config);
        if !docs_root.is_dir() {
            return Ok(0);
        }

        let mut count = 0;
        write_indexes_recursive(&docs_root, &self.docs_config, &mut count)?;
        Ok(count)
    }

    /// Get the docs config.
    pub fn config(&self) -> &DocsConfig {
        &self.docs_config
    }
}

/// Recursively collect generated indexes for all directories.
fn collect_indexes_recursive(
    dir: &Path,
    config: &DocsConfig,
    indexes: &mut Vec<index::GeneratedIndex>,
) -> Result<(), DocError> {
    let generated = index::generate_index(dir, &config.index)?;
    if !generated.entries.is_empty() {
        indexes.push(generated);
    }

    // Recurse into subdirectories
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(());
    };

    let exclude_set: std::collections::HashSet<&str> = config
        .readme
        .exclude_dirs
        .iter()
        .map(|s| s.as_str())
        .collect();

    for entry in entries.filter_map(|e| e.ok()) {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || exclude_set.contains(name.as_str()) {
            continue;
        }
        collect_indexes_recursive(&entry.path(), config, indexes)?;
    }

    Ok(())
}

/// Recursively write indexes into README.md files.
fn write_indexes_recursive(
    dir: &Path,
    config: &DocsConfig,
    count: &mut usize,
) -> Result<(), DocError> {
    let readme_path = dir.join("README.md");
    if readme_path.exists() {
        index::update_readme_index(&readme_path, dir, &config.index)?;
        *count += 1;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(());
    };

    let exclude_set: std::collections::HashSet<&str> = config
        .readme
        .exclude_dirs
        .iter()
        .map(|s| s.as_str())
        .collect();

    for entry in entries.filter_map(|e| e.ok()) {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || exclude_set.contains(name.as_str()) {
            continue;
        }
        write_indexes_recursive(&entry.path(), config, count)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes_are_unique() {
        let codes = vec![
            error_codes::MISSING_README,
            error_codes::MISSING_CLAUDE_MD,
            error_codes::MISSING_AGENTS_MD,
            error_codes::MISSING_INDEX,
            error_codes::STALE_INDEX,
            error_codes::MISSING_ROOT_CLAUDE_MD,
            error_codes::MISSING_ROOT_AGENTS_MD,
            error_codes::MISSING_DOCS_REFERENCE,
            error_codes::INVALID_CONFIG,
        ];
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(codes.len(), unique.len(), "error codes must be unique");
    }
}
