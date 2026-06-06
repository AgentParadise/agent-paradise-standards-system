//! ADR (Architecture Decision Record) enforcement substandard.
//!
//! Validates ADR directory structure, file naming conventions (`ADR-XXX-<name>.md`),
//! required front matter, keyword-based required ADRs, and backlinking from
//! implementation files back to governing ADRs.

use apss_core::{Diagnostic, Diagnostics, diagnostics::Location};
use documentation::config::{self, DocsConfig};
use glob::Pattern;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Substandard identifier.
pub const SUBSTANDARD_ID: &str = "EXP-V1-0004.ADR01";

/// Diagnostic codes emitted by the ADR substandard.
///
/// Codes use the form `ADR01-<verb-phrase>` so the substandard prefix stays
/// visible while the suffix is descriptive in CLI output. Matches the operator
/// invariant for human-readable codes (example: `ADR01-dir-not-found`).
pub mod error_codes {
    /// ADR directory does not exist at the configured path.
    pub const MISSING_ADR_DIR: &str = "ADR01-dir-not-found";
    /// ADR filename does not match the configured naming pattern.
    pub const INVALID_ADR_NAMING: &str = "ADR01-invalid-naming";
    /// ADR file is missing required front matter fields (name, description).
    pub const MISSING_ADR_FRONTMATTER: &str = "ADR01-missing-frontmatter";
    /// A configured required-keyword ADR is missing from the directory.
    pub const MISSING_REQUIRED_ADR: &str = "ADR01-missing-required-keyword";
    /// The configured naming pattern is not a valid regex.
    pub const INVALID_NAMING_REGEX: &str = "ADR01-invalid-naming-regex";
    /// ADR directory is missing CLAUDE.md or AGENTS.md.
    pub const MISSING_ADR_CONTEXT_FILE: &str = "ADR01-missing-context-file";
    /// ADR CLAUDE.md or AGENTS.md does not document how code references ADRs.
    pub const ADR_CONTEXT_MISSING_GUIDANCE: &str = "ADR01-context-missing-guidance";
    /// Source file references an ADR token that has no matching ADR file.
    pub const UNKNOWN_ADR_REFERENCE: &str = "ADR01-unknown-reference";
    /// The old code name before precision accuracy was renamed in 2026-06.
    pub const DEAD_ADR_REFERENCE: &str = "ADR01-dead-reference";
    /// A docs.backlinking include glob is invalid.
    pub const INVALID_ADR_REFERENCE_GLOB: &str = "ADR01-invalid-reference-glob";
    /// A deprecated backlinking list key was used and is still honored.
    pub const BACKLINKING_FILE_TYPES_DEPRECATED: &str = "ADR01-backlinking-file-types-deprecated";
    /// ADR file is missing a required section header (Context, Decision, Consequences).
    pub const MISSING_ADR_HEADER: &str = "ADR01-missing-header";
    /// ADR file is missing the `status` field, or its value is not a valid lifecycle state.
    pub const INVALID_ADR_STATUS: &str = "ADR01-invalid-status";
    /// Source file references an ADR whose status is superseded or deprecated.
    pub const SUPERSEDED_ADR_REFERENCE: &str = "ADR01-superseded-reference";
    /// A scanned path is outside the repo root and was matched with fallback logic.
    pub const SCAN_PATH_OUTSIDE_REPO: &str = "ADR01-scan-path-outside-repo";
}

/// ADR validator that loads config and runs all ADR checks.
pub struct AdrValidator {
    config: DocsConfig,
    repo_root: PathBuf,
}

impl AdrValidator {
    /// Load the ADR validator from a repository root.
    /// Reads `APSS.yaml` for configuration.
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

