//! Template engine for scaffolding APS packages.
//!
//! Provides utilities for rendering templates with variable substitution.

use handlebars::Handlebars;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Template rendering engine.
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine {
    /// Create a new template engine.
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        Self { handlebars }
    }

    /// Render a template string with the given context.
    pub fn render_string<T: Serialize>(
        &self,
        template: &str,
        context: &T,
    ) -> Result<String, TemplateError> {
        self.handlebars
            .render_template(template, context)
            .map_err(|e| TemplateError::Render(e.to_string()))
    }

    /// Render a template file with the given context.
    pub fn render_file<T: Serialize>(
        &self,
        template_path: &Path,
        context: &T,
    ) -> Result<String, TemplateError> {
        let template = fs::read_to_string(template_path).map_err(|e| TemplateError::Io {
            path: template_path.to_path_buf(),
            source: e,
        })?;

        self.render_string(&template, context)
    }

    /// Render a skeleton directory to an output directory.
    ///
    /// Walks the skeleton directory and renders each file as a template.
    pub fn render_skeleton<T: Serialize>(
        &self,
        skeleton_dir: &Path,
        output_dir: &Path,
        context: &T,
    ) -> Result<Vec<PathBuf>, TemplateError> {
        let mut created_files = Vec::new();

        if !skeleton_dir.exists() {
            return Err(TemplateError::SkeletonNotFound(skeleton_dir.to_path_buf()));
        }

        // Walk the skeleton directory
        for entry in walkdir::WalkDir::new(skeleton_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let rel_path = entry
                .path()
                .strip_prefix(skeleton_dir)
                .map_err(|_| TemplateError::PathError)?;

            // Render the path itself (allows {{slug}} in filenames)
            let rendered_path = self.render_path(rel_path, context)?;
            let target_path = output_dir.join(&rendered_path);

            if entry.file_type().is_dir() {
                fs::create_dir_all(&target_path).map_err(|e| TemplateError::Io {
                    path: target_path.clone(),
                    source: e,
                })?;
            } else if entry.file_type().is_file() {
                // Create parent directories
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| TemplateError::Io {
                        path: parent.to_path_buf(),
                        source: e,
                    })?;
                }

                // Check if file should be rendered as template
                if should_render_as_template(entry.path()) {
                    let content = self.render_file(entry.path(), context)?;
                    fs::write(&target_path, content).map_err(|e| TemplateError::Io {
                        path: target_path.clone(),
                        source: e,
                    })?;
                } else {
                    // Copy as-is
                    fs::copy(entry.path(), &target_path).map_err(|e| TemplateError::Io {
                        path: target_path.clone(),
                        source: e,
                    })?;
                }

                created_files.push(target_path);
            }
        }

        Ok(created_files)
    }

    /// Render template variables in a path.
    fn render_path<T: Serialize>(
        &self,
        path: &Path,
        context: &T,
    ) -> Result<PathBuf, TemplateError> {
        let path_str = path.to_string_lossy();

        // Only render if path contains template markers
        if path_str.contains("{{") {
            let rendered = self.render_string(&path_str, context)?;
            Ok(PathBuf::from(rendered))
        } else {
            Ok(path.to_path_buf())
        }
    }
}

/// Check if a file should be rendered as a template.
fn should_render_as_template(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    // Template-eligible extensions
    matches!(
        ext,
        "toml" | "md" | "rs" | "yaml" | "yml" | "json" | "txt" | "hbs"
    )
}

/// Context for rendering a standard package template.
#[derive(Debug, Clone, Serialize)]
pub struct StandardContext {
    /// Standard ID (e.g., "APS-V1-0001").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Filesystem-safe slug (kebab-case).
    pub slug: String,
    /// SemVer version.
    pub version: String,
    /// Category: governance, technical, design, process, security.
    pub category: String,
    /// Maintainers list.
    pub maintainers: Vec<String>,
}

impl StandardContext {
    /// Create a new standard context with defaults.
    pub fn new(id: &str, name: &str, slug: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            slug: slug.to_string(),
            version: "1.0.0".to_string(),
            category: "governance".to_string(),
            maintainers: vec!["AgentParadise".to_string()],
        }
    }
}

