//! Approved-list loader, type definitions, and manifest matcher.
//!
//! The approved list is a TOML file (`approved-deps.toml` at the repo root)
//! whose schema is `"aps.approved-deps/v1"`. Each entry declares a third-party
//! dependency the repo has audited and sanctioned for use.

use crate::manifests::ParsedManifest;
use crate::DepError;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Category of an approved dependency.
///
/// - `Standard`: allowed in shippable standard crates (`standards/**`).
/// - `Tooling`: allowed only in tooling crates (`crates/**`) or as a
///   `[dev-dependencies]` entry of members explicitly listed in `allowed_for`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Standard,
    Tooling,
}

/// A single approved dependency entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedEntry {
    pub name: String,
    pub justification: String,
    pub category: Category,
    /// Glob-style crate-name list that may depend on this entry. `"*"` matches
    /// anything.
    pub allowed_for: Vec<String>,
    pub transitive_audit_date: NaiveDate,
    #[serde(default)]
    pub notes: Option<String>,
}

/// A loaded and indexed approved list.
#[derive(Debug, Clone)]
pub struct ApprovedList {
    entries: Vec<ApprovedEntry>,
    index: HashMap<String, usize>,
}

impl ApprovedList {
    pub fn entries(&self) -> &[ApprovedEntry] {
        &self.entries
    }

    pub fn get(&self, name: &str) -> Option<&ApprovedEntry> {
        self.index.get(name).map(|i| &self.entries[*i])
    }
}

/// TOML shape for the approved-deps.toml file.
#[derive(Debug, Deserialize)]
struct ApprovedListDoc {
    schema: String,
    #[serde(default, rename = "dep")]
    deps: Vec<ApprovedEntry>,
}

/// Load and index the approved list.
pub fn load(path: &Path) -> Result<ApprovedList, DepError> {
    let raw = std::fs::read_to_string(path).map_err(|e| DepError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    let doc: ApprovedListDoc = toml::from_str(&raw).map_err(|e| DepError::Toml {
        path: path.display().to_string(),
        source: e,
    })?;
    if doc.schema != "aps.approved-deps/v1" {
        return Err(DepError::SchemaMismatch(doc.schema));
    }
    let mut index = HashMap::with_capacity(doc.deps.len());
    for (i, entry) in doc.deps.iter().enumerate() {
        index.insert(entry.name.clone(), i);
    }
    Ok(ApprovedList {
        entries: doc.deps,
        index,
    })
}

/// Reason a dependency failed the approved-list check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationReason {
    Unapproved,
    NotAllowedForCrate,
    WrongCategory,
    AuditStale,
}

/// A dep that failed the approved-list check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DepViolation {
    pub crate_id: String,
    pub dep: String,
    pub reason: ViolationReason,
}

/// Match a parsed manifest against the approved list.
///
/// Scaffold: returns an empty `Vec`. Full implementation lands in Commit 2.
pub fn match_manifest(
    _parsed: &ParsedManifest,
    _list: &ApprovedList,
    _crate_id: &str,
) -> Vec<DepViolation> {
    Vec::new()
}
