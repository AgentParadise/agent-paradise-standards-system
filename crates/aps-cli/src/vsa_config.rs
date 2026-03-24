//! VSA (Vertical Slice Architecture) configuration schema.
//!
//! Parses and validates `vsa.yaml` files that define which bounded contexts
//! participate in VSA visualization. Supports both version 1 (explicit context
//! map) and version 2 (root-only) formats.

use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

// ─── Error types ─────────────────────────────────────────────────────────────

/// Errors that can occur when loading or validating a VSA config.
#[derive(Debug)]
pub enum VsaConfigError {
    /// File exists but could not be read.
    Io(PathBuf, std::io::Error),
    /// YAML syntax is invalid.
    Parse(String),
    /// Schema validation failed (missing/invalid fields).
    Validation(Vec<String>),
}

impl fmt::Display for VsaConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(path, err) => write!(f, "failed to read {}: {err}", path.display()),
            Self::Parse(err) => write!(f, "invalid YAML in vsa.yaml: {err}"),
            Self::Validation(errors) => {
                writeln!(f, "vsa.yaml validation failed:")?;
                for e in errors {
                    writeln!(f, "  - {e}")?;
                }
                Ok(())
            }
        }
    }
}

// ─── Raw deserialization (maps directly to YAML) ─────────────────────────────

/// Raw YAML structure — deserializes both v1 and v2 formats.
#[derive(Debug, Deserialize)]
struct RawVsaConfig {
    version: Option<u8>,
    root: Option<String>,
    language: Option<String>,
    architecture: Option<String>,
    contexts: Option<HashMap<String, RawContext>>,
    validation: Option<RawValidation>,
}

/// A context entry in v1 format.
#[derive(Debug, Deserialize)]
struct RawContext {
    description: Option<String>,
}

/// Validation settings (optional).
#[derive(Debug, Deserialize)]
struct RawValidation {
    require_tests: Option<bool>,
    max_nesting_depth: Option<u32>,
    domain_level_commands: Option<bool>,
}

// ─── Validated config (public API) ───────────────────────────────────────────

/// A validated VSA configuration.
#[derive(Debug, Clone)]
pub struct VsaConfig {
    /// Config format version (1 or 2).
    pub version: u8,
    /// Root directory containing bounded contexts (relative to repo root).
    pub root: String,
    /// Programming language.
    pub language: Option<String>,
    /// Architecture style (v2 only).
    pub architecture: Option<String>,
    /// Named bounded contexts (v1 only; None in v2 means "discover from root").
    pub contexts: Option<HashMap<String, ContextConfig>>,
}

/// A validated bounded context entry.
#[derive(Debug, Clone)]
pub struct ContextConfig {
    pub description: Option<String>,
}

impl VsaConfig {
    /// Attempt to load `vsa.yaml` from a directory. Returns `None` if the file
    /// does not exist, `Err` if it exists but is invalid.
    pub fn load(dir: &Path) -> Result<Option<Self>, VsaConfigError> {
        let path = dir.join("vsa.yaml");
        if !path.exists() {
            // Also check vsa.yml
            let alt = dir.join("vsa.yml");
            if !alt.exists() {
                return Ok(None);
            }
            return Self::parse_file(&alt).map(Some);
        }
        Self::parse_file(&path).map(Some)
    }

