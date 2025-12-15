//! Self-validation tests for the APS repository.
//!
//! These tests validate the actual standards in the repository,
//! ensuring they conform to the meta-standard.

mod fixtures;

use aps_core::discovery::discover_v1_packages;
use aps_v1_0000_meta::{MetaStandard, Standard};
use fixtures::repo_root;
use std::process::Command;

#[test]
fn test_meta_standard_passes_validation() {
    let repo = repo_root();
    let meta_path = repo.join("standards/v1/APS-V1-0000-meta");

    assert!(meta_path.exists(), "Meta-standard directory should exist");

    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&meta_path);

    assert!(
        !diagnostics.has_errors(),
        "APS-V1-0000-meta should pass validation.\nErrors: {:?}",
        diagnostics.errors().collect::<Vec<_>>()
    );
}

#[test]
fn test_all_v1_standards_pass_validation() {
    let repo = repo_root();
    let packages = discover_v1_packages(&repo);

    assert!(!packages.is_empty(), "Should have at least one V1 package");

    let meta = MetaStandard::new();

    for pkg in &packages {
        let diagnostics = meta.validate_package(&pkg.path);

        assert!(
            !diagnostics.has_errors(),
            "Package {:?} should pass validation.\nErrors: {:?}",
            pkg.path.file_name(),
            diagnostics.errors().collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_repo_validation_via_cli() {
    let repo = repo_root();

    let output = Command::new(fixtures::aps_binary())
        .args(["v1", "validate", "repo"])
        .current_dir(&repo)
        .output()
        .expect("Failed to execute aps v1 validate repo");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "aps v1 validate repo should succeed.\nstdout: {}\nstderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_repo_has_required_structure() {
    let repo = repo_root();

    // Check required directories exist
    assert!(repo.join("standards/v1").exists(), "standards/v1/ required");
    assert!(
        repo.join("standards-experimental/v1").exists(),
        "standards-experimental/v1/ required"
    );
    assert!(
        repo.join("crates/aps-core").exists(),
        "crates/aps-core/ required"
    );
    assert!(
        repo.join("crates/aps-cli").exists(),
        "crates/aps-cli/ required"
    );

    // Check meta-standard exists
    assert!(
        repo.join("standards/v1/APS-V1-0000-meta").exists(),
        "APS-V1-0000-meta required"
    );
}

#[test]
fn test_all_standards_have_required_files() {
    let repo = repo_root();
    let packages = discover_v1_packages(&repo);

    for pkg in &packages {
        // All packages need these
        assert!(
            pkg.path.join("Cargo.toml").exists(),
            "{:?} missing Cargo.toml",
            pkg.path.file_name()
        );
        assert!(
            pkg.path.join("src/lib.rs").exists(),
            "{:?} missing src/lib.rs",
            pkg.path.file_name()
        );
        assert!(
            pkg.path.join("docs/01_spec.md").exists(),
            "{:?} missing docs/01_spec.md",
            pkg.path.file_name()
        );
    }
}
