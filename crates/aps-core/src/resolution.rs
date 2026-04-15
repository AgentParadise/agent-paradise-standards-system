//! Cascading configuration resolution for monorepos.
//!
//! When a project uses workspace-style `apss.toml` files, child configs
//! inherit from and override the root config. This module handles the
//! merge logic and version resolution.
//!
//! See `APS-V1-0000.CF01` for the normative specification.

use crate::config::{ConfigError, ProjectConfig, ProjectInfo, StandardEntry, ToolConfig};
use crate::{Diagnostic, Diagnostics};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during configuration resolution.
#[derive(Debug, Error)]
pub enum ResolutionError {
    /// Failed to load a configuration file.
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// APSS version mismatch between root and child.
    #[error("apss_version mismatch: root={root}, child={child} (in {child_path})")]
    VersionMismatch {
        root: String,
        child: String,
        child_path: PathBuf,
    },

    /// Child config contains a [workspace] section.
    #[error("child config {path} must not contain [workspace] section")]
    WorkspaceInChild { path: PathBuf },

    /// Version range conflict between root and child.
    #[error(
        "version range conflict for standard '{slug}': root requires {root_req}, child requires {child_req} — no satisfying version exists"
    )]
    VersionRangeConflict {
        slug: String,
        root_req: String,
        child_req: String,
    },
}

// ============================================================================
// Resolved Types
// ============================================================================

/// A fully resolved project configuration after cascading merge.
#[derive(Debug, Clone)]
pub struct ResolvedProjectConfig {
    /// Project identity (from the nearest `apss.toml`).
    pub project: ProjectInfo,

    /// Resolved standards with merged config.
    pub standards: BTreeMap<String, ResolvedStandard>,

    /// Resolved tool configuration.
    pub tool: ToolConfig,

    /// Which `apss.toml` files contributed to this resolution.
    pub source_files: Vec<PathBuf>,
}

/// A standard entry after resolution.
#[derive(Debug, Clone)]
pub struct ResolvedStandard {
    /// Standard ID (e.g., `"APS-V1-0001"`).
    pub id: String,

    /// CLI dispatch slug.
    pub slug: String,

    /// Version requirement string.
    pub version_req: String,

    /// Whether this standard is enabled.
    pub enabled: bool,

    /// Enabled substandard profile codes.
    pub substandards: Option<Vec<String>>,

    /// Standard-specific configuration.
    pub config: toml::Value,

    /// Expected crate name for this standard.
    pub crate_name: String,
}

// ============================================================================
// Resolution Logic
// ============================================================================

/// Resolve a project configuration from a single `apss.toml` (no cascading).
pub fn resolve_single(config: ProjectConfig, source: PathBuf) -> ResolvedProjectConfig {
    let standards = config
        .standards
        .into_iter()
        .map(|(slug, entry)| {
            let crate_name = standard_id_to_crate_name(&entry.id);
            let resolved = ResolvedStandard {
                id: entry.id,
                slug: slug.clone(),
                version_req: entry.version,
                enabled: entry.enabled,
                substandards: entry.substandards,
                config: entry.config,
                crate_name,
            };
            (slug, resolved)
        })
        .collect();

    ResolvedProjectConfig {
        project: config.project,
        standards,
        tool: config.tool.unwrap_or_default(),
        source_files: vec![source],
    }
}

