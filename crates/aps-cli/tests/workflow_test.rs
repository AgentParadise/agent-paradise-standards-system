//! End-to-end workflow tests.
//!
//! These tests verify complete workflows: create → validate → promote → version.

mod fixtures;

use aps_core::promotion::promote_experiment;
use aps_core::versioning::{BumpPart, bump_version, get_version};
use aps_v1_0000_meta::{MetaStandard, Standard};
use fixtures::{create_test_workspace, create_valid_experiment, create_valid_standard};
use std::fs;

#[test]
fn test_create_validate_workflow() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    // Step 1: Create a new standard using fixture helper
    let pkg_dir = create_valid_standard(
        temp_dir.path(),
        "APS-V1-0200",
        "Workflow Test Standard",
        "workflow-test",
    );

    // Step 2: Validate the created standard
    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&pkg_dir);

    assert!(
        !diagnostics.has_errors(),
        "Created standard should pass validation"
    );
}

#[test]
fn test_experiment_promotion_workflow() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    // Step 1: Create an experiment using fixture helper
    let _exp_dir = create_valid_experiment(
        temp_dir.path(),
        "EXP-V1-0002",
        "Promotable Experiment",
        "promotable",
    );

    // Verify experiment is valid
    let meta = MetaStandard::new();
    let exp_diagnostics = meta.validate_package(&_exp_dir);
    assert!(!exp_diagnostics.has_errors(), "Experiment should be valid");

    // Step 2: Promote to official standard
    let result = promote_experiment(temp_dir.path(), "EXP-V1-0002", Some("APS-V1-0002")).unwrap();

    // Step 3: Verify the promoted standard is valid
    let std_diagnostics = meta.validate_package(&result.new_path);
    assert!(
        !std_diagnostics.has_errors(),
        "Promoted standard should pass validation. Errors: {:?}",
        std_diagnostics.errors().collect::<Vec<_>>()
    );

    // Verify metadata was updated correctly
    assert!(
        result.new_path.join("standard.toml").exists(),
        "Should have standard.toml"
    );
    assert!(
        !result.new_path.join("experiment.toml").exists(),
        "Should not have experiment.toml"
    );

    let standard_content = fs::read_to_string(result.new_path.join("standard.toml")).unwrap();
    assert!(
        standard_content.contains("APS-V1-0002"),
        "Should have new ID"
    );
}

#[test]
fn test_version_bump_workflow() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    // Step 1: Create a standard
    let _pkg_dir = create_valid_standard(
        temp_dir.path(),
        "APS-V1-0300",
        "Version Test Standard",
        "version-test",
    );

    // Verify initial version
    let initial_version = get_version(temp_dir.path(), "APS-V1-0300").unwrap();
    assert_eq!(initial_version, "1.0.0");

    // Step 2: Bump patch version
    let result = bump_version(temp_dir.path(), "APS-V1-0300", BumpPart::Patch).unwrap();
    assert_eq!(result.old_version, "1.0.0");
    assert_eq!(result.new_version, "1.0.1");

    let new_version = get_version(temp_dir.path(), "APS-V1-0300").unwrap();
    assert_eq!(new_version, "1.0.1");

    // Step 3: Bump minor version
    let result = bump_version(temp_dir.path(), "APS-V1-0300", BumpPart::Minor).unwrap();
    assert_eq!(result.new_version, "1.1.0");

    // Step 4: Validate still passes
    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&_pkg_dir);
    assert!(
        !diagnostics.has_errors(),
        "Version bumped standard should still be valid"
    );
}

#[test]
fn test_full_lifecycle_workflow() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());
    let meta = MetaStandard::new();

    // 1. Create an experiment
    let exp_dir = create_valid_experiment(
        temp_dir.path(),
        "EXP-V1-0003",
        "Full Lifecycle Test",
        "lifecycle",
    );

    // 2. Validate experiment
    assert!(!meta.validate_package(&exp_dir).has_errors());

    // 3. Iterate on experiment (bump version)
    bump_version(temp_dir.path(), "EXP-V1-0003", BumpPart::Minor).unwrap();
    let exp_version = get_version(temp_dir.path(), "EXP-V1-0003").unwrap();
    assert_eq!(exp_version, "0.2.0");

    // 4. Promote to official
    let result = promote_experiment(temp_dir.path(), "EXP-V1-0003", Some("APS-V1-0003")).unwrap();

    // 5. Validate promoted standard
    assert!(!meta.validate_package(&result.new_path).has_errors());

    // 6. Release updates
    bump_version(temp_dir.path(), "APS-V1-0003", BumpPart::Major).unwrap();
    let std_version = get_version(temp_dir.path(), "APS-V1-0003").unwrap();
    // Version after promotion is 0.2.0, then major bump makes it 1.0.0
    assert_eq!(std_version, "1.0.0");

    // 7. Final validation
    assert!(!meta.validate_package(&result.new_path).has_errors());
}
