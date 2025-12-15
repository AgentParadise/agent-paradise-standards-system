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

pub mod diagnostics;
pub mod discovery;
pub mod metadata;
pub mod promotion;
pub mod templates;

pub use diagnostics::{Diagnostic, Diagnostics, Severity};
pub use promotion::{PromotionError, PromotionResult, promote_experiment};
pub use templates::{ExperimentContext, StandardContext, SubstandardContext, TemplateEngine};
