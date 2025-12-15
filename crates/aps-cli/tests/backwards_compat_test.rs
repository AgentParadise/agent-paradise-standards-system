//! Backwards compatibility tests.
//!
//! These tests ensure that V1 standards maintain backwards compatibility
//! within the major version, as required by the meta-standard.

mod fixtures;

use aps_core::discovery::discover_v1_packages;
use aps_core::metadata::parse_standard_metadata;
use fixtures::repo_root;

/// All V1 standards must declare backwards_compatible_major_required.
#[test]
fn test_all_v1_standards_have_backwards_compat_field() {
    let repo = repo_root();
    let packages = discover_v1_packages(&repo);

    for pkg in &packages {
        let standard_toml = pkg.path.join("standard.toml");
        if !standard_toml.exists() {
            continue; // Skip experiments/substandards
        }

        let metadata = parse_standard_metadata(&standard_toml)
            .unwrap_or_else(|_| panic!("Failed to parse {:?}", standard_toml));

        // All V1 standards should have this field set
        assert!(
            metadata.aps.backwards_compatible_major_required.is_some(),
            "Package {:?} should have backwards_compatible_major_required field",
            pkg.path.file_name()
        );
    }
}

/// Standards marked as backwards_compatible_major_required=true must follow
/// semver rules within V1.
#[test]
fn test_backwards_compat_flag_meaning() {
    let repo = repo_root();
    let packages = discover_v1_packages(&repo);

    for pkg in &packages {
        let standard_toml = pkg.path.join("standard.toml");
        if !standard_toml.exists() {
            continue;
        }

        let metadata = parse_standard_metadata(&standard_toml).unwrap();

        if metadata.aps.backwards_compatible_major_required == Some(true) {
            // This standard must maintain backwards compatibility
            // Future breaking changes require APS-V2

            // Verify the standard's aps_major is v1
            assert_eq!(
                metadata.aps.aps_major, "v1",
                "Standards with backwards_compatible_major_required must be V1"
            );
        }
    }
}

/// Test that the meta-standard itself is marked as requiring backwards compatibility.
#[test]
fn test_meta_standard_requires_backwards_compat() {
    let repo = repo_root();
    let meta_standard_toml = repo.join("standards/v1/APS-V1-0000-meta/standard.toml");

    let metadata = parse_standard_metadata(&meta_standard_toml)
        .expect("Meta-standard should have valid metadata");

    assert_eq!(
        metadata.aps.backwards_compatible_major_required,
        Some(true),
        "Meta-standard must require backwards compatibility within V1"
    );
}

/// Ensure version numbers in V1 standards follow semver.
#[test]
fn test_v1_standards_follow_semver() {
    let repo = repo_root();
    let packages = discover_v1_packages(&repo);

    let semver_regex = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();

    for pkg in &packages {
        let standard_toml = pkg.path.join("standard.toml");
        if !standard_toml.exists() {
            continue;
        }

        let metadata = parse_standard_metadata(&standard_toml).unwrap();

        assert!(
            semver_regex.is_match(&metadata.standard.version),
            "Package {:?} version '{}' should follow semver X.Y.Z format",
            pkg.path.file_name(),
            metadata.standard.version
        );
    }
}

/// Test that ID format follows the V1 pattern.
#[test]
fn test_v1_standard_id_format() {
    let repo = repo_root();
    let packages = discover_v1_packages(&repo);

    let id_regex = regex::Regex::new(r"^APS-V1-\d{4}$").unwrap();

    for pkg in &packages {
        let standard_toml = pkg.path.join("standard.toml");
        if !standard_toml.exists() {
            continue;
        }

        let metadata = parse_standard_metadata(&standard_toml).unwrap();

        assert!(
            id_regex.is_match(&metadata.standard.id),
            "Package {:?} ID '{}' should match APS-V1-XXXX format",
            pkg.path.file_name(),
            metadata.standard.id
        );
    }
}
