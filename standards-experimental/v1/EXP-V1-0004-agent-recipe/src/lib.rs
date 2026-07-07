//! Agent Recipe Standard (Experimental)
//!
//! This crate is the reference implementation of **EXP-V1-0004**, a
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
//!
//! ⚠️ EXPERIMENTAL: This standard is in incubation and may change significantly.

pub mod schema;

pub use schema::{
    AgentKind, AgentManifest, EffortLevel, InstructionMode, ModelSpec, Recipe, RecipeLoadError,
    RecipeManifest, SystemInstructions, load_recipe_dir, resolved_system,
};

/// Register this package with a composed APSS runner.
///
/// No composed CLI commands are exposed yet: this experiment's surface area
/// is the [`schema`] module, consumed directly by Rust callers (see
/// `agents/skills/README.md`). A CLI subcommand (e.g. `aps v1 validate
/// recipe <dir>`) is a natural follow-up once `validate_recipe_dir` (Task 2)
/// lands.
pub fn register(registry: &mut dyn apss_core::registry::StandardRegistry) {
    registry.register(
        apss_core::registry::RegisteredStandard {
            id: "EXP-V1-0004".to_string(),
            slug: "agent-recipe".to_string(),
            name: "Agent Recipe Standard".to_string(),
            description: "Harness-neutral, directory-based agent recipe schema experiment"
                .to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            commands: Vec::new(),
        },
        Box::new(NoopCommandHandler),
    );
}

struct NoopCommandHandler;

impl apss_core::registry::CommandHandler for NoopCommandHandler {
    fn execute(&self, _command: &str, _args: &[String], _config: &toml::Value) -> i32 {
        eprintln!("No composed CLI commands are registered for agent-recipe yet.");
        5
    }

    fn commands(&self) -> Vec<apss_core::registry::CommandInfo> {
        Vec::new()
    }
}
