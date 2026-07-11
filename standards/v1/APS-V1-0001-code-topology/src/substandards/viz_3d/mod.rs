//! 3D Force-Directed Coupling Visualization (APS-V1-0001.FD01)
//!
//! This projector renders the coupling matrix from code topology artifacts
//! as an interactive 3D visualization using force-directed layout.
//!
//! ## Layout
//!
//! - [`model`] holds the typed config and scene graph.
//! - [`layout`] runs the deterministic force simulation (pure math).
//! - [`render`] builds the `Scene3D` from a `Topology` and wraps it in
//!   the Three.js HTML viewer (template at `templates/scene.hbs`).
//!
//! This module keeps the public [`ForceDirectedProjector`] plus the
//! `Projector` trait impl so external callers see the same API surface
//! that existed before the split.
//!
//! ## Usage
//!
//! ```ignore
//! use code_topology::substandards::viz_3d::ForceDirectedProjector;
//! use code_topology::{Projector, OutputFormat};
//!
//! let projector = ForceDirectedProjector::new();
//! let topology = projector.load(Path::new(".topology"))?;
//! let scene = projector.render(&topology, OutputFormat::WebGL, None)?;
//! ```
//!
//! EXPERIMENTAL: This substandard is in incubation.

use std::path::Path;

use crate::{OutputFormat, Projector, ProjectorConfig, ProjectorError, Topology};

pub mod layout;
pub mod model;
pub mod render;

pub use model::{
    Camera, ColorScheme, ForceDirectedConfig, NodeMetrics, Scene3D, SceneEdge, SceneNode,
};

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
        if !topology_dir.exists() {
            return Err(ProjectorError {
                code: "TOPOLOGY_NOT_FOUND",
                message: format!("Directory not found: {}", topology_dir.display()),
                source: None,
            });
        }

        let coupling_matrix = topology_dir.join("graphs/coupling-matrix.json");
        if !coupling_matrix.exists() {
            return Err(ProjectorError {
                code: "REQUIRED_FILE_MISSING",
                message: "graphs/coupling-matrix.json is required for 3D visualization".into(),
                source: None,
            });
        }

        // TODO: Actually load and parse the topology
        Ok(Topology::default())
    }

    fn render(
        &self,
        topology: &Topology,
        format: OutputFormat,
        config: Option<&ProjectorConfig>,
    ) -> Result<Vec<u8>, ProjectorError> {
        let cfg = if let Some(proj_config) = config {
            serde_json::from_value(proj_config.raw.clone()).unwrap_or_else(|_| self.config.clone())
        } else {
            self.config.clone()
        };

        match format {
            OutputFormat::WebGL | OutputFormat::Json => {
                let scene = render::build_scene(topology, &cfg)?;
                let json = serde_json::to_vec_pretty(&scene).map_err(|e| ProjectorError {
                    code: "RENDER_FAILED",
                    message: "Failed to serialize scene".into(),
                    source: Some(Box::new(e)),
                })?;
                Ok(json)
            }
            OutputFormat::Html => {
                let scene = render::build_scene(topology, &cfg)?;
                let html = render::wrap_in_html(&scene)?;
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

/// Register this package with a composed APSS runner.
pub fn register(registry: &mut dyn apss_core::registry::StandardRegistry) {
    registry.register(
        apss_core::registry::RegisteredStandard {
            id: "APS-V1-0001.FD01".to_string(),
            slug: "force-directed".to_string(),
            name: "3D Force Directed".to_string(),
            description: "3D force-directed topology visualization substandard".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            commands: Vec::new(),
        },
        Box::new(NoopCommandHandler),
    );
}

struct NoopCommandHandler;

impl apss_core::registry::CommandHandler for NoopCommandHandler {
    fn execute(&self, _command: &str, _args: &[String], _config: &toml::Value) -> i32 {
        eprintln!("No composed CLI commands are registered for 3d01-force-directed yet.");
        5
    }

    fn commands(&self) -> Vec<apss_core::registry::CommandInfo> {
        Vec::new()
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
}
