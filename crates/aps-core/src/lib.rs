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
    ConsumerError, ConsumerResult, PackageValidation, ResolvedSource, cache_dir, compute_checksum,
    compute_checksum_bytes, generate_index, github_release_url, init_manifest, is_package_cached,
    load_lock, load_manifest, load_or_create_lock, lock_package, package_cache_path,
    parse_checksum_file, resolve_source, validate_package_structure, verify_checksum, write_index,
    write_lock,
};
pub use diagnostics::{Diagnostic, Diagnostics, Severity};
pub use manifest::{
    AgentIndex, ConsumerManifest, IndexedStandard, IndexedSubstandard, LOCK_SCHEMA, LockedPackage,
    MANIFEST_SCHEMA, ManifestLock, PackageRegistry, ProjectInfo, REGISTRY_SCHEMA, RegistryStandard,
    RegistryVersion, SourceConfig, SourceOverride, StandardRef, SubstandardConfig,
};
pub use promotion::{PromotionError, PromotionResult, promote_experiment};
pub use templates::{ExperimentContext, StandardContext, SubstandardContext, TemplateEngine};
pub use versioning::{BumpPart, VersionBumpResult, VersionError, bump_version, get_version};
pub use views::{Registry, ViewsError, generate_all_views, generate_registry};
