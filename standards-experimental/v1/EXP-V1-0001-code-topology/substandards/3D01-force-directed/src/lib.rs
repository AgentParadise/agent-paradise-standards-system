//! 3D Force-Directed Coupling Visualization (EXP-V1-0001.3D01)
//!
//! This projector renders the coupling matrix from code topology artifacts
//! as an interactive 3D visualization using force-directed layout.
//!
//! ## Key Features
//!
//! - **Force-directed layout** — Tightly coupled modules cluster together
//! - **Deterministic positions** — Saves layout positions for reproducibility
//! - **Multiple formats** — WebGL scene, GLTF model, HTML viewer
//! - **Metric-driven sizing** — Node size reflects complexity
//!
//! ## Usage
//!
//! ```ignore
//! use code_topology_3d::ForceDirectedProjector;
//! use code_topology::{Projector, OutputFormat};
//!
//! let projector = ForceDirectedProjector::new();
//! let topology = projector.load(Path::new(".topology"))?;
//! let scene = projector.render(&topology, OutputFormat::WebGL, None)?;
//! ```
//!
//! ⚠️ EXPERIMENTAL: This substandard is in incubation.

use std::path::Path;

use code_topology::{OutputFormat, Projector, ProjectorConfig, ProjectorError, Topology};
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

/// The 3D Force-Directed Projector.
pub struct ForceDirectedProjector {
    config: ForceDirectedConfig,
}

impl ForceDirectedProjector {
    /// Create a new projector with default configuration.
    pub fn new() -> Self {
        Self {
            config: ForceDirectedConfig::default(),
        }
    }

    /// Create a projector with custom configuration.
    pub fn with_config(config: ForceDirectedConfig) -> Self {
        Self { config }
    }

    /// Calculate node color based on health score.
    ///
    /// Color scheme (intuitive):
    /// - 🔴 Red = needs attention (Zone of Pain, high coupling to stable concrete modules)
    /// - 🟡 Yellow = moderate concern
    /// - 🟢 Green = healthy
    ///
    /// Health is based on distance from main sequence:
    /// - D near 0 = on the main sequence = healthy (green)
    /// - D near 1 = Zone of Pain or Zone of Uselessness = needs attention (red)
    fn health_color(distance_from_main_sequence: f64) -> String {
        // Green (healthy, D≈0) to Red (needs attention, D≈1)
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

    /// Legacy: Calculate color based on instability (for backwards compat).
    #[allow(dead_code)]
    fn instability_color(instability: f64) -> String {
        // Map instability to distance approximation for color
        // Stable (I≈0) modules that are concrete are in Zone of Pain
        // Unstable (I≈1) modules that are abstract are in Zone of Uselessness
        // Middle is healthy
        let distance = (instability - 0.5).abs() * 2.0; // 0.5 = healthy, 0 or 1 = edges
        Self::health_color(distance * 0.5) // Scale down for less dramatic colors
    }

    /// Calculate edge color based on coupling strength.
    fn edge_color(strength: f64) -> String {
        // Strong coupling = bright, weak = dim
        let intensity = (strength * 200.0 + 55.0) as u8;
        format!("#{intensity:02x}{intensity:02x}{intensity:02x}")
    }
}

impl Default for ForceDirectedProjector {
    fn default() -> Self {
        Self::new()
    }
}

impl Projector for ForceDirectedProjector {
    fn id(&self) -> &'static str {
        "3d-force"
    }