        if self.config.disable || self.config.adr.disable {
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
                    "Create the directory at '{}' or configure docs.adr.directory in apss.yaml",
                    adr_dir.display()
                )),
            );
            return diagnostics;
        }

        // Compile naming pattern
        let naming_regex = match Regex::new(&format!("^{}$", self.config.adr.naming_pattern)) {
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
                    .with_hint("Check docs.adr.naming_pattern in apss.yaml"),
                );
                return diagnostics;
            }
        };

        // Collect ADR files
        let adr_files = collect_adr_files(&adr_dir);

        // ADR01-invalid-naming: Validate naming convention
        validate_naming(
            &adr_dir,
            &adr_files,
            &naming_regex,
            &self.config,
            &mut diagnostics,
        );

        // ADR01-missing-frontmatter / ADR01-invalid-status: Validate front matter
        validate_frontmatter(&adr_dir, &adr_files, &mut diagnostics);

        // ADR01-missing-required-keyword: Check required ADR keywords
        validate_required_keywords(
            &adr_dir,
            &adr_files,
            &naming_regex,
            &self.config,
            &mut diagnostics,
        );

        // ADR01-007/008: Check ADR context files (CLAUDE.md, AGENTS.md)
        validate_adr_context_files(&adr_dir, &mut diagnostics);

        // ADR01-009: Scan source files for dead ADR references
        if !self.config.backlinking.disable {
            validate_adr_references(
                &self.repo_root,
                &adr_dir,
                &adr_files,
                &self.config,
                &mut diagnostics,
            );
        }

        // ADR01-010: Required headers in ADR files
        validate_adr_headers(&adr_dir, &adr_files, &mut diagnostics);

        diagnostics
    }
}

/// Valid ADR status values per the Fowler ADR lifecycle.
const VALID_ADR_STATUSES: &[&str] = &["proposed", "accepted", "deprecated", "superseded"];

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

