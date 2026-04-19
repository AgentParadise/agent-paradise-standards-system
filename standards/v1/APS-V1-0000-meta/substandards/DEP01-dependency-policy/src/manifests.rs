//! Manifest readers for Rust (`Cargo.toml`), Python (`pyproject.toml`), and
//! Node (`package.json`).
//!
//! Each reader extracts the minimum needed to run the approved-list matcher: a
//! list of direct dependency names together with whether each is a dev/test
//! dependency. Internal/path deps (Cargo `path = "..."`, npm `file:...`) are
//! filtered out because they name workspace members, not third-party crates.

use crate::DepError;
use std::path::{Path, PathBuf};

/// Kind of manifest file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestKind {
    Cargo,
    Pyproject,
    PackageJson,
}

/// A direct dependency extracted from a manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepRef {
    pub name: String,
    pub dev_only: bool,
}

/// A parsed manifest reduced to the fields the approved-list matcher needs.
#[derive(Debug, Clone)]
pub struct ParsedManifest {
    pub path: PathBuf,
    pub kind: ManifestKind,
    /// The crate/package name, if discoverable. Workspace-root Cargo.toml
    /// files without `[package]` yield `None`.
    pub package_name: Option<String>,
    pub deps: Vec<DepRef>,
}

/// Detect the manifest kind from a file path.
pub fn kind_for(path: &Path) -> Result<ManifestKind, DepError> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    match name {
        "Cargo.toml" => Ok(ManifestKind::Cargo),
        "pyproject.toml" => Ok(ManifestKind::Pyproject),
        "package.json" => Ok(ManifestKind::PackageJson),
        _ => Err(DepError::UnknownManifest(path.display().to_string())),
    }
}