    fn name(&self) -> &'static str {
        "3D Force-Directed Coupling Visualization"
    }

    fn description(&self) -> &'static str {
        "Renders coupling matrix as interactive 3D visualization where tightly coupled modules cluster together"
    }

    fn load(&self, topology_dir: &Path) -> Result<Topology, ProjectorError> {
        // Verify directory exists
        if !topology_dir.exists() {
            return Err(ProjectorError {
                code: "TOPOLOGY_NOT_FOUND",
                message: format!("Directory not found: {}", topology_dir.display()),
                source: None,
            });
        }

        // Check for required files
        let coupling_matrix = topology_dir.join("graphs/coupling-matrix.json");
        if !coupling_matrix.exists() {
            return Err(ProjectorError {
                code: "REQUIRED_FILE_MISSING",
                message: "graphs/coupling-matrix.json is required for 3D visualization".into(),
                source: None,
            });
        }

        // TODO: Actually load and parse the topology
        // For now, return placeholder
        Ok(Topology::default())
    }

    fn render(
        &self,
        topology: &Topology,
        format: OutputFormat,
        config: Option<&ProjectorConfig>,
    ) -> Result<Vec<u8>, ProjectorError> {
        // Merge config if provided
        let cfg = if let Some(proj_config) = config {
            serde_json::from_value(proj_config.raw.clone()).unwrap_or_else(|_| self.config.clone())
        } else {
            self.config.clone()
        };

        match format {
            OutputFormat::WebGL | OutputFormat::Json => {
                let scene = self.build_scene(topology, &cfg)?;
                let json = serde_json::to_vec_pretty(&scene).map_err(|e| ProjectorError {
                    code: "RENDER_FAILED",
                    message: "Failed to serialize scene".into(),
                    source: Some(Box::new(e)),
                })?;
                Ok(json)
            }
            OutputFormat::Html => {
                let scene = self.build_scene(topology, &cfg)?;
                let html = self.wrap_in_html(&scene)?;
                Ok(html.into_bytes())
            }
            _ => Err(ProjectorError {
                code: "UNSUPPORTED_FORMAT",
                message: format!("Format {format:?} not supported by 3d-force projector"),
                source: None,
            }),
        }
    }

    fn supported_formats(&self) -> &[OutputFormat] {
        &[OutputFormat::WebGL, OutputFormat::Json, OutputFormat::Html]
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "ForceDirectedConfig",
            "type": "object",
            "properties": {
                "nodeScale": { "type": "number", "default": 1.0 },
                "minEdgeStrength": { "type": "number", "default": 0.1, "minimum": 0, "maximum": 1 },
                "iterations": { "type": "integer", "default": 300 },
                "repulsion": { "type": "number", "default": 100.0 },
                "attraction": { "type": "number", "default": 0.5 },
                "seed": { "type": "integer", "default": 42 },
                "colorScheme": { "type": "string", "enum": ["instability", "complexity", "language", "custom"] }
            }
        }))
    }
}

impl ForceDirectedProjector {
    /// Run force-directed layout simulation to compute positions.
    fn run_force_simulation(
        modules: &[String],
        matrix: &[Vec<f64>],
        cfg: &ForceDirectedConfig,
    ) -> std::collections::HashMap<String, [f64; 3]> {
        use std::collections::HashMap;

        let n = modules.len();
        if n == 0 {
            return HashMap::new();
        }

        // Initialize positions in a circle (not a line!)
        let mut positions: Vec<[f64; 3]> = (0..n)
            .map(|i| {
                let angle = (i as f64 / n as f64) * 2.0 * std::f64::consts::PI;
                let radius = 5.0;
                [
                    angle.cos() * radius,
                    (i as f64 * 0.5) - (n as f64 * 0.25),
                    angle.sin() * radius,
                ]
            })
            .collect();

        // Run force simulation
        for _iter in 0..cfg.iterations {
            let mut forces: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0]; n];

            // Repulsion between all pairs
            for i in 0..n {
                for j in (i + 1)..n {
                    let dx = positions[j][0] - positions[i][0];
                    let dy = positions[j][1] - positions[i][1];
                    let dz = positions[j][2] - positions[i][2];
                    let dist_sq = dx * dx + dy * dy + dz * dz + 0.01; // Avoid division by zero
                    let dist = dist_sq.sqrt();

                    // Repulsion force (Coulomb's law)
                    let repulsion = cfg.repulsion / dist_sq;
                    let fx = (dx / dist) * repulsion;
                    let fy = (dy / dist) * repulsion;
                    let fz = (dz / dist) * repulsion;

                    forces[i][0] -= fx;
                    forces[i][1] -= fy;
                    forces[i][2] -= fz;
                    forces[j][0] += fx;
                    forces[j][1] += fy;
                    forces[j][2] += fz;
                }
            }

