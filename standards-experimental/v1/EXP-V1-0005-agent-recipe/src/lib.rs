//! Agent Recipe Standard (Experimental)
//!
//! This crate is the reference implementation of **EXP-V1-0005**, a
//! directory-based recipe artifact: a versioned directory describing *what
//! agent(s) to run* (harness, model, reasoning effort, skills, system
//! instructions), independent of *where* or *how* it is executed.
//!
//! See `docs/01_spec.md` for the normative specification. This crate
//! provides:
//!
//! - [`schema`] - typed Rust structures for the recipe directory shape
//!   ([`schema::RecipeManifest`], [`schema::AgentManifest`], [`schema::Recipe`])
//!   and the canonical loader ([`schema::load_recipe_dir`]). No CLI-only
//!   dependencies: downstream consumers (e.g. `itmux`, Plan B) can depend on
//!   this crate as a plain library.
//! - [`validate`] - [`validate::validate_recipe_dir`], a diagnostics-producing
//!   validator built on top of the loader (loading and validation share one
//!   code path).
//! - [`generate`] - [`generate::scaffold_recipe`], which writes a conformant
//!   recipe directory from the embedded template. Generator output always
//!   validates (see `tests/round_trip_test.rs`).
//! - [`cli`] - the composed-CLI command handler backing
//!   `apss-dev run agent-recipe validate <recipe-dir>` and
//!   `apss-dev run agent-recipe create <name>`.
//!
//! ⚠️ EXPERIMENTAL: This standard is in incubation and may change significantly.

pub mod cli;
pub mod generate;
pub mod schema;
pub mod validate;

pub use generate::{RecipeTemplateContext, scaffold_recipe};
pub use schema::{
    AgentManifest, EffortLevel, HarnessKind, HarnessPromptMode, InstructionMode, ModelSpec,
    Recipe, RecipeLoadError, RecipeManifest, SystemInstructions, load_recipe_dir, resolved_system,
};
pub use validate::validate_recipe_dir;

/// Immutable standard identifier.
pub const ID: &str = "EXP-V1-0005";

/// CLI dispatch slug.
pub const SLUG: &str = "agent-recipe";

/// Human-readable standard name.
pub const NAME: &str = "Agent Recipe Standard";

/// Version of this crate / standard.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Register this package with a composed APSS runner.
///
/// Exposes two commands, `validate <recipe-dir>` and `create <name>`, backed
/// by [`cli::AgentRecipeCommandHandler`], which wraps
/// [`validate::validate_recipe_dir`] and [`generate::scaffold_recipe`].
pub fn register(registry: &mut dyn apss_core::registry::StandardRegistry) {
    registry.register(
        apss_core::registry::RegisteredStandard {
            id: ID.to_string(),
            slug: SLUG.to_string(),
            name: NAME.to_string(),
            description: "Harness-neutral, directory-based agent recipe schema experiment"
                .to_string(),
            version: VERSION.to_string(),
            commands: cli::COMMAND_NAMES.iter().map(|s| s.to_string()).collect(),
        },
        Box::new(cli::AgentRecipeCommandHandler::new()),
    );
}
