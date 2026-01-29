//! Artifact type definitions for TODO/FIXME tracking.
//!
//! This module defines the core data structures that represent the
//! TODO/FIXME tracking artifacts as specified in the standard.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema version for artifacts
pub const ARTIFACT_SCHEMA_VERSION: &str = "1.0.0";

/// Manifest schema identifier
pub const MANIFEST_SCHEMA: &str = "aps.todo-tracker/v1";

/// Root manifest for a todo-tracker scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerManifest {
    /// Schema identifier
    pub schema: String,

    /// ISO 8601 timestamp of when scan was generated
    pub generated_at: String,

    /// Version of the scanner that generated this
    pub scanner_version: String,

    /// Scan metadata
    pub scan: ScanMetadata,

    /// Configuration snapshot
    pub config: ConfigSnapshot,

    /// GitHub configuration (if enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<GitHubConfig>,
}

impl TrackerManifest {
    /// Create a new manifest with default schema
    pub fn new(
        scanner_version: String,
        scan: ScanMetadata,
        config: ConfigSnapshot,
        github: Option<GitHubConfig>,
    ) -> Self {
        Self {
            schema: MANIFEST_SCHEMA.to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            scanner_version,
            scan,
            config,
            github,
        }
    }
}

/// Metadata about the scan operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMetadata {
    /// Root path that was scanned
    pub root_path: String,

    /// Number of files scanned
    pub files_scanned: usize,

    /// Total lines scanned across all files
    pub lines_scanned: usize,

    /// Total TODO/FIXME items found
    pub items_found: usize,
}

/// Snapshot of configuration used for this scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    /// Tags that were scanned for
    pub tags: Vec<String>,

    /// Whether issue references are required
    pub require_issue: bool,

    /// Enforcement level: "off", "warn", or "error"
    pub enforcement: String,
}

/// GitHub API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// Whether GitHub API validation was enabled
    pub enabled: bool,

    /// Repository that was validated against
    pub repo: String,
}

/// Collection of TODO/FIXME items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItems {
    /// Schema version
    pub schema_version: String,

    /// ISO 8601 timestamp of generation
    pub generated_at: String,

    /// All items found
    pub items: Vec<TodoItem>,
}

impl TodoItems {
    /// Create a new items collection
    pub fn new(items: Vec<TodoItem>) -> Self {
        Self {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            items,
        }
    }
}

/// A single TODO/FIXME item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    /// Unique identifier (SHA256 hash of file+line+text)
    pub id: String,

    /// Tag type (TODO, FIXME, etc.)
    pub tag: String,

    /// File path (relative to repo root)
    pub file: String,

    /// Line number (1-indexed)
    pub line: usize,

    /// Column number (1-indexed)
    pub column: usize,

    /// Full comment text
    pub text: String,

    /// Extracted description (after issue reference)
    pub description: String,

    /// Issue reference (if present)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<IssueReference>,

    /// Code context (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<CodeContext>,
}

impl TodoItem {
    /// Generate a unique ID for this item
    pub fn generate_id(file: &str, line: usize, text: &str) -> String {
        use sha2::{Digest, Sha256};
        let input = format!("{file}:{line}:{text}");
        let hash = Sha256::digest(input.as_bytes());
        hex::encode(hash)
    }

    /// Check if this item has an issue reference
    pub fn is_tracked(&self) -> bool {
        self.issue.is_some()
    }
}

/// Reference to an issue tracker item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueReference {
    /// Type of issue tracker (currently only "github")
    #[serde(rename = "type")]
    pub issue_type: String,

    /// Issue number
    pub number: u32,

    /// Repository (if validated via API)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,

    /// Full issue URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Whether issue was validated via API
    pub validated: bool,

    /// Issue state (open, closed) from API
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// Issue title from API
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// ISO 8601 timestamp of when validation occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validated_at: Option<String>,
}

