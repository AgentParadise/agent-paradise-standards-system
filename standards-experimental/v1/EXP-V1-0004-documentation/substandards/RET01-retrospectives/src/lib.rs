//! Retrospectives (EXP-V1-0004.RET01) substandard.
//!
//! This crate is scaffolded. The full validator implementation lands in a
//! follow up PR. The normative contract lives in `docs/01_spec.md`; this
//! module exposes the diagnostic code constants and shared defaults so the
//! parent crate and downstream tooling can reference them today.

use aps_core::{Diagnostic, Diagnostics};
use std::path::Path;

/// Diagnostic codes emitted by the Retrospectives substandard.
///
/// See `docs/01_spec.md` Section 9 for descriptions.
pub mod error_codes {
    pub const DIR_NOT_FOUND: &str = "RET01-dir-not-found";
    pub const NAMING_MISMATCH: &str = "RET01-naming-mismatch";
    pub const INVALID_NAMING_REGEX: &str = "RET01-invalid-naming-regex";
    pub const FRONTMATTER_MISSING: &str = "RET01-frontmatter-missing";
    pub const FRONTMATTER_FIELD_MISSING: &str = "RET01-frontmatter-field-missing";
    pub const INVALID_STATUS: &str = "RET01-invalid-status";
    pub const MISSING_SECTION: &str = "RET01-missing-section";
    pub const HISTORY_MODIFIED: &str = "RET01-history-modified";
}

/// Allowed lifecycle status values for retrospectives.
pub const ALLOWED_STATUSES: &[&str] = &["proposed", "active", "deprecated", "superseded"];

/// Required H2 section titles in a retrospective.
pub const REQUIRED_SECTIONS: &[&str] = &["Context", "What Went Well", "What Did Not", "Followups"];

/// Default directory relative to the repo root.
pub const DEFAULT_DIRECTORY: &str = "docs/retrospectives";

/// Default naming pattern (3 to 5 digit numeric block, kebab slug, `.md`).
pub const DEFAULT_NAMING_PATTERN: &str = r"RET-\d{3,5}-[a-zA-Z0-9-]+\.md";

/// Sentinel marking the start of an append-only region inside a retrospective.
pub const APPEND_REGION_SENTINEL: &str = "<!-- RET01:append-only -->";

/// Scaffolded validator entry point.
///
/// Returns an empty diagnostic set today. The implementation lands in a
/// follow up PR per the spec.
pub fn validate(_repo_root: &Path) -> Diagnostics {
    Diagnostics::new()
}

/// Placeholder factory for the canonical `RET01-dir-not-found` diagnostic so
/// downstream code can already assert against the contract.
#[doc(hidden)]
pub fn dir_not_found_diagnostic(dir: &Path) -> Diagnostic {
    Diagnostic::error(
        error_codes::DIR_NOT_FOUND,
        format!("Retrospective directory not found: {}", dir.display()),
    )
    .with_path(dir)
    .with_hint(format!(
        "Create the directory at '{}' or set docs.retrospectives.disable = true in .apss/config.toml",
        dir.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_statuses_match_parent_vocabulary() {
        assert_eq!(
            ALLOWED_STATUSES,
            &["proposed", "active", "deprecated", "superseded"]
        );
    }

    #[test]
    fn required_sections_present_in_order() {
        assert_eq!(
            REQUIRED_SECTIONS,
            &["Context", "What Went Well", "What Did Not", "Followups"]
        );
    }

    #[test]
    fn default_naming_pattern_compiles_and_accepts_expected_range() {
        let re = regex::Regex::new(&format!("^{DEFAULT_NAMING_PATTERN}$")).unwrap();
        assert!(re.is_match("RET-001-q1-launch.md"));
        assert!(re.is_match("RET-99999-late-stage.md"));
        assert!(!re.is_match("RET-01-too-short.md"));
        assert!(!re.is_match("RET-123456-too-long.md"));
        assert!(!re.is_match("RET-001-q1-launch")); // no .md
    }

    #[test]
    fn validate_returns_empty_today() {
        let temp = tempfile::tempdir().unwrap();
        let report = validate(temp.path());
        assert_eq!(report.len(), 0);
    }
}