            // Attraction along edges (coupling)
            for i in 0..n {
                for j in 0..n {
                    if i != j && matrix[i][j] > cfg.min_edge_strength {
                        let coupling = matrix[i][j];
                        let dx = positions[j][0] - positions[i][0];
                        let dy = positions[j][1] - positions[i][1];
                        let dz = positions[j][2] - positions[i][2];
                        let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.1);

                        // Attraction force (spring)
                        let attraction = cfg.attraction * coupling * dist;
                        let fx = (dx / dist) * attraction;
                        let fy = (dy / dist) * attraction;
                        let fz = (dz / dist) * attraction;

                        forces[i][0] += fx;
                        forces[i][1] += fy;
                        forces[i][2] += fz;
                    }
                }
            }

            // Apply forces with damping
            let damping = 0.85;
            let max_displacement = 0.5;
            for i in 0..n {
                for d in 0..3 {
                    let displacement =
                        (forces[i][d] * 0.01).clamp(-max_displacement, max_displacement);
                    positions[i][d] += displacement * damping;
                }
            }
        }

        // Center the layout
        let mut center = [0.0, 0.0, 0.0];
        for pos in &positions {
            center[0] += pos[0];
            center[1] += pos[1];
            center[2] += pos[2];
        }
        for c in &mut center {
            *c /= n as f64;
        }
        for pos in &mut positions {
            pos[0] -= center[0];
            pos[1] -= center[1];
            pos[2] -= center[2];
        }

        // Build result map
        modules
            .iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), positions[i]))
            .collect()
    }

    /// Build the 3D scene from topology.
    fn build_scene(
        &self,
        topology: &Topology,
        cfg: &ForceDirectedConfig,
    ) -> Result<Scene3D, ProjectorError> {
        // Build module metrics lookup
        let module_metrics: std::collections::HashMap<_, _> =
            topology.modules.iter().map(|m| (m.id.clone(), m)).collect();

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Build nodes from coupling matrix modules
        if let Some(matrix) = &topology.coupling_matrix {
            // Run force simulation to compute positions
            let positions = Self::run_force_simulation(&matrix.modules, &matrix.values, cfg);

            for (i, module_id) in matrix.modules.iter().enumerate() {
                let pos = positions
                    .get(module_id)
                    .cloned()
                    .unwrap_or([i as f64 * 2.0, 0.0, 0.0]);

                // Look up metrics for this module
                let metrics = module_metrics.get(module_id);

                let cyclomatic = metrics.map(|m| m.total_cyclomatic).unwrap_or(0);
                let cognitive = metrics.map(|m| m.total_cognitive).unwrap_or(0);
                let instability = metrics.map(|m| m.martin.instability).unwrap_or(0.5);
                let distance = metrics
                    .map(|m| m.martin.distance_from_main_sequence)
                    .unwrap_or(0.5);
                let function_count = metrics.map(|m| m.function_count).unwrap_or(0);

                // Size based on function count, scaled
                let size = (function_count as f64 / 5.0 + 0.5).min(2.5) * cfg.node_scale;

                // Color based on health (distance from main sequence)
                // Red = needs attention (high distance), Green = healthy (low distance)
                let color = Self::health_color(distance);

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

            // Build edges from coupling matrix
            for (i, row) in matrix.values.iter().enumerate() {
                for (j, &strength) in row.iter().enumerate() {
                    // Only upper triangle, skip diagonal, skip weak edges
                    if j > i && strength >= self.config.min_edge_strength {
                        edges.push(SceneEdge {
                            from: matrix.modules[i].clone(),
                            to: matrix.modules[j].clone(),
                            strength,
                            color: Self::edge_color(strength),
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

    /// Wrap scene in self-contained HTML with Three.js viewer.
    fn wrap_in_html(&self, scene: &Scene3D) -> Result<String, ProjectorError> {
        let scene_json = serde_json::to_string(scene).map_err(|e| ProjectorError {
            code: "RENDER_FAILED",
            message: "Failed to serialize scene for HTML".into(),
            source: Some(Box::new(e)),
        })?;

        Ok(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Code Topology - 3D Coupling Visualization</title>
    <style>
        body {{ margin: 0; overflow: hidden; font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace; display: flex; background: #0f0f1a; }}
        #main {{ flex: 1; position: relative; }}
        #info {{
            position: absolute;
            top: 10px;
            left: 10px;
            padding: 15px 20px;
            background: linear-gradient(135deg, rgba(20,20,40,0.95), rgba(40,30,60,0.9));
            color: #e0e0e0;
            border-radius: 12px;
            font-size: 13px;
            z-index: 100;
            border: 1px solid rgba(255,255,255,0.1);
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 32px rgba(0,0,0,0.4);
        }}
        #info h3 {{ 
            margin: 0 0 12px 0; 
            font-size: 18px;
            background: linear-gradient(90deg, #ff6b9d, #c44569);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            font-weight: 700;
        }}
        #info p {{ margin: 4px 0; color: #b0b0b0; }}
        #info em {{ color: #888; font-size: 11px; }}
        #legend {{
            position: absolute;
            bottom: 20px;
            left: 10px;
            padding: 15px 20px;
            background: linear-gradient(135deg, rgba(20,20,40,0.95), rgba(40,30,60,0.9));
            color: #e0e0e0;
            border-radius: 12px;
            font-size: 12px;
            z-index: 100;
            border: 1px solid rgba(255,255,255,0.1);
        }}
        #legend h4 {{ margin: 0 0 10px 0; color: #aaa; font-weight: 500; }}
        .legend-item {{ display: flex; align-items: center; margin: 6px 0; }}
        .legend-color {{ width: 14px; height: 14px; border-radius: 50%; margin-right: 10px; }}
        #filter {{
            margin-top: 12px;
            padding-top: 12px;
            border-top: 1px solid rgba(255,255,255,0.1);
        }}
        #filter label {{ display: block; font-size: 11px; color: #888; margin-bottom: 6px; }}
        #filter input[type="range"] {{ width: 100%; cursor: pointer; }}
        #filter-value {{ float: right; color: #ff6b9d; }}
        #sidebar {{
            width: 280px;
            height: 100vh;
            background: linear-gradient(180deg, rgba(15,15,30,0.98), rgba(20,20,35,0.98));
            border-left: 1px solid rgba(255,255,255,0.1);
            overflow-y: scroll;
            padding: 15px;
            z-index: 100;
            box-sizing: border-box;
        }}
        #module-list {{
            max-height: calc(100vh - 80px);
            overflow-y: auto;
        }}
        #sidebar h2 {{ 
            font-size: 14px; 
            color: #888; 
            margin-bottom: 15px; 
            display: flex; 
            justify-content: space-between;
            align-items: center;
        }}
        #sidebar h2 span {{ color: #666; font-weight: normal; }}
        .module-item {{
            padding: 10px 12px;
            margin-bottom: 6px;
            background: rgba(255,255,255,0.03);
            border-radius: 8px;
            cursor: pointer;
            transition: all 0.2s;
            border: 1px solid transparent;
        }}
        .module-item:hover {{ border-color: rgba(255,255,255,0.1); background: rgba(255,255,255,0.06); }}
        .module-item.selected {{ border-color: #ff6b9d; background: rgba(255,107,157,0.1); }}
        .module-item.filtered-out {{ opacity: 0.3; pointer-events: none; }}
        .module-name {{ 
            font-weight: 500; 
            margin-bottom: 4px; 
            font-size: 12px; 
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }}
        .module-stats {{ font-size: 10px; color: #666; display: flex; gap: 10px; }}
        .module-coupling {{ color: #88aaff; }}
        .module-complexity {{ color: #ffaa88; }}
        #tooltip {{
            position: fixed;
            padding: 16px 20px;
            background: rgba(20,20,35,0.98);
            color: #fff;
            border-radius: 12px;
            font-size: 13px;
            pointer-events: none;
            display: none;
            z-index: 10000;
            border: 2px solid rgba(100,180,255,0.6);
            min-width: 220px;
            max-width: 320px;
            box-shadow: 0 8px 32px rgba(0,0,0,0.7), 0 0 20px rgba(100,180,255,0.3);
            backdrop-filter: blur(10px);
        }}
        #tooltip .name {{
            font-weight: 700;
            font-size: 16px;
            margin-bottom: 12px;
            color: #6bf;
            word-break: break-word;
        }}
        #tooltip .metric {{
            display: flex;
            justify-content: space-between;
            margin: 6px 0;
            border-bottom: 1px solid rgba(255,255,255,0.15);
            padding-bottom: 6px;
        }}
        #tooltip .metric-label {{ color: #aaa; font-size: 12px; }}
        #tooltip .metric-value {{ color: #fff; font-weight: 600; font-size: 14px; }}
        #tooltip .active-hint {{
            margin-top: 12px;
            padding-top: 10px;
            border-top: 1px solid rgba(100,180,255,0.4);
            font-size: 12px;
            color: #8cf;
            text-align: center;
            font-style: italic;
        }}
        .node-label {{
            color: #fff;
            font-size: 12px;
            font-weight: 600;
            text-shadow: 0 2px 8px rgba(0,0,0,0.8), 0 0 20px rgba(0,0,0,0.6);
            pointer-events: none;
            white-space: nowrap;
        }}
        .edge-label {{
            color: rgba(255,255,255,0.6);
            font-size: 10px;
            text-shadow: 0 1px 4px rgba(0,0,0,0.9);
            pointer-events: none;
        }}
    </style>
