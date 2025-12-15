//! Mermaid Diagram Projector (EXP-V1-0001.VIZ01)
//!
//! Generates Mermaid diagrams from code topology artifacts:
//! - Dependency flowcharts
//! - C4 Context/Container diagrams
//! - Module relationship graphs
//!
//! ## Usage
//!
//! ```ignore
//! use code_topology_mermaid::MermaidProjector;
//! use code_topology::{Projector, OutputFormat};
//!
//! let projector = MermaidProjector::new();
//! let mermaid = projector.render(&topology, OutputFormat::Mermaid, None)?;
//! println!("{}", String::from_utf8_lossy(&mermaid));
//! ```
//!
//! ⚠️ EXPERIMENTAL: This substandard is in incubation.

use std::path::Path;

use code_topology::{OutputFormat, Projector, ProjectorConfig, ProjectorError, Topology};
use serde::{Deserialize, Serialize};

/// Mermaid diagram style.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagramStyle {
    /// Simple flowchart (graph LR/TD)
    #[default]
    Flowchart,
    /// C4 Context diagram
    C4Context,
    /// C4 Container diagram
    C4Container,
    /// Class diagram (for module structure)
    ClassDiagram,
}

/// Configuration for the Mermaid projector.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MermaidConfig {
    /// Diagram style
    #[serde(default)]
    pub style: DiagramStyle,

    /// Direction: LR (left-right), TD (top-down), etc.
    #[serde(default = "default_direction")]
    pub direction: String,

    /// Minimum coupling to show edges
    #[serde(default = "default_min_coupling")]
    pub min_coupling: f64,

    /// Show coupling strength on edges
    #[serde(default = "default_show_strength")]
    pub show_strength: bool,

    /// Theme: default, dark, forest, neutral
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_direction() -> String {
    "LR".into()
}
fn default_min_coupling() -> f64 {
    0.3
}
fn default_show_strength() -> bool {
    true
}
fn default_theme() -> String {
    "dark".into()
}

impl Default for MermaidConfig {
    fn default() -> Self {
        Self {
            style: DiagramStyle::default(),
            direction: default_direction(),
            min_coupling: default_min_coupling(),
            show_strength: default_show_strength(),
            theme: default_theme(),
        }
    }
}

/// The Mermaid Diagram Projector.
pub struct MermaidProjector {
    config: MermaidConfig,
}

impl MermaidProjector {
    /// Create a new projector with default configuration.
    pub fn new() -> Self {
        Self {
            config: MermaidConfig::default(),
        }
    }

    /// Create a projector with custom configuration.
    pub fn with_config(config: MermaidConfig) -> Self {
        Self { config }
    }

    /// Generate a flowchart diagram.
    fn render_flowchart(&self, topology: &Topology, cfg: &MermaidConfig) -> String {
        let mut lines = vec![format!("graph {}", cfg.direction)];

        // Add theme
        if cfg.theme != "default" {
            lines.insert(0, format!("%%{{init: {{'theme': '{}'}}}}%%", cfg.theme));
        }

        // Build module metrics lookup for styling
        let module_metrics: std::collections::HashMap<_, _> = topology
            .modules
            .iter()
            .map(|m| (m.id.clone(), m))
            .collect();

        if let Some(matrix) = &topology.coupling_matrix {
            // Add nodes with styling based on health
            for module_id in &matrix.modules {
                let safe_id = Self::sanitize_id(module_id);
                let display_name = module_id.replace("::", "/");

                // Get metrics for styling
                let metrics = module_metrics.get(module_id);
                let distance = metrics
                    .map(|m| m.martin.distance_from_main_sequence)
                    .unwrap_or(0.5);

                // Style based on health
                let style = if distance > 0.6 {
                    ":::danger" // Red - needs attention
                } else if distance > 0.3 {
                    ":::warning" // Yellow - moderate
                } else {
                    ":::success" // Green - healthy
                };

                lines.push(format!("    {}[\"📦 {}\"]{}",safe_id, display_name, style));
            }

            // Add edges for coupling relationships
            for (i, row) in matrix.values.iter().enumerate() {
                for (j, &strength) in row.iter().enumerate() {
                    if j > i && strength >= cfg.min_coupling {
                        let from = Self::sanitize_id(&matrix.modules[i]);
                        let to = Self::sanitize_id(&matrix.modules[j]);

                        let edge = if cfg.show_strength {
                            let pct = (strength * 100.0) as u32;
                            if strength >= 0.7 {
                                format!("    {} ==>|{}%| {}", from, pct, to)
                            } else {
                                format!("    {} -->|{}%| {}", from, pct, to)
                            }
                        } else if strength >= 0.7 {
                            format!("    {} ==> {}", from, to)
                        } else {
                            format!("    {} --> {}", from, to)
                        };
                        lines.push(edge);
                    }
                }
            }
        }

        // Add class definitions for styling
        lines.push(String::new());
        lines.push("    classDef danger fill:#ff6b6b,stroke:#c92a2a,color:#fff".into());
        lines.push("    classDef warning fill:#ffa94d,stroke:#e67700,color:#000".into());
        lines.push("    classDef success fill:#51cf66,stroke:#2b8a3e,color:#000".into());

        lines.join("\n")
    }