impl IssueReference {
    /// Create a new GitHub issue reference (not yet validated)
    pub fn github(number: u32) -> Self {
        Self {
            issue_type: "github".to_string(),
            number,
            repo: None,
            url: None,
            validated: false,
            state: None,
            title: None,
            validated_at: None,
        }
    }

    /// Create a GitHub issue reference with full details (validated)
    pub fn github_validated(number: u32, repo: String, state: String, title: String) -> Self {
        let url = format!("https://github.com/{repo}/issues/{number}");
        Self {
            issue_type: "github".to_string(),
            number,
            repo: Some(repo),
            url: Some(url),
            validated: true,
            state: Some(state),
            title: Some(title),
            validated_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
}

/// Code context around the TODO item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeContext {
    /// Function/method name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,

    /// Module/class name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,

    /// Line before the comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,

    /// Line after the comment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}

/// Summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSummary {
    /// Schema version
    pub schema_version: String,

    /// ISO 8601 timestamp of generation
    pub generated_at: String,

    /// Total statistics
    pub totals: TotalStats,

    /// Statistics by tag
    pub by_tag: HashMap<String, TagStats>,

    /// Count by file
    pub by_file: HashMap<String, usize>,

    /// Count by issue number (or "null" for untracked)
    pub by_issue: HashMap<String, usize>,

    /// Validation statistics (if GitHub API was used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationStats>,
}

impl ItemSummary {
    /// Generate summary from a list of items
    pub fn from_items(items: &[TodoItem], github_enabled: bool) -> Self {
        let mut by_tag: HashMap<String, TagStats> = HashMap::new();
        let mut by_file: HashMap<String, usize> = HashMap::new();
        let mut by_issue: HashMap<String, usize> = HashMap::new();

        let mut tracked = 0;
        let mut untracked = 0;
        let mut validated_count = 0;
        let failed_count = 0;
        let mut open_issues = 0;
        let mut closed_issues = 0;

        let mut files_set = std::collections::HashSet::new();

        for item in items {
            // Track file
            files_set.insert(item.file.clone());
            *by_file.entry(item.file.clone()).or_insert(0) += 1;

            // Track by tag
            let tag_stats = by_tag.entry(item.tag.clone()).or_insert(TagStats {
                total: 0,
                tracked: 0,
                untracked: 0,
            });
            tag_stats.total += 1;

            // Track issue status
            if let Some(issue) = &item.issue {
                tracked += 1;
                tag_stats.tracked += 1;

                let issue_key = issue.number.to_string();
                *by_issue.entry(issue_key).or_insert(0) += 1;

                if issue.validated {
                    validated_count += 1;
                    if let Some(state) = &issue.state {
                        if state == "open" {
                            open_issues += 1;
                        } else {
                            closed_issues += 1;
                        }
                    }
                }
            } else {
                untracked += 1;
                tag_stats.untracked += 1;
                *by_issue.entry("null".to_string()).or_insert(0) += 1;
            }
        }

        let validation = if github_enabled {
            Some(ValidationStats {
                github_api_enabled: true,
                validated_count,
                failed_count,
                open_issues,
                closed_issues,
            })
        } else {
            None
        };

        Self {
            schema_version: ARTIFACT_SCHEMA_VERSION.to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            totals: TotalStats {
                items: items.len(),
                files: files_set.len(),
                tracked,
                untracked,
            },
            by_tag,
            by_file,
            by_issue,
            validation,
        }
    }
}

/// Total statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotalStats {
    /// Total number of items
    pub items: usize,

    /// Number of unique files with items
    pub files: usize,

    /// Number of tracked items (with issue references)
    pub tracked: usize,

    /// Number of untracked items (without issue references)
    pub untracked: usize,
}

/// Statistics for a specific tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagStats {
    /// Total items with this tag
    pub total: usize,

    /// Tracked items with this tag
    pub tracked: usize,

    /// Untracked items with this tag
    pub untracked: usize,
}

