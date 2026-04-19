//! APS-V1-0000.DEP01 — Dependency Policy
//!
//! Declares the **zero-external-dependencies-by-default** principle for APS
//! shippable standards and enforces it via an approved list. Any crate listed
//! in `approved-deps.toml` at the repo root is an explicit exception with
//! justification, category, and one-level transitive audit date.
//!
//! # Principle
//!
//! Shippable standard crates (anything under `standards/v1/APS-*/**`) MUST NOT
//! declare third-party dependencies unless the entry appears in the repo-root
//! `approved-deps.toml`. Tooling crates (anything under `crates/*`) MAY
//! declare any dependency whose approved-list entry names them in
//! `allowed_for`.

use std::path::Path;
use thiserror::Error;

pub mod manifests;
pub mod policy;

pub use manifests::{ManifestKind, ParsedManifest};
pub use policy::{ApprovedEntry, ApprovedList, Category, DepViolation, ViolationReason};

/// Errors produced while loading the approved list or scanning manifests.
#[derive(Debug, Error)]
pub enum DepError {
    #[error("i/o error reading {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("toml parse error in {path}: {source}")]
    Toml {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("json parse error in {path}: {source}")]
    Json {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("approved-deps.toml schema mismatch: expected 'aps.approved-deps/v1', got '{0}'")]
    SchemaMismatch(String),
    #[error("unknown manifest kind for {0}")]
    UnknownManifest(String),
}

/// Error codes emitted by DEP01 in diagnostics.
pub mod error_codes {
    pub const UNAPPROVED_DEPENDENCY: &str = "UNAPPROVED_DEPENDENCY";
    pub const DEPENDENCY_NOT_ALLOWED_FOR_CRATE: &str = "DEPENDENCY_NOT_ALLOWED_FOR_CRATE";
    pub const DEPENDENCY_WRONG_CATEGORY: &str = "DEPENDENCY_WRONG_CATEGORY";
    pub const APPROVED_DEP_AUDIT_STALE: &str = "APPROVED_DEP_AUDIT_STALE";
}

/// Load the approved list from `approved-deps.toml`.
pub fn load_approved(path: &Path) -> Result<ApprovedList, DepError> {
    policy::load(path)
}

/// Scan a manifest file and return any dependencies that violate the policy.
pub fn scan_crate(
    manifest: &Path,
    list: &ApprovedList,
    crate_id: &str,
) -> Result<Vec<DepViolation>, DepError> {
    let parsed = manifests::parse(manifest)?;
    Ok(policy::match_manifest(&parsed, list, crate_id))
}
