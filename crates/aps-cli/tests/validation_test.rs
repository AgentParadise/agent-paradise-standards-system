//! Validation tests using fixtures.
//!
//! Tests that valid packages pass validation and invalid packages
//! produce the correct error codes.

mod fixtures;

use aps_v1_0000_meta::{MetaStandard, Standard, error_codes};
use fixtures::{
    InvalidKind, create_invalid_standard, create_test_workspace, create_valid_standard,
};

#[test]
fn test_valid_standard_passes_validation() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    let pkg_path = create_valid_standard(
        temp_dir.path(),
        "APS-V1-0001",
        "Test Standard",
        "test-standard",
    );

    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&pkg_path);

    assert!(
        !diagnostics.has_errors(),
        "Valid standard should pass. Errors: {:?}",
        diagnostics.errors().collect::<Vec<_>>()
    );
}

#[test]
fn test_invalid_missing_metadata() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    let pkg_path = create_invalid_standard(temp_dir.path(), InvalidKind::MissingMetadata);

    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&pkg_path);

    assert!(diagnostics.has_errors(), "Should have errors");

    let has_metadata_error = diagnostics
        .errors()
        .any(|d| d.code == error_codes::MISSING_METADATA_FILE);

    assert!(
        has_metadata_error,
        "Should have MISSING_METADATA_FILE error. Got: {:?}",
        diagnostics.errors().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn test_invalid_missing_docs() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    let pkg_path = create_invalid_standard(temp_dir.path(), InvalidKind::MissingDocs);

    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&pkg_path);

    assert!(diagnostics.has_errors(), "Should have errors");

    let has_docs_error = diagnostics.errors().any(|d| {
        d.code == error_codes::MISSING_SPEC_DOC || d.code == error_codes::MISSING_REQUIRED_DIR
    });

    assert!(
        has_docs_error,
        "Should have MISSING_SPEC_DOC or MISSING_REQUIRED_DIR error. Got: {:?}",
        diagnostics.errors().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn test_invalid_missing_cargo() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    let pkg_path = create_invalid_standard(temp_dir.path(), InvalidKind::MissingCargo);

    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&pkg_path);

    assert!(diagnostics.has_errors(), "Should have errors");

    let has_cargo_error = diagnostics
        .errors()
        .any(|d| d.code == error_codes::MISSING_CARGO_TOML);

    assert!(
        has_cargo_error,
        "Should have MISSING_CARGO_TOML error. Got: {:?}",
        diagnostics.errors().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn test_invalid_missing_lib_rs() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    let pkg_path = create_invalid_standard(temp_dir.path(), InvalidKind::MissingLibRs);

    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&pkg_path);

    assert!(diagnostics.has_errors(), "Should have errors");

    let has_lib_error = diagnostics
        .errors()
        .any(|d| d.code == error_codes::MISSING_LIB_RS);

    assert!(
        has_lib_error,
        "Should have MISSING_LIB_RS error. Got: {:?}",
        diagnostics.errors().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn test_invalid_missing_required_dirs() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    let pkg_path = create_invalid_standard(temp_dir.path(), InvalidKind::MissingRequiredDirs);

    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&pkg_path);

    assert!(diagnostics.has_errors(), "Should have errors");

    let has_dir_error = diagnostics
        .errors()
        .any(|d| d.code == error_codes::MISSING_REQUIRED_DIR);

    assert!(
        has_dir_error,
        "Should have MISSING_REQUIRED_DIR error. Got: {:?}",
        diagnostics.errors().map(|d| &d.code).collect::<Vec<_>>()
    );
}

#[test]
fn test_invalid_bad_id_format() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    let pkg_path = create_invalid_standard(temp_dir.path(), InvalidKind::BadIdFormat);

    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&pkg_path);

    assert!(diagnostics.has_errors(), "Should have errors");

    let has_id_error = diagnostics
        .errors()
        .any(|d| d.code == error_codes::INVALID_STANDARD_ID);

    assert!(
        has_id_error,
        "Should have INVALID_STANDARD_ID error. Got: {:?}",
        diagnostics.errors().map(|d| &d.code).collect::<Vec<_>>()
    );
}
