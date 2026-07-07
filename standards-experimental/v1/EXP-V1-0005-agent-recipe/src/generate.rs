//! Recipe directory generator for EXP-V1-0005 (Agent Recipe Standard).
//!
//! [`scaffold_recipe`] writes a conformant recipe directory from the canonical
//! template in `templates/recipe/skeleton/`. The template files are embedded at
//! compile time via `include_str!`, so the generator is self-contained (it does
//! not resolve the crate directory at runtime) and its output can never drift
//! from the reviewed on-disk skeleton.
//!
//! The strongest correctness guarantee for this leg of the standard is that
//! generator output always validates: see `tests/round_trip_test.rs`, which
//! scaffolds into a temp dir and asserts [`crate::validate_recipe_dir`] reports
//! zero errors.

use apss_core::TemplateEngine;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// `recipe.yaml`, the only template file with a `{{name}}` variable.
const RECIPE_YAML: &str = include_str!("../templates/recipe/skeleton/recipe.yaml");
/// The default `claude` agent plus the commented `codex` example.
const AGENT_MAIN_YAML: &str = include_str!("../templates/recipe/skeleton/agents/main.yaml");
/// Starter shared base instructions.
const SYSTEM_MD: &str = include_str!("../templates/recipe/skeleton/SYSTEM.md");
/// Keeps the initially empty `skills/` directory tracked.
const SKILLS_GITKEEP: &str = include_str!("../templates/recipe/skeleton/skills/.gitkeep");

/// Handlebars context for rendering the recipe template.
#[derive(Debug, Clone, Serialize)]
pub struct RecipeTemplateContext {
    /// Recipe name, substituted for `{{name}}` in `recipe.yaml`.
    pub name: String,
}

/// Failure modes of [`scaffold_recipe`].
#[derive(Debug, thiserror::Error)]
pub enum GenerateError {
    /// The destination already exists; the generator never overwrites.
    #[error("destination already exists: {0}")]
    AlreadyExists(PathBuf),
    /// Rendering the `recipe.yaml` template failed.
    #[error("failed to render recipe template: {0}")]
    Render(String),
    /// A filesystem operation failed.
    #[error("io error at {path}: {source}")]
    Io {
        /// Path being written when the error occurred.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

/// Scaffold a conformant recipe directory named `name` at `dest`.
///
/// `dest` is the recipe directory itself (for example `./my-recipe`). It MUST
/// NOT already exist. On success, returns the list of files written, sorted.
///
/// The emitted directory always passes [`crate::validate_recipe_dir`].
pub fn scaffold_recipe(name: &str, dest: &Path) -> Result<Vec<PathBuf>, GenerateError> {
    if dest.exists() {
        return Err(GenerateError::AlreadyExists(dest.to_path_buf()));
    }

    let engine = TemplateEngine::new();
    let context = RecipeTemplateContext {
        name: name.to_string(),
    };
    let recipe_yaml = engine
        .render_string(RECIPE_YAML, &context)
        .map_err(|error| GenerateError::Render(error.to_string()))?;

    // (relative path, contents) for every file the skeleton emits. Only
    // recipe.yaml is templated; the rest are literal starter content.
    let files: [(&str, &str); 4] = [
        ("recipe.yaml", &recipe_yaml),
        ("agents/main.yaml", AGENT_MAIN_YAML),
        ("skills/.gitkeep", SKILLS_GITKEEP),
        ("SYSTEM.md", SYSTEM_MD),
    ];

    let mut written = Vec::with_capacity(files.len());
    for (rel_path, contents) in files {
        let target = dest.join(rel_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| GenerateError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::write(&target, contents).map_err(|source| GenerateError::Io {
            path: target.clone(),
            source,
        })?;
        written.push(target);
    }

    written.sort();
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_writes_expected_files() {
        let temp = std::env::temp_dir().join(format!("apss-recipe-gen-{}", std::process::id()));
        let dest = temp.join("my-recipe");
        // Best-effort clean slate.
        let _ = fs::remove_dir_all(&temp);

        let written = scaffold_recipe("my-recipe", &dest).expect("scaffold should succeed");
        assert_eq!(written.len(), 4);
        assert!(dest.join("recipe.yaml").is_file());
        assert!(dest.join("agents/main.yaml").is_file());
        assert!(dest.join("skills/.gitkeep").is_file());
        assert!(dest.join("SYSTEM.md").is_file());

        let recipe = fs::read_to_string(dest.join("recipe.yaml")).unwrap();
        assert!(recipe.contains("name: my-recipe"));
        assert!(recipe.contains("default_agent: main"));

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn scaffold_refuses_existing_destination() {
        let temp = std::env::temp_dir().join(format!("apss-recipe-gen-x-{}", std::process::id()));
        let dest = temp.join("exists");
        fs::create_dir_all(&dest).unwrap();

        let error = scaffold_recipe("exists", &dest).expect_err("should refuse existing dest");
        assert!(matches!(error, GenerateError::AlreadyExists(_)));

        let _ = fs::remove_dir_all(&temp);
    }
}
