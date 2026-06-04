//! Scaffold smoke tests for the Retrospectives substandard.
//!
//! Real validator tests will land in the follow up PR. These pin the public
//! surface (error codes, required sections, default pattern) so it cannot
//! drift before the implementation arrives.

use documentation_retrospectives::{
    error_codes, ALLOWED_STATUSES, APPEND_REGION_SENTINEL, DEFAULT_DIRECTORY,
    DEFAULT_NAMING_PATTERN, REQUIRED_SECTIONS,
};

#[test]
fn error_codes_use_human_readable_scheme() {
    for code in [
        error_codes::DIR_NOT_FOUND,
        error_codes::NAMING_MISMATCH,
        error_codes::INVALID_NAMING_REGEX,
        error_codes::FRONTMATTER_MISSING,
        error_codes::FRONTMATTER_FIELD_MISSING,
        error_codes::INVALID_STATUS,
        error_codes::MISSING_SECTION,
        error_codes::HISTORY_MODIFIED,
    ] {
        assert!(code.starts_with("RET01-"), "code missing RET01- prefix: {code}");
    }
}

#[test]
fn default_directory_is_docs_retrospectives() {
    assert_eq!(DEFAULT_DIRECTORY, "docs/retrospectives");
}

#[test]
fn default_pattern_is_three_to_five_digit() {
    assert!(DEFAULT_NAMING_PATTERN.contains("d{3,5}"));
}

#[test]
fn append_region_sentinel_is_html_comment() {
    assert!(APPEND_REGION_SENTINEL.starts_with("<!--"));
    assert!(APPEND_REGION_SENTINEL.ends_with("-->"));
}

#[test]
fn allowed_statuses_and_sections_are_canonical() {
    assert_eq!(
        ALLOWED_STATUSES,
        &["proposed", "active", "deprecated", "superseded"]
    );
    assert_eq!(
        REQUIRED_SECTIONS,
        &["Context", "What Went Well", "What Did Not", "Followups"]
    );
}
