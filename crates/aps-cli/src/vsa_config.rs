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
    #[allow(dead_code)]
    language: Option<String>,
    architecture: Option<String>,
    contexts: Option<HashMap<String, RawContext>>,
    #[serde(default)]
    #[allow(dead_code)]
    validation: Option<RawValidation>,
}

/// A context entry in v1 format.
#[derive(Debug, Deserialize)]
struct RawContext {
    description: Option<String>,
}

/// Validation settings (optional, reserved for future use).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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
    /// Named bounded contexts (v1 only; None in v2 means "discover from root").
    pub contexts: Option<HashMap<String, ContextConfig>>,
}

/// A validated bounded context entry.
#[derive(Debug, Clone)]
pub struct ContextConfig {
    _description: Option<String>,
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
                            _description: raw_ctx.description,
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

    /// Split the root into path components for boundary-safe matching.
    fn root_components(&self) -> Vec<&str> {
        self.normalized_root()
            .split('/')
            .filter(|c| !c.is_empty())
            .collect()
    }

    /// Normalize a module path/ID to `/`-separated components.
    /// Handles `::` (Rust), `.` (Python), `\` (Windows), and `/` (path-like).
    fn normalize_to_components(module_path: &str) -> Vec<&str> {
        // Split on all known separators. For `::` we first replace, then split on `/`.
        // We can't simply replace `.` because it appears in filenames, but Python module
        // IDs don't have `/` so we can use that to disambiguate.
        if module_path.contains('/') {
            // Path-like: split on `/` (preserves `.` in filenames like `[[...slug]]`)
            module_path.split('/').filter(|c| !c.is_empty()).collect()
        } else if module_path.contains("::") {
            // Rust-style
            module_path.split("::").filter(|c| !c.is_empty()).collect()
        } else {
            // Python-style (dot-separated)
            module_path.split('.').filter(|c| !c.is_empty()).collect()
        }
    }

    /// Check if a module path falls under the VSA root using component-boundary matching.
    pub fn contains_path(&self, module_path: &str) -> bool {
        let root_parts = self.root_components();
        if root_parts.is_empty() {
            return false;
        }
        let path_parts = Self::normalize_to_components(module_path);
        // Look for the root components as a contiguous window in the path
        path_parts
            .windows(root_parts.len())
            .any(|window| window == root_parts.as_slice())
    }

    /// Extract the bounded context name from a module path/ID, given the VSA root.
    /// Returns the first path component after the root component sequence.
    ///
    /// Uses component-boundary matching to avoid substring false positives
    /// (e.g., `contexts_backup` won't match `contexts`).
    pub fn extract_context(&self, module_path: &str) -> Option<String> {
        let root_parts = self.root_components();
        if root_parts.is_empty() {
            return None;
        }
        let path_parts = Self::normalize_to_components(module_path);
        let root_len = root_parts.len();

        // Find the window matching root components, then take the next component
        for start in 0..=path_parts.len().saturating_sub(root_len) {
            if path_parts[start..start + root_len] == root_parts[..] {
                let context_index = start + root_len;
                if let Some(&ctx) = path_parts.get(context_index) {
                    if !ctx.is_empty() {
                        return Some(ctx.to_string());
                    }
                }
                return None;
            }
        }
        None
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
        // Boundary check: should not match partial directory names
        assert!(!config.contains_path("packages/syn-domain/contexts_backup/orchestration"));
    }

    #[test]
    fn contains_path_python_modules() {
        let config = VsaConfig::parse_str(
            "root: ./src/syn_domain/contexts\ncontexts:\n  orchestration:\n    description: test\n",
        )
        .unwrap();

        // Python dot-separated module IDs
        assert!(config.contains_path("src.syn_domain.contexts.orchestration.core"));
        assert!(!config.contains_path("src.syn_api.routes"));
    }

    #[test]
    fn contains_path_rust_modules() {
        let config = VsaConfig::parse_str(
            "root: ./src/syn_domain/contexts\ncontexts:\n  orchestration:\n    description: test\n",
        )
        .unwrap();

        // Rust :: separated module IDs
        assert!(config.contains_path("src::syn_domain::contexts::orchestration::core"));
        assert!(!config.contains_path("src::syn_api::routes"));
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
        // Boundary: partial match should not work
        assert_eq!(
            config.extract_context("packages/syn-domain/contexts_backup/orchestration"),
            None
        );
    }

    #[test]
    fn extract_context_python_modules() {
        let config = VsaConfig::parse_str(
            "root: ./src/syn_domain/contexts\ncontexts:\n  orchestration:\n    description: test\n",
        )
        .unwrap();

        assert_eq!(
            config.extract_context("src.syn_domain.contexts.orchestration.core"),
            Some("orchestration".to_string())
        );
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
        let dir = tempfile::tempdir().unwrap();
        assert!(VsaConfig::load(dir.path()).unwrap().is_none());
    }
}
