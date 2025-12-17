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
//! - [`manifest`] - Consumer manifest types for APS adoption
//! - [`consumer`] - Consumer-side functionality for adopting standards

pub mod consumer;
pub mod diagnostics;
pub mod discovery;
pub mod manifest;
pub mod metadata;
pub mod promotion;
pub mod templates;
pub mod versioning;
pub mod views;

pub use consumer::{
    ConsumerError, ConsumerResult, ResolvedSource, generate_index, init_manifest, load_manifest,
    resolve_source, write_index,
};
pub use diagnostics::{Diagnostic, Diagnostics, Severity};
pub use manifest::{
    AgentIndex, ConsumerManifest, IndexedStandard, IndexedSubstandard, MANIFEST_SCHEMA,
    ProjectInfo, SourceConfig, SourceOverride, StandardRef, SubstandardConfig,
};
pub use promotion::{PromotionError, PromotionResult, promote_experiment};
pub use templates::{ExperimentContext, StandardContext, SubstandardContext, TemplateEngine};
pub use versioning::{BumpPart, VersionBumpResult, VersionError, bump_version, get_version};
pub use views::{Registry, ViewsError, generate_all_views, generate_registry};
