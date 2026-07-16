//! Data model for the 3D force-directed projector.
//!
//! Contains the config (`ForceDirectedConfig`, `ColorScheme`) and the
//! JSON-serialised scene graph (`Scene3D`, `Camera`, `SceneNode`,
//! `SceneEdge`, `NodeMetrics`) that gets embedded in the HTML viewer.

use serde::{Deserialize, Serialize};

/// Configuration for the 3D force-directed projector.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceDirectedConfig {
    /// Scale factor for node sizes (default: 1.0)
    #[serde(default = "default_node_scale")]
    pub node_scale: f64,

    /// Minimum edge strength to render (0.0-1.0, default: 0.1)
    #[serde(default = "default_min_edge_strength")]
    pub min_edge_strength: f64,

    /// Force simulation iterations (default: 300)
    #[serde(default = "default_iterations")]
    pub iterations: u32,

    /// Repulsion strength between nodes (default: 100.0)
    #[serde(default = "default_repulsion")]
    pub repulsion: f64,

    /// Attraction strength along edges (default: 0.5)
    #[serde(default = "default_attraction")]
    pub attraction: f64,

    /// Random seed for layout (default: 42)
    #[serde(default = "default_seed")]
    pub seed: u64,

    /// Color scheme for nodes
    #[serde(default)]
    pub color_scheme: ColorScheme,
}

fn default_node_scale() -> f64 {
    1.0
}
fn default_min_edge_strength() -> f64 {
    0.1
}
fn default_iterations() -> u32 {
    300
}
fn default_repulsion() -> f64 {
    100.0
}
fn default_attraction() -> f64 {
    0.5
}
fn default_seed() -> u64 {
    42
}

impl Default for ForceDirectedConfig {
    fn default() -> Self {
        Self {
            node_scale: default_node_scale(),
            min_edge_strength: default_min_edge_strength(),
            iterations: default_iterations(),
            repulsion: default_repulsion(),
            attraction: default_attraction(),
            seed: default_seed(),
            color_scheme: ColorScheme::default(),
        }
    }
}

/// Color scheme for 3D visualization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
    /// Colors based on coupling instability (red = unstable, blue = stable)
    #[default]
    Instability,
    /// Colors based on complexity (red = high, green = low)
    Complexity,
    /// Colors based on module/language
    Language,
    /// Custom colors provided in config
    Custom,
}

/// 3D scene output format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene3D {
    /// Format identifier
    pub format: String,
    /// Camera configuration
    pub camera: Camera,
    /// Nodes (modules)
    pub nodes: Vec<SceneNode>,
    /// Edges (coupling relationships)
    pub edges: Vec<SceneEdge>,
}

/// Camera configuration for 3D scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    /// Camera position [x, y, z]
    pub position: [f64; 3],
    /// Look-at target [x, y, z]
    pub target: [f64; 3],
    /// Up vector [x, y, z]
    #[serde(default = "default_up")]
    pub up: [f64; 3],
}

fn default_up() -> [f64; 3] {
    [0.0, 1.0, 0.0]
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: [0.0, 5.0, 10.0],
            target: [0.0, 0.0, 0.0],
            up: default_up(),
        }
    }
}

/// A node in the 3D scene (represents a module).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    /// Module ID
    pub id: String,
    /// Display label
    pub label: String,
    /// 3D position [x, y, z]
    pub position: [f64; 3],
    /// Node size (based on complexity)
    pub size: f64,
    /// Node color (hex)
    pub color: String,
    /// Associated metrics
    pub metrics: NodeMetrics,
}

/// Metrics attached to a scene node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// Total cyclomatic complexity
    pub cyclomatic: u32,
    /// Total cognitive complexity
    pub cognitive: u32,
    /// Instability (Martin's metric)
    pub instability: f64,
    /// Function count
    pub function_count: u32,
}

/// An edge in the 3D scene (represents coupling).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneEdge {
    /// Source module ID
    pub from: String,
    /// Target module ID
    pub to: String,
    /// Coupling strength (0.0-1.0)
    pub strength: f64,
    /// Edge color (hex)
    pub color: String,
    /// Edge width (based on strength)
    pub width: f64,
}
