//! Test fixtures and shared helpers for APS CLI tests.
//!
//! Provides utilities for creating valid and invalid package fixtures
//! for testing validation logic.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

/// Get the repository root (where Cargo.toml with [workspace] lives).
pub fn repo_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates/aps-cli -> crates
    path.pop(); // crates -> repo root
    path
}

/// Get the path to the aps binary.
pub fn aps_binary() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove deps
    path.push("aps");
    path
}

/// Create a minimal valid workspace in a temp directory.
pub fn create_test_workspace(dir: &Path) {
    let cargo_toml = r#"[workspace]
resolver = "2"
members = []

[workspace.package]
version = "0.1.0"
edition = "2021"
"#;
    fs::write(dir.join("Cargo.toml"), cargo_toml).unwrap();
    fs::create_dir_all(dir.join("standards/v1")).unwrap();
    fs::create_dir_all(dir.join("standards-experimental/v1")).unwrap();
}

/// Create a minimal valid standard package.
pub fn create_valid_standard(dir: &Path, id: &str, name: &str, slug: &str) -> PathBuf {
    let pkg_dir = dir.join(format!("standards/v1/{}-{}", id, slug));
    fs::create_dir_all(&pkg_dir).unwrap();

    // Required directories
    fs::create_dir_all(pkg_dir.join("docs")).unwrap();
    fs::create_dir_all(pkg_dir.join("examples")).unwrap();
    fs::create_dir_all(pkg_dir.join("tests")).unwrap();
    fs::create_dir_all(pkg_dir.join("agents/skills")).unwrap();
    fs::create_dir_all(pkg_dir.join("src")).unwrap();

    // standard.toml
    let standard_toml = format!(
        r#"schema = "aps.standard/v1"

[standard]
id = "{id}"
name = "{name}"
slug = "{slug}"
version = "1.0.0"
category = "governance"
status = "active"

[aps]
aps_major = "v1"
backwards_compatible_major_required = true

[ownership]
maintainers = ["Test"]
"#
    );
    fs::write(pkg_dir.join("standard.toml"), standard_toml).unwrap();

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{slug}"
version = "1.0.0"
edition = "2021"
"#
    );
    fs::write(pkg_dir.join("Cargo.toml"), cargo_toml).unwrap();

    // src/lib.rs
    fs::write(pkg_dir.join("src/lib.rs"), "// Minimal lib\n").unwrap();

    // docs/01_spec.md
    let spec = format!(
        "# {} — {} (Canonical Specification)\n\n**Version**: 1.0.0\n",
        id, name
    );
    fs::write(pkg_dir.join("docs/01_spec.md"), spec).unwrap();

    // READMEs
    fs::write(pkg_dir.join("examples/README.md"), "# Examples\n").unwrap();
    fs::write(pkg_dir.join("tests/README.md"), "# Tests\n").unwrap();
    fs::write(pkg_dir.join("agents/skills/README.md"), "# Skills\n").unwrap();

    pkg_dir
}

