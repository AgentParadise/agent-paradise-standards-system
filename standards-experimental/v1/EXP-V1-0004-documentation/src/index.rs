//! Index generation from front matter.
//!
//! Scans `.md` files in a directory, extracts front matter, and generates
//! a `## Index` table for insertion into `README.md`.

use crate::config::IndexConfig;
use crate::frontmatter::{self, FrontMatter};
use std::path::Path;

/// A single entry in the generated index.
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub filename: String,
    pub name: String,
    pub description: String,
}

/// Result of index generation for a directory.
#[derive(Debug, Clone)]
pub struct GeneratedIndex {
    pub dir: std::path::PathBuf,
    pub entries: Vec<IndexEntry>,
    pub markdown: String,
}

/// Generate index entries by scanning `.md` files in a directory.
///
/// Excludes `README.md`, `CLAUDE.md`, and `AGENTS.md` from the index
/// (they are structural files, not content documents).
pub fn generate_index(dir: &Path, config: &IndexConfig) -> Result<GeneratedIndex, IndexError> {
    let mut entries = Vec::new();
    let structural_files = ["readme.md", "claude.md", "agents.md"];

    let read_dir = std::fs::read_dir(dir).map_err(|e| IndexError::ReadDir {
        path: dir.to_path_buf(),
        source: e,
    })?;

    let mut md_files: Vec<_> = read_dir
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            name.ends_with(".md") && !structural_files.contains(&name.as_str())
        })
        .collect();

    // Sort by filename for deterministic output
    md_files.sort_by_key(|e| e.file_name());

    for entry in &md_files {
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().to_string();

        let fm = frontmatter::parse_frontmatter_from_file(&path)
            .ok()
            .flatten();

        let name = fm
            .as_ref()
            .and_then(FrontMatter::name)
            .unwrap_or_else(|| filename.trim_end_matches(".md"))
            .to_string();

        let description = fm
            .as_ref()
            .and_then(FrontMatter::description)
            .unwrap_or("")
            .to_string();

        entries.push(IndexEntry {
            filename,
            name,
            description,
        });
    }

    let markdown = render_index_table(&entries, config);

    Ok(GeneratedIndex {
        dir: dir.to_path_buf(),
        entries,
        markdown,
    })
}

/// Render index entries as a Markdown table.
fn render_index_table(entries: &[IndexEntry], config: &IndexConfig) -> String {
    if entries.is_empty() {
        return "## Index\n\n_No documents found._\n".to_string();
    }

    // Build header row from configured frontmatter fields
    let fields = &config.frontmatter_fields;
    let mut out = String::from("## Index\n\n");

    if fields.len() == 2 && fields[0] == "name" && fields[1] == "description" {
        // Default: two-column table with name as link text
        out.push_str("| Document | Description |\n");
        out.push_str("|----------|-------------|\n");
        for entry in entries {
            out.push_str(&format!(
                "| [{}]({}) | {} |\n",
                entry.name, entry.filename, entry.description
            ));
        }
    } else {
        // Custom fields: always start with a linked name column, then remaining fields
        out.push_str("| Document |");
        for field in fields.iter().filter(|f| f.as_str() != "name") {
            let title = capitalize(field);
            out.push_str(&format!(" {title} |"));
        }
        out.push('\n');

        out.push_str("|----------|");
        for _ in fields.iter().filter(|f| f.as_str() != "name") {
            out.push_str("-------------|");
        }
        out.push('\n');

        for entry in entries {
            out.push_str(&format!("| [{}]({})", entry.name, entry.filename));
            for field in fields.iter().filter(|f| f.as_str() != "name") {
                let val = if field == "description" {
                    &entry.description
                } else {
                    ""
                };
                out.push_str(&format!(" | {val}"));
            }
            out.push_str(" |\n");
        }
    }

    out
}

/// Extract the `## Index` section from a README's content.
///
/// Returns the byte range `(start, end)` of the section, or `None` if missing.
/// Uses line-based matching so `## Indexing` is not a false positive.
pub fn find_index_section(content: &str) -> Option<(usize, usize)> {
    let mut offset = 0usize;
    let mut start = None;
    let mut after_header = content.len();

    for line in content.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed == "## Index" {
            start = Some(offset);
            after_header = offset + line.len();
            break;
        }
        offset += line.len();
    }

    let start = start?;

    // Find the next `## ` heading or end of file
    let mut end = content.len();
    let mut search_offset = after_header;
    for line in content[after_header..].split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.starts_with("## ") {
            end = search_offset;
            break;
        }
        search_offset += line.len();
    }

    Some((start, end))
}

/// Validate that a README's `## Index` section matches the actual files.
///
/// Returns the expected index markdown and whether it matches.
pub fn validate_index(
    readme_content: &str,
    dir: &Path,
    config: &IndexConfig,
) -> Result<IndexValidation, IndexError> {
    let generated = generate_index(dir, config)?;

    let current_section = find_index_section(readme_content);
    match current_section {
        None => Ok(IndexValidation {
            expected: generated.markdown,
            is_valid: false,
            reason: IndexIssue::Missing,
        }),
        Some((start, end)) => {
            let current = readme_content[start..end].trim_end();
            let expected = generated.markdown.trim_end();
            if current == expected {
                Ok(IndexValidation {
                    expected: generated.markdown,
                    is_valid: true,
                    reason: IndexIssue::None,
                })
            } else {
                Ok(IndexValidation {
                    expected: generated.markdown,
                    is_valid: false,
                    reason: IndexIssue::Stale,
                })
            }
        }
    }
}

/// Update the `## Index` section in a README file in-place.
pub fn update_readme_index(
    readme_path: &Path,
    dir: &Path,
    config: &IndexConfig,
) -> Result<(), IndexError> {
    let content = std::fs::read_to_string(readme_path).map_err(|e| IndexError::ReadFile {
        path: readme_path.to_path_buf(),
        source: e,
    })?;

    let generated = generate_index(dir, config)?;
    let new_content = match find_index_section(&content) {
        Some((start, end)) => {
            let mut result = String::with_capacity(content.len());
            result.push_str(&content[..start]);
            result.push_str(generated.markdown.trim_end());
            result.push_str(&content[end..]);
            result
        }
        None => {
            // Append the index section at the end
            let mut result = content;
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');
            result.push_str(&generated.markdown);
            result
        }
    };

    std::fs::write(readme_path, new_content).map_err(|e| IndexError::WriteError {
        path: readme_path.to_path_buf(),
        source: e,
    })?;

    Ok(())
}

/// Result of index validation.
#[derive(Debug)]
pub struct IndexValidation {
    pub expected: String,
    pub is_valid: bool,
    pub reason: IndexIssue,
}

#[derive(Debug, PartialEq, Eq)]
pub enum IndexIssue {
    None,
    Missing,
    Stale,
}

// ─── Errors ────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("failed to read directory {path}: {source}")]
    ReadDir {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("failed to read file {path}: {source}")]
    ReadFile {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("failed to write file {path}: {source}")]
    WriteError {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
}

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}