    /// Generate a C4 Context diagram.
    fn render_c4_context(&self, topology: &Topology, _cfg: &MermaidConfig) -> String {
        let mut lines = vec![
            "C4Context".to_string(),
            "    title System Context - Code Topology".to_string(),
        ];

        // Group modules by prefix (e.g., "aps-core::" becomes a system)
        let mut systems: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        if let Some(matrix) = &topology.coupling_matrix {
            for module_id in &matrix.modules {
                let system = if module_id.contains("::") {
                    module_id.split("::").next().unwrap_or(module_id).to_string()
                } else {
                    module_id.clone()
                };
                systems.entry(system).or_default().push(module_id.clone());
            }
        }

        // Render systems
        for (system, modules) in &systems {
            let safe_id = Self::sanitize_id(system);
            let desc = format!("{} modules", modules.len());
            lines.push(format!(
                "    System({}, \"{}\", \"{}\")",
                safe_id, system, desc
            ));
        }

        // Render relationships between systems
        if let Some(matrix) = &topology.coupling_matrix {
            let mut system_rels: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();

            for (i, row) in matrix.values.iter().enumerate() {
                for (j, &strength) in row.iter().enumerate() {
                    if j > i && strength >= 0.3 {
                        let sys_i = matrix.modules[i].split("::").next().unwrap_or(&matrix.modules[i]);
                        let sys_j = matrix.modules[j].split("::").next().unwrap_or(&matrix.modules[j]);
                        if sys_i != sys_j {
                            let key = if sys_i < sys_j {
                                (sys_i.to_string(), sys_j.to_string())
                            } else {
                                (sys_j.to_string(), sys_i.to_string())
                            };
                            system_rels.insert(key);
                        }
                    }
                }
            }

            for (from, to) in system_rels {
                lines.push(format!(
                    "    Rel({}, {}, \"depends on\")",
                    Self::sanitize_id(&from),
                    Self::sanitize_id(&to)
                ));
            }
        }

        lines.join("\n")
    }

    /// Generate a C4 Container diagram.
    fn render_c4_container(&self, topology: &Topology, _cfg: &MermaidConfig) -> String {
        let mut lines = vec![
            "C4Container".to_string(),
            "    title Container Diagram - Code Topology".to_string(),
        ];

        // Build module metrics lookup
        let module_metrics: std::collections::HashMap<_, _> = topology
            .modules
            .iter()
            .map(|m| (m.id.clone(), m))
            .collect();

        if let Some(matrix) = &topology.coupling_matrix {
            // Each module is a container
            for module_id in &matrix.modules {
                let safe_id = Self::sanitize_id(module_id);
                let metrics = module_metrics.get(module_id);

                let tech = metrics
                    .map(|m| m.languages.first().cloned().unwrap_or_else(|| "rust".into()))
                    .unwrap_or_else(|| "rust".into());

                let desc = metrics
                    .map(|m| format!("CC:{} LOC:{}", m.total_cyclomatic, m.lines_of_code))
                    .unwrap_or_default();

                lines.push(format!(
                    "    Container({}, \"{}\", \"{}\", \"{}\")",
                    safe_id,
                    module_id.replace("::", "/"),
                    tech,
                    desc
                ));
            }

            // Add relationships
            for (i, row) in matrix.values.iter().enumerate() {
                for (j, &strength) in row.iter().enumerate() {
                    if j > i && strength >= 0.3 {
                        let from = Self::sanitize_id(&matrix.modules[i]);
                        let to = Self::sanitize_id(&matrix.modules[j]);
                        let pct = (strength * 100.0) as u32;
                        lines.push(format!("    Rel({}, {}, \"{}% coupled\")", from, to, pct));
                    }
                }
            }
        }

        lines.join("\n")
    }

