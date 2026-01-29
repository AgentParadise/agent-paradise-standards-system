//! File scanning and TODO/FIXME extraction.
//!
//! This module implements the core scanning logic to find TODO/FIXME
//! comments across polyglot codebases.

use crate::artifact::{IssueReference, TodoItem};
use crate::config::TrackerConfig;
use crate::languages::{GENERIC, LanguageConfig};
use regex::Regex;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

/// Scanner for finding TODO/FIXME comments in source code
pub struct Scanner {
    config: TrackerConfig,
    tag_pattern: Regex,
}

impl Scanner {
    /// Create a new scanner with the given configuration
    pub fn new(config: TrackerConfig) -> Result<Self, ScannerError> {
        // Build regex pattern for all configured tags
        let tags = config.tracker.tags.join("|");
        let pattern = format!(r"({tags})(\(#(\d+)\))?:\s*(.+)");
        let tag_pattern = Regex::new(&pattern)?;

        Ok(Self {
            config,
            tag_pattern,
        })
    }

    /// Scan a repository and return all TODO/FIXME items found
    pub fn scan_repo(&self, repo_root: &Path) -> Result<ScanResult, ScannerError> {
        let mut items = Vec::new();
        let mut files_scanned = 0;
        let mut lines_scanned = 0;

        for entry in self.walk_files(repo_root) {
            let entry = entry?;
            let path = entry.path();

            if !self.should_scan_file(path) {
                continue;
            }

            match self.scan_file(path, repo_root) {
                Ok(file_result) => {
                    files_scanned += 1;
                    lines_scanned += file_result.lines_scanned;
                    items.extend(file_result.items);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to scan {}: {}", path.display(), e);
                }
            }
        }

        Ok(ScanResult {
            items,
            files_scanned,
            lines_scanned,
        })
    }

    /// Walk through files in the repository
    fn walk_files(&self, root: &Path) -> impl Iterator<Item = Result<DirEntry, walkdir::Error>> {
        WalkDir::new(root).follow_links(false).into_iter()
    }

    /// Check if a file should be scanned
    fn should_scan_file(&self, path: &Path) -> bool {
        // Must be a file
        if !path.is_file() {
            return false;
        }

        // Check file size
        if let Ok(metadata) = fs::metadata(path) {
            let size_mb = metadata.len() / (1024 * 1024);
            if size_mb > self.config.scan.max_file_size as u64 {
                return false;
            }
        }

        // Check if extension is allowed (if specified)
        if !self.config.scan.extensions.is_empty() {
            if let Some(ext) = path.extension() {
                let ext_str = format!(".{}", ext.to_string_lossy());
                if !self.config.scan.extensions.contains(&ext_str) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check exclusion patterns
        let path_str = path.to_string_lossy();
        for exclude in &self.config.scan.exclude {
            if self.matches_glob(&path_str, exclude) {
                return false;
            }
        }

        true
    }

    /// Simple glob matching (supports ** and *)
    fn matches_glob(&self, path: &str, pattern: &str) -> bool {
        // Simple implementation - just check if pattern is contained
        // A full implementation would use the `glob` crate
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0].trim_matches('/');
                let suffix = parts[1].trim_matches('/');
                return (prefix.is_empty() || path.contains(prefix))
                    && (suffix.is_empty() || path.contains(suffix));
            }
        }
        path.contains(pattern.trim_matches('*'))
    }

    /// Scan a single file for TODO/FIXME comments
    fn scan_file(&self, path: &Path, repo_root: &Path) -> Result<FileScanResult, ScannerError> {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);

        let lang = LanguageConfig::from_path(path).unwrap_or(&GENERIC);
        let relative_path = path
            .strip_prefix(repo_root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let mut items = Vec::new();
        let mut lines_scanned = 0;

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result?;
            lines_scanned += 1;

            // Check if line is a comment
            if !lang.could_be_comment(&line) {
                continue;
            }

            // Extract TODO/FIXME if present
            if let Some(item) = self.extract_todo(&line, &relative_path, line_num + 1)? {
                items.push(item);
            }
        }

        Ok(FileScanResult {
            items,
            lines_scanned,
        })
    }

    /// Extract TODO/FIXME from a comment line
    fn extract_todo(
        &self,
        line: &str,
        file: &str,
        line_num: usize,
    ) -> Result<Option<TodoItem>, ScannerError> {
        // Find the start of the comment content
        let comment_start = self.find_comment_start(line);
        let content = &line[comment_start..];

        // Try to match the TODO/FIXME pattern
        if let Some(captures) = self.tag_pattern.captures(content) {
            let tag = captures.get(1).unwrap().as_str().to_string();
            let issue_num = captures.get(3).and_then(|m| m.as_str().parse::<u32>().ok());
            let description = captures.get(4).unwrap().as_str().trim().to_string();

            let text = line.trim().to_string();
            let id = TodoItem::generate_id(file, line_num, &text);

            let issue = issue_num.map(IssueReference::github);

            let item = TodoItem {
                id,
                tag,
                file: file.to_string(),
                line: line_num,
                column: comment_start + 1, // Convert to 1-indexed
                text,
                description,
                issue,
                context: None,
            };

            return Ok(Some(item));
        }

        Ok(None)
    }

    /// Find the start index of the actual comment content
    fn find_comment_start(&self, line: &str) -> usize {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        // Skip common comment prefixes
        for prefix in &["//!", "///", "//", "#", "/*", "*"] {
            if trimmed.starts_with(prefix) {
                return indent + prefix.len();
            }
        }

        indent
    }
}