</head>
<body>
    <div id="main">
        <div id="info">
            <h3>🌐 Code Topology</h3>
            <p><strong>{node_count}</strong> modules</p>
            <p><strong>{edge_count}</strong> coupling relationships</p>
            <p><em>Drag to rotate • Scroll to zoom • Hover for details</em></p>
            <div id="filter">
                <label>Coupling Threshold <span id="filter-value">0%</span></label>
                <input type="range" id="coupling-slider" min="0" max="100" value="0">
            </div>
        </div>
        <div id="legend">
            <h4>Module Health</h4>
            <div class="legend-item"><div class="legend-color" style="background: #00cc88;"></div>🟢 Healthy (on main sequence)</div>
            <div class="legend-item"><div class="legend-color" style="background: #ffaa40;"></div>🟡 Moderate concern</div>
            <div class="legend-item"><div class="legend-color" style="background: #ff4040;"></div>🔴 Needs attention (Zone of Pain)</div>
            <h4 style="margin-top: 12px;">Coupling Strength</h4>
            <div class="legend-item"><div class="legend-color" style="background: #ffffff; border: 1px solid #666;"></div>Strong (≥0.7)</div>
            <div class="legend-item"><div class="legend-color" style="background: #888888;"></div>Medium (0.3-0.7)</div>
            <div class="legend-item"><div class="legend-color" style="background: #444444;"></div>Weak (&lt;0.3)</div>
        </div>
        <div id="tooltip"></div>
    </div>
    <div id="sidebar">
        <h2>Modules <span id="module-count"></span></h2>
        <div id="module-list"></div>
    </div>
    <script type="importmap">
    {{
        "imports": {{
            "three": "https://cdn.jsdelivr.net/npm/three@0.160.0/build/three.module.js",
            "three/addons/": "https://cdn.jsdelivr.net/npm/three@0.160.0/examples/jsm/"
        }}
    }}
    </script>
    <script type="module">
        import * as THREE from 'three';
        import {{ OrbitControls }} from 'three/addons/controls/OrbitControls.js';
        import {{ CSS2DRenderer, CSS2DObject }} from 'three/addons/renderers/CSS2DRenderer.js';
        
        const scene = new THREE.Scene();
        scene.background = new THREE.Color(0x0f0f1a);
        
        const mainContainer = document.getElementById('main');
        const mainWidth = mainContainer.clientWidth;
        
        const camera = new THREE.PerspectiveCamera(75, mainWidth / window.innerHeight, 0.1, 1000);
        camera.position.set(0, 5, 10);
        
        const renderer = new THREE.WebGLRenderer({{ antialias: true }});
        renderer.setSize(mainWidth, window.innerHeight);
        renderer.setPixelRatio(window.devicePixelRatio);
        mainContainer.appendChild(renderer.domElement);
        
        // CSS2D Renderer for labels
        const labelRenderer = new CSS2DRenderer();
        labelRenderer.setSize(mainWidth, window.innerHeight);
        labelRenderer.domElement.style.position = 'absolute';
        labelRenderer.domElement.style.top = '0px';
        labelRenderer.domElement.style.pointerEvents = 'none';
        mainContainer.appendChild(labelRenderer.domElement);
        
        const controls = new OrbitControls(camera, renderer.domElement);
        controls.enableDamping = true;
        controls.dampingFactor = 0.05;
        
        // Lighting
        scene.add(new THREE.AmbientLight(0xffffff, 0.4));
        const dirLight = new THREE.DirectionalLight(0xffffff, 0.8);
        dirLight.position.set(5, 10, 7);
        scene.add(dirLight);
        const backLight = new THREE.DirectionalLight(0x6060ff, 0.3);
        backLight.position.set(-5, -5, -5);
        scene.add(backLight);
        
        // Load topology data
        const data = {scene_json};
        
        const nodeMeshes = [];
        const tooltip = document.getElementById('tooltip');
        const raycaster = new THREE.Raycaster();
        const mouse = new THREE.Vector2();
        
        // Create nodes with labels
        data.nodes.forEach(node => {{
            // Create sphere - minimum size for easy clicking
            const nodeRadius = Math.max(0.6, node.size * 0.5);
            const geometry = new THREE.SphereGeometry(nodeRadius, 32, 32);
            const material = new THREE.MeshPhongMaterial({{
                color: node.color,
                emissive: node.color,
                emissiveIntensity: 0.4,
                shininess: 100,
                transparent: true,
                opacity: 1.0
            }});
            const mesh = new THREE.Mesh(geometry, material);
            mesh.position.set(...node.position);
            mesh.userData = node;
            scene.add(mesh);
            nodeMeshes.push(mesh);
            
            // Create text label
            const labelDiv = document.createElement('div');
            labelDiv.className = 'node-label';
            labelDiv.textContent = node.label;
            const label = new CSS2DObject(labelDiv);
            label.position.set(0, node.size * 0.5 + 0.3, 0);
            mesh.add(label);
        }});
        
        // Create edges with labels
        data.edges.forEach(edge => {{
            const fromNode = data.nodes.find(n => n.id === edge.from);
            const toNode = data.nodes.find(n => n.id === edge.to);
            if (fromNode && toNode) {{
                const start = new THREE.Vector3(...fromNode.position);
                const end = new THREE.Vector3(...toNode.position);
                
                // Create tube for thicker edges
                const path = new THREE.LineCurve3(start, end);
                const tubeGeometry = new THREE.TubeGeometry(path, 1, edge.strength * 0.08 + 0.02, 8, false);
                const tubeMaterial = new THREE.MeshBasicMaterial({{ 
                    color: edge.color,
                    opacity: 0.4 + edge.strength * 0.4,
                    transparent: true
                }});
                const tube = new THREE.Mesh(tubeGeometry, tubeMaterial);
                scene.add(tube);
                
                // Edge label at midpoint (only for strong connections)
                if (edge.strength >= 0.5) {{
                    const midpoint = start.clone().add(end).multiplyScalar(0.5);
                    const edgeLabelDiv = document.createElement('div');
                    edgeLabelDiv.className = 'edge-label';
                    edgeLabelDiv.textContent = edge.strength.toFixed(2);
                    const edgeLabel = new CSS2DObject(edgeLabelDiv);
                    edgeLabel.position.copy(midpoint);
                    scene.add(edgeLabel);
                }}
            }}
        }});
        
        // Collect edge meshes for filtering/highlighting
        const edgeMeshes = [];
        scene.children.forEach(child => {{
            if (child.geometry && child.geometry.type === 'TubeGeometry') {{
                edgeMeshes.push(child);
            }}
        }});
        
        // Highlight module and its connections, fade everything else
        let hoveredModule = null;
        let hoverSource = null; // 'sidebar' or '3d'
        let currentThreshold = 0;
        
        function highlightModule(moduleId, source) {{
            hoveredModule = moduleId;
            hoverSource = source || '3d';
            
            // Find connected modules
            const connectedModules = new Set([moduleId]);
            data.edges.forEach(edge => {{
                if (edge.from === moduleId) connectedModules.add(edge.to);
                if (edge.to === moduleId) connectedModules.add(edge.from);
            }});
            
            // Fade out non-connected nodes
            nodeMeshes.forEach(mesh => {{
                const isConnected = connectedModules.has(mesh.userData.id);
                mesh.material.opacity = isConnected ? 1.0 : 0.15;
                mesh.material.transparent = true;
                mesh.scale.setScalar(mesh.userData.id === moduleId ? 1.4 : (isConnected ? 1.1 : 0.8));
            }});
            
            // Fade out non-connected edges
            edgeMeshes.forEach((mesh, i) => {{
                if (i < data.edges.length) {{
                    const edge = data.edges[i];
                    const isConnected = edge.from === moduleId || edge.to === moduleId;
                    mesh.material.opacity = isConnected ? 1.0 : 0.05;
                    mesh.material.transparent = true;
                }}
            }});
            
            // Highlight sidebar item
            document.querySelectorAll('.module-item').forEach(el => {{
                const isConnected = connectedModules.has(el.dataset.id);
                el.style.opacity = isConnected ? '1' : '0.3';
            }});
        }}
        
        function clearHighlight(source) {{
            // Only clear if source matches or no source specified
            if (!hoveredModule) return;
            if (source && hoverSource !== source) return;
            
            hoveredModule = null;
            hoverSource = null;
            
            // Restore all nodes
            nodeMeshes.forEach(mesh => {{
                mesh.material.opacity = 1.0;
                mesh.scale.setScalar(1.0);
            }});
            
            // Restore all edges (respecting threshold filter)
            edgeMeshes.forEach((mesh, i) => {{
                if (i < data.edges.length) {{
                    mesh.material.opacity = 0.6;
                    mesh.visible = data.edges[i].strength >= currentThreshold;
                }}
            }});
            
            // Restore sidebar
            document.querySelectorAll('.module-item').forEach(el => {{
                el.style.opacity = '';
            }});
        }}
        
        // Track active (clicked) module in 3D view
        let activeModule = null;
        
        // Mouse hover for tooltips only (no highlighting on hover)
        function onMouseMove(event) {{
            // Calculate mouse position relative to the canvas, not the window
            const rect = renderer.domElement.getBoundingClientRect();
            mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
            mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
            
            raycaster.setFromCamera(mouse, camera);
            const intersects = raycaster.intersectObjects(nodeMeshes);
            
            if (intersects.length > 0) {{
                const node = intersects[0].object.userData;
                tooltip.style.display = 'block';
                
                // Position tooltip, keeping it on screen
                const tooltipWidth = 280;
                const tooltipHeight = 200;
                let left = event.clientX + 20;
                let top = event.clientY + 20;
                
                // Keep on right side of screen
                if (left + tooltipWidth > window.innerWidth) {{
                    left = event.clientX - tooltipWidth - 20;
                }}
                // Keep on bottom of screen
                if (top + tooltipHeight > window.innerHeight) {{
                    top = event.clientY - tooltipHeight - 20;
                }}
                
                tooltip.style.left = left + 'px';
                tooltip.style.top = top + 'px';
                
                // Show cursor as pointer and highlight hovered node
                document.body.style.cursor = 'pointer';
                
                // Glow effect on hovered node
                const hoveredMesh = intersects[0].object;
                if (!hoveredMesh.userData.isHovered) {{
                    // Reset previous hovered node
                    nodeMeshes.forEach(m => {{
                        if (m.userData.isHovered && m !== hoveredMesh) {{
                            m.userData.isHovered = false;
                            m.material.emissiveIntensity = 0.4;
                            m.scale.setScalar(1.0);
                        }}
                    }});
                    // Highlight current
                    hoveredMesh.userData.isHovered = true;
                    hoveredMesh.material.emissiveIntensity = 0.8;
                    hoveredMesh.scale.setScalar(1.15);
                }}

                // Find connections
                const connections = data.edges
                    .filter(e => e.from === node.id || e.to === node.id)
                    .map(e => {{
                        const other = e.from === node.id ? e.to : e.from;
                        return `${{other}} (${{e.strength.toFixed(2)}})`;
                    }})
                    .join(', ');
                
                tooltip.innerHTML = `
                    <div class="name">${{node.label}}</div>
                    <div class="metric">
                        <span class="metric-label">Cyclomatic</span>
                        <span class="metric-value">${{node.metrics.cyclomatic}}</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Cognitive</span>
                        <span class="metric-value">${{node.metrics.cognitive}}</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Instability</span>
                        <span class="metric-value">${{(node.metrics.instability * 100).toFixed(0)}}%</span>
                    </div>
                    <div class="metric">
                        <span class="metric-label">Connected to</span>
                        <span class="metric-value">${{connections || 'none'}}</span>
                    </div>
                    ${{activeModule === node.id ? '<div class="active-hint">Click to deactivate</div>' : '<div class="active-hint">Click to focus connections</div>'}}
                `;
            }} else {{
                tooltip.style.display = 'none';
                document.body.style.cursor = 'default';
                
                // Reset any hovered node glow
                nodeMeshes.forEach(m => {{
                    if (m.userData.isHovered) {{
                        m.userData.isHovered = false;
                        m.material.emissiveIntensity = 0.4;
                        m.scale.setScalar(1.0);
                    }}
                }});
            }}
        }}

        // Click to activate/deactivate connection highlighting
        function onClick(event) {{
            raycaster.setFromCamera(mouse, camera);
            const intersects = raycaster.intersectObjects(nodeMeshes);
            
            if (intersects.length > 0) {{
                const node = intersects[0].object.userData;
                
                if (activeModule === node.id) {{
                    // Deactivate
                    activeModule = null;
                    clearHighlight('3d');
                }} else {{
                    // Activate new module
                    activeModule = node.id;
                    highlightModule(node.id, '3d');
                }}
            }} else {{
                // Clicked on empty space - deactivate
                if (activeModule) {{
                    activeModule = null;
                    clearHighlight('3d');
                }}
            }}
        }}

        window.addEventListener('mousemove', onMouseMove);
        window.addEventListener('click', onClick);
        
        // Calculate total coupling per module
        const moduleCoupling = {{}};
        data.nodes.forEach(n => moduleCoupling[n.id] = 0);
        data.edges.forEach(e => {{
            moduleCoupling[e.from] = (moduleCoupling[e.from] || 0) + e.strength;
            moduleCoupling[e.to] = (moduleCoupling[e.to] || 0) + e.strength;
        }});
        
        // Sort modules by coupling (descending)
        const sortedModules = [...data.nodes].sort((a, b) => 
            (moduleCoupling[b.id] || 0) - (moduleCoupling[a.id] || 0)
        );
        
        // Populate sidebar
        let selectedModule = null;
        function renderSidebar() {{
            document.getElementById('module-count').textContent = `(${{data.nodes.length}})`;
            const list = document.getElementById('module-list');
            list.innerHTML = sortedModules.map(m => {{
                const coupling = moduleCoupling[m.id] || 0;
                const shortName = m.label.split('::').pop() || m.label;
                return `
                    <div class="module-item ${{selectedModule === m.id ? 'selected' : ''}}" data-id="${{m.id}}">
                        <div class="module-name" style="color:${{m.color}}">${{shortName}}</div>
                        <div class="module-stats">
                            <span class="module-coupling">⚡ ${{coupling.toFixed(1)}}</span>
                            <span class="module-complexity">📊 CC:${{m.metrics.cyclomatic}}</span>
                            <span>🔧 ${{m.metrics.function_count}}</span>
                        </div>
                    </div>
                `;
            }}).join('');
            
            // Add click and hover handlers
            list.querySelectorAll('.module-item').forEach(item => {{
                item.addEventListener('click', () => {{
                    const id = item.dataset.id;
                    selectedModule = selectedModule === id ? null : id;
                    renderSidebar();
                    
                    // Focus camera on selected module
                    if (selectedModule) {{
                        const node = data.nodes.find(n => n.id === selectedModule);
                        if (node) {{
                            const targetPos = new THREE.Vector3(...node.position);
                            controls.target.copy(targetPos);
                            camera.position.set(
                                targetPos.x + 5,
                                targetPos.y + 3,
                                targetPos.z + 5
                            );
                        }}
                    }}
                    
                    // Highlight selected node
                    nodeMeshes.forEach(mesh => {{
                        const isSelected = mesh.userData.id === selectedModule;
                        mesh.material.emissiveIntensity = isSelected ? 0.6 : 0.2;
                        mesh.scale.setScalar(isSelected ? 1.3 : 1.0);
                    }});
                }});
                
                // Hover handlers for focus effect
                item.addEventListener('mouseenter', () => {{
                    const id = item.dataset.id;
                    highlightModule(id, 'sidebar');
                }});
                
                item.addEventListener('mouseleave', () => {{
                    clearHighlight('sidebar');
                }});
            }});
        }}
        renderSidebar();
        
        // Coupling threshold slider
        document.getElementById('coupling-slider').addEventListener('input', e => {{
            currentThreshold = parseInt(e.target.value) / 100;
            document.getElementById('filter-value').textContent = `${{e.target.value}}%`;
            
            // Filter edges based on threshold
            edgeMeshes.forEach((mesh, i) => {{
                if (i < data.edges.length) {{
                    mesh.visible = data.edges[i].strength >= currentThreshold;
                }}
            }});
            
            // Find modules that still have visible edges
            const visibleModules = new Set();
            data.edges.forEach(edge => {{
                if (edge.strength >= currentThreshold) {{
                    visibleModules.add(edge.from);
                    visibleModules.add(edge.to);
                }}
            }});
            
            // Grey out modules in sidebar that have no visible edges
            document.querySelectorAll('.module-item').forEach(item => {{
                const moduleId = item.dataset.id;
                const hasVisibleEdge = visibleModules.has(moduleId);
                item.classList.toggle('filtered-out', currentThreshold > 0 && !hasVisibleEdge);
            }});
            
            // Also fade out 3D nodes that have no visible edges
            nodeMeshes.forEach(mesh => {{
                const hasVisibleEdge = visibleModules.has(mesh.userData.id);
                mesh.material.opacity = (currentThreshold > 0 && !hasVisibleEdge) ? 0.2 : 1.0;
                mesh.material.transparent = true;
            }});
        }});
        
        // Animation loop
        function animate() {{
            requestAnimationFrame(animate);
            controls.update();
            renderer.render(scene, camera);
            labelRenderer.render(scene, camera);
        }}
        animate();
        
        // Handle resize
        window.addEventListener('resize', () => {{
            const mainWidth = document.getElementById('main').clientWidth;
            camera.aspect = mainWidth / window.innerHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(mainWidth, window.innerHeight);
            labelRenderer.setSize(mainWidth, window.innerHeight);
        }});
        
        // Trigger initial resize
        window.dispatchEvent(new Event('resize'));
    </script>
