//! ADR (Architecture Decision Record) enforcement substandard.
//!
//! Validates ADR directory structure, file naming conventions (`ADR-XXX-<name>.md`),
//! required front matter, keyword-based required ADRs, and backlinking from
//! implementation files back to governing ADRs.

use documentation::config::{self, DocsConfig};
use aps_core::{Diagnostic, Diagnostics};
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Error codes emitted by the ADR substandard.
pub mod error_codes {
    pub const MISSING_ADR_DIR: &str = "ADR01-001";
    pub const INVALID_ADR_NAMING: &str = "ADR01-002";
    pub const MISSING_ADR_FRONTMATTER: &str = "ADR01-003";
    pub const MISSING_REQUIRED_ADR: &str = "ADR01-004";
    /// Reserved — not emitted. Forward backlink enforcement is not feasible;
    /// use ADR01-009 (dead reference detection) for reference integrity instead.
    pub const MISSING_ADR_BACKLINK: &str = "ADR01-005";
    pub const INVALID_NAMING_REGEX: &str = "ADR01-006";
    pub const MISSING_ADR_CONTEXT_FILE: &str = "ADR01-007";
    pub const ADR_CONTEXT_MISSING_GUIDANCE: &str = "ADR01-008";
    pub const DEAD_ADR_REFERENCE: &str = "ADR01-009";
    pub const MISSING_ADR_HEADER: &str = "ADR01-010";
}

/// ADR validator that loads config and runs all ADR checks.
pub struct AdrValidator {
    config: DocsConfig,
    repo_root: PathBuf,
}

impl AdrValidator {
    /// Load the ADR validator from a repository root.
    /// Reads `.apss/config.toml` for configuration.
    pub fn load(repo_root: &Path) -> Result<Self, config::ConfigError> {
        let apss_config = config::load_config(repo_root)?;
        Ok(Self {
            config: apss_config.docs,
            repo_root: repo_root.to_path_buf(),
        })
    }

    /// Create a validator with an explicit config (useful for testing).
    pub fn with_config(repo_root: &Path, config: DocsConfig) -> Self {
        Self {
            config,
            repo_root: repo_root.to_path_buf(),
        }
    }

    /// Run all ADR validation checks and return diagnostics.
    pub fn validate(&self) -> Diagnostics {
        let mut diagnostics = Diagnostics::new();

        if !self.config.adr.enabled {
            return diagnostics;
        }

        let adr_dir = config::resolve_adr_dir(&self.repo_root, &self.config);



        // ADR01-001: ADR directory must exist
        if !adr_dir.is_dir() {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::MISSING_ADR_DIR,
                    format!("ADR directory not found: {}", adr_dir.display()),
                )
                .with_path(&adr_dir)
                .with_hint(format!(
                    "Create the directory at '{}' or configure docs.adr.directory in .apss/config.toml",
                    adr_dir.display()
                )),
            );
            return diagnostics;
        }

        // Compile naming pattern
        let naming_regex = match Regex::new(&format!("^{}$", &self.config.adr.naming_pattern)) {
            Ok(re) => re,
            Err(e) => {
                diagnostics.push(
                    Diagnostic::error(
                        error_codes::INVALID_NAMING_REGEX,
                        format!(
                            "Invalid ADR naming regex '{}': {e}",
                            self.config.adr.naming_pattern
                        ),
                    )
                    .with_hint("Check docs.adr.naming_pattern in .apss/config.toml"),
                );
                return diagnostics;
            }
        };

        // Collect ADR files
        let adr_files = collect_adr_files(&adr_dir);

        // ADR01-002: Validate naming convention
        validate_naming(&adr_dir, &adr_files, &naming_regex, &self.config, &mut diagnostics);

        // ADR01-003: Validate front matter
        validate_frontmatter(&adr_dir, &adr_files, &mut diagnostics);

        // ADR01-004: Check required ADR keywords
        validate_required_keywords(&adr_dir, &adr_files, &self.config, &mut diagnostics);

        // ADR01-007/008: Check ADR context files (CLAUDE.md, AGENTS.md)
        validate_adr_context_files(&adr_dir, &mut diagnostics);

        // ADR01-009: Scan source files for dead ADR references
        if self.config.adr.backlinking {
            validate_adr_references(&self.repo_root, &adr_dir, &adr_files, &self.config, &mut diagnostics);
        }

        // ADR01-010: Required headers in ADR files
        validate_adr_headers(&adr_dir, &adr_files, &mut diagnostics);

        diagnostics
    }
}

