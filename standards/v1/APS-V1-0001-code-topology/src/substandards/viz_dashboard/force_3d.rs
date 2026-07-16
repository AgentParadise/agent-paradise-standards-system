//! 3D Force-Directed Visualization
//!
//! Interactive 3D graph where nodes represent modules and edges represent coupling.
//! Uses Three.js for WebGL rendering with OrbitControls.
//!
//! ## Features
//! - Force-directed layout clusters coupled modules together
//! - Node color based on health (distance from main sequence)
//! - Edge thickness based on coupling strength
//! - Interactive sidebar with module list and coupling filter
//! - Click to focus, hover for tooltips
//!
//! Markup lives in `templates/force_3d.hbs`.

use super::{escape_json_for_html, render::render_template};
use serde::Serialize;

const TEMPLATE: &str = include_str!("templates/force_3d.hbs");

#[derive(Serialize)]
struct Force3dCtx<'a> {
    scene_json: &'a str,
    node_count: usize,
    edge_count: usize,
}

/// Generate 3D Force-Directed HTML visualization.
///
/// This function generates a complete, self-contained HTML document with
/// embedded Three.js for 3D rendering. The scene data is embedded as JSON.
///
/// # Arguments
/// * `scene_json` - Serialized Scene3D data (nodes, edges, camera)
/// * `node_count` - Number of nodes (for info panel)
/// * `edge_count` - Number of edges (for info panel)
///
/// # Returns
/// Complete HTML document as a string
pub fn generate(scene_json: &str, node_count: usize, edge_count: usize) -> String {
    let scene_escaped = escape_json_for_html(scene_json);
    let ctx = Force3dCtx {
        scene_json: &scene_escaped,
        node_count,
        edge_count,
    };
    render_template(TEMPLATE, &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_contains_doctype() {
        let html = generate("{}", 0, 0);
        assert!(html.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn test_generate_contains_title() {
        let html = generate("{}", 5, 3);
        assert!(html.contains("<title>Code Topology"));
    }

    #[test]
    fn test_generate_contains_counts() {
        let html = generate("{}", 42, 17);
        assert!(html.contains("42"));
        assert!(html.contains("17"));
    }

    #[test]
    fn test_generate_embeds_scene() {
        let html = generate(r#"{"n":1}"#, 1, 0);
        assert!(html.contains(r#"const data = {"n":1};"#));
    }
}
