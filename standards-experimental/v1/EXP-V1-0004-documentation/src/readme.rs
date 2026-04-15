//! README.md and context file (CLAUDE.md, AGENTS.md) enforcement.
//!
//! Validates that every directory under the docs root has required
//! structural files with proper index sections.

use crate::config::DocsConfig;
use crate::error_codes;
use crate::index::{self, IndexIssue};
use aps_core::{Diagnostic, Diagnostics};
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

/// Validate README, CLAUDE.md, AGENTS.md presence and index freshness.
pub fn validate_readmes(repo_root: &Path, docs_config: &DocsConfig, diagnostics: &mut Diagnostics) {
    let readme_config = &docs_config.readme;
    if !readme_config.enabled {
        return;
    }

    let docs_root = crate::config::resolve_docs_root(repo_root, docs_config);
    if !docs_root.is_dir() {
        diagnostics.push(
            Diagnostic::error(
                error_codes::MISSING_README,
                format!("Docs root directory not found: {}", docs_root.display()),
            )
            .with_path(&docs_root)
            .with_hint(format!(
                "Create '{}' or configure docs.root in .apss/config.toml",
                docs_root.display()
            )),
        );
        return;
    }

    let exclude_set: HashSet<&str> = readme_config
        .exclude_dirs
        .iter()
        .map(|s| s.as_str())
        .collect();

    let mut walkdir = WalkDir::new(&docs_root).follow_links(false);
    // Use WalkDir's max_depth to prune traversal early.
    // max_depth == -1 means unlimited; WalkDir depth 0 == docs_root itself,
    // so we add 1 to translate from "directory nesting levels" to WalkDir depth.
    if readme_config.max_depth >= 0 {
        walkdir = walkdir.max_depth(readme_config.max_depth as usize + 1);
    }

    let walker = walkdir.into_iter().filter_entry(|entry| {
        if !entry.file_type().is_dir() {
            return true;
        }
        let name = entry.file_name().to_string_lossy();
        // Skip hidden directories and excluded dirs
        !name.starts_with('.') && !exclude_set.contains(name.as_ref())
    });

    for entry in walker {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_dir() {
            continue;
        }

        let dir = entry.path();

        // DOC02-001: README.md must exist
        let readme_path = dir.join("README.md");
        if !readme_path.exists() {
            diagnostics.push(
                Diagnostic::error(
                    error_codes::MISSING_README,
                    format!("Missing README.md in {}", dir.display()),
                )
                .with_path(dir),
            );
        } else if docs_config.index.enabled {
            // Validate index freshness
            validate_readme_index(&readme_path, dir, docs_config, diagnostics);
        }

        // DOC02-002: CLAUDE.md (warning)
        if docs_config.context_files.require_claude_md && !dir.join("CLAUDE.md").exists() {
            diagnostics.push(
                Diagnostic::warning(
                    error_codes::MISSING_CLAUDE_MD,
                    format!("Missing CLAUDE.md in {}", dir.display()),
                )
                .with_path(dir)
                .with_hint("Create a CLAUDE.md that points to README.md for context"),
            );
        }

        // DOC02-003: AGENTS.md (warning)
        if docs_config.context_files.require_agents_md && !dir.join("AGENTS.md").exists() {
            diagnostics.push(
                Diagnostic::warning(
                    error_codes::MISSING_AGENTS_MD,
                    format!("Missing AGENTS.md in {}", dir.display()),
                )
                .with_path(dir)
                .with_hint("Create an AGENTS.md that points to README.md for context"),
            );
        }
    }
}

/// Validate that a README's `## Index` section is up to date.
fn validate_readme_index(
    readme_path: &Path,
    dir: &Path,
    docs_config: &DocsConfig,
    diagnostics: &mut Diagnostics,
) {
    let Ok(content) = std::fs::read_to_string(readme_path) else {
        return;
    };

    match index::validate_index(&content, dir, &docs_config.index) {
        Ok(validation) => {
            if !validation.is_valid {
                let code = match validation.reason {
                    IndexIssue::Missing => error_codes::MISSING_INDEX,
                    IndexIssue::Stale => error_codes::STALE_INDEX,
                    IndexIssue::None => return,
                };
                let msg = match validation.reason {
                    IndexIssue::Missing => {
                        format!("README.md at {} is missing ## Index section", dir.display())
                    }
                    IndexIssue::Stale => {
                        format!(
                            "README.md at {} has a stale ## Index section",
                            dir.display()
                        )
                    }
                    IndexIssue::None => return,
                };
                diagnostics.push(
                    Diagnostic::warning(code, msg)
                        .with_path(readme_path)
                        .with_hint("Run `aps run docs index --write` to regenerate"),
                );
            }
        }
        Err(_) => {
            // Non-fatal: index validation failed, skip
        }
    }
}
