//! Typed configuration contract for APS standards.
//!
//! Standards that accept runtime configuration via `[standards.<slug>.config]`
//! in `apss.toml` MUST implement the [`StandardConfig`] trait. Standards that
//! accept no configuration MUST use the [`NoConfig`] marker type.
//!
//! See meta-standard §8.3 for the normative specification.

use crate::Diagnostics;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// Trait for typed standard configuration.
///
/// Implementing this trait enables:
/// - **Type-safe validation** — consumer config blocks are deserialized into this type
/// - **Semantic validation** — `validate()` checks value ranges, cross-field consistency
/// - **Schema generation** — `json_schema()` produces JSON Schema for IDE tooling
/// - **Scaffolding** — `toml_template()` generates documented default config snippets
pub trait StandardConfig: DeserializeOwned + Serialize + Default + Send + Sync {
    /// Validate config values beyond type checking.
    ///
    /// Called after successful deserialization. Use this for:
    /// - Value ranges (e.g., threshold between 0.0 and 1.0)
    /// - Path existence checks
    /// - Cross-field consistency
    fn validate(&self) -> Diagnostics;

    /// Generate a JSON Schema for this config type.
    ///
    /// Used for IDE completion, documentation, and external tooling.
    fn json_schema() -> serde_json::Value;

    /// Generate a commented TOML snippet showing all options with defaults.
    ///
    /// Used by `apss init` and `apss config template` to scaffold config blocks.
    fn toml_template() -> String;
}

/// Marker type for standards that accept no configuration.
///
/// Use this when your standard has no configurable options:
///
/// ```ignore
/// use aps_core::standard_config::NoConfig;
/// pub type Config = NoConfig;
/// ```
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct NoConfig {}

impl StandardConfig for NoConfig {
    fn validate(&self) -> Diagnostics {
        Diagnostics::new()
    }

    fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "description": "This standard has no configuration options.",
            "additionalProperties": false
        })
    }

    fn toml_template() -> String {
        "# No configuration options.\n".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_config_validates() {
        let config = NoConfig {};
        let diags = config.validate();
        assert!(!diags.has_errors());
        assert!(diags.is_empty());
    }

    #[test]
    fn test_no_config_json_schema() {
        let schema = NoConfig::json_schema();
        assert_eq!(schema["type"], "object");
    }

    #[test]
    fn test_no_config_toml_template() {
        let template = NoConfig::toml_template();
        assert!(template.starts_with('#'));
    }

    #[test]
    fn test_no_config_roundtrip() {
        // NoConfig deserializes from empty TOML (unit structs can't serialize to TOML)
        let _: NoConfig = toml::from_str("").unwrap();
        // Also works from an empty table
        let _: NoConfig = serde_json::from_str("{}").unwrap();
    }
}