/// Parse a manifest file into the common `ParsedManifest` shape.
pub fn parse(path: &Path) -> Result<ParsedManifest, DepError> {
    let kind = kind_for(path)?;
    let raw = std::fs::read_to_string(path).map_err(|e| DepError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    match kind {
        ManifestKind::Cargo => parse_cargo(path, &raw),
        ManifestKind::Pyproject => parse_pyproject(path, &raw),
        ManifestKind::PackageJson => parse_package_json(path, &raw),
    }
}

fn parse_cargo(path: &Path, raw: &str) -> Result<ParsedManifest, DepError> {
    let doc: toml::Value = toml::from_str(raw).map_err(|e| DepError::Toml {
        path: path.display().to_string(),
        source: e,
    })?;

    let package_name = doc
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(str::to_string);

    let mut deps = Vec::new();
    collect_cargo_table(&doc, "dependencies", false, &mut deps);
    collect_cargo_table(&doc, "dev-dependencies", true, &mut deps);
    collect_cargo_table(&doc, "build-dependencies", false, &mut deps);
    // Target-gated deps: [target.'cfg(...)'.dependencies]
    if let Some(table) = doc.get("target").and_then(toml::Value::as_table) {
        for (_cfg, inner) in table {
            collect_cargo_table(inner, "dependencies", false, &mut deps);
            collect_cargo_table(inner, "dev-dependencies", true, &mut deps);
            collect_cargo_table(inner, "build-dependencies", false, &mut deps);
        }
    }

    Ok(ParsedManifest {
        path: path.to_path_buf(),
        kind: ManifestKind::Cargo,
        package_name,
        deps,
    })
}

/// Push third-party dep names from `doc[section]` into `deps`. Skips path deps.
fn collect_cargo_table(doc: &toml::Value, section: &str, dev_only: bool, deps: &mut Vec<DepRef>) {
    let Some(table) = doc.get(section).and_then(toml::Value::as_table) else {
        return;
    };
    for (name, value) in table {
        if let toml::Value::Table(spec) = value {
            if spec.contains_key("path") {
                continue;
            }
        }
        deps.push(DepRef {
            name: name.clone(),
            dev_only,
        });
    }
}

fn parse_pyproject(path: &Path, raw: &str) -> Result<ParsedManifest, DepError> {
    let doc: toml::Value = toml::from_str(raw).map_err(|e| DepError::Toml {
        path: path.display().to_string(),
        source: e,
    })?;

    let package_name = doc
        .get("project")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(str::to_string)
        .or_else(|| {
            doc.get("tool")
                .and_then(|t| t.get("poetry"))
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .map(str::to_string)
        });

    let mut deps = Vec::new();

    // PEP 621: [project].dependencies is a list of requirement strings.
    if let Some(arr) = doc
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(toml::Value::as_array)
    {
        for val in arr {
            if let Some(req) = val.as_str() {
                if let Some(name) = pep508_name(req) {
                    deps.push(DepRef {
                        name,
                        dev_only: false,
                    });
                }
            }
        }
    }

    // PEP 621 optional-dependencies — treat as dev-only (they gate on extras).
    if let Some(table) = doc
        .get("project")
        .and_then(|p| p.get("optional-dependencies"))
        .and_then(toml::Value::as_table)
    {
        for (_group, val) in table {
            if let Some(arr) = val.as_array() {
                for item in arr {
                    if let Some(req) = item.as_str() {
                        if let Some(name) = pep508_name(req) {
                            deps.push(DepRef {
                                name,
                                dev_only: true,
                            });
                        }
                    }
                }
            }
        }
    }

    // Poetry: [tool.poetry.dependencies] is a table { name = "version" | { table } }.
    if let Some(table) = doc
        .get("tool")
        .and_then(|t| t.get("poetry"))
        .and_then(|p| p.get("dependencies"))
        .and_then(toml::Value::as_table)
    {
        for (name, _) in table {
            if name == "python" {
                continue;
            }
            deps.push(DepRef {
                name: name.clone(),
                dev_only: false,
            });
        }
    }
    if let Some(table) = doc
        .get("tool")
        .and_then(|t| t.get("poetry"))
        .and_then(|p| p.get("dev-dependencies"))
        .and_then(toml::Value::as_table)
    {
        for (name, _) in table {
            deps.push(DepRef {
                name: name.clone(),
                dev_only: true,
            });
        }
    }
    // Newer poetry: [tool.poetry.group.*.dependencies]
    if let Some(groups) = doc
        .get("tool")
        .and_then(|t| t.get("poetry"))
        .and_then(|p| p.get("group"))
        .and_then(toml::Value::as_table)
    {
        for (_gname, group) in groups {
            if let Some(table) = group.get("dependencies").and_then(toml::Value::as_table) {
                for (name, _) in table {
                    deps.push(DepRef {
                        name: name.clone(),
                        dev_only: true,
                    });
                }
            }
        }
    }

    Ok(ParsedManifest {
        path: path.to_path_buf(),
        kind: ManifestKind::Pyproject,
        package_name,
        deps,
    })
}

/// Extract the distribution name from a PEP 508 requirement string.
/// Strips extras, version specifiers, markers, and whitespace.
fn pep508_name(req: &str) -> Option<String> {
    let trimmed = req.trim();
    if trimmed.is_empty() {
        return None;
    }
    let end = trimmed
        .find(|c: char| {
            matches!(
                c,
                '[' | '(' | '=' | '<' | '>' | '!' | '~' | ';' | ' ' | '\t'
            )
        })
        .unwrap_or(trimmed.len());
    let name = trimmed[..end].trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn parse_package_json(path: &Path, raw: &str) -> Result<ParsedManifest, DepError> {
    let doc: serde_json::Value = serde_json::from_str(raw).map_err(|e| DepError::Json {
        path: path.display().to_string(),
        source: e,
    })?;

    let package_name = doc
        .get("name")
        .and_then(|n| n.as_str())
        .map(str::to_string);

    let mut deps = Vec::new();
    collect_npm_object(&doc, "dependencies", false, &mut deps);
    collect_npm_object(&doc, "devDependencies", true, &mut deps);
    collect_npm_object(&doc, "peerDependencies", false, &mut deps);
    collect_npm_object(&doc, "optionalDependencies", true, &mut deps);

    Ok(ParsedManifest {
        path: path.to_path_buf(),
        kind: ManifestKind::PackageJson,
        package_name,
        deps,
    })
}

fn collect_npm_object(
    doc: &serde_json::Value,
    section: &str,
    dev_only: bool,
    deps: &mut Vec<DepRef>,
) {
    let Some(obj) = doc.get(section).and_then(serde_json::Value::as_object) else {
        return;
    };
    for (name, value) in obj {
        if let Some(spec) = value.as_str() {
            if spec.starts_with("file:") || spec.starts_with("link:") {
                continue;
            }
        }
        deps.push(DepRef {
            name: name.clone(),
            dev_only,
        });
    }
}
