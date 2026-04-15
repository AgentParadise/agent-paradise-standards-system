//! Root context file validation.
//!
//! Validates that the repository root has CLAUDE.md and AGENTS.md
//! files that reference the documentation location.

use crate::config::DocsConfig;
use crate::error_codes;
use aps_core::{Diagnostic, Diagnostics};
use std::path::Path;

/// Validate root-level context files (CLAUDE.md, AGENTS.md).
pub fn validate_root_context(
    repo_root: &Path,
    docs_config: &DocsConfig,
    diagnostics: &mut Diagnostics,
) {
    let root_config = &docs_config.root_context;
    if !root_config.enabled {
        return;
    }

    // DOC03-001: Root CLAUDE.md must exist
    let claude_path = repo_root.join("CLAUDE.md");
    if !claude_path.exists() {
        diagnostics.push(
            Diagnostic::error(
                error_codes::MISSING_ROOT_CLAUDE_MD,
                "Missing CLAUDE.md at repository root",
            )
            .with_path(repo_root)
            .with_hint("Create a CLAUDE.md that provides AI context and references documentation location"),
        );
    } else {
        // DOC03-003: Should reference docs location
        validate_docs_reference(&claude_path, &root_config.docs_reference_pattern, diagnostics);
    }

    // DOC03-002: Root AGENTS.md must exist
    let agents_path = repo_root.join("AGENTS.md");
    if !agents_path.exists() {
        diagnostics.push(
            Diagnostic::error(
                error_codes::MISSING_ROOT_AGENTS_MD,
                "Missing AGENTS.md at repository root",
            )
            .with_path(repo_root)
            .with_hint("Create an AGENTS.md that provides agent operational context"),
        );
    }
}

/// Check that a file contains a reference to the docs location.
fn validate_docs_reference(
    file_path: &Path,
    pattern: &str,
    diagnostics: &mut Diagnostics,
) {
    let Ok(content) = std::fs::read_to_string(file_path) else {
        return;
    };

    if !content.contains(pattern) {
        diagnostics.push(
            Diagnostic::warning(
                error_codes::MISSING_DOCS_REFERENCE,
                format!(
                    "{} does not reference documentation location (expected '{}' to appear)",
                    file_path.display(),
                    pattern
                ),
            )
            .with_path(file_path)
            .with_hint("Add a reference to your documentation directory so agents can find it"),
        );
    }
}
