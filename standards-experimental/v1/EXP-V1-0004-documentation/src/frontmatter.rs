//! YAML front matter parser for Markdown files.
//!
//! Parses the `---`-delimited YAML block at the top of `.md` files.
//! Shared by ADR validation and index generation.

use std::collections::HashMap;
use std::path::Path;

/// Parsed front matter from a Markdown file.
#[derive(Debug, Clone, Default)]
pub struct FrontMatter {
    /// All key-value pairs from the YAML block.
    pub fields: HashMap<String, String>,
}

impl FrontMatter {
    /// Get the `name` field.
    pub fn name(&self) -> Option<&str> {
        self.fields.get("name").map(|s| s.as_str())
    }

    /// Get the `description` field.
    pub fn description(&self) -> Option<&str> {
        self.fields.get("description").map(|s| s.as_str())
    }

    /// Get an arbitrary field by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(|s| s.as_str())
    }
}

/// Parse YAML front matter from a string.
///
/// Returns `None` if the content does not start with a `---` block.
/// Returns `Err` if the block exists but cannot be parsed.
pub fn parse_frontmatter(content: &str) -> Result<Option<FrontMatter>, FrontMatterError> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Ok(None);
    }

    // Find the closing `---`
    let after_open = &trimmed[3..];
    let close_pos = after_open.find("\n---").ok_or(FrontMatterError::Unclosed)?;

    let yaml_block = &after_open[..close_pos].trim();
    if yaml_block.is_empty() {
        return Ok(Some(FrontMatter::default()));
    }

    let fields = parse_simple_yaml(yaml_block)?;
    Ok(Some(FrontMatter { fields }))
}

/// Parse front matter from a file path.
///
/// Returns `None` if the file has no front matter block.
pub fn parse_frontmatter_from_file(path: &Path) -> Result<Option<FrontMatter>, FrontMatterError> {
    let content = std::fs::read_to_string(path).map_err(|e| FrontMatterError::IoError {
        path: path.to_path_buf(),
        source: e,
    })?;
    parse_frontmatter(&content)
}

/// Simple YAML parser for flat key-value pairs (no nesting).
///
/// Handles: `key: value`, `key: "quoted value"`, `key: 'quoted value'`
fn parse_simple_yaml(yaml: &str) -> Result<HashMap<String, String>, FrontMatterError> {
    let mut map = HashMap::new();
    for line in yaml.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(colon_pos) = line.find(':') else {
            continue;
        };
        let key = line[..colon_pos].trim().to_string();
        let mut value = line[colon_pos + 1..].trim().to_string();

        // Strip quotes
        if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            value = value[1..value.len() - 1].to_string();
        }

        if !key.is_empty() {
            map.insert(key, value);
        }
    }
    Ok(map)
}

// ─── Errors ────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum FrontMatterError {
    #[error("front matter block is not closed (missing closing ---)")]
    Unclosed,
    #[error("failed to read file {path}: {source}")]
    IoError {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
}