/// ADR01-003/011: Each ADR file must have front matter with `name`, `description`, and `status`.
fn validate_frontmatter(adr_dir: &Path, adr_files: &[String], diagnostics: &mut Diagnostics) {
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
                            format!(
                                "ADR '{filename}' is missing required front matter field: name"
                            ),
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
                // ADR01-011: status must exist and be a valid lifecycle value
                match fm.get("status") {
                    None | Some("") => {
                        diagnostics.push(
                            Diagnostic::error(
                                error_codes::INVALID_ADR_STATUS,
                                format!("ADR '{filename}' is missing required front matter field: status"),
                            )
                            .with_path(&path)
                            .with_hint(format!(
                                "Add a 'status' field with one of: {}",
                                VALID_ADR_STATUSES.join(", ")
                            )),
                        );
                    }
                    Some(status) => {
                        let normalized = status.to_lowercase();
                        if !VALID_ADR_STATUSES.contains(&normalized.as_str()) {
                            diagnostics.push(
                                Diagnostic::error(
                                    error_codes::INVALID_ADR_STATUS,
                                    format!("ADR '{filename}' has invalid status '{status}'"),
                                )
                                .with_path(&path)
                                .with_hint(format!(
                                    "Valid statuses: {}",
                                    VALID_ADR_STATUSES.join(", ")
                                )),
                            );
                        }
                    }
                }
            }
            Ok(None) => {
                diagnostics.push(
                    Diagnostic::error(
                        error_codes::MISSING_ADR_FRONTMATTER,
                        format!("ADR '{filename}' has no front matter block"),
                    )
                    .with_path(&path)
                    .with_hint(
                        "Add a --- delimited YAML block with 'name', 'description', and 'status' fields",
                    ),
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

/// For each keyword in `required_adr_keywords`, at least one ADR file whose
/// stem ends in `-<keyword>` (and which matches the configured naming pattern)
/// must exist. Emits `ADR01-missing-required-keyword` when missing.
fn validate_required_keywords(
    adr_dir: &Path,
    adr_files: &[String],
    naming_regex: &Regex,
    config: &DocsConfig,
    diagnostics: &mut Diagnostics,
) {
    for keyword in &config.adr.required_adr_keywords {
        let exists = adr_files
            .iter()
            .any(|f| config::adr_filename_has_keyword(naming_regex, f, keyword));
        if !exists {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::MISSING_REQUIRED_ADR,
                    format!(
                        "Required ADR keyword '{keyword}' not satisfied - no file matching the configured naming pattern with stem ending '-{keyword}' found in {}",
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

/// Lowercase keyword fragments that indicate the file contains ADR backlinking
/// guidance. Matched against a lowercased copy of the file so casing variants
/// like `Reference` and `BACKLINK` do not produce false `ADR01-context-missing-guidance`
/// warnings.
const ADR_REFERENCE_KEYWORDS: &[&str] = &["adr-", "comment", "backlink", "reference in code"];

/// The ADR directory must contain CLAUDE.md and AGENTS.md with guidance on how
/// ADRs should be referenced in implementation files. Emits
/// `ADR01-missing-context-file` and `ADR01-context-missing-guidance`.
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

        // Check that the file contains ADR referencing guidance.
        // Normalise to lowercase so casing variants do not produce false warnings.
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let lowered = content.to_ascii_lowercase();

        let has_adr_prefix = lowered.contains("adr-");
        let has_guidance_fragment = ADR_REFERENCE_KEYWORDS.iter().any(|kw| lowered.contains(kw));
        let has_guidance = has_adr_prefix && has_guidance_fragment;

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

// ─── ADR reference scanning (ADR01-unknown-reference) ────────────────

/// Extract ADR identifiers from text with line numbers, where line is 1-indexed.
fn extract_adr_references_with_lines(
    content: &str,
    reference_re: &Regex,
    stem_re: &Regex,
    adjacent_splitter: &Option<Regex>,
) -> Vec<(String, usize)> {
    let mut refs = Vec::new();
    for (line_number, line) in content.lines().enumerate() {
        let line_number = line_number + 1;
        for captures in reference_re.captures_iter(line) {
            let Some(m) = captures.get(1) else {
                continue;
            };
            for reference in split_adr_references(m.as_str(), stem_re, adjacent_splitter) {
                refs.push((reference, line_number));
            }
        }
    }
    refs
}

/// Build the regex used to detect ADR references.
///
/// The matcher uses left-boundary protection for embedded identifiers and a
/// right-boundary check so references ending in punctuation are still accepted
/// while most false positives are avoided.
fn compile_reference_matcher(stem_re: &Regex) -> Result<Regex, regex::Error> {
    Regex::new(&format!(
        r"(?:^|[^A-Za-z0-9-])({})(?:$|[^A-Za-z0-9-])",
        stem_re.as_str()
    ))
}

/// Build a splitter pattern that finds the start of a second reference that is
/// glued to a previous one with only a hyphen, for example
/// `ADR-001-foo-ADR-002-bar`.
fn compile_adjacent_splitter(stem_re: &str) -> Option<Regex> {
    let Some(split_prefix_end) = stem_re.find(r"\d") else {
        return None;
    };
    let prefix = stem_re[..split_prefix_end].trim();
    if prefix.is_empty() {
        return None;
    }
    Regex::new(&format!(r"-{}[0-9]", regex::escape(prefix))).ok()
}

/// Split a raw matched ADR-like token into one or more references.
///
/// This function handles adjacent references by splitting at a hyphen followed
/// by a second prefix and ensures trailing hyphens are trimmed to avoid false
/// negatives for valid references followed by punctuation.
fn split_adr_references(
    reference: &str,
    stem_re: &Regex,
    adjacent_splitter: &Option<Regex>,
) -> Vec<String> {
    let mut remaining = reference;
    let mut refs = Vec::new();

    while let Some(splitter) = adjacent_splitter.as_ref() {
        if let Some(m) = splitter.find(remaining) {
            let head = remaining[..m.start()].trim_end_matches('-');
            if head.is_empty() {
                remaining = &remaining[m.start() + 1..];
                continue;
            }

            if stem_re.is_match(head) {
                refs.push(head.to_string());
                remaining = &remaining[m.start() + 1..];
                continue;
            }
        }
        break;
    }

    let tail = remaining.trim_end_matches('-');
    if stem_re.is_match(tail) {
        refs.push(tail.to_string());
    }

    refs
}

/// Build a set of valid ADR stems (filename without `.md`) from the ADR directory.
fn adr_stems(adr_files: &[String]) -> HashSet<String> {
    adr_files
        .iter()
        .filter_map(|f| f.strip_suffix(".md").map(|s| s.to_string()))
        .collect()
}

/// Build a map of ADR stem → status (lowercase) by reading frontmatter.
fn adr_statuses(adr_dir: &Path, adr_files: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for filename in adr_files {
        let lower = filename.to_lowercase();
        if lower == "readme.md" || lower == "claude.md" || lower == "agents.md" {
            continue;
        }
        let Some(stem) = filename.strip_suffix(".md") else {
            continue;
        };
        let path = adr_dir.join(filename);
        if let Ok(Some(fm)) = documentation::frontmatter::parse_frontmatter_from_file(&path) {
            if let Some(status) = fm.get("status") {
                map.insert(stem.to_string(), status.to_lowercase());
            }
        }
    }
    map
}

fn compile_backlinking_patterns(
    repo_root: &Path,
    config: &DocsConfig,
    diagnostics: &mut Diagnostics,
) -> Vec<Pattern> {
    let mut raw_patterns = config
        .backlinking
        .scan
        .clone()
        .unwrap_or_else(config::default_backlinking_scan);

    let deprecated_file_patterns = config
        .backlinking
        .file_types
        .iter()
        .map(|ext| format!("**/*.{}", ext));
    raw_patterns.extend(deprecated_file_patterns);

    let mut patterns = Vec::new();
    if !config.backlinking.file_types.is_empty() {
        diagnostics.push(
            Diagnostic::warning(
                error_codes::BACKLINKING_FILE_TYPES_DEPRECATED,
                "docs.backlinking.file_types is deprecated",
            )
            .with_path(repo_root)
            .with_hint(
                "Use docs.backlinking.scan instead. The deprecated key is still honored for now",
            ),
        );
    }

    for pattern in raw_patterns {
        match Pattern::new(&pattern) {
            Ok(compiled) => patterns.push(compiled),
            Err(e) => {
                diagnostics.push(
                    Diagnostic::error(
                        error_codes::INVALID_ADR_REFERENCE_GLOB,
                        format!(
                            "Invalid ADR reference scan glob '{pattern}' in docs.backlinking.scan: {e}"
                        ),
                    )
                    .with_path(repo_root)
                    .with_hint(
                        "Update docs.backlinking.scan to valid glob syntax. Other patterns still apply.",
                    ),
            );
            }
        }
    }
    patterns
}

/// ADR01-009/012: Scan source files for ADR-XXX-name references and flag any
/// that don't correspond to an actual ADR file (unknown), or reference a
/// superseded/deprecated ADR (012).
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

    // Build the reference-extraction regex from the configured naming pattern so
    // projects that customise the prefix still get backlink scanning.
    // On regex error, fall back to skipping (ADR01-invalid-naming-regex would
    // have been emitted earlier from the same pattern).
    let stem_pattern = config::adr_stem_pattern_from_naming(&config.adr.naming_pattern);
    let stem_re = match Regex::new(&stem_pattern) {
        Ok(re) => re,
        Err(_) => return,
    };
    let reference_re = match compile_reference_matcher(&stem_re) {
        Ok(re) => re,
        Err(_) => return,
    };
    let adjacent_splitter = compile_adjacent_splitter(&stem_pattern);

    let statuses = adr_statuses(adr_dir, adr_files);
    let patterns = compile_backlinking_patterns(repo_root, config, diagnostics);
    if patterns.is_empty() {
        return;
    }
    let exclude: HashSet<&str> = config
        .readme
        .exclude_dirs
        .iter()
        .map(|s| s.as_str())
        .collect();

    // Canonicalize roots once for reliable comparison (handles macOS /var -> /private/var)
    let canonical_repo = repo_root
        .canonicalize()
        .unwrap_or_else(|_| repo_root.to_path_buf());
    let canonical_adr_dir = adr_dir
        .canonicalize()
        .unwrap_or_else(|_| adr_dir.to_path_buf());
    let mut scan_path_outside_repo_warning_emitted = false;
    let mut seen_references: HashSet<(PathBuf, usize, String)> = HashSet::new();

    for entry in WalkDir::new(&canonical_repo)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() && e.depth() > 0 {
                let name = e.file_name().to_string_lossy();
                return !name.starts_with('.') && !exclude.contains(name.as_ref());
            }
            true
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let rel_str = match path.strip_prefix(&canonical_repo) {
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => {
                if !scan_path_outside_repo_warning_emitted {
                    scan_path_outside_repo_warning_emitted = true;
                    diagnostics.push(
                        Diagnostic::warning(
                            error_codes::SCAN_PATH_OUTSIDE_REPO,
                            format!(
                                "Path '{}' is outside '{}' and matched as fallback text",
                                path.display(),
                                canonical_repo.display()
                            ),
                        )
                        .with_path(path)
                        .with_hint(
                            "The path is being matched as an absolute path against scan globs. Audit scan coverage if this is unexpected.",
                        ),
                    );
                }
                path.to_string_lossy().replace('\\', "/")
            }
        };
        if !patterns.iter().any(|pattern| pattern.matches(&rel_str)) {
            continue;
        }

        // Skip files inside the ADR directory (they reference themselves).
        // Walking from canonical_repo means paths are already canonical - no per-file canonicalize.
        if path.starts_with(&canonical_adr_dir) {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };

        let refs = extract_adr_references_with_lines(
            &content,
            &reference_re,
            &stem_re,
            &adjacent_splitter,
        );
        for (adr_ref, line_number) in refs {
            if !seen_references.insert((path.to_path_buf(), line_number, adr_ref.clone())) {
                continue;
            }
            if !valid_stems.contains(&adr_ref) {
                diagnostics.push(
                    Diagnostic::error(
                        error_codes::UNKNOWN_ADR_REFERENCE,
                        format!(
                            "Reference to '{adr_ref}' in {}:{line_number} does not match any ADR file",
                            path.display()
                        ),
                    )
                    .with_location(Location {
                        path: Some(path.to_path_buf()),
                        line: Some(line_number),
                        column: None,
                    })
                    .with_hint(format!(
                        "No file '{adr_ref}.md' found in {}. Update or remove the stale reference.",
                        adr_dir.display()
                    )),
                );
            } else if let Some(status) = statuses.get(&adr_ref) {
                if status == "superseded" || status == "deprecated" {
                    diagnostics.push(
                        Diagnostic::warning(
                            error_codes::SUPERSEDED_ADR_REFERENCE,
                            format!(
                                "Reference to '{adr_ref}' points to a {status} ADR"
                            ),
                        )
                        .with_path(path)
                        .with_hint(format!(
                            "ADR '{adr_ref}.md' has status '{status}'. Update this reference to the current ADR.",
                        )),
                    );
                }
            }
        }
    }
}

// ─── Required ADR headers (ADR01-010) ────────────────────────────────────

/// Headers that every ADR file MUST contain.
const REQUIRED_ADR_HEADERS: &[&str] = &["## Context", "## Decision", "## Consequences"];

/// ADR01-010: Each ADR file must contain required section headers.
fn validate_adr_headers(adr_dir: &Path, adr_files: &[String], diagnostics: &mut Diagnostics) {
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
            error_codes::BACKLINKING_FILE_TYPES_DEPRECATED,
            error_codes::SCAN_PATH_OUTSIDE_REPO,
            error_codes::INVALID_ADR_NAMING,
            error_codes::MISSING_ADR_FRONTMATTER,
            error_codes::MISSING_REQUIRED_ADR,
            error_codes::INVALID_NAMING_REGEX,
            error_codes::MISSING_ADR_CONTEXT_FILE,
            error_codes::ADR_CONTEXT_MISSING_GUIDANCE,
            error_codes::UNKNOWN_ADR_REFERENCE,
            error_codes::INVALID_ADR_REFERENCE_GLOB,
            error_codes::MISSING_ADR_HEADER,
            error_codes::INVALID_ADR_STATUS,
            error_codes::SUPERSEDED_ADR_REFERENCE,
        ];
        let unique: HashSet<_> = codes.iter().collect();
        assert_eq!(codes.len(), unique.len(), "error codes must be unique");
    }

    fn reference_inputs(stem_pattern: &str) -> (Regex, Regex, Option<Regex>) {
        let stem_re = Regex::new(stem_pattern).expect("ADR stem pattern must compile");
        let reference_re =
            compile_reference_matcher(&stem_re).expect("reference matcher must compile");
        let adjacent_splitter = compile_adjacent_splitter(stem_pattern);
        (stem_re, reference_re, adjacent_splitter)
    }

    fn default_reference_inputs() -> (Regex, Regex, Option<Regex>) {
        let stem_pattern =
            config::adr_stem_pattern_from_naming(&config::default_adr_filename_pattern());
        reference_inputs(&stem_pattern)
    }

    fn custom_reference_inputs(pattern: &str) -> (Regex, Regex, Option<Regex>) {
        let stem_pattern = config::adr_stem_pattern_from_naming(pattern);
        reference_inputs(&stem_pattern)
    }

    fn extract_adr_references(
        content: &str,
        reference_re: &Regex,
        stem_re: &Regex,
        adjacent_splitter: &Option<Regex>,
    ) -> Vec<String> {
        extract_adr_references_with_lines(content, reference_re, stem_re, adjacent_splitter)
            .into_iter()
            .map(|(r, _)| r)
            .collect()
    }

    fn extract_adr_references_with_line_numbers(
        content: &str,
        reference_re: &Regex,
        stem_re: &Regex,
        adjacent_splitter: &Option<Regex>,
    ) -> Vec<(String, usize)> {
        extract_adr_references_with_lines(content, reference_re, stem_re, adjacent_splitter)
    }

    #[test]
    fn extract_adr_references_finds_patterns() {
        let (stem_re, reference_re, adjacent_splitter) = default_reference_inputs();
        let content = r#"
            // Implements ADR-001-security
            // Also see ADR-042-testing for context
            let x = 42; // not an ADR reference
        "#;
        let refs = extract_adr_references(content, &reference_re, &stem_re, &adjacent_splitter);
        assert_eq!(refs, vec!["ADR-001-security", "ADR-042-testing"]);
    }

    #[test]
    fn extract_adr_references_ignores_short_numbers() {
        // ADR-01-foo has only 2 digits - should not match (minimum 3)
        let (stem_re, reference_re, adjacent_splitter) = default_reference_inputs();
        let content = "// ADR-01-short";
        let refs = extract_adr_references(content, &reference_re, &stem_re, &adjacent_splitter);
        assert!(refs.is_empty());
    }

    #[test]
    fn extract_adr_references_handles_inline() {
        let content = "# see ADR-001-auth for rationale, and ADR-002-db for schema";
        let (stem_re, reference_re, adjacent_splitter) = default_reference_inputs();
        let refs = extract_adr_references(content, &reference_re, &stem_re, &adjacent_splitter);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0], "ADR-001-auth");
        assert_eq!(refs[1], "ADR-002-db");
    }

    #[test]
    fn extract_adr_references_splits_adjacent_references() {
        let (stem_re, reference_re, adjacent_splitter) = default_reference_inputs();
        let content = "ADR-001-foo-ADR-002-bar";
        let refs = extract_adr_references(content, &reference_re, &stem_re, &adjacent_splitter);
        assert_eq!(refs, vec!["ADR-001-foo", "ADR-002-bar"]);
    }

    #[test]
    fn extract_adr_references_handles_trailing_hyphen() {
        let (stem_re, reference_re, adjacent_splitter) = default_reference_inputs();
        let content = "ADR-001-foo-";
        let refs = extract_adr_references(content, &reference_re, &stem_re, &adjacent_splitter);
        assert_eq!(refs, vec!["ADR-001-foo"]);
    }

    #[test]
    fn extract_adr_references_ignores_embedded_prefix() {
        let (stem_re, reference_re, adjacent_splitter) = default_reference_inputs();
        let content = "BADR-001-embedded ADR-001-real";
        let refs = extract_adr_references(content, &reference_re, &stem_re, &adjacent_splitter);
        assert_eq!(refs, vec!["ADR-001-real"]);
    }

    #[test]
    fn extract_adr_references_follows_custom_naming() {
        // A project that customises naming_pattern to `DEC-...` should have its
        // references picked up by the stem regex derived from that pattern.
        let (stem_re, reference_re, adjacent_splitter) =
            custom_reference_inputs(r"DEC-\d{3,5}-[a-zA-Z0-9-]+\.md");
        let content = "// Implements DEC-042-payments\n// Old ref ADR-001-auth";
        let refs = extract_adr_references(content, &reference_re, &stem_re, &adjacent_splitter);
        assert_eq!(refs, vec!["DEC-042-payments"]);
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

    #[test]
    fn valid_statuses_accepted() {
        for status in VALID_ADR_STATUSES {
            assert!(
                VALID_ADR_STATUSES.contains(status),
                "'{status}' should be valid"
            );
        }
    }

    #[test]
    fn invalid_status_rejected() {
        assert!(!VALID_ADR_STATUSES.contains(&"draft"));
        assert!(!VALID_ADR_STATUSES.contains(&"approved"));
        assert!(!VALID_ADR_STATUSES.contains(&""));
    }

    #[test]
    fn extract_adr_references_accepts_five_digit_numbers() {
        let (stem_re, reference_re, adjacent_splitter) = default_reference_inputs();
        let content = "// ADR-99999-max-digits";
        let refs = extract_adr_references(content, &reference_re, &stem_re, &adjacent_splitter);
        assert_eq!(refs, vec!["ADR-99999-max-digits"]);
    }

    #[test]
    fn extract_adr_references_rejects_six_digit_numbers() {
        let (stem_re, reference_re, adjacent_splitter) = default_reference_inputs();
        let content = "// ADR-123456-too-long";
        let refs = extract_adr_references(content, &reference_re, &stem_re, &adjacent_splitter);
        assert!(refs.is_empty());
    }

    #[test]
    fn extract_adr_references_tracks_line_numbers() {
        let (stem_re, reference_re, adjacent_splitter) = default_reference_inputs();
        let content = "\n// first\nADR-001-security\ntext ADR-002-testing\n";
        let refs = extract_adr_references_with_line_numbers(
            content,
            &reference_re,
            &stem_re,
            &adjacent_splitter,
        );
        assert_eq!(
            refs,
            vec![
                ("ADR-001-security".to_string(), 3),
                ("ADR-002-testing".to_string(), 4)
            ]
        );
    }
}