/// Merge a child config into a root config.
///
/// ## Cascading Rules
///
/// - Child `apss_version` MUST match root (error if different)
/// - Child MUST NOT contain `[workspace]` (error if present)
/// - Standards present only in root: inherited as-is
/// - Standards present only in child: added
/// - Standards present in both: child's entry fully replaces root's (no deep merge)
/// - `enabled = false` in child disables that standard for this member only
/// - `[tool]` fields from child override root's individual fields
pub fn merge_configs(
    root: &ProjectConfig,
    root_path: &Path,
    child: &ProjectConfig,
    child_path: &Path,
) -> Result<ResolvedProjectConfig, ResolutionError> {
    // Validate: apss_version must match
    if root.project.apss_version != child.project.apss_version {
        return Err(ResolutionError::VersionMismatch {
            root: root.project.apss_version.clone(),
            child: child.project.apss_version.clone(),
            child_path: child_path.to_path_buf(),
        });
    }

    // Validate: child must not have [workspace]
    if child.workspace.is_some() {
        return Err(ResolutionError::WorkspaceInChild {
            path: child_path.to_path_buf(),
        });
    }

    // Merge standards: start with root, override with child
    let mut standards: BTreeMap<String, StandardEntry> = root.standards.clone();
    for (slug, child_entry) in &child.standards {
        standards.insert(slug.clone(), child_entry.clone());
    }

    // Merge tool config
    let root_tool = root.tool.clone().unwrap_or_default();
    let merged_tool = match &child.tool {
        Some(child_tool) => ToolConfig {
            bin_dir: child_tool.bin_dir.clone(),
            registry: child_tool.registry.clone(),
            offline: child_tool.offline,
            log_level: child_tool.log_level.clone(),
        },
        None => root_tool,
    };

    // Build resolved config
    let resolved_standards = standards
        .into_iter()
        .map(|(slug, entry)| {
            let crate_name = standard_id_to_crate_name(&entry.id);
            let resolved = ResolvedStandard {
                id: entry.id,
                slug: slug.clone(),
                version_req: entry.version,
                enabled: entry.enabled,
                substandards: entry.substandards,
                config: entry.config,
                crate_name,
            };
            (slug, resolved)
        })
        .collect();

    Ok(ResolvedProjectConfig {
        project: child.project.clone(),
        standards: resolved_standards,
        tool: merged_tool,
        source_files: vec![root_path.to_path_buf(), child_path.to_path_buf()],
    })
}

/// Validate a resolved config for internal consistency.
pub fn validate_resolved(config: &ResolvedProjectConfig) -> Diagnostics {
    let mut diags = Diagnostics::new();

    // Check for duplicate standard IDs across different slugs
    let mut id_to_slug: BTreeMap<&str, &str> = BTreeMap::new();
    for (slug, standard) in &config.standards {
        if let Some(existing_slug) = id_to_slug.insert(&standard.id, slug) {
            diags.push(
                Diagnostic::error(
                    "CF_DUPLICATE_STANDARD_ID",
                    format!(
                        "Standard ID '{}' is declared under both '{}' and '{}'",
                        standard.id, existing_slug, slug
                    ),
                )
                .with_hint("Each standard ID must map to exactly one slug"),
            );
        }
    }

    diags
}

// ============================================================================
// Helpers
// ============================================================================