    /// Parse and validate a specific vsa.yaml file.
    fn parse_file(path: &Path) -> Result<Self, VsaConfigError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| VsaConfigError::Io(path.to_path_buf(), e))?;
        Self::parse_str(&content)
    }

    /// Parse and validate from a YAML string (useful for testing).
    pub fn parse_str(yaml: &str) -> Result<Self, VsaConfigError> {
        let raw: RawVsaConfig =
            serde_yaml::from_str(yaml).map_err(|e| VsaConfigError::Parse(e.to_string()))?;
        Self::validate_raw(raw)
    }

    /// Validate the raw deserialized config and produce a typed, validated config.
    fn validate_raw(raw: RawVsaConfig) -> Result<Self, VsaConfigError> {
        let mut errors = Vec::new();

        // Version: must be 1 or 2 (default to 1 if omitted)
        let version = raw.version.unwrap_or(1);
        if version != 1 && version != 2 {
            errors.push(format!("version must be 1 or 2, got {version}"));
        }

        // Root: required
        let root = match &raw.root {
            Some(r) if !r.is_empty() => r.clone(),
            Some(_) => {
                errors.push("root must not be empty".to_string());
                String::new()
            }
            None => {
                errors.push("root is required".to_string());
                String::new()
            }
        };

        // V2 should have architecture
        if version == 2 && raw.architecture.is_none() {
            errors.push("version 2 configs should include an 'architecture' field".to_string());
        }

        // Contexts map validation (v1)
        let contexts = raw.contexts.map(|ctx_map| {
            if ctx_map.is_empty() {
                errors.push("contexts map must not be empty when specified".to_string());
            }
            ctx_map
                .into_iter()
                .map(|(name, raw_ctx)| {
                    (
                        name,
                        ContextConfig {
                            description: raw_ctx.description,
                        },
                    )
                })
                .collect()
        });

        if !errors.is_empty() {
            return Err(VsaConfigError::Validation(errors));
        }

        Ok(Self {
            version,
            root,
            language: raw.language,
            architecture: raw.architecture,
            contexts,
        })
    }

    /// Normalize the root path: strip leading `./` and trailing `/`.
    pub fn normalized_root(&self) -> &str {
        self.root
            .strip_prefix("./")
            .unwrap_or(&self.root)
            .trim_end_matches('/')
    }

    /// Check if a module path falls under the VSA root.
    pub fn contains_path(&self, module_path: &str) -> bool {
        let root = self.normalized_root();
        // Module path or ID should contain the root path segments
        let normalized = module_path.replace("::", "/");
        normalized.starts_with(root) || normalized.contains(root)
    }

    /// Extract the bounded context name from a module path/ID, given the VSA root.
    /// Returns the first path segment after the root.
    ///
    /// Example: root="packages/syn-domain/src/syn_domain/contexts"
    ///   module_path="packages/syn-domain/src/syn_domain/contexts/orchestration/core"
    ///   → Some("orchestration")
    pub fn extract_context(&self, module_path: &str) -> Option<String> {
        let root = self.normalized_root();
        let normalized = module_path.replace("::", "/");

        // Find the root in the path and take the next segment
        if let Some(pos) = normalized.find(root) {
            let after_root = &normalized[pos + root.len()..];
            let after_root = after_root.trim_start_matches('/');
            let context = after_root.split('/').next().unwrap_or("");
            if context.is_empty() {
                None
            } else {
                Some(context.to_string())
            }
        } else {
            None
        }
    }

    /// Check if a context name is allowed by this config.
    /// If contexts map is defined (v1), only listed contexts are allowed.
    /// If no contexts map (v2), all contexts under root are allowed.
    pub fn is_context_allowed(&self, context: &str) -> bool {
        match &self.contexts {
            Some(map) => map.contains_key(context),
            None => true, // v2: all contexts under root are valid
        }
    }

    /// Returns the list of explicitly configured context names (v1 only).
    pub fn context_names(&self) -> Vec<&str> {
        match &self.contexts {
            Some(map) => {
                let mut names: Vec<&str> = map.keys().map(|s| s.as_str()).collect();
                names.sort();
                names
            }
            None => Vec::new(),
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_v1_config() {
        let yaml = r#"
version: 1
root: ./packages/syn-domain/src/syn_domain/contexts
language: python

contexts:
  orchestration:
    description: "Workflow execution"
  agent_sessions:
    description: "Agent sessions and metrics"
  artifacts:
    description: "Artifact storage"
"#;
        let config = VsaConfig::parse_str(yaml).unwrap();
        assert_eq!(config.version, 1);
        assert_eq!(
            config.normalized_root(),
            "packages/syn-domain/src/syn_domain/contexts"
        );
        assert_eq!(config.language.as_deref(), Some("python"));
        assert!(config.contexts.is_some());

        let ctx = config.contexts.as_ref().unwrap();
        assert_eq!(ctx.len(), 3);
        assert!(ctx.contains_key("orchestration"));
        assert!(ctx.contains_key("agent_sessions"));
        assert!(ctx.contains_key("artifacts"));
    }

    #[test]
    fn parse_v2_config() {
        let yaml = r#"
version: 2
architecture: hexagonal-event-sourced-vsa
language: python
root: src/syn_domain/contexts
"#;
        let config = VsaConfig::parse_str(yaml).unwrap();
        assert_eq!(config.version, 2);
        assert_eq!(config.normalized_root(), "src/syn_domain/contexts");
        assert_eq!(
            config.architecture.as_deref(),
            Some("hexagonal-event-sourced-vsa")
        );
        assert!(config.contexts.is_none());
    }

    #[test]
    fn missing_root_fails() {
        let yaml = "version: 1\nlanguage: python\n";
        let err = VsaConfig::parse_str(yaml).unwrap_err();
        match err {
            VsaConfigError::Validation(errors) => {
                assert!(errors.iter().any(|e| e.contains("root is required")));
            }
            other => panic!("expected Validation error, got: {other}"),
        }
    }

    #[test]
    fn invalid_version_fails() {
        let yaml = "version: 5\nroot: ./src\n";
        let err = VsaConfig::parse_str(yaml).unwrap_err();
        match err {
            VsaConfigError::Validation(errors) => {
                assert!(errors.iter().any(|e| e.contains("version must be 1 or 2")));
            }
            other => panic!("expected Validation error, got: {other}"),
        }
    }

    #[test]
    fn invalid_yaml_fails() {
        let yaml = "{{not valid yaml";
        let err = VsaConfig::parse_str(yaml).unwrap_err();
        assert!(matches!(err, VsaConfigError::Parse(_)));
    }

    #[test]
    fn contains_path_works() {
        let config = VsaConfig::parse_str(
            "version: 1\nroot: ./packages/syn-domain/contexts\ncontexts:\n  orchestration:\n    description: test\n",
        )
        .unwrap();

        assert!(config.contains_path("packages/syn-domain/contexts/orchestration/core"));
        assert!(!config.contains_path("packages/syn-api/routes"));
    }

    #[test]
    fn extract_context_works() {
        let config = VsaConfig::parse_str(
            "version: 1\nroot: ./packages/syn-domain/contexts\ncontexts:\n  orchestration:\n    description: test\n",
        )
        .unwrap();

        assert_eq!(
            config.extract_context("packages/syn-domain/contexts/orchestration/core"),
            Some("orchestration".to_string())
        );
        assert_eq!(
            config.extract_context("packages/syn-domain/contexts/artifacts/storage"),
            Some("artifacts".to_string())
        );
        assert_eq!(config.extract_context("packages/syn-api/routes"), None);
    }

    #[test]
    fn is_context_allowed_v1() {
        let config = VsaConfig::parse_str(
            "version: 1\nroot: ./src\ncontexts:\n  orchestration:\n    description: test\n",
        )
        .unwrap();
        assert!(config.is_context_allowed("orchestration"));
        assert!(!config.is_context_allowed("unknown_context"));
    }

    #[test]
    fn is_context_allowed_v2_allows_all() {
        let config = VsaConfig::parse_str(
            "version: 2\nroot: ./src\narchitecture: hexagonal-event-sourced-vsa\n",
        )
        .unwrap();
        assert!(config.is_context_allowed("anything"));
        assert!(config.is_context_allowed("whatever"));
    }

    #[test]
    fn version_defaults_to_1() {
        let config =
            VsaConfig::parse_str("root: ./src\ncontexts:\n  foo:\n    description: bar\n").unwrap();
        assert_eq!(config.version, 1);
    }

    #[test]
    fn load_returns_none_when_missing() {
        let dir = std::env::temp_dir().join("vsa_config_test_missing");
        std::fs::create_dir_all(&dir).ok();
        assert!(VsaConfig::load(&dir).unwrap().is_none());
    }

    #[test]
    fn context_names_sorted() {
        let config = VsaConfig::parse_str(
            "root: ./src\ncontexts:\n  zebra:\n    description: z\n  alpha:\n    description: a\n  mid:\n    description: m\n",
        )
        .unwrap();
        assert_eq!(config.context_names(), vec!["alpha", "mid", "zebra"]);
    }
}
