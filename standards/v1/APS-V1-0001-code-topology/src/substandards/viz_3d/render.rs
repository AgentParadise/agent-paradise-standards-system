//! Scene construction and HTML rendering for the 3D force-directed projector.
//!
//! `build_scene` turns a `Topology` (plus a per-call config) into a
//! `Scene3D`; `wrap_in_html` embeds that scene into the Three.js viewer
//! template at `templates/scene.hbs`.

use crate::{ProjectorError, Topology};
use serde::Serialize;

use super::layout::run_force_simulation;
use super::model::{Camera, ForceDirectedConfig, NodeMetrics, Scene3D, SceneEdge, SceneNode};
use crate::substandards::render_shared::render_template;

const TEMPLATE: &str = include_str!("templates/scene.hbs");

#[derive(Serialize)]
struct SceneCtx<'a> {
    scene_json: &'a str,
    node_count: usize,
    edge_count: usize,
}

/// Calculate node color based on distance from Martin's main sequence.
///
/// - D near 0 (on main sequence) = healthy, green
/// - D near 0.5 = moderate, yellow/orange
/// - D near 1 (Zone of Pain / Uselessness) = red
pub fn health_color(distance_from_main_sequence: f64) -> String {
    let health = 1.0 - distance_from_main_sequence.clamp(0.0, 1.0);

    if health > 0.7 {
        // Healthy: green
        format!(
            "#{:02x}cc{:02x}",
            ((1.0 - health) * 100.0) as u8,
            (health * 200.0) as u8
        )
    } else if health > 0.4 {
        // Moderate: yellow/orange
        let yellow = ((health - 0.4) / 0.3 * 255.0) as u8;
        format!("#ff{yellow:02x}40")
    } else {
        // Needs attention: red
        format!("#ff{:02x}40", (health * 150.0) as u8)
    }
}

/// Legacy: instability-based color, kept for backwards compatibility with
/// downstream code that still calls it directly.
#[allow(dead_code)]
pub fn instability_color(instability: f64) -> String {
    let distance = (instability - 0.5).abs() * 2.0;
    health_color(distance * 0.5)
}

/// Edge shade based on coupling strength.
pub fn edge_color(strength: f64) -> String {
    let intensity = (strength * 200.0 + 55.0) as u8;
    format!("#{intensity:02x}{intensity:02x}{intensity:02x}")
}

/// Build a `Scene3D` for the supplied topology using the given config.
pub fn build_scene(
    topology: &Topology,
    cfg: &ForceDirectedConfig,
) -> Result<Scene3D, ProjectorError> {
    // Build module metrics lookup
    let module_metrics: std::collections::HashMap<_, _> =
        topology.modules.iter().map(|m| (m.id.clone(), m)).collect();

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    if let Some(matrix) = &topology.coupling_matrix {
        let positions = run_force_simulation(&matrix.modules, &matrix.values, cfg);

        for (i, module_id) in matrix.modules.iter().enumerate() {
            let pos = positions
                .get(module_id)
                .cloned()
                .unwrap_or([i as f64 * 2.0, 0.0, 0.0]);

            let metrics = module_metrics.get(module_id);

            let cyclomatic = metrics.map(|m| m.total_cyclomatic).unwrap_or(0);
            let cognitive = metrics.map(|m| m.total_cognitive).unwrap_or(0);
            let instability = metrics.map(|m| m.martin.instability).unwrap_or(0.5);
            let distance = metrics
                .map(|m| m.martin.distance_from_main_sequence)
                .unwrap_or(0.5);
            let function_count = metrics.map(|m| m.function_count).unwrap_or(0);

            let size = (function_count as f64 / 5.0 + 0.5).min(2.5) * cfg.node_scale;
            let color = health_color(distance);

            nodes.push(SceneNode {
                id: module_id.clone(),
                label: module_id.clone(),
                position: pos,
                size,
                color,
                metrics: NodeMetrics {
                    cyclomatic,
                    cognitive,
                    instability,
                    function_count,
                },
            });
        }

        // Build edges from upper triangle of coupling matrix.
        for (i, row) in matrix.values.iter().enumerate() {
            for (j, &strength) in row.iter().enumerate() {
                if j > i && strength >= cfg.min_edge_strength {
                    edges.push(SceneEdge {
                        from: matrix.modules[i].clone(),
                        to: matrix.modules[j].clone(),
                        strength,
                        color: edge_color(strength),
                        width: strength * 2.0,
                    });
                }
            }
        }
    }

    Ok(Scene3D {
        format: "topology-webgl/v1".into(),
        camera: Camera::default(),
        nodes,
        edges,
    })
}

/// Render the Three.js HTML viewer around a `Scene3D`.
pub fn wrap_in_html(scene: &Scene3D) -> Result<String, ProjectorError> {
    let scene_json = serde_json::to_string(scene).map_err(|e| ProjectorError {
        code: "RENDER_FAILED",
        message: "Failed to serialize scene for HTML".into(),
        source: Some(Box::new(e)),
    })?;
    let ctx = SceneCtx {
        scene_json: &scene_json,
        node_count: scene.nodes.len(),
        edge_count: scene.edges.len(),
    };
    Ok(render_template(TEMPLATE, &ctx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_color_is_green_when_on_main_sequence() {
        let color = health_color(0.0);
        assert!(
            color.contains("cc"),
            "healthy color should have green: {color}"
        );
    }

    #[test]
    fn health_color_is_red_when_far_from_main_sequence() {
        let color = health_color(1.0);
        assert!(
            color.starts_with("#ff"),
            "needs attention should be red: {color}"
        );
    }

    #[test]
    fn instability_color_returns_valid_hex() {
        for i in [0.0_f64, 0.5, 1.0] {
            let c = instability_color(i);
            assert!(c.starts_with("#"));
            assert_eq!(c.len(), 7);
        }
    }
}
