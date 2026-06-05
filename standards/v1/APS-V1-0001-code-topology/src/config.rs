//! Typed configuration for the Code Topology standard.
//!
//! Implements `StandardConfig` to provide type-safe validation,
//! JSON Schema generation, and TOML scaffolding for topology config blocks.

use apss_core::standard_config::StandardConfig;
use apss_core::{Diagnostic, Diagnostics};
use serde::{Deserialize, Serialize};

use crate::OutputFormat;

/// Configuration for the Code Topology standard (`APS-V1-0001`).
///
/// Used in `APSS.yaml`:
/// ```yaml
/// standards:
///   code-topology:
///     config:
///       output_dir: .topology
///       languages: ["rust", "python"]
///       format: json
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TopologyConfig {
    /// Output directory for topology artifacts.
    #[serde(default = "default_output_dir")]
    pub output_dir: String,

    /// Languages to analyze. Empty means auto-detect from source files.
    #[serde(default)]
    pub languages: Vec<String>,

    /// Output format for generated artifacts.
    #[serde(default = "default_format")]
    pub format: OutputFormat,
}

fn default_output_dir() -> String {
    ".topology".to_string()
}

fn default_format() -> OutputFormat {
    OutputFormat::Json
}

impl Default for TopologyConfig {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            languages: Vec::new(),
            format: default_format(),
        }
    }
}

impl StandardConfig for TopologyConfig {
    fn validate(&self) -> Diagnostics {
        let mut diags = Diagnostics::new();

        if self.output_dir.trim().is_empty() {
            diags.push(Diagnostic::error(
                "TOPOLOGY_EMPTY_OUTPUT_DIR",
                "output_dir must not be empty",
            ));
        }

        diags
    }

    fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "title": "TopologyConfig",
            "description": "Configuration for the Code Topology standard (APS-V1-0001)",
            "type": "object",
            "properties": {
                "output_dir": {
                    "type": "string",
                    "description": "Output directory for topology artifacts",
                    "default": ".topology"
                },
                "languages": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Languages to analyze (empty = auto-detect)",
                    "default": []
                },
                "format": {
                    "type": "string",
                    "enum": ["dot", "svg", "png", "mermaid", "markdown", "json", "webgl", "html", "gltf"],
                    "description": "Output format for generated artifacts",
                    "default": "json"
                }
            },
            "additionalProperties": false
        })
    }

    fn toml_template() -> String {
        r#"# Output directory for topology artifacts
# output_dir = ".topology"

# Languages to analyze (auto-detected if omitted)
# languages = ["rust", "python", "typescript"]

# Output format: dot, svg, png, mermaid, markdown, json, webgl, html, gltf
# format = "json"
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_default_config_validates() {
        let config = TopologyConfig::default();
        let diags = config.validate();
        assert!(!diags.has_errors());
    }

    #[test]
    fn test_empty_output_dir_fails() {
        let config = TopologyConfig {
            output_dir: "".to_string(),
            ..Default::default()
        };
        let diags = config.validate();
        assert!(diags.has_errors());
    }

    #[test]
    fn test_roundtrip_toml() {
        let config = TopologyConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: TopologyConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.output_dir, config.output_dir);
    }

    #[test]
    fn test_json_schema_structure() {
        let schema = TopologyConfig::json_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["output_dir"].is_object());
        assert!(schema["properties"]["languages"].is_object());
        assert!(schema["properties"]["format"].is_object());
    }

    #[test]
    fn test_toml_template_is_commented() {
        let template = TopologyConfig::toml_template();
        for line in template.lines() {
            assert!(
                line.is_empty() || line.starts_with('#'),
                "Template line should be commented: {line}"
            );
        }
    }

    #[test]
    fn config_schema_is_fresh() {
        let schema = TopologyConfig::json_schema();
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.schema.json");
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("config.schema.json not found at {}", path.display()));
        let expected: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("config.schema.json is invalid JSON: {e}"));
        assert_eq!(
            schema, expected,
            "config.schema.json is stale  -  update it from TopologyConfig::json_schema()"
        );
    }
}
