//! Topology Visualization Dashboard (EXP-V1-0001.VIZ01)
//!
//! This substandard provides interactive HTML visualizations for code topology data.
//! Each visualization offers a different perspective on the codebase structure and health.
//!
//! ## Visualization Types
//!
//! - **3D Force-Directed** — Coupling relationships as a 3D graph
//! - **CodeCity** — 3D city metaphor (buildings = modules, height = complexity)
//! - **Package Clusters** — 2D force-directed package relationships
//! - **VSA Diagram** — Vertical Slice Architecture matrix
//! - **Dashboard Index** — Landing page linking to all visualizations
//!
//! ## Usage
//!
//! ```ignore
//! use code_topology_viz::{force_3d, codecity, clusters, vsa, index};
//!
//! let modules_json = serde_json::to_string(&modules)?;
//! let coupling_json = serde_json::to_string(&coupling)?;
//!
//! let html = force_3d::generate(&modules_json, &coupling_json);
//! std::fs::write("topology-3d.html", html)?;
//! ```
//!
//! ⚠️ EXPERIMENTAL: This substandard is in incubation.

pub mod clusters;
pub mod codecity;
pub mod force_3d;
pub mod index;
pub mod vsa;

// Re-exports for convenience
pub use clusters::generate as generate_clusters;
pub use codecity::generate as generate_codecity;
pub use force_3d::generate as generate_force_3d;
pub use index::generate as generate_index;
pub use vsa::generate as generate_vsa;

/// Available visualization types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VizType {
    /// 3D force-directed coupling graph
    Force3D,
    /// 3D city metaphor
    CodeCity,
    /// 2D package clusters
    Clusters,
    /// Vertical slice architecture matrix
    Vsa,
    /// Dashboard index
    Index,
}

impl VizType {
    /// Get all visualization types (excluding index)
    pub fn all() -> &'static [VizType] {
        &[
            VizType::Force3D,
            VizType::CodeCity,
            VizType::Clusters,
            VizType::Vsa,
        ]
    }

    /// Get the default output filename for this visualization type
    pub fn default_filename(&self) -> &'static str {
        match self {
            VizType::Force3D => "topology-3d.html",
            VizType::CodeCity => "codecity.html",
            VizType::Clusters => "clusters.html",
            VizType::Vsa => "vsa.html",
            VizType::Index => "index.html",
        }
    }

    /// Get a human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            VizType::Force3D => "3D Force-Directed",
            VizType::CodeCity => "CodeCity",
            VizType::Clusters => "Package Clusters",
            VizType::Vsa => "VSA Diagram",
            VizType::Index => "Dashboard Index",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viz_types() {
        assert_eq!(VizType::all().len(), 4);
        assert_eq!(VizType::Force3D.default_filename(), "topology-3d.html");
        assert_eq!(VizType::CodeCity.name(), "CodeCity");
    }
}
