//! Manifest readers for Rust (`Cargo.toml`), Python (`pyproject.toml`), and
//! Node (`package.json`).
//!
//! Each reader extracts the minimum needed to run the approved-list matcher: a
//! list of direct dependency names together with whether each is a dev/test
//! dependency. This module performs no matching itself.

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
    /// The crate/package name, if discoverable. For workspace Cargo.toml files
    /// this may be `None`.
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
    let _kind = kind_for(path)?;
    // Implementation comes in Commit 2; scaffold returns an empty manifest so
    // downstream code can be wired and compiled first.
    Ok(ParsedManifest {
        path: path.to_path_buf(),
        kind: _kind,
        package_name: None,
        deps: Vec::new(),
    })
}
