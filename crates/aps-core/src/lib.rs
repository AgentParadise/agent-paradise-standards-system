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
//! - [`config`] - Project configuration parsing (`apss.toml`)
//! - [`standard_config`] - Typed configuration contract for standards
//! - [`resolution`] - Cascading configuration resolution for monorepos
//! - [`lockfile`] - Lockfile types for reproducible installations
//! - [`registry`] - Dynamic standard composition and CLI dispatch

pub mod config;
pub mod diagnostics;
pub mod discovery;
pub mod lockfile;
pub mod metadata;
pub mod promotion;
pub mod registry;
pub mod resolution;
pub mod standard_config;
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

// Project configuration and distribution
pub use config::{ConfigError, ProjectConfig, StandardEntry};
pub use lockfile::{Lockfile, LockfileError};
pub use registry::{CommandHandler, ProjectRunner, RegisteredStandard, StandardRegistry};
pub use resolution::{ResolutionError, ResolvedProjectConfig, ResolvedStandard};
pub use standard_config::{NoConfig, StandardConfig};