/// Create a minimal valid experiment package.
pub fn create_valid_experiment(dir: &Path, id: &str, name: &str, slug: &str) -> PathBuf {
    let pkg_dir = dir.join(format!("standards-experimental/v1/{}-{}", id, slug));
    fs::create_dir_all(&pkg_dir).unwrap();

    // Required directories
    fs::create_dir_all(pkg_dir.join("docs")).unwrap();
    fs::create_dir_all(pkg_dir.join("examples")).unwrap();
    fs::create_dir_all(pkg_dir.join("tests")).unwrap();
    fs::create_dir_all(pkg_dir.join("agents/skills")).unwrap();
    fs::create_dir_all(pkg_dir.join("src")).unwrap();

    // experiment.toml
    let experiment_toml = format!(
        r#"schema = "aps.experiment/v1"

[experiment]
id = "{id}"
name = "{name}"
slug = "{slug}"
version = "0.1.0"
category = "technical"

[aps]
aps_major = "v1"

[ownership]
maintainers = ["Test"]
"#
    );
    fs::write(pkg_dir.join("experiment.toml"), experiment_toml).unwrap();

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{slug}"
version = "0.1.0"
edition = "2021"
"#
    );
    fs::write(pkg_dir.join("Cargo.toml"), cargo_toml).unwrap();

    // src/lib.rs
    fs::write(pkg_dir.join("src/lib.rs"), "// Minimal lib\n").unwrap();

    // docs/01_spec.md
    let spec = format!(
        "# {} — {} (Experimental Specification)\n\n**Version**: 0.1.0\n",
        id, name
    );
    fs::write(pkg_dir.join("docs/01_spec.md"), spec).unwrap();

    // READMEs
    fs::write(pkg_dir.join("examples/README.md"), "# Examples\n").unwrap();
    fs::write(pkg_dir.join("tests/README.md"), "# Tests\n").unwrap();
    fs::write(pkg_dir.join("agents/skills/README.md"), "# Skills\n").unwrap();

    pkg_dir
}

/// Create a minimal valid substandard package.
pub fn create_valid_substandard(
    dir: &Path,
    parent_id: &str,
    parent_slug: &str,
    profile: &str,
    name: &str,
    slug: &str,
) -> PathBuf {
    let substandard_id = format!("{}.{}", parent_id, profile);
    let pkg_dir = dir.join(format!(
        "standards/v1/{}-{}/substandards/{}-{}",
        parent_id, parent_slug, profile, slug
    ));
    fs::create_dir_all(&pkg_dir).unwrap();

    // Required directories
    fs::create_dir_all(pkg_dir.join("docs")).unwrap();
    fs::create_dir_all(pkg_dir.join("examples")).unwrap();
    fs::create_dir_all(pkg_dir.join("tests")).unwrap();
    fs::create_dir_all(pkg_dir.join("agents/skills")).unwrap();
    fs::create_dir_all(pkg_dir.join("src")).unwrap();

    // substandard.toml
    let substandard_toml = format!(
        r#"schema = "aps.substandard/v1"

[substandard]
id = "{substandard_id}"
name = "{name}"
slug = "{slug}"
version = "1.0.0"
parent_id = "{parent_id}"
parent_major = "1"

[ownership]
maintainers = ["Test"]
"#
    );
    fs::write(pkg_dir.join("substandard.toml"), substandard_toml).unwrap();

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{slug}"
version = "1.0.0"
edition = "2021"
"#
    );
    fs::write(pkg_dir.join("Cargo.toml"), cargo_toml).unwrap();

    // src/lib.rs
    fs::write(pkg_dir.join("src/lib.rs"), "// Minimal substandard lib\n").unwrap();

    // docs/01_spec.md
    let spec = format!(
        "# {} — {} (Canonical Specification)\n\n**Version**: 1.0.0\n",
        substandard_id, name
    );
    fs::write(pkg_dir.join("docs/01_spec.md"), spec).unwrap();

    // READMEs
    fs::write(pkg_dir.join("examples/README.md"), "# Examples\n").unwrap();
    fs::write(pkg_dir.join("tests/README.md"), "# Tests\n").unwrap();
    fs::write(pkg_dir.join("agents/skills/README.md"), "# Skills\n").unwrap();

    pkg_dir
}

/// Types of invalid fixtures.
#[derive(Debug, Clone, Copy)]
pub enum InvalidKind {
    MissingMetadata,
    MissingDocs,
    MissingCargo,
    MissingLibRs,
    MissingRequiredDirs,
    BadIdFormat,
}

/// Types of invalid substandard fixtures.
#[derive(Debug, Clone, Copy)]
pub enum InvalidSubstandardKind {
    BadIdFormat,
    MismatchedParent,
}

