//! APSS Bootstrap CLI
//!
//! Lightweight entry point for the Agent Paradise Standards System.
//! Handles project initialization, standard installation, and delegates
//! standard-specific commands to the composed project binary.

use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process;

mod init;
mod install;

// ============================================================================
// CLI Definition
// ============================================================================

#[derive(Parser)]
#[command(
    name = "apss",
    about = "Agent Paradise Standards System",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new APSS project
    Init(init::InitArgs),

    /// Install/update standards and build the project CLI
    Install(install::InstallArgs),

    /// Show project configuration and status
    Status,

    /// Validate project configuration
    Validate {
        /// Only validate apss.toml structure (skip standard-specific validation)
        #[arg(long)]
        config_only: bool,
    },

    /// Run a standard's command (delegates to composed binary)
    Run {
        /// Standard slug and command arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Show or generate standard configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Generate apss.toml with all defaults and comments
    Template,
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Commands::Init(args) => init::run(args),
        Commands::Install(args) => install::run(args),
        Commands::Status => cmd_status(),
        Commands::Validate { config_only } => cmd_validate(config_only),
        Commands::Run { args } => cmd_run(&args),
        Commands::Config { action } => match action {
            ConfigAction::Template => cmd_config_template(),
        },
    };

    process::exit(exit_code);
}

// ============================================================================
// Commands
// ============================================================================

fn cmd_status() -> i32 {
    let config_path = match find_config() {
        Some(p) => p,
        None => {
            eprintln!("No apss.toml found. Run 'apss init' to create one.");
            return 1;
        }
    };

    let config = match aps_core::config::parse_project_config(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load apss.toml: {e}");
            return 1;
        }
    };

    println!("Project: {}", config.project.name);
    println!("APSS Version: {}", config.project.apss_version);
    println!("Config: {}", config_path.display());
    println!();

    if config.standards.is_empty() {
        println!("Standards: (none declared)");
    } else {
        println!("Standards:");
        for (slug, entry) in &config.standards {
            let status = if entry.enabled { "" } else { " (disabled)" };
            let subs = entry
                .substandards
                .as_ref()
                .map(|s| format!(" [{}]", s.join(", ")))
                .unwrap_or_else(|| " [all]".to_string());
            println!(
                "  {:<20} {} {}{}{}",
                slug, entry.id, entry.version, subs, status
            );
        }
    }

    if let Some(ws) = &config.workspace {
        println!();
        println!("Workspace members: {}", ws.members.join(", "));
    }

    // Check installation state
    let project_root = config_path.parent().unwrap_or(Path::new("."));
    let binary_path = project_root
        .join(
            config
                .tool
                .as_ref()
                .map(|t| t.bin_dir.as_str())
                .unwrap_or(".apss/bin"),
        )
        .join("apss");

    println!();
    if binary_path.exists() {
        println!("Binary: {} (installed)", binary_path.display());
    } else {
        println!("Binary: not installed (run 'apss install')");
    }

    0
}

fn cmd_validate(config_only: bool) -> i32 {
    let config_path = match find_config() {
        Some(p) => p,
        None => {
            eprintln!("No apss.toml found. Run 'apss init' to create one.");
            return 1;
        }
    };

    let diags = apss_project_config::validate_project_config(&config_path);

    if !config_only {
        // Also validate installation state
        let project_root = config_path.parent().unwrap_or(Path::new("."));
        let install_diags = apss_distribution::validate_installation(project_root);
        let mut all = diags;
        all.merge(install_diags);
        print_and_exit(all)
    } else {
        print_and_exit(diags)
    }
}

fn cmd_run(args: &[String]) -> i32 {
    let config_path = match find_config() {
        Some(p) => p,
        None => {
            eprintln!("No apss.toml found. Run 'apss init' to create one.");
            return 1;
        }
    };

    let config = match aps_core::config::parse_project_config(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load apss.toml: {e}");
            return 1;
        }
    };

    let project_root = config_path.parent().unwrap_or(Path::new("."));
    let bin_dir = config
        .tool
        .as_ref()
        .map(|t| t.bin_dir.as_str())
        .unwrap_or(".apss/bin");
    let binary_path = project_root.join(bin_dir).join("apss");

    if !binary_path.exists() {
        eprintln!("Composed binary not found at {}", binary_path.display());
        eprintln!("Run 'apss install' first to build the project CLI.");
        return 1;
    }

    // Delegate to composed binary
    let mut cmd_args = vec!["run".to_string()];
    cmd_args.extend_from_slice(args);

    let status = process::Command::new(&binary_path).args(&cmd_args).status();

    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("Failed to run composed binary: {e}");
            1
        }
    }
}

fn cmd_config_template() -> i32 {
    println!(
        r#"# apss.toml — APSS Project Configuration
# See: https://github.com/AgentParadise/agent-paradise-standards-system

schema = "apss.project/v1"

[project]
name = "my-project"
apss_version = "v1"

# Declare which standards this project implements.
# Each key is a slug used for CLI dispatch (e.g., `apss run topology ...`).

# [standards.topology]
# id = "APS-V1-0001"
# version = ">=1.0.0, <2.0.0"
# substandards = ["RS01", "CI01"]  # omit for all
#
# [standards.topology.config]
# output_dir = ".topology"
# languages = ["rust", "python"]

# Monorepo workspace configuration (optional).
# [workspace]
# members = ["packages/*", "services/*"]
# exclude = ["packages/deprecated-*"]

# Tool configuration (optional).
# [tool]
# bin_dir = ".apss/bin"
# registry = "https://crates.io"
# offline = false
# log_level = "warn"
"#
    );
    0
}

// ============================================================================
// Helpers
// ============================================================================

fn find_config() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    aps_core::config::find_project_config(&cwd)
}

fn print_and_exit(diags: aps_core::Diagnostics) -> i32 {
    if !diags.is_empty() {
        eprintln!("{diags}");
    }

    if diags.has_errors() {
        1
    } else if diags.has_warnings() {
        eprintln!("\nValidation passed with warnings.");
        2
    } else {
        eprintln!("Validation passed.");
        0
    }
}