/// Result of scanning a repository
#[derive(Debug)]
pub struct ScanResult {
    /// All TODO/FIXME items found
    pub items: Vec<TodoItem>,

    /// Number of files scanned
    pub files_scanned: usize,

    /// Total lines scanned
    pub lines_scanned: usize,
}

/// Result of scanning a single file
#[derive(Debug)]
struct FileScanResult {
    items: Vec<TodoItem>,
    lines_scanned: usize,
}

/// Scanner errors
#[derive(Debug, thiserror::Error)]
pub enum ScannerError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Regex error
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Walkdir error
    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_todo_with_issue() {
        let config = TrackerConfig::default();
        let scanner = Scanner::new(config).unwrap();

        let line = "// TODO(#123): Add error handling";
        let item = scanner.extract_todo(line, "test.rs", 10).unwrap().unwrap();

        assert_eq!(item.tag, "TODO");
        assert_eq!(item.description, "Add error handling");
        assert_eq!(item.line, 10);
        assert!(item.issue.is_some());
        assert_eq!(item.issue.as_ref().unwrap().number, 123);
    }

    #[test]
    fn test_extract_fixme_with_issue() {
        let config = TrackerConfig::default();
        let scanner = Scanner::new(config).unwrap();

        let line = "# FIXME(#456): This breaks with empty input";
        let item = scanner.extract_todo(line, "test.py", 5).unwrap().unwrap();

        assert_eq!(item.tag, "FIXME");
        assert_eq!(item.description, "This breaks with empty input");
        assert_eq!(item.line, 5);
        assert!(item.issue.is_some());
        assert_eq!(item.issue.as_ref().unwrap().number, 456);
    }

    #[test]
    fn test_extract_todo_without_issue() {
        let config = TrackerConfig::default();
        let scanner = Scanner::new(config).unwrap();

        let line = "// TODO: Add tests";
        let item = scanner.extract_todo(line, "test.rs", 1).unwrap().unwrap();

        assert_eq!(item.tag, "TODO");
        assert_eq!(item.description, "Add tests");
        assert!(item.issue.is_none());
    }

    #[test]
    fn test_extract_no_match() {
        let config = TrackerConfig::default();
        let scanner = Scanner::new(config).unwrap();

        let line = "// This is a regular comment";
        let item = scanner.extract_todo(line, "test.rs", 1).unwrap();

        assert!(item.is_none());
    }

    #[test]
    fn test_matches_glob() {
        let config = TrackerConfig::default();
        let scanner = Scanner::new(config).unwrap();

        assert!(scanner.matches_glob("target/debug/foo", "target/**"));
        assert!(scanner.matches_glob("node_modules/pkg/index.js", "node_modules/**"));
        assert!(scanner.matches_glob(".git/config", ".git/**"));
        assert!(!scanner.matches_glob("src/main.rs", "target/**"));
    }
}
