//! APS Core Engine
//!
//! Provides shared primitives for APS validation and tooling.
//!
//! # Modules
//!
//! - [`diagnostics`] - Structured error/warning reporting
//! - [`discovery`] - Filesystem traversal and package discovery
//! - [`metadata`] - TOML metadata parsing for standards/substandards/experiments

pub mod diagnostics;
pub mod discovery;
pub mod metadata;

pub use diagnostics::{Diagnostic, Diagnostics, Severity};
