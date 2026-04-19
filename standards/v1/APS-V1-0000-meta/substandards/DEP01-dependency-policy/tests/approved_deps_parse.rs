//! Round-trip the approved-list TOML: load, query, check schema guard.

use aps_v1_0000_dep01_dependency_policy as dep01;
use std::io::Write;

fn write_toml(content: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

#[test]
fn loads_well_formed_approved_list() {
    let toml = r#"
schema = "aps.approved-deps/v1"

[[dep]]
name = "serde"
category = "standard"
justification = "Canonical serde."
allowed_for = ["*"]
transitive_audit_date = "2026-04-18"

[[dep]]
name = "jsonschema"
category = "tooling"
justification = "Schema drift testing."
allowed_for = ["aps-schema-test"]
transitive_audit_date = "2026-04-18"
"#;
    let f = write_toml(toml);
    let list = dep01::load_approved(f.path()).expect("load");
    assert_eq!(list.entries().len(), 2);
    assert_eq!(list.get("serde").unwrap().category, dep01::Category::Standard);
    assert_eq!(
        list.get("jsonschema").unwrap().category,
        dep01::Category::Tooling
    );
    assert!(list.get("not-there").is_none());
}

#[test]
fn rejects_wrong_schema() {
    let toml = r#"
schema = "aps.wrong/v1"
[[dep]]
name = "x"
category = "standard"
justification = "."
allowed_for = ["*"]
transitive_audit_date = "2026-04-18"
"#;
    let f = write_toml(toml);
    let err = dep01::load_approved(f.path()).unwrap_err();
    matches!(err, dep01::DepError::SchemaMismatch(_));
}

#[test]
fn accepts_optional_notes_field() {
    let toml = r#"
schema = "aps.approved-deps/v1"
[[dep]]
name = "tempfile"
category = "standard"
justification = "Test fixtures."
allowed_for = ["*"]
transitive_audit_date = "2026-04-18"
notes = "dev-only"
"#;
    let f = write_toml(toml);
    let list = dep01::load_approved(f.path()).unwrap();
    assert_eq!(list.get("tempfile").unwrap().notes.as_deref(), Some("dev-only"));
}

#[test]
fn repo_root_approved_deps_loads() {
    // The seeded approved-deps.toml at repo root must parse cleanly.
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = std::path::Path::new(&root).join("../../../../../approved-deps.toml");
    let list = dep01::load_approved(&path).expect("load root approved-deps.toml");
    // Sanity-check some of the seeded entries are present.
    for name in ["serde", "serde_json", "toml", "thiserror", "jsonschema", "tempfile"] {
        assert!(
            list.get(name).is_some(),
            "expected {name} in approved-deps.toml"
        );
    }
}
