//! APS Core Engine
//!
//! Provides shared primitives for APS validation and tooling.
//!
//! # Modules
//!
//! - [`diagnostics`] - Structured error/warning reporting
//! - [`discovery`] - Filesystem traversal and package discovery
//! - [`metadata`] - TOML metadata parsing for standards/substandards/experiments
//! - [`templates`] - Template rendering for package scaffolding
//! - [`promotion`] - Experiment to standard promotion workflow
//! - [`views`] - Derived views generator (registry.json, INDEX.md)
//! - [`versioning`] - Version management for packages

pub mod diagnostics;
pub mod discovery;
pub mod metadata;
pub mod promotion;
pub mod templates;
pub mod versioning;
pub mod views;

pub use diagnostics::{Diagnostic, Diagnostics, Severity};
pub use promotion::{PromotionError, PromotionResult, promote_experiment};
pub use templates::{ExperimentContext, StandardContext, SubstandardContext, TemplateEngine};
pub use versioning::{
    BumpPart, VersionBumpResult, VersionError, VersionValidation, bump_version, get_version,
    is_valid_semver, parse_semver, validate_backwards_compat, validate_version,
};
pub use views::{Registry, ViewsError, generate_all_views, generate_registry};
