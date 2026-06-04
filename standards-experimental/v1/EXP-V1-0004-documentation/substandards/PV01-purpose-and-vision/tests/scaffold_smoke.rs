//! Scaffold smoke tests for the Purpose and Vision substandard.
//!
//! Real validator tests will land in the follow up PR. The placeholder tests
//! here pin the scaffolded public surface (error codes, required sections,
//! allowed statuses) so it cannot drift silently before the implementation
//! arrives.

use documentation_purpose_and_vision::{
    ALLOWED_STATUSES, DEFAULT_LOCATION, REQUIRED_SECTIONS, error_codes,
};

#[test]
fn error_codes_use_human_readable_scheme() {
    for code in [
        error_codes::DOCUMENT_MISSING,
        error_codes::FRONTMATTER_MISSING,
        error_codes::FRONTMATTER_FIELD_MISSING,
        error_codes::MISSING_PURPOSE_SECTION,
        error_codes::MISSING_VISION_SECTION,
        error_codes::MISSING_NON_GOALS_SECTION,
        error_codes::INVALID_STATUS,
        error_codes::SUPERSEDED_WITHOUT_POINTER,
        error_codes::DEPRECATED_ACTIVE,
    ] {
        assert!(
            code.starts_with("PV01-"),
            "code missing PV01- prefix: {code}"
        );
        // Check the SUFFIX (after the PV01- prefix) has no uppercase. The
        // earlier "starts_with prefix" disjunction was a no-op because the
        // prefix check itself is always true, so an unintended uppercase
        // in the verb phrase would slip through.
        let suffix = code
            .strip_prefix("PV01-")
            .expect("prefix check passed above");
        assert!(
            !suffix.contains(char::is_uppercase),
            "code suffix has unexpected uppercase: {code}"
        );
    }
}

#[test]
fn required_sections_are_the_spec_three() {
    assert_eq!(REQUIRED_SECTIONS, &["Purpose", "Vision", "Non-Goals"]);
}

#[test]
fn allowed_statuses_match_lifecycle_vocabulary() {
    assert_eq!(
        ALLOWED_STATUSES,
        &["proposed", "active", "deprecated", "superseded"]
    );
}

#[test]
fn default_location_is_docs_vision_md() {
    assert_eq!(DEFAULT_LOCATION, "docs/vision.md");
}
