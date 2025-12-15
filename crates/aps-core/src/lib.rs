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

pub mod diagnostics;
pub mod discovery;
pub mod metadata;
pub mod promotion;
pub mod templates;
pub mod views;

pub use diagnostics::{Diagnostic, Diagnostics, Severity};
pub use promotion::{PromotionError, PromotionResult, promote_experiment};
pub use templates::{ExperimentContext, StandardContext, SubstandardContext, TemplateEngine};
pub use views::{Registry, ViewsError, generate_all_views, generate_registry};