/// Context for rendering an experiment package template.
#[derive(Debug, Clone, Serialize)]
pub struct ExperimentContext {
    /// Experiment ID (e.g., "EXP-V1-0001").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Filesystem-safe slug.
    pub slug: String,
    /// SemVer version.
    pub version: String,
    /// Category.
    pub category: String,
    /// Maintainers list.
    pub maintainers: Vec<String>,
}

impl ExperimentContext {
    /// Create a new experiment context with defaults.
    pub fn new(id: &str, name: &str, slug: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            slug: slug.to_string(),
            version: "0.1.0".to_string(),
            category: "technical".to_string(),
            maintainers: vec!["AgentParadise".to_string()],
        }
    }
}

/// Context for rendering a substandard package template.
#[derive(Debug, Clone, Serialize)]
pub struct SubstandardContext {
    /// Substandard ID (e.g., "APS-V1-0001.GH01").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Filesystem-safe slug.
    pub slug: String,
    /// SemVer version.
    pub version: String,
    /// Parent standard ID.
    pub parent_id: String,
    /// Parent major version.
    pub parent_major: String,
    /// Maintainers list.
    pub maintainers: Vec<String>,
}

impl SubstandardContext {
    /// Create a new substandard context with defaults.
    pub fn new(id: &str, name: &str, slug: &str, parent_id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            slug: slug.to_string(),
            version: "1.0.0".to_string(),
            parent_id: parent_id.to_string(),
            parent_major: "1".to_string(),
            maintainers: vec!["AgentParadise".to_string()],
        }
    }
}

/// Errors that can occur during template operations.
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    /// Template rendering error.
    #[error("template render error: {0}")]
    Render(String),

    /// IO error.
    #[error("IO error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Skeleton directory not found.
    #[error("skeleton directory not found: {0}")]
    SkeletonNotFound(PathBuf),

    /// Path manipulation error.
    #[error("path manipulation error")]
    PathError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_render_string() {
        let engine = TemplateEngine::new();
        let mut context = HashMap::new();
        context.insert("name", "Test");

        let result = engine.render_string("Hello, {{name}}!", &context).unwrap();
        assert_eq!(result, "Hello, Test!");
    }

    #[test]
    fn test_standard_context() {
        let ctx = StandardContext::new("APS-V1-0001", "Test Standard", "test-standard");

        assert_eq!(ctx.id, "APS-V1-0001");
        assert_eq!(ctx.version, "1.0.0");
        assert_eq!(ctx.category, "governance");
    }

    #[test]
    fn test_experiment_context() {
        let ctx = ExperimentContext::new("EXP-V1-0001", "Test Experiment", "test-experiment");

        assert_eq!(ctx.id, "EXP-V1-0001");
        assert_eq!(ctx.version, "0.1.0"); // Experiments start at 0.x
    }

    #[test]
    fn test_should_render_as_template() {
        assert!(should_render_as_template(Path::new("file.toml")));
        assert!(should_render_as_template(Path::new("file.md")));
        assert!(should_render_as_template(Path::new("file.rs")));
        assert!(!should_render_as_template(Path::new("file.png")));
        assert!(!should_render_as_template(Path::new("file.exe")));
    }

    #[test]
    fn test_render_skeleton() {
        let engine = TemplateEngine::new();
        let temp_dir = tempfile::tempdir().unwrap();

        // Create a simple skeleton
        let skeleton_dir = temp_dir.path().join("skeleton");
        fs::create_dir_all(&skeleton_dir).unwrap();
        fs::write(
            skeleton_dir.join("README.md"),
            "# {{name}}\n\nVersion: {{version}}",
        )
        .unwrap();

        // Render it
        let output_dir = temp_dir.path().join("output");
        let context = StandardContext::new("APS-V1-0001", "My Standard", "my-standard");

        let files = engine
            .render_skeleton(&skeleton_dir, &output_dir, &context)
            .unwrap();

        assert_eq!(files.len(), 1);

        let content = fs::read_to_string(output_dir.join("README.md")).unwrap();
        assert!(content.contains("My Standard"));
        assert!(content.contains("1.0.0"));
    }
}
