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

    /// Calculate node color based on instability.
    fn instability_color(instability: f64) -> String {
        // Red (unstable) to Blue (stable)
        let r = (instability * 255.0) as u8;
        let b = ((1.0 - instability) * 255.0) as u8;
        format!("#{r:02x}40{b:02x}")
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
    /// Build the 3D scene from topology.
    fn build_scene(
        &self,
        topology: &Topology,
        _cfg: &ForceDirectedConfig,
    ) -> Result<Scene3D, ProjectorError> {
        // Get positions from coupling matrix if available
        let positions = topology
            .coupling_matrix
            .as_ref()
            .and_then(|m| m.positions.as_ref());

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Build nodes from coupling matrix modules
        if let Some(matrix) = &topology.coupling_matrix {
            for (i, module_id) in matrix.modules.iter().enumerate() {
                let pos = positions
                    .and_then(|p| p.get(module_id))
                    .cloned()
                    .unwrap_or([i as f64 * 2.0, 0.0, 0.0]);

                nodes.push(SceneNode {
                    id: module_id.clone(),
                    label: module_id.clone(),
                    position: pos,
                    size: 1.0, // TODO: Calculate from metrics
                    color: Self::instability_color(0.5), // TODO: Get from metrics
                    metrics: NodeMetrics {
                        cyclomatic: 0,
                        cognitive: 0,
                        instability: 0.5,
                        function_count: 0,
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
        body {{ margin: 0; overflow: hidden; font-family: system-ui, sans-serif; }}
        #info {{
            position: absolute;
            top: 10px;
            left: 10px;
            padding: 10px;
            background: rgba(0,0,0,0.7);
            color: white;
            border-radius: 8px;
            font-size: 14px;
            z-index: 100;
        }}
        #info h3 {{ margin: 0 0 8px 0; }}
    </style>
</head>
<body>
    <div id="info">
        <h3>Code Topology</h3>
        <p>Nodes: {node_count} modules</p>
        <p>Edges: {edge_count} coupling relationships</p>
        <p><em>Drag to rotate, scroll to zoom</em></p>
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
        
        const scene = new THREE.Scene();
        scene.background = new THREE.Color(0x1a1a2e);
        
        const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
        camera.position.set(0, 5, 10);
        
        const renderer = new THREE.WebGLRenderer({{ antialias: true }});
        renderer.setSize(window.innerWidth, window.innerHeight);
        document.body.appendChild(renderer.domElement);
        
        const controls = new OrbitControls(camera, renderer.domElement);
        controls.enableDamping = true;
        
        // Add ambient light
        scene.add(new THREE.AmbientLight(0xffffff, 0.5));
        const dirLight = new THREE.DirectionalLight(0xffffff, 0.8);
        dirLight.position.set(5, 10, 7);
        scene.add(dirLight);
        
        // Load topology data
        const data = {scene_json};
        
        // Create nodes
        data.nodes.forEach(node => {{
            const geometry = new THREE.SphereGeometry(node.size * 0.3, 32, 32);
            const material = new THREE.MeshPhongMaterial({{ color: node.color }});
            const mesh = new THREE.Mesh(geometry, material);
            mesh.position.set(...node.position);
            mesh.userData = node;
            scene.add(mesh);
            
            // TODO: Add labels
        }});
        
        // Create edges
        data.edges.forEach(edge => {{
            const fromNode = data.nodes.find(n => n.id === edge.from);
            const toNode = data.nodes.find(n => n.id === edge.to);
            if (fromNode && toNode) {{
                const points = [
                    new THREE.Vector3(...fromNode.position),
                    new THREE.Vector3(...toNode.position)
                ];
                const geometry = new THREE.BufferGeometry().setFromPoints(points);
                const material = new THREE.LineBasicMaterial({{ 
                    color: edge.color,
                    linewidth: edge.width,
                    opacity: edge.strength,
                    transparent: true
                }});
                scene.add(new THREE.Line(geometry, material));
            }}
        }});
        
        // Animation loop
        function animate() {{
            requestAnimationFrame(animate);
            controls.update();
            renderer.render(scene, camera);
        }}
        animate();
        
        // Handle resize
        window.addEventListener('resize', () => {{
            camera.aspect = window.innerWidth / window.innerHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(window.innerWidth, window.innerHeight);
        }});
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
    fn test_instability_color() {
        // Fully stable (0.0) should be blue-ish
        let stable = ForceDirectedProjector::instability_color(0.0);
        assert!(stable.starts_with("#00"));

        // Fully unstable (1.0) should be red-ish
        let unstable = ForceDirectedProjector::instability_color(1.0);
        assert!(unstable.starts_with("#ff"));
    }
}