</body>
</html>"#,
            node_count = scene.nodes.len(),
            edge_count = scene.edges.len(),
            scene_json = scene_json
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projector_creation() {
        let projector = ForceDirectedProjector::new();
        assert_eq!(projector.id(), "3d-force");
    }

    #[test]
    fn test_supported_formats() {
        let projector = ForceDirectedProjector::new();
        let formats = projector.supported_formats();
        assert!(formats.contains(&OutputFormat::WebGL));
        assert!(formats.contains(&OutputFormat::Json));
        assert!(formats.contains(&OutputFormat::Html));
    }

    #[test]
    fn test_config_schema() {
        let projector = ForceDirectedProjector::new();
        let schema = projector.config_schema();
        assert!(schema.is_some());
    }

    #[test]
    fn test_default_config() {
        let config = ForceDirectedConfig::default();
        assert_eq!(config.node_scale, 1.0);
        assert_eq!(config.iterations, 300);
        assert_eq!(config.seed, 42);
    }

    #[test]
    fn test_health_color() {
        // Healthy (distance = 0) should be green-ish
        let healthy = ForceDirectedProjector::health_color(0.0);
        assert!(
            healthy.contains("cc"),
            "Healthy color should have green: {healthy}"
        );

        // Needs attention (distance = 1.0) should be red-ish
        let needs_attention = ForceDirectedProjector::health_color(1.0);
        assert!(
            needs_attention.starts_with("#ff"),
            "Needs attention should be red: {needs_attention}"
        );
    }

    #[test]
    fn test_instability_color_backwards_compat() {
        // Just verify it returns a valid hex color
        let stable = ForceDirectedProjector::instability_color(0.0);
        assert!(stable.starts_with("#"));
        assert_eq!(stable.len(), 7);

        let unstable = ForceDirectedProjector::instability_color(1.0);
        assert!(unstable.starts_with("#"));
        assert_eq!(unstable.len(), 7);
    }
}