/// Validation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    /// Whether GitHub API was enabled
    pub github_api_enabled: bool,

    /// Number of items successfully validated
    pub validated_count: usize,

    /// Number of validation failures
    pub failed_count: usize,

    /// Number of open issues found
    pub open_issues: usize,

    /// Number of closed issues found
    pub closed_issues: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id1 = TodoItem::generate_id("src/main.rs", 10, "TODO(#123): test");
        let id2 = TodoItem::generate_id("src/main.rs", 10, "TODO(#123): test");
        let id3 = TodoItem::generate_id("src/main.rs", 11, "TODO(#123): test");

        // Same input should generate same ID
        assert_eq!(id1, id2);

        // Different input should generate different ID
        assert_ne!(id1, id3);

        // ID should be hex-encoded SHA256 (64 characters)
        assert_eq!(id1.len(), 64);
    }

    #[test]
    fn test_issue_reference_github() {
        let issue = IssueReference::github(123);
        assert_eq!(issue.issue_type, "github");
        assert_eq!(issue.number, 123);
        assert!(!issue.validated);
        assert!(issue.repo.is_none());
    }

    #[test]
    fn test_issue_reference_github_validated() {
        let issue = IssueReference::github_validated(
            123,
            "owner/repo".to_string(),
            "open".to_string(),
            "Test issue".to_string(),
        );

        assert_eq!(issue.issue_type, "github");
        assert_eq!(issue.number, 123);
        assert!(issue.validated);
        assert_eq!(issue.repo, Some("owner/repo".to_string()));
        assert_eq!(issue.state, Some("open".to_string()));
        assert_eq!(issue.title, Some("Test issue".to_string()));
        assert!(issue.url.is_some());
        assert!(issue.validated_at.is_some());
    }

    #[test]
    fn test_todo_item_is_tracked() {
        let tracked = TodoItem {
            id: "test".to_string(),
            tag: "TODO".to_string(),
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
            text: "TODO(#123): test".to_string(),
            description: "test".to_string(),
            issue: Some(IssueReference::github(123)),
            context: None,
        };

        let untracked = TodoItem {
            id: "test".to_string(),
            tag: "TODO".to_string(),
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
            text: "TODO: test".to_string(),
            description: "test".to_string(),
            issue: None,
            context: None,
        };

        assert!(tracked.is_tracked());
        assert!(!untracked.is_tracked());
    }

    #[test]
    fn test_item_summary_from_items() {
        let items = vec![
            TodoItem {
                id: "1".to_string(),
                tag: "TODO".to_string(),
                file: "a.rs".to_string(),
                line: 1,
                column: 1,
                text: "TODO(#123): test".to_string(),
                description: "test".to_string(),
                issue: Some(IssueReference::github(123)),
                context: None,
            },
            TodoItem {
                id: "2".to_string(),
                tag: "FIXME".to_string(),
                file: "a.rs".to_string(),
                line: 2,
                column: 1,
                text: "FIXME: test".to_string(),
                description: "test".to_string(),
                issue: None,
                context: None,
            },
            TodoItem {
                id: "3".to_string(),
                tag: "TODO".to_string(),
                file: "b.rs".to_string(),
                line: 1,
                column: 1,
                text: "TODO(#456): test".to_string(),
                description: "test".to_string(),
                issue: Some(IssueReference::github(456)),
                context: None,
            },
        ];

        let summary = ItemSummary::from_items(&items, false);

        assert_eq!(summary.totals.items, 3);
        assert_eq!(summary.totals.files, 2);
        assert_eq!(summary.totals.tracked, 2);
        assert_eq!(summary.totals.untracked, 1);

        assert_eq!(summary.by_tag.get("TODO").unwrap().total, 2);
        assert_eq!(summary.by_tag.get("FIXME").unwrap().total, 1);

        assert_eq!(summary.by_file.get("a.rs"), Some(&2));
        assert_eq!(summary.by_file.get("b.rs"), Some(&1));

        assert_eq!(summary.by_issue.get("123"), Some(&1));
        assert_eq!(summary.by_issue.get("456"), Some(&1));
        assert_eq!(summary.by_issue.get("null"), Some(&1));
    }
}
