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
/// Returns `None` if the content does not begin with a delimiter line that is
/// exactly `---` (LF or CRLF terminated). Markdown horizontal rules like
/// `----` therefore do not look like front matter, so they no longer produce
/// false `Unclosed` errors.
///
/// Returns `Err` if a valid opening delimiter is found but the closing
/// delimiter is missing.
pub fn parse_frontmatter(content: &str) -> Result<Option<FrontMatter>, FrontMatterError> {
    let trimmed = content.trim_start();

    // Require the opener to be exactly `---` followed by a line break. This
    // rules out `----` rules and inline dashes while still tolerating CRLF.
    let after_open = if let Some(rest) = trimmed.strip_prefix("---\n") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("---\r\n") {
        rest
    } else {
        return Ok(None);
    };

    // Walk the remaining lines to find a delimiter line that is exactly `---`,
    // so we accept both LF and CRLF and never mistake a longer rule like
    // `----` for the close.
    let mut consumed = 0usize;
    let mut close_pos = None;
    for line in after_open.split_inclusive('\n') {
        let body = line.trim_end_matches(['\r', '\n']);
        if body == "---" {
            close_pos = Some(consumed);
            break;
        }
        consumed += line.len();
    }

    let close_pos = close_pos.ok_or(FrontMatterError::Unclosed)?;
    let yaml_block = after_open[..close_pos].trim();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lf_delimited_block() {
        let src = "---\nname: foo\ndescription: bar\n---\nbody\n";
        let fm = parse_frontmatter(src).unwrap().expect("fm present");
        assert_eq!(fm.name(), Some("foo"));
        assert_eq!(fm.description(), Some("bar"));
    }

    #[test]
    fn parses_crlf_delimited_block() {
        let src = "---\r\nname: foo\r\n---\r\nbody\r\n";
        let fm = parse_frontmatter(src).unwrap().expect("fm present");
        assert_eq!(fm.name(), Some("foo"));
    }

    #[test]
    fn rejects_quadruple_dash_rule() {
        // A markdown horizontal rule starts with `----`, not exactly `---`,
        // so it must not look like a front matter opener.
        let src = "----\nNot front matter, just a horizontal rule.\n";
        assert!(parse_frontmatter(src).unwrap().is_none());
    }

    #[test]
    fn rejects_inline_dashes_without_newline() {
        // `--- name: foo` on the same line is not a delimiter line.
        let src = "--- inline\nname: foo\n";
        assert!(parse_frontmatter(src).unwrap().is_none());
    }

    #[test]
    fn closing_delimiter_only_matches_exact_line() {
        // A `----` rule inside the YAML region must not be mistaken for the
        // closing delimiter; closure must be exactly `---` on its own line.
        let src = "---\nname: foo\n----\ndescription: still inside\n---\nbody\n";
        let fm = parse_frontmatter(src).unwrap().expect("fm present");
        assert_eq!(fm.description(), Some("still inside"));
    }

    #[test]
    fn unclosed_block_returns_error() {
        let src = "---\nname: foo\n";
        let err = parse_frontmatter(src).expect_err("expected unclosed error");
        assert!(matches!(err, FrontMatterError::Unclosed));
    }
}
