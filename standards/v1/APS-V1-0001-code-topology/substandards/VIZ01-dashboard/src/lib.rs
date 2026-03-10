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
// Note: index::generate takes a repo_name parameter — use index::generate directly
pub use index::generate as generate_index;
pub use vsa::generate as generate_vsa;

// ============================================================================
// Shared Utilities
// ============================================================================

/// Health classification bands used for colors and labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthBand {
    Excellent,
    Good,
    Ok,
    Warning,
    Poor,
    Critical,
}

impl HealthBand {
    /// Classify a numeric health score (0.0-1.0) into a health band.
    pub fn from_score(health: f64) -> Self {
        if health >= 0.80 {
            HealthBand::Excellent
        } else if health >= 0.65 {
            HealthBand::Good
        } else if health >= 0.50 {
            HealthBand::Ok
        } else if health >= 0.35 {
            HealthBand::Warning
        } else if health >= 0.20 {
            HealthBand::Poor
        } else {
            HealthBand::Critical
        }
    }

    /// Get the color hex string for this health band.
    pub fn color(&self) -> &'static str {
        match self {
            HealthBand::Excellent => "#00ff88",
            HealthBand::Good => "#44dd77",
            HealthBand::Ok => "#88cc55",
            HealthBand::Warning => "#ddaa33",
            HealthBand::Poor => "#ff7744",
            HealthBand::Critical => "#ff3333",
        }
    }

    /// Get the label for this health band.
    pub fn label(&self) -> &'static str {
        match self {
            HealthBand::Excellent => "Excellent",
            HealthBand::Good => "Good",
            HealthBand::Ok => "OK",
            HealthBand::Warning => "Warning",
            HealthBand::Poor => "Poor",
            HealthBand::Critical => "Critical",
        }
    }
}

/// Convert health score (0.0-1.0) to color hex string.
pub fn health_to_color(health: f64) -> &'static str {
    HealthBand::from_score(health).color()
}

/// Get health label from score (0.0-1.0).
pub fn health_label(health: f64) -> &'static str {
    HealthBand::from_score(health).label()
}

/// Escape JSON for safe embedding in HTML script tags.
///
/// This escapes characters that could break out of the JavaScript context:
/// - `</script>` sequences that would close the script tag
/// - Backticks that could interfere with template literals
pub fn escape_json_for_html(json: &str) -> String {
    json.replace("</script>", "<\\/script>")
        .replace("</Script>", "<\\/Script>")
        .replace("</SCRIPT>", "<\\/SCRIPT>")
}

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

    #[test]
    fn test_health_band_from_score() {
        assert_eq!(HealthBand::from_score(0.90), HealthBand::Excellent);
        assert_eq!(HealthBand::from_score(0.70), HealthBand::Good);
        assert_eq!(HealthBand::from_score(0.50), HealthBand::Ok);
        assert_eq!(HealthBand::from_score(0.35), HealthBand::Warning);
        assert_eq!(HealthBand::from_score(0.20), HealthBand::Poor);
        assert_eq!(HealthBand::from_score(0.10), HealthBand::Critical);
    }

    #[test]
    fn test_health_to_color() {
        assert_eq!(health_to_color(0.90), "#00ff88");
        assert_eq!(health_to_color(0.50), "#88cc55");
        assert_eq!(health_to_color(0.10), "#ff3333");
    }

    #[test]
    fn test_health_label() {
        assert_eq!(health_label(0.90), "Excellent");
        assert_eq!(health_label(0.50), "OK");
        assert_eq!(health_label(0.10), "Critical");
    }

    #[test]
    fn test_escape_json_for_html() {
        assert_eq!(escape_json_for_html("normal json"), "normal json");
        assert_eq!(
            escape_json_for_html("</script>alert('xss')"),
            "<\\/script>alert('xss')"
        );
        assert_eq!(
            escape_json_for_html("</Script></SCRIPT>"),
            "<\\/Script><\\/SCRIPT>"
        );
    }
}