/// Convert a standard ID like `"APS-V1-0001"` to a crate name like `"apss-v1-0001"`.
fn standard_id_to_crate_name(id: &str) -> String {
    id.to_lowercase().replace("aps-", "apss-")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_root() -> ProjectConfig {
        toml::from_str(
            r#"
schema = "apss.project/v1"

[project]
name = "root"
apss_version = "v1"

[standards.topology]
id = "APS-V1-0001"
version = ">=1.0.0"

[standards.topology.config]
output_dir = ".topology"

[workspace]
members = ["packages/*"]
"#,
        )
        .unwrap()
    }

    fn minimal_child() -> ProjectConfig {
        toml::from_str(
            r#"
schema = "apss.project/v1"

[project]
name = "child-pkg"
apss_version = "v1"

[standards.topology]
id = "APS-V1-0001"
version = ">=1.0.0, <2.0.0"

[standards.topology.config]
output_dir = ".custom-topology"
"#,
        )
        .unwrap()
    }

    #[test]
    fn test_resolve_single() {
        let config = minimal_root();
        let resolved = resolve_single(config, PathBuf::from("apss.toml"));

        assert_eq!(resolved.project.name, "root");
        assert_eq!(resolved.standards.len(), 1);

        let topo = &resolved.standards["topology"];
        assert_eq!(topo.id, "APS-V1-0001");
        assert_eq!(topo.crate_name, "apss-v1-0001");
        assert!(topo.enabled);
    }

    #[test]
    fn test_merge_child_overrides_config() {
        let root = minimal_root();
        let child = minimal_child();

        let resolved = merge_configs(
            &root,
            Path::new("apss.toml"),
            &child,
            Path::new("packages/a/apss.toml"),
        )
        .unwrap();

        let topo = &resolved.standards["topology"];
        // Child's version requirement replaces root's
        assert_eq!(topo.version_req, ">=1.0.0, <2.0.0");
        // Child's config replaces root's
        assert_eq!(
            topo.config["output_dir"].as_str().unwrap(),
            ".custom-topology"
        );
    }

    #[test]
    fn test_merge_inherits_root_standards() {
        let root: ProjectConfig = toml::from_str(
            r#"
schema = "apss.project/v1"
[project]
name = "root"
apss_version = "v1"

[standards.topology]
id = "APS-V1-0001"
version = ">=1.0.0"

[standards.fitness]
id = "APS-V1-0003"
version = ">=1.0.0"

[workspace]
members = ["packages/*"]
"#,
        )
        .unwrap();

        let child: ProjectConfig = toml::from_str(
            r#"
schema = "apss.project/v1"
[project]
name = "child"
apss_version = "v1"
"#,
        )
        .unwrap();

        let resolved = merge_configs(
            &root,
            Path::new("apss.toml"),
            &child,
            Path::new("packages/a/apss.toml"),
        )
        .unwrap();

        // Child inherits both standards from root
        assert_eq!(resolved.standards.len(), 2);
        assert!(resolved.standards.contains_key("topology"));
        assert!(resolved.standards.contains_key("fitness"));
    }

    #[test]
    fn test_merge_version_mismatch_error() {
        let root = minimal_root();
        let mut child = minimal_child();
        child.project.apss_version = "v2".to_string();

        let result = merge_configs(
            &root,
            Path::new("apss.toml"),
            &child,
            Path::new("packages/a/apss.toml"),
        );

        assert!(matches!(
            result,
            Err(ResolutionError::VersionMismatch { .. })
        ));
    }

    #[test]
    fn test_merge_workspace_in_child_error() {
        let root = minimal_root();
        let mut child = minimal_child();
        child.workspace = Some(crate::config::WorkspaceConfig {
            members: vec!["sub/*".to_string()],
            exclude: vec![],
        });

        let result = merge_configs(
            &root,
            Path::new("apss.toml"),
            &child,
            Path::new("packages/a/apss.toml"),
        );

        assert!(matches!(
            result,
            Err(ResolutionError::WorkspaceInChild { .. })
        ));
    }

    #[test]
    fn test_validate_duplicate_ids() {
        let config = ResolvedProjectConfig {
            project: ProjectInfo {
                name: "test".to_string(),
                apss_version: "v1".to_string(),
            },
            standards: BTreeMap::from([
                (
                    "topology".to_string(),
                    ResolvedStandard {
                        id: "APS-V1-0001".to_string(),
                        slug: "topology".to_string(),
                        version_req: ">=1.0.0".to_string(),
                        enabled: true,
                        substandards: None,
                        config: toml::Value::Table(Default::default()),
                        crate_name: "apss-v1-0001".to_string(),
                    },
                ),
                (
                    "topo".to_string(),
                    ResolvedStandard {
                        id: "APS-V1-0001".to_string(),
                        slug: "topo".to_string(),
                        version_req: ">=1.0.0".to_string(),
                        enabled: true,
                        substandards: None,
                        config: toml::Value::Table(Default::default()),
                        crate_name: "apss-v1-0001".to_string(),
                    },
                ),
            ]),
            tool: ToolConfig::default(),
            source_files: vec![],
        };

        let diags = validate_resolved(&config);
        assert!(diags.has_errors());
        assert!(diags.iter().any(|d| d.code == "CF_DUPLICATE_STANDARD_ID"));
    }

    #[test]
    fn test_standard_id_to_crate_name() {
        assert_eq!(standard_id_to_crate_name("APS-V1-0001"), "apss-v1-0001");
        assert_eq!(standard_id_to_crate_name("APS-V1-0003"), "apss-v1-0003");
    }
}
