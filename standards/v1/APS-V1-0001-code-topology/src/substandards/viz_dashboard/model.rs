//! Typed data model shared across dashboard visualizations.
//!
//! `VizModule` is the per-module row consumed by every dashboard viz
//! (codecity, clusters, vsa, force_3d). It is populated in the CLI layer
//! (`cli::viz::topology_viz`) from the raw topology artifact plus the
//! health scoring helpers, and then serialized as JSON and embedded into
//! each HTML template.

use serde::Serialize;

/// Per-module summary embedded in every dashboard viz payload.
///
/// Kept flat and JSON-friendly so that the front-end JS can index it
/// without extra unwrapping. Field order matches the JSON contract that
/// the current dashboard JS relies on.
#[derive(Debug, Clone, Serialize)]
pub struct VizModule {
    pub id: String,
    pub name: String,
    pub path: String,
    pub slice: String,
    pub layer: String,
    pub function_count: u32,
    pub total_cyclomatic: u32,
    pub total_cognitive: u32,
    pub lines_of_code: u32,
    pub ca: u32,
    pub ce: u32,
    pub health: f64,
    pub color: String,
    pub health_label: String,
}

/// Aggregate stats used by the index / landing page.
#[derive(Debug, Clone, Copy)]
pub struct VizStats {
    pub module_count: usize,
    pub slice_count: usize,
    pub avg_health: f64,
}