/// Collect `.md` filenames from the ADR directory (non-recursive).
fn collect_adr_files(adr_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(adr_dir) else {
        return Vec::new();
    };

    let mut files: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file()
                && e.file_name()
                    .to_string_lossy()
                    .to_lowercase()
                    .ends_with(".md")
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    files.sort();
    files
}

/// ADR01-002: Each .md file in the ADR directory must match the naming pattern.
fn validate_naming(
    adr_dir: &Path,
    adr_files: &[String],
    naming_regex: &Regex,
    config: &DocsConfig,
    diagnostics: &mut Diagnostics,
) {
    for filename in adr_files {
        // Skip structural files (README.md, CLAUDE.md, AGENTS.md)
        let lower = filename.to_lowercase();
        if lower == "readme.md" || lower == "claude.md" || lower == "agents.md" {
            continue;
        }

        if !naming_regex.is_match(filename) {
            let adr_path = adr_dir.join(filename);
            diagnostics.push(
                Diagnostic::error(
                    error_codes::INVALID_ADR_NAMING,
                    format!(
                        "ADR file '{filename}' does not match naming pattern '{}'",
                        config.adr.naming_pattern
                    ),
                )
                .with_path(&adr_path)
                .with_hint("Expected format: ADR-XXX-<adr-name>.md (e.g., ADR-001-initial-architecture.md)"),
            );
        }
    }
}

/// ADR01-003: Each ADR file must have front matter with `name` and `description`.
fn validate_frontmatter(
    adr_dir: &Path,
    adr_files: &[String],
    diagnostics: &mut Diagnostics,
) {
    for filename in adr_files {
        let lower = filename.to_lowercase();
        if lower == "readme.md" || lower == "claude.md" || lower == "agents.md" {
            continue;
        }

        let path = adr_dir.join(filename);
        match documentation::frontmatter::parse_frontmatter_from_file(&path) {
            Ok(Some(fm)) => {
                if fm.name().is_none() || fm.name().is_some_and(|n| n.is_empty()) {
                    diagnostics.push(
                        Diagnostic::error(
                            error_codes::MISSING_ADR_FRONTMATTER,
                            format!("ADR '{filename}' is missing required front matter field: name"),
                        )
                        .with_path(&path),
                    );
                }
                if fm.description().is_none() || fm.description().is_some_and(|d| d.is_empty()) {
                    diagnostics.push(
                        Diagnostic::error(
                            error_codes::MISSING_ADR_FRONTMATTER,
                            format!("ADR '{filename}' is missing required front matter field: description"),
                        )
                        .with_path(&path),
                    );
                }
            }
            Ok(None) => {
                diagnostics.push(
                    Diagnostic::error(
                        error_codes::MISSING_ADR_FRONTMATTER,
                        format!("ADR '{filename}' has no front matter block"),
                    )
                    .with_path(&path)
                    .with_hint("Add a --- delimited YAML block with 'name' and 'description' fields"),
                );
            }
            Err(e) => {
                diagnostics.push(
                    Diagnostic::error(
                        error_codes::MISSING_ADR_FRONTMATTER,
                        format!("Failed to parse front matter in '{filename}': {e}"),
                    )
                    .with_path(&path),
                );
            }
        }
    }
}

/// ADR01-004: For each keyword in `required_adr_keywords`, at least one file
/// matching `ADR-\d+-<keyword>\.md` must exist.
fn validate_required_keywords(
    adr_dir: &Path,
    adr_files: &[String],
    config: &DocsConfig,
    diagnostics: &mut Diagnostics,
) {
    for keyword in &config.adr.required_adr_keywords {
        let pattern = format!(r"^ADR-\d+-{keyword}\.md$");
        let re = match Regex::new(&pattern) {
            Ok(re) => re,
            Err(_) => {
                // Keyword contains regex-unsafe characters — try case-insensitive literal match
                let escaped = regex::escape(keyword);
                match Regex::new(&format!(r"(?i)^ADR-\d+-{escaped}\.md$")) {
                    Ok(re) => re,
                    Err(_) => continue,
                }
            }
        };

        let exists = adr_files.iter().any(|f| re.is_match(f));
        if !exists {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::MISSING_REQUIRED_ADR,
                    format!(
                        "Required ADR keyword '{keyword}' not satisfied — no file matching 'ADR-*-{keyword}.md' found in {}",
                        adr_dir.display()
                    ),
                )
                .with_path(adr_dir)
                .with_hint(format!(
                    "Create an ADR file like '{}'",
                    adr_dir.join(format!("ADR-001-{keyword}.md")).display()
                )),
            );
        }
    }
}