/// Create an invalid standard fixture.
pub fn create_invalid_standard(dir: &Path, kind: InvalidKind) -> PathBuf {
    let pkg_dir = dir.join("standards/v1/APS-V1-9999-invalid");
    fs::create_dir_all(&pkg_dir).unwrap();

    match kind {
        InvalidKind::MissingMetadata => {
            // Create everything except standard.toml
            fs::create_dir_all(pkg_dir.join("docs")).unwrap();
            fs::create_dir_all(pkg_dir.join("examples")).unwrap();
            fs::create_dir_all(pkg_dir.join("tests")).unwrap();
            fs::create_dir_all(pkg_dir.join("agents/skills")).unwrap();
            fs::create_dir_all(pkg_dir.join("src")).unwrap();
            fs::write(pkg_dir.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
            fs::write(pkg_dir.join("src/lib.rs"), "// lib\n").unwrap();
            fs::write(pkg_dir.join("docs/01_spec.md"), "# Spec\n").unwrap();
        }
        InvalidKind::MissingDocs => {
            // Create everything except docs/01_spec.md
            fs::create_dir_all(pkg_dir.join("examples")).unwrap();
            fs::create_dir_all(pkg_dir.join("tests")).unwrap();
            fs::create_dir_all(pkg_dir.join("agents/skills")).unwrap();
            fs::create_dir_all(pkg_dir.join("src")).unwrap();
            fs::write(pkg_dir.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
            fs::write(pkg_dir.join("src/lib.rs"), "// lib\n").unwrap();
            fs::write(
                pkg_dir.join("standard.toml"),
                "schema = \"aps.standard/v1\"\n[standard]\nid = \"APS-V1-9999\"\nname = \"Test\"\nslug = \"test\"\nversion = \"1.0.0\"\ncategory = \"governance\"\nstatus = \"active\"\n[aps]\naps_major = \"v1\"\n[ownership]\nmaintainers = [\"Test\"]\n",
            )
            .unwrap();
        }
        InvalidKind::MissingCargo => {
            // Create everything except Cargo.toml
            fs::create_dir_all(pkg_dir.join("docs")).unwrap();
            fs::create_dir_all(pkg_dir.join("examples")).unwrap();
            fs::create_dir_all(pkg_dir.join("tests")).unwrap();
            fs::create_dir_all(pkg_dir.join("agents/skills")).unwrap();
            fs::create_dir_all(pkg_dir.join("src")).unwrap();
            fs::write(pkg_dir.join("src/lib.rs"), "// lib\n").unwrap();
            fs::write(pkg_dir.join("docs/01_spec.md"), "# Spec\n").unwrap();
            fs::write(
                pkg_dir.join("standard.toml"),
                "schema = \"aps.standard/v1\"\n[standard]\nid = \"APS-V1-9999\"\nname = \"Test\"\nslug = \"test\"\nversion = \"1.0.0\"\ncategory = \"governance\"\nstatus = \"active\"\n[aps]\naps_major = \"v1\"\n[ownership]\nmaintainers = [\"Test\"]\n",
            )
            .unwrap();
        }
        InvalidKind::MissingLibRs => {
            // Create everything except src/lib.rs
            fs::create_dir_all(pkg_dir.join("docs")).unwrap();
            fs::create_dir_all(pkg_dir.join("examples")).unwrap();
            fs::create_dir_all(pkg_dir.join("tests")).unwrap();
            fs::create_dir_all(pkg_dir.join("agents/skills")).unwrap();
            fs::create_dir_all(pkg_dir.join("src")).unwrap();
            fs::write(pkg_dir.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
            fs::write(pkg_dir.join("docs/01_spec.md"), "# Spec\n").unwrap();
            fs::write(
                pkg_dir.join("standard.toml"),
                "schema = \"aps.standard/v1\"\n[standard]\nid = \"APS-V1-9999\"\nname = \"Test\"\nslug = \"test\"\nversion = \"1.0.0\"\ncategory = \"governance\"\nstatus = \"active\"\n[aps]\naps_major = \"v1\"\n[ownership]\nmaintainers = [\"Test\"]\n",
            )
            .unwrap();
        }
        InvalidKind::MissingRequiredDirs => {
            // Only create standard.toml, nothing else
            fs::write(
                pkg_dir.join("standard.toml"),
                "schema = \"aps.standard/v1\"\n[standard]\nid = \"APS-V1-9999\"\nname = \"Test\"\nslug = \"test\"\nversion = \"1.0.0\"\ncategory = \"governance\"\nstatus = \"active\"\n[aps]\naps_major = \"v1\"\n[ownership]\nmaintainers = [\"Test\"]\n",
            )
            .unwrap();
        }
        InvalidKind::BadIdFormat => {
            // Create everything but with invalid ID format
            fs::create_dir_all(pkg_dir.join("docs")).unwrap();
            fs::create_dir_all(pkg_dir.join("examples")).unwrap();
            fs::create_dir_all(pkg_dir.join("tests")).unwrap();
            fs::create_dir_all(pkg_dir.join("agents/skills")).unwrap();
            fs::create_dir_all(pkg_dir.join("src")).unwrap();
            fs::write(pkg_dir.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
            fs::write(pkg_dir.join("src/lib.rs"), "// lib\n").unwrap();
            fs::write(pkg_dir.join("docs/01_spec.md"), "# Spec\n").unwrap();
            // Invalid ID: should be APS-V1-XXXX, using INVALID-ID instead
            fs::write(
                pkg_dir.join("standard.toml"),
                "schema = \"aps.standard/v1\"\n[standard]\nid = \"INVALID-ID\"\nname = \"Test\"\nslug = \"test\"\nversion = \"1.0.0\"\ncategory = \"governance\"\nstatus = \"active\"\n[aps]\naps_major = \"v1\"\n[ownership]\nmaintainers = [\"Test\"]\n",
            )
            .unwrap();
        }
    }

    pkg_dir
}

/// Create an invalid substandard fixture.
pub fn create_invalid_substandard(dir: &Path, kind: InvalidSubstandardKind) -> PathBuf {
    let pkg_dir = dir.join("standards/v1/APS-V1-0001-test/substandards/XX01-invalid");
    fs::create_dir_all(&pkg_dir).unwrap();

    // Create valid structure
    fs::create_dir_all(pkg_dir.join("docs")).unwrap();
    fs::create_dir_all(pkg_dir.join("examples")).unwrap();
    fs::create_dir_all(pkg_dir.join("tests")).unwrap();
    fs::create_dir_all(pkg_dir.join("agents/skills")).unwrap();
    fs::create_dir_all(pkg_dir.join("src")).unwrap();
    fs::write(pkg_dir.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
    fs::write(pkg_dir.join("src/lib.rs"), "// lib\n").unwrap();
    fs::write(pkg_dir.join("docs/01_spec.md"), "# Spec\n").unwrap();

    match kind {
        InvalidSubstandardKind::BadIdFormat => {
            // Invalid substandard ID format
            fs::write(
                pkg_dir.join("substandard.toml"),
                "schema = \"aps.substandard/v1\"\n[substandard]\nid = \"INVALID-ID\"\nname = \"Test\"\nslug = \"test\"\nversion = \"1.0.0\"\nparent_id = \"APS-V1-0001\"\nparent_major = \"1\"\n[ownership]\nmaintainers = [\"Test\"]\n",
            )
            .unwrap();
        }
        InvalidSubstandardKind::MismatchedParent => {
            // Valid ID but parent_id doesn't match
            fs::write(
                pkg_dir.join("substandard.toml"),
                "schema = \"aps.substandard/v1\"\n[substandard]\nid = \"APS-V1-0001.GH01\"\nname = \"Test\"\nslug = \"test\"\nversion = \"1.0.0\"\nparent_id = \"APS-V1-9999\"\nparent_major = \"1\"\n[ownership]\nmaintainers = [\"Test\"]\n",
            )
            .unwrap();
        }
    }

    pkg_dir
}
