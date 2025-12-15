//! Substandard validation tests.
//!
//! Tests that substandards are validated correctly with proper
//! ID format and parent reference checks.

mod fixtures;

use aps_v1_0000_meta::{MetaStandard, Standard, error_codes};
use fixtures::{
    InvalidSubstandardKind, create_invalid_substandard, create_test_workspace,
    create_valid_standard, create_valid_substandard,
};

#[test]
fn test_valid_substandard_passes_validation() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    // Create parent standard first
    let _parent =
        create_valid_standard(temp_dir.path(), "APS-V1-0001", "Parent Standard", "parent");

    // Create substandard
    let pkg_path = create_valid_substandard(
        temp_dir.path(),
        "APS-V1-0001",
        "parent",
        "GH01",
        "GitHub Profile",
        "github",
    );

    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&pkg_path);

    assert!(
        !diagnostics.has_errors(),
        "Valid substandard should pass. Errors: {:?}",
        diagnostics.errors().collect::<Vec<_>>()
    );
}

#[test]
fn test_substandard_with_invalid_id() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    let pkg_path = create_invalid_substandard(temp_dir.path(), InvalidSubstandardKind::BadIdFormat);

    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&pkg_path);

    assert!(diagnostics.has_errors(), "Should have errors");

    let has_id_error = diagnostics
        .errors()
        .any(|d| d.code == error_codes::INVALID_SUBSTANDARD_ID);

    assert!(
        has_id_error,
        "Should have INVALID_SUBSTANDARD_ID error. Got: {:?}",
        diagnostics.errors().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn test_substandard_with_mismatched_parent() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    let pkg_path =
        create_invalid_substandard(temp_dir.path(), InvalidSubstandardKind::MismatchedParent);

    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&pkg_path);

    assert!(diagnostics.has_errors(), "Should have errors");

    let has_parent_error = diagnostics
        .errors()
        .any(|d| d.code == error_codes::INVALID_PARENT_REF);

    assert!(
        has_parent_error,
        "Should have INVALID_PARENT_REF error. Got: {:?}",
        diagnostics.errors().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn test_real_substandard_passes_validation() {
    // Validate the actual SS01 substandard in the repo
    let repo_root = fixtures::repo_root();
    let ss01_path =
        repo_root.join("standards/v1/APS-V1-0000-meta/substandards/SS01-substandard-structure");

    if !ss01_path.exists() {
        // Skip if substandard doesn't exist yet
        return;
    }

    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&ss01_path);

    assert!(
        !diagnostics.has_errors(),
        "SS01 substandard should pass validation. Errors: {:?}",
        diagnostics.errors().collect::<Vec<_>>()
    );
}
