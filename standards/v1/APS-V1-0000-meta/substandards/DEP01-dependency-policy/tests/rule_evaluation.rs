//! End-to-end: parse a fixture crate, match against a fixture approved list,
//! assert we emit the right violation classes.

use aps_v1_0000_dep01_dependency_policy::{
    load_approved, scan_crate, ViolationReason,
};
use std::io::Write;

const APPROVED: &str = r#"
schema = "aps.approved-deps/v1"

[[dep]]
name = "serde"
category = "standard"
justification = "."
allowed_for = ["*"]
transitive_audit_date = "2026-04-18"

[[dep]]
name = "jsonschema"
category = "tooling"
justification = "."
allowed_for = ["aps-schema-test"]
transitive_audit_date = "2026-04-18"

[[dep]]
name = "handlebars"
category = "tooling"
justification = "."
allowed_for = ["aps-core"]
transitive_audit_date = "2026-04-18"
"#;

fn fixture_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let approved_path = dir.path().join("approved-deps.toml");
    let mut f = std::fs::File::create(&approved_path).unwrap();
    f.write_all(APPROVED.as_bytes()).unwrap();
    dir
}

fn write_cargo(dir: &tempfile::TempDir, rel: &str, body: &str) -> std::path::PathBuf {
    let path = dir.path().join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(body.as_bytes()).unwrap();
    path
}

#[test]
fn unapproved_dep_emits_violation() {
    let dir = fixture_dir();
    let manifest = write_cargo(
        &dir,
        "standards/v1/APS-V1-9999-demo/Cargo.toml",
        r#"
[package]
name = "demo-std"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
unknown-crate = "0.1"
"#,
    );
    let list = load_approved(&dir.path().join("approved-deps.toml")).unwrap();
    let vs = scan_crate(&manifest, &list, "demo-std").unwrap();
    assert_eq!(vs.len(), 1);
    assert_eq!(vs[0].dep, "unknown-crate");
    assert_eq!(vs[0].reason, ViolationReason::Unapproved);
}

#[test]
fn tooling_dep_in_non_allowed_standard_is_not_allowed_for_crate() {
    let dir = fixture_dir();
    let manifest = write_cargo(
        &dir,
        "standards/v1/APS-V1-9999-demo/Cargo.toml",
        r#"
[package]
name = "demo-std"
version = "0.1.0"
edition = "2021"

[dev-dependencies]
jsonschema = "0.46"
"#,
    );
    let list = load_approved(&dir.path().join("approved-deps.toml")).unwrap();
    let vs = scan_crate(&manifest, &list, "demo-std").unwrap();
    assert_eq!(vs.len(), 1);
    assert_eq!(vs[0].dep, "jsonschema");
    assert_eq!(vs[0].reason, ViolationReason::NotAllowedForCrate);
}

#[test]
fn tooling_regular_dep_in_shippable_standard_is_wrong_category() {
    let dir = fixture_dir();
    // Allow the dep so we don't trip on scope — isolate the category rule.
    let approved = APPROVED.to_string()
        + r#"
[[dep]]
name = "handlebars2"
category = "tooling"
justification = "."
allowed_for = ["demo-std"]
transitive_audit_date = "2026-04-18"
"#;
    let approved_path = dir.path().join("approved-deps.toml");
    std::fs::write(&approved_path, approved).unwrap();
    let manifest = write_cargo(
        &dir,
        "standards/v1/APS-V1-9999-demo/Cargo.toml",
        r#"
[package]
name = "demo-std"
version = "0.1.0"
edition = "2021"

[dependencies]
handlebars2 = "0.1"
"#,
    );
    let list = load_approved(&approved_path).unwrap();
    let vs = scan_crate(&manifest, &list, "demo-std").unwrap();
    assert_eq!(vs.len(), 1);
    assert_eq!(vs[0].dep, "handlebars2");
    assert_eq!(vs[0].reason, ViolationReason::WrongCategory);
}

#[test]
fn tooling_dev_dep_in_scoped_tooling_crate_is_clean() {
    let dir = fixture_dir();
    let manifest = write_cargo(
        &dir,
        "crates/aps-schema-test/Cargo.toml",
        r#"
[package]
name = "aps-schema-test"
version = "0.1.0"
edition = "2021"

[dev-dependencies]
jsonschema = "0.46"
"#,
    );
    let list = load_approved(&dir.path().join("approved-deps.toml")).unwrap();
    let vs = scan_crate(&manifest, &list, "aps-schema-test").unwrap();
    assert!(vs.is_empty(), "expected no violations, got {vs:?}");
}

#[test]
fn path_deps_are_ignored() {
    let dir = fixture_dir();
    let manifest = write_cargo(
        &dir,
        "standards/v1/APS-V1-9999-demo/Cargo.toml",
        r#"
[package]
name = "demo-std"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
internal-lib = { path = "../internal" }
"#,
    );
    let list = load_approved(&dir.path().join("approved-deps.toml")).unwrap();
    let vs = scan_crate(&manifest, &list, "demo-std").unwrap();
    assert!(vs.is_empty(), "path deps should be skipped; got {vs:?}");
}
