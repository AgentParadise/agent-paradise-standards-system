//! Provider-neutral agent workspace contract.
//!
//! EXP-V1-0006 defines the launch manifest passed from an orchestrator to a
//! workspace provider. It separates workflow-domain decisions from lifecycle
//! mechanics and provider implementation details.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const ID: &str = "EXP-V1-0006";
pub const SLUG: &str = "workspace";
pub const NAME: &str = "Agentic Workspace";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MANIFEST_SCHEMA: &str = "apss.workspace-launch/v1";

pub fn register(registry: &mut dyn apss_core::registry::StandardRegistry) {
    registry.register(
        apss_core::registry::RegisteredStandard {
            id: ID.into(),
            slug: SLUG.into(),
            name: NAME.into(),
            description: "Provider-neutral agent workspace launch contract".into(),
            version: VERSION.into(),
            commands: vec!["validate".into()],
        },
        Box::new(WorkspaceCommandHandler),
    );
}

struct WorkspaceCommandHandler;

impl apss_core::registry::CommandHandler for WorkspaceCommandHandler {
    fn execute(&self, command: &str, args: &[String], _config: &toml::Value) -> i32 {
        if command != "validate" || args.len() != 1 {
            eprintln!("usage: apss run workspace validate <workspace-launch.json>");
            return 3;
        }
        let path = std::path::Path::new(&args[0]);
        let contents = match std::fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(error) => {
                eprintln!("cannot read {}: {error}", path.display());
                return 1;
            }
        };
        let manifest: LaunchManifest = match serde_json::from_str(&contents) {
            Ok(manifest) => manifest,
            Err(error) => {
                eprintln!("invalid JSON manifest: {error}");
                return 1;
            }
        };
        match manifest.validate() {
            Ok(()) => {
                println!("valid workspace launch manifest");
                0
            }
            Err(error) => {
                eprintln!("invalid workspace launch manifest: {error}");
                1
            }
        }
    }

    fn commands(&self) -> Vec<apss_core::registry::CommandInfo> {
        vec![apss_core::registry::CommandInfo {
            name: "validate".into(),
            description: "Validate a workspace launch manifest".into(),
            usage: "validate <workspace-launch.json>".into(),
        }]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchManifest {
    pub schema: String,
    pub execution_id: String,
    pub workspace: WorkspaceRequest,
    #[serde(default)]
    pub content: ContentRequest,
    pub agent: AgentRequest,
    #[serde(default)]
    pub skills: Vec<SkillRequest>,
    #[serde(default)]
    pub tools: ToolPolicy,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub transcript: TranscriptRequest,
    #[serde(default)]
    pub outputs: OutputRequest,
    #[serde(default)]
    pub limits: ResourceLimits,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceRequest {
    pub working_directory: String,
    pub security_profile: SecurityProfile,
    #[serde(default)]
    pub provider_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecurityProfile {
    InsecureLocal,
    Isolated,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentRequest {
    #[serde(default)]
    pub repositories: Vec<RepositoryRequest>,
    #[serde(default)]
    pub inputs: Vec<FileInput>,
    #[serde(default)]
    pub context_files: Vec<FileInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryRequest {
    pub url: String,
    pub revision: String,
    pub destination: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileInput {
    pub source: String,
    pub destination: String,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentRequest {
    pub harness: String,
    #[serde(default)]
    pub model: Option<String>,
    pub prompt: String,
    #[serde(default)]
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillRequest {
    pub name: String,
    pub source: String,
    pub revision: String,
    pub digest: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolPolicy {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default)]
    pub sandbox: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TranscriptRequest {
    pub session_id: String,
    pub source_format: String,
    pub destination: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputRequest {
    #[serde(default)]
    pub artifacts: Vec<String>,
    #[serde(default)]
    pub collect_logs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceLimits {
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default)]
    pub memory_mb: Option<u64>,
    #[serde(default)]
    pub cpu_millis: Option<u32>,
    #[serde(default)]
    pub disk_mb: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            timeout_seconds: default_timeout(),
            memory_mb: None,
            cpu_millis: None,
            disk_mb: None,
        }
    }
}

const fn default_timeout() -> u64 {
    3600
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ManifestError {
    #[error("unsupported manifest schema: {0}")]
    UnsupportedSchema(String),
    #[error("required field is empty: {0}")]
    Empty(&'static str),
    #[error("path must be workspace-relative and traversal-free: {0}")]
    UnsafePath(String),
    #[error("tool appears in both allow and deny: {0}")]
    ConflictingTool(String),
    #[error("duplicate skill name: {0}")]
    DuplicateSkill(String),
    #[error("skill revision and digest are required: {0}")]
    UnpinnedSkill(String),
    #[error("timeout_seconds must be greater than zero")]
    ZeroTimeout,
}

impl LaunchManifest {
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.schema != MANIFEST_SCHEMA {
            return Err(ManifestError::UnsupportedSchema(self.schema.clone()));
        }
        require_non_empty(&self.execution_id, "execution_id")?;
        require_non_empty(&self.agent.harness, "agent.harness")?;
        require_non_empty(&self.agent.prompt, "agent.prompt")?;
        require_non_empty(&self.transcript.session_id, "transcript.session_id")?;
        require_safe_path(&self.workspace.working_directory)?;
        for repository in &self.content.repositories {
            require_non_empty(&repository.url, "content.repositories.url")?;
            require_non_empty(&repository.revision, "content.repositories.revision")?;
            require_safe_path(&repository.destination)?;
        }
        for input in self
            .content
            .inputs
            .iter()
            .chain(self.content.context_files.iter())
        {
            require_non_empty(&input.source, "content input source")?;
            require_safe_path(&input.destination)?;
        }
        let denied: BTreeSet<_> = self.tools.deny.iter().collect();
        if let Some(tool) = self.tools.allow.iter().find(|tool| denied.contains(tool)) {
            return Err(ManifestError::ConflictingTool(tool.clone()));
        }
        let mut names = BTreeSet::new();
        for skill in &self.skills {
            if !names.insert(&skill.name) {
                return Err(ManifestError::DuplicateSkill(skill.name.clone()));
            }
            if skill.revision.trim().is_empty() || skill.digest.trim().is_empty() {
                return Err(ManifestError::UnpinnedSkill(skill.name.clone()));
            }
        }
        if self.limits.timeout_seconds == 0 {
            return Err(ManifestError::ZeroTimeout);
        }
        Ok(())
    }
}

fn require_non_empty(value: &str, field: &'static str) -> Result<(), ManifestError> {
    if value.trim().is_empty() {
        Err(ManifestError::Empty(field))
    } else {
        Ok(())
    }
}

fn require_safe_path(path: &str) -> Result<(), ManifestError> {
    let path = std::path::Path::new(path);
    let safe = !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path.components().all(|component| {
            matches!(
                component,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        });
    if safe {
        Ok(())
    } else {
        Err(ManifestError::UnsafePath(path.display().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal() -> LaunchManifest {
        serde_json::from_str(include_str!("../examples/minimal/workspace-launch.json")).unwrap()
    }

    #[test]
    fn example_is_valid() {
        minimal().validate().unwrap();
    }

    #[test]
    fn rejects_path_traversal() {
        let mut manifest = minimal();
        manifest.workspace.working_directory = "../host".into();
        assert!(matches!(
            manifest.validate(),
            Err(ManifestError::UnsafePath(_))
        ));
    }

    #[test]
    fn rejects_conflicting_tool_policy() {
        let mut manifest = minimal();
        manifest.tools.deny.push("Read".into());
        assert_eq!(
            manifest.validate(),
            Err(ManifestError::ConflictingTool("Read".into()))
        );
    }
}
