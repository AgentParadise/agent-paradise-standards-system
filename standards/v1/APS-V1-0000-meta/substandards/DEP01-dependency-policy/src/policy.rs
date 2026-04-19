//! Approved-list loader, type definitions, and manifest matcher.
//!
//! The approved list is a TOML file (`approved-deps.toml` at the repo root)
//! whose schema is `"aps.approved-deps/v1"`. Each entry declares a third-party
//! dependency the repo has audited and sanctioned for use.

use crate::manifests::{DepRef, ParsedManifest};
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
/// `crate_id` is the consumer name to match against `allowed_for` patterns.
/// Normally this is the manifest's `[package].name`; callers supply it so the
/// matcher stays agnostic to file-system layout.
pub fn match_manifest(
    parsed: &ParsedManifest,
    list: &ApprovedList,
    crate_id: &str,
) -> Vec<DepViolation> {
    let mut out = Vec::new();
    for DepRef { name, dev_only } in &parsed.deps {
        let Some(entry) = list.get(name) else {
            out.push(DepViolation {
                crate_id: crate_id.to_string(),
                dep: name.clone(),
                reason: ViolationReason::Unapproved,
            });
            continue;
        };
        if !allowed_for_matches(&entry.allowed_for, crate_id) {
            out.push(DepViolation {
                crate_id: crate_id.to_string(),
                dep: name.clone(),
                reason: ViolationReason::NotAllowedForCrate,
            });
            continue;
        }
        // Regular [dependencies] of a shippable standard must not be tooling-category.
        if !*dev_only
            && matches!(entry.category, Category::Tooling)
            && is_shippable_standard(crate_id, &parsed.path)
        {
            out.push(DepViolation {
                crate_id: crate_id.to_string(),
                dep: name.clone(),
                reason: ViolationReason::WrongCategory,
            });
        }
    }
    out
}

/// True if the crate lives under `standards/v1/APS-*/**` — our definition of
/// a shippable standard.
fn is_shippable_standard(_crate_id: &str, manifest_path: &Path) -> bool {
    let s = manifest_path.to_string_lossy();
    s.contains("/standards/v1/APS-") || s.starts_with("standards/v1/APS-")
}

fn allowed_for_matches(patterns: &[String], crate_id: &str) -> bool {
    for pat in patterns {
        if pat == "*" || pat == crate_id {
            return true;
        }
        if let Some(prefix) = pat.strip_suffix('*') {
            if crate_id.starts_with(prefix) {
                return true;
            }
        }
    }
    false
}