/// Keyword fragments that indicate the file contains ADR backlinking guidance.
const ADR_REFERENCE_KEYWORDS: &[&str] = &[
    "ADR-",
    "backlink",
    "reference",
    "comment block",
    "comment at the top",
];

/// ADR01-007/008: The ADR directory must contain CLAUDE.md and AGENTS.md with
/// guidance on how ADRs should be referenced in implementation files.
fn validate_adr_context_files(adr_dir: &Path, diagnostics: &mut Diagnostics) {
    for filename in ["CLAUDE.md", "AGENTS.md"] {
        let path = adr_dir.join(filename);
        if !path.exists() {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::MISSING_ADR_CONTEXT_FILE,
                    format!("ADR directory is missing {filename}"),
                )
                .with_path(&path)
                .with_hint(format!(
                    "Create {filename} in '{}' with guidance on referencing ADRs in code files",
                    adr_dir.display()
                )),
            );
            continue;
        }

        // Check that the file contains ADR referencing guidance
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };

        let has_guidance = ADR_REFERENCE_KEYWORDS
            .iter()
            .any(|kw| content.contains(kw));

        if !has_guidance {
            diagnostics.push(
                Diagnostic::warning(
                    error_codes::ADR_CONTEXT_MISSING_GUIDANCE,
                    format!(
                        "{filename} in ADR directory does not mention how to reference ADRs in code"
                    ),
                )
                .with_path(&path)
                .with_hint(
                    "Include guidance that implementation files should reference their governing ADR \
                     (e.g., a comment block at the top of the file like `// Implements ADR-001-security`)",
                ),
            );
        }
    }
}

// ─── Dead ADR reference scanning (ADR01-009) ─────────────────────────────

/// Extract ADR identifiers from text. Matches patterns like `ADR-001-security`
/// (the filename stem without `.md`).
fn extract_adr_references(content: &str) -> Vec<String> {
    let re = Regex::new(r"ADR-\d{3,}-[a-zA-Z0-9-]+").unwrap();
    re.find_iter(content)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Build a set of valid ADR stems (filename without `.md`) from the ADR directory.
fn adr_stems(adr_files: &[String]) -> HashSet<String> {
    adr_files
        .iter()
        .filter_map(|f| f.strip_suffix(".md").map(|s| s.to_string()))
        .collect()
}

/// File extensions to scan for ADR references.
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "py", "ts", "tsx", "js", "jsx", "go", "java", "kt", "rb", "sh",
    "yaml", "yml", "toml", "json", "md",
];

/// ADR01-009: Scan source files for ADR-XXX-name references and flag any
/// that don't correspond to an actual ADR file.
fn validate_adr_references(
    repo_root: &Path,
    adr_dir: &Path,
    adr_files: &[String],
    config: &DocsConfig,
    diagnostics: &mut Diagnostics,
) {
    let valid_stems = adr_stems(adr_files);
    if valid_stems.is_empty() {
        return;
    }

    let exclude: HashSet<&str> = config.readme.exclude_dirs.iter().map(|s| s.as_str()).collect();

    // Canonicalize the ADR dir for reliable comparison (handles macOS /var -> /private/var)
    let canonical_adr_dir = adr_dir.canonicalize().unwrap_or_else(|_| adr_dir.to_path_buf());

    for entry in WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() && e.depth() > 0 {
                let name = e.file_name().to_string_lossy();
                return !name.starts_with('.')
                    && !exclude.contains(name.as_ref());
            }
            true
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Only scan known source extensions
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !SOURCE_EXTENSIONS.contains(&ext) {
            continue;
        }

        // Skip files inside the ADR directory (they reference themselves)
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        if canonical_path.starts_with(&canonical_adr_dir) {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };

        let refs = extract_adr_references(&content);
        for adr_ref in refs {
            if !valid_stems.contains(&adr_ref) {
                diagnostics.push(
                    Diagnostic::warning(
                        error_codes::DEAD_ADR_REFERENCE,
                        format!("Reference to '{adr_ref}' does not match any ADR file"),
                    )
                    .with_path(path)
                    .with_hint(format!(
                        "No file '{adr_ref}.md' found in {}. Update or remove the stale reference.",
                        adr_dir.display()
                    )),
                );
            }
        }
    }
}

// ─── Required ADR headers (ADR01-010) ────────────────────────────────────

/// Headers that every ADR file MUST contain.
const REQUIRED_ADR_HEADERS: &[&str] = &[
    "## Context",
    "## Decision",
    "## Consequences",
];

