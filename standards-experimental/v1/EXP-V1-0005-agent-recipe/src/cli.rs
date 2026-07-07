//! Composed CLI for the Agent Recipe Standard (EXP-V1-0005).
//!
//! This module hosts the command implementations that back
//! `apss-dev run agent-recipe validate <recipe-dir>` and
//! `apss-dev run agent-recipe create <name>` (and the `recipe` alias) in the
//! development CLI. It dispatches through [`AgentRecipeCommandHandler`], which
//! implements [`apss_core::registry::CommandHandler`].
//!
//! Impedance notes (mirroring the topology / fitness / documentation handlers):
//! - `CommandHandler::execute` receives no repo root, so the handler resolves
//!   `repo_root = std::env::current_dir()` and joins relative paths onto it.
//! - command functions return `i32` (0 success, 1 validation errors, 3
//!   usage/unknown).

use apss_core::registry::{CommandHandler, CommandInfo};

use crate::generate::scaffold_recipe;
use crate::validate::validate_recipe_dir;

/// Handler that backs `run agent-recipe <command>` in the dev CLI.
pub struct AgentRecipeCommandHandler;

impl AgentRecipeCommandHandler {
    /// Create a new handler instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AgentRecipeCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHandler for AgentRecipeCommandHandler {
    fn execute(&self, command: &str, args: &[String], _config: &toml::Value) -> i32 {
        let repo_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        dispatch(command, args, &repo_root)
    }

    fn commands(&self) -> Vec<CommandInfo> {
        command_infos()
    }
}

/// Dispatch an agent-recipe command to its implementation.
fn dispatch(command: &str, args: &[String], repo_root: &std::path::Path) -> i32 {
    match command {
        "--help" | "-h" | "help" => {
            print_help();
            0
        }
        "validate" => validate(args, repo_root),
        "create" => create(args, repo_root),
        other => {
            eprintln!("Error: Unknown agent-recipe command '{other}'");
            eprintln!("Use 'apss-dev run agent-recipe --help' for available commands.");
            3
        }
    }
}

/// Scaffold a new conformant recipe directory from the embedded template.
///
/// `create <name> [--dir <parent>]` writes `<parent>/<name>/` (parent defaults
/// to the current directory).
fn create(args: &[String], repo_root: &std::path::Path) -> i32 {
    let mut name: Option<&str> = None;
    let mut parent: Option<std::path::PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dir" => {
                parent = args.get(i + 1).map(std::path::PathBuf::from);
                i += 2;
            }
            arg if !arg.starts_with('-') && name.is_none() => {
                name = Some(arg);
                i += 1;
            }
            _ => i += 1,
        }
    }

    let Some(name) = name else {
        eprintln!("Error: a recipe name is required");
        eprintln!("Usage: apss-dev run agent-recipe create <name> [--dir <parent>]");
        return 3;
    };

    let parent = parent.unwrap_or_else(|| repo_root.to_path_buf());
    let parent = if parent.is_absolute() {
        parent
    } else {
        repo_root.join(parent)
    };
    let dest = parent.join(name);

    match scaffold_recipe(name, &dest) {
        Ok(files) => {
            println!("Created recipe '{name}' at {}", dest.display());
            println!("\n{} file(s):", files.len());
            for file in &files {
                let shown = file.strip_prefix(repo_root).unwrap_or(file);
                println!("  {}", shown.display());
            }
            println!(
                "\nNext steps:\n  1. Edit agents/main.yaml and SYSTEM.md\n  2. Add skills under skills/\n  3. Validate: apss-dev run agent-recipe validate {}",
                dest.display()
            );
            0
        }
        Err(error) => {
            eprintln!("Error: {error}");
            1
        }
    }
}

/// Validate a recipe directory and print a human-readable report.
fn validate(args: &[String], repo_root: &std::path::Path) -> i32 {
    let positional = args.iter().find(|a| !a.starts_with('-'));
    let Some(path) = positional else {
        eprintln!("Error: a recipe directory path is required");
        eprintln!("Usage: apss-dev run agent-recipe validate <recipe-dir>");
        return 3;
    };

    let target = if std::path::Path::new(path).is_absolute() {
        std::path::PathBuf::from(path)
    } else {
        repo_root.join(path)
    };

    let diagnostics = validate_recipe_dir(&target);

    println!("Agent Recipe Validation Report");
    println!("==============================\n");
    println!("Recipe: {}", target.display());

    if diagnostics.is_empty() {
        println!("\nValidation passed with no issues.");
        return 0;
    }

    println!();
    println!("{diagnostics}");

    if diagnostics.has_errors() { 1 } else { 0 }
}

fn print_help() {
    println!("Agent Recipe Standard ({}) v{}", crate::ID, crate::VERSION);
    println!();
    println!("USAGE:");
    println!("    apss-dev run agent-recipe <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    validate <recipe-dir>       Validate a recipe directory against EXP-V1-0005");
    println!("    create <name> [--dir <p>]   Scaffold a conformant recipe directory");
    println!();
    println!("OPTIONS:");
    println!("    --dir <parent>              (create) parent directory for the new recipe");
    println!("    --help                      Show this help message");
}

/// The command list returned by `commands()` and used by `register()`.
fn command_infos() -> Vec<CommandInfo> {
    vec![
        CommandInfo {
            name: "validate".to_string(),
            description: "Validate a recipe directory against EXP-V1-0005".to_string(),
            usage: "validate <recipe-dir>".to_string(),
        },
        CommandInfo {
            name: "create".to_string(),
            description: "Scaffold a conformant recipe directory from the template".to_string(),
            usage: "create <name> [--dir <parent>]".to_string(),
        },
    ]
}

/// Command names registered by `register()`; kept in sync with [`command_infos`].
pub(crate) const COMMAND_NAMES: [&str; 2] = ["validate", "create"];