    /// Sanitize module ID for Mermaid (remove special chars).
    fn sanitize_id(id: &str) -> String {
        id.replace("::", "_")
            .replace('-', "_")
            .replace('.', "_")
            .replace('/', "_")
    }
}

impl Default for MermaidProjector {
    fn default() -> Self {
        Self::new()
    }
}

impl Projector for MermaidProjector {
    fn id(&self) -> &'static str {
        "mermaid"
    }

    fn name(&self) -> &'static str {
        "Mermaid Diagram Projector"
    }

    fn description(&self) -> &'static str {
        "Generates Mermaid diagrams (flowchart, C4) from topology artifacts for embedding in markdown"
    }

    fn load(&self, topology_dir: &Path) -> Result<Topology, ProjectorError> {
        if !topology_dir.exists() {
            return Err(ProjectorError {
                code: "TOPOLOGY_NOT_FOUND",
                message: format!("Directory not found: {}", topology_dir.display()),
                source: None,
            });
        }
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

        let diagram = match format {
            OutputFormat::Mermaid => match cfg.style {
                DiagramStyle::Flowchart => self.render_flowchart(topology, &cfg),
                DiagramStyle::C4Context => self.render_c4_context(topology, &cfg),
                DiagramStyle::C4Container => self.render_c4_container(topology, &cfg),
                DiagramStyle::ClassDiagram => self.render_flowchart(topology, &cfg), // TODO
            },
            OutputFormat::Markdown => {
                let inner = match cfg.style {
                    DiagramStyle::Flowchart => self.render_flowchart(topology, &cfg),
                    DiagramStyle::C4Context => self.render_c4_context(topology, &cfg),
                    DiagramStyle::C4Container => self.render_c4_container(topology, &cfg),
                    DiagramStyle::ClassDiagram => self.render_flowchart(topology, &cfg),
                };
                format!("```mermaid\n{}\n```", inner)
            }
            _ => {
                return Err(ProjectorError {
                    code: "UNSUPPORTED_FORMAT",
                    message: format!("Format {format:?} not supported by mermaid projector"),
                    source: None,
                })
            }
        };

        Ok(diagram.into_bytes())
    }

    fn supported_formats(&self) -> &[OutputFormat] {
        &[OutputFormat::Mermaid, OutputFormat::Markdown]
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "MermaidConfig",
            "type": "object",
            "properties": {
                "style": {
                    "type": "string",
                    "enum": ["flowchart", "c4-context", "c4-container", "class-diagram"],
                    "default": "flowchart"
                },
                "direction": { "type": "string", "default": "LR" },
                "minCoupling": { "type": "number", "default": 0.3 },
                "showStrength": { "type": "boolean", "default": true },
                "theme": { "type": "string", "default": "dark" }
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projector_creation() {
        let projector = MermaidProjector::new();
        assert_eq!(projector.id(), "mermaid");
    }

    #[test]
    fn test_sanitize_id() {
        assert_eq!(MermaidProjector::sanitize_id("aps-core::discovery"), "aps_core_discovery");
        assert_eq!(MermaidProjector::sanitize_id("my.module"), "my_module");
    }

    #[test]
    fn test_supported_formats() {
        let projector = MermaidProjector::new();
        let formats = projector.supported_formats();
        assert!(formats.contains(&OutputFormat::Mermaid));
        assert!(formats.contains(&OutputFormat::Markdown));
    }
}