/// ADR01-010: Each ADR file must contain required section headers.
fn validate_adr_headers(
    adr_dir: &Path,
    adr_files: &[String],
    diagnostics: &mut Diagnostics,
) {
    for filename in adr_files {
        let lower = filename.to_lowercase();
        if lower == "readme.md" || lower == "claude.md" || lower == "agents.md" {
            continue;
        }

        let path = adr_dir.join(filename);
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };

        for &header in REQUIRED_ADR_HEADERS {
            if !contains_header(&content, header) {
                diagnostics.push(
                    Diagnostic::warning(
                        error_codes::MISSING_ADR_HEADER,
                        format!("ADR '{filename}' is missing required section: {header}"),
                    )
                    .with_path(&path)
                    .with_hint(format!(
                        "ADR files should include {header} as part of the standard ADR structure"
                    )),
                );
            }
        }
    }
}

/// Check if content contains a markdown header, matching case-insensitively
/// and allowing for extra whitespace.
fn contains_header(content: &str, header: &str) -> bool {
    let prefix = header.split_once(' ').map(|(p, _)| p).unwrap_or(header);
    let text = header.split_once(' ').map(|(_, t)| t).unwrap_or("");

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(after_prefix) = trimmed.strip_prefix(prefix) {
            let rest = after_prefix.trim();
            if rest.eq_ignore_ascii_case(text) {
                return true;
            }
        }
    }
    false
}

// ─── Unit tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_are_unique() {
        let codes = vec![
            error_codes::MISSING_ADR_DIR,
            error_codes::INVALID_ADR_NAMING,
            error_codes::MISSING_ADR_FRONTMATTER,
            error_codes::MISSING_REQUIRED_ADR,
            error_codes::MISSING_ADR_BACKLINK,
            error_codes::INVALID_NAMING_REGEX,
            error_codes::MISSING_ADR_CONTEXT_FILE,
            error_codes::ADR_CONTEXT_MISSING_GUIDANCE,
            error_codes::DEAD_ADR_REFERENCE,
            error_codes::MISSING_ADR_HEADER,
        ];
        let unique: HashSet<_> = codes.iter().collect();
        assert_eq!(codes.len(), unique.len(), "error codes must be unique");
    }

    #[test]
    fn extract_adr_references_finds_patterns() {
        let content = r#"
            // Implements ADR-001-security
            // Also see ADR-042-testing for context
            let x = 42; // not an ADR reference
        "#;
        let refs = extract_adr_references(content);
        assert_eq!(refs, vec!["ADR-001-security", "ADR-042-testing"]);
    }

    #[test]
    fn extract_adr_references_ignores_short_numbers() {
        // ADR-01-foo has only 2 digits — should not match (minimum 3)
        let content = "// ADR-01-short";
        let refs = extract_adr_references(content);
        assert!(refs.is_empty());
    }

    #[test]
    fn extract_adr_references_handles_inline() {
        let content = "# see ADR-001-auth for rationale, and ADR-002-db for schema";
        let refs = extract_adr_references(content);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0], "ADR-001-auth");
        assert_eq!(refs[1], "ADR-002-db");
    }

    #[test]
    fn adr_stems_strips_extension() {
        let files = vec![
            "ADR-001-init.md".to_string(),
            "ADR-002-security.md".to_string(),
            "README.md".to_string(),
        ];
        let stems = adr_stems(&files);
        assert!(stems.contains("ADR-001-init"));
        assert!(stems.contains("ADR-002-security"));
        assert!(stems.contains("README")); // structural files still get stemmed, that's fine
        assert_eq!(stems.len(), 3);
    }

    #[test]
    fn contains_header_matches_exact() {
        let content = "# Title\n\n## Context\n\nSome context.\n\n## Decision\n\nWe decided.";
        assert!(contains_header(content, "## Context"));
        assert!(contains_header(content, "## Decision"));
        assert!(!contains_header(content, "## Consequences"));
    }

    #[test]
    fn contains_header_case_insensitive() {
        let content = "## context\n\nLowercase header.";
        assert!(contains_header(content, "## Context"));
    }

    #[test]
    fn contains_header_with_extra_whitespace() {
        let content = "##   Context  \n\nExtra spaces.";
        assert!(contains_header(content, "## Context"));
    }

    #[test]
    fn contains_header_rejects_partial() {
        let content = "## Contextual Analysis\n\nNot the same header.";
        assert!(!contains_header(content, "## Context"));
    }

    #[test]
    fn contains_header_rejects_wrong_level() {
        let content = "### Context\n\nWrong heading level.";
        assert!(!contains_header(content, "## Context"));
    }
}
