//! Verify the manifest parsers extract direct dep names and skip path deps.

use aps_v1_0000_dep01_dependency_policy::manifests;
use std::io::Write;
use std::path::PathBuf;

fn write(name: &str, content: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    (dir, path)
}

#[test]
fn cargo_toml_names_deps_and_skips_paths() {
    let toml = r#"
[package]
name = "my-crate"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
thiserror = "2"
internal = { path = "../other" }

[dev-dependencies]
tempfile = "3"

[target.'cfg(unix)'.dependencies]
nix = "0.29"
"#;
    let (_dir, path) = write("Cargo.toml", toml);
    let parsed = manifests::parse(&path).unwrap();
    assert_eq!(parsed.kind, manifests::ManifestKind::Cargo);
    assert_eq!(parsed.package_name.as_deref(), Some("my-crate"));
    let names: Vec<_> = parsed.deps.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"serde"));
    assert!(names.contains(&"thiserror"));
    assert!(names.contains(&"tempfile"));
    assert!(names.contains(&"nix"));
    assert!(
        !names.contains(&"internal"),
        "path deps must be filtered out"
    );
    let tempfile_entry = parsed.deps.iter().find(|d| d.name == "tempfile").unwrap();
    assert!(tempfile_entry.dev_only);
    let serde_entry = parsed.deps.iter().find(|d| d.name == "serde").unwrap();
    assert!(!serde_entry.dev_only);
}

#[test]
fn pyproject_reads_pep621_dependencies() {
    let toml = r#"
[project]
name = "my-pkg"
version = "0.1.0"
dependencies = [
  "httpx>=0.27",
  "pydantic[email]~=2.9",
  "click ; python_version >= '3.10'",
]

[project.optional-dependencies]
dev = ["pytest>=8.0", "ruff"]
"#;
    let (_dir, path) = write("pyproject.toml", toml);
    let parsed = manifests::parse(&path).unwrap();
    assert_eq!(parsed.kind, manifests::ManifestKind::Pyproject);
    assert_eq!(parsed.package_name.as_deref(), Some("my-pkg"));
    let names: Vec<_> = parsed.deps.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"httpx"));
    assert!(names.contains(&"pydantic"));
    assert!(names.contains(&"click"));
    assert!(names.contains(&"pytest"));
    assert!(names.contains(&"ruff"));
}

#[test]
fn pyproject_reads_poetry_groups() {
    let toml = r#"
[tool.poetry]
name = "poetry-pkg"
version = "0.1.0"

[tool.poetry.dependencies]
python = "^3.12"
requests = "^2.32"

[tool.poetry.group.dev.dependencies]
pytest = "^8"
"#;
    let (_dir, path) = write("pyproject.toml", toml);
    let parsed = manifests::parse(&path).unwrap();
    assert_eq!(parsed.package_name.as_deref(), Some("poetry-pkg"));
    let names: Vec<_> = parsed.deps.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"requests"));
    assert!(names.contains(&"pytest"));
    assert!(
        !names.contains(&"python"),
        "the python version pseudo-dep must be filtered out"
    );
}

#[test]
fn package_json_reads_all_sections_and_skips_file_specs() {
    let json = r#"
{
  "name": "@acme/app",
  "version": "1.0.0",
  "dependencies": {
    "react": "^18.3.0",
    "local": "file:../shared"
  },
  "devDependencies": {
    "typescript": "~5.5.0"
  },
  "peerDependencies": {
    "react-dom": "^18.3.0"
  }
}
"#;
    let (_dir, path) = write("package.json", json);
    let parsed = manifests::parse(&path).unwrap();
    assert_eq!(parsed.kind, manifests::ManifestKind::PackageJson);
    assert_eq!(parsed.package_name.as_deref(), Some("@acme/app"));
    let names: Vec<_> = parsed.deps.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"react"));
    assert!(names.contains(&"typescript"));
    assert!(names.contains(&"react-dom"));
    assert!(
        !names.contains(&"local"),
        "file: deps must be filtered out"
    );
    let ts_entry = parsed.deps.iter().find(|d| d.name == "typescript").unwrap();
    assert!(ts_entry.dev_only);
}
