//! Purpose and Vision (EXP-V1-0004.PV01) substandard.
//!
//! This crate is scaffolded. The full validator implementation lands in a
//! follow up PR. The normative contract is documented in
//! `docs/01_spec.md`; this module exposes the diagnostic code constants so
//! the parent crate's tests can reference them today.

use apss_core::{Diagnostic, Diagnostics};
use std::path::Path;

/// Diagnostic codes emitted by the Purpose and Vision substandard.
///
/// See `docs/01_spec.md` Section 8 for descriptions.
pub mod error_codes {
    pub const DOCUMENT_MISSING: &str = "PV01-document-missing";
    pub const FRONTMATTER_MISSING: &str = "PV01-frontmatter-missing";
    pub const FRONTMATTER_FIELD_MISSING: &str = "PV01-frontmatter-field-missing";
    pub const MISSING_MISSION_SECTION: &str = "PV01-missing-mission-section";
    pub const MISSING_VISION_SECTION: &str = "PV01-missing-vision-section";
    pub const MISSING_POSITION_SECTION: &str = "PV01-missing-position-section";
    pub const INVALID_STATUS: &str = "PV01-invalid-status";
    pub const SUPERSEDED_WITHOUT_POINTER: &str = "PV01-superseded-without-pointer";
    pub const DEPRECATED_ACTIVE: &str = "PV01-deprecated-active";
}

/// Allowed lifecycle status values for the Purpose and Vision document.
pub const ALLOWED_STATUSES: &[&str] = &["proposed", "active", "deprecated", "superseded"];

/// Required H2 sections in the Purpose and Vision document.
pub const REQUIRED_SECTIONS: &[&str] = &["Mission", "Vision", "Position"];

/// Default location relative to the repo root.
pub const DEFAULT_LOCATION: &str = "docs/north-star.md";

/// Scaffolded validator entry point.
///
/// Returns an empty diagnostic set today. The implementation lands in a
/// follow up PR per the spec.
pub fn validate(_repo_root: &Path) -> Diagnostics {
    Diagnostics::new()
}

/// Build a placeholder diagnostic for the spec's contract surface so tests
/// and downstream tooling can assert against the exposed codes today.
#[doc(hidden)]
pub fn document_missing_diagnostic(location: &Path) -> Diagnostic {
    Diagnostic::error(
        error_codes::DOCUMENT_MISSING,
        format!(
            "Purpose and Vision document not found: {}",
            location.display()
        ),
    )
    .with_path(location)
    .with_hint(format!(
        "Create the file at '{}' or set docs.north-star.disable = true in apss.yaml",
        location.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_statuses_are_lowercase_unique() {
        let mut seen = std::collections::HashSet::new();
        for s in ALLOWED_STATUSES {
            assert_eq!(*s, s.to_lowercase());
            assert!(seen.insert(*s), "duplicate status: {s}");
        }
    }

    #[test]
    fn required_sections_present() {
        assert_eq!(REQUIRED_SECTIONS, &["Mission", "Vision", "Position"]);
    }

    #[test]
    fn validate_returns_empty_today() {
        let temp = tempfile::tempdir().unwrap();
        let report = validate(temp.path());
        assert_eq!(report.len(), 0);
    }
}
