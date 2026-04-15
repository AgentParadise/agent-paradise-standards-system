//! Dynamic standard composition and CLI dispatch.
//!
//! This module provides the [`StandardRegistry`] trait and [`ProjectRunner`]
//! for composing multiple standards into a single CLI binary. Standards
//! register themselves via `register()` functions, and the runner dispatches
//! commands based on the project's `apss.toml` configuration.
//!
//! See `APS-V1-0000.DI01` for the normative specification.

use crate::config::{self, ConfigError};
use crate::resolution::{self, ResolvedProjectConfig};
use std::path::{Path, PathBuf};
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

/// Errors from the project runner.
#[derive(Debug, Error)]
pub enum RunnerError {
    /// Failed to load configuration.
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Standard not found.
    #[error("standard '{slug}' not found in project configuration")]
    StandardNotFound { slug: String },

    /// Standard not registered (declared in config but not linked).
    #[error("standard '{slug}' is declared in apss.toml but not registered in this binary")]
    StandardNotRegistered { slug: String },

    /// No configuration file found.
    #[error("no apss.toml found (searched from {start_dir})")]
    NoConfig { start_dir: PathBuf },
}

// ============================================================================
// Registry Trait
// ============================================================================

/// Information about a registered standard's CLI capabilities.
#[derive(Debug, Clone)]
pub struct RegisteredStandard {
    /// Standard ID (e.g., `"APS-V1-0001"`).
    pub id: String,

    /// CLI dispatch slug (e.g., `"topology"`).
    pub slug: String,

    /// Human-readable name.
    pub name: String,

    /// Short description.
    pub description: String,

    /// Version of the standard.
    pub version: String,

    /// Available commands.
    pub commands: Vec<String>,
}

/// Trait for registering standards into a composed CLI.
///
/// Standards publish a `register()` function that calls methods on this trait
/// to make their CLI commands available.
pub trait StandardRegistry {
    /// Register a standard's CLI capabilities.
    fn register(&mut self, standard: RegisteredStandard, handler: Box<dyn CommandHandler>);
}

/// Handler for executing standard commands.
///
/// Each standard implements this to handle its CLI commands.
pub trait CommandHandler: Send + Sync {
    /// Execute a command with the given arguments.
    ///
    /// Returns the exit code (0 = success, 1 = error, 2 = warnings).
    fn execute(&self, command: &str, args: &[String], config: &toml::Value) -> i32;

    /// List available commands.
    fn commands(&self) -> Vec<CommandInfo>;
}

/// Information about a CLI command.
#[derive(Debug, Clone)]
pub struct CommandInfo {
    /// Command name (e.g., `"analyze"`).
    pub name: String,

    /// Short description.
    pub description: String,

    /// Usage pattern (e.g., `"analyze <path>"`).
    pub usage: String,
}

// ============================================================================
// Project Runner
// ============================================================================

/// Entry in the runner's registry.
struct RegistryEntry {
    info: RegisteredStandard,
    handler: Box<dyn CommandHandler>,
}

/// Config-driven CLI runner for consumer projects.
///
/// This is the main entry point for composed binaries. It:
/// 1. Loads `apss.toml` configuration
/// 2. Accepts standard registrations via [`StandardRegistry`]
/// 3. Dispatches CLI commands to the appropriate standard handler
pub struct ProjectRunner {
    config: ResolvedProjectConfig,
    entries: Vec<RegistryEntry>,
}

impl ProjectRunner {
    /// Create a runner from an `apss.toml` file path.
    pub fn from_config_file(path: &Path) -> Result<Self, RunnerError> {
        let project_config = config::parse_project_config(path)?;
        let resolved = resolution::resolve_single(project_config, path.to_path_buf());

        Ok(Self {
            config: resolved,
            entries: Vec::new(),
        })
    }

    /// Create a runner from an already-resolved configuration.
    pub fn from_resolved(config: ResolvedProjectConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
        }
    }

    /// Get the resolved configuration.
    pub fn config(&self) -> &ResolvedProjectConfig {
        &self.config
    }

    /// Run the CLI with the given arguments.
    ///
    /// Expects args in the form: `["run", "<slug>", "<command>", ...]`
    /// Returns the process exit code.
    pub fn run(&self, args: &[String]) -> i32 {
        if args.is_empty() {
            self.print_usage();
            return 0;
        }

        match args[0].as_str() {
            "run" => self.handle_run(&args[1..]),
            "--list" | "list" => {
                self.print_standards();
                0
            }
            "--help" | "help" | "-h" => {
                self.print_usage();
                0
            }
            _ => {
                eprintln!("Unknown command: {}", args[0]);
                self.print_usage();
                3
            }
        }
    }

    fn handle_run(&self, args: &[String]) -> i32 {
        if args.is_empty() {
            eprintln!("Usage: apss run <standard> <command> [args...]");
            eprintln!("\nUse 'apss run --list' to see available standards.");
            return 3;
        }

        let slug = &args[0];

        if slug == "--list" {
            self.print_standards();
            return 0;
        }

        // Check if standard is in config
        let resolved = match self.config.standards.get(slug.as_str()) {
            Some(s) if s.enabled => s,
            Some(_) => {
                eprintln!("Standard '{slug}' is disabled in apss.toml");
                return 1;
            }
            None => {
                eprintln!("Standard '{slug}' not found in apss.toml");
                return 1;
            }
        };

        // Find registered handler
        let entry = match self.entries.iter().find(|e| e.info.slug == *slug) {
            Some(e) => e,
            None => {
                eprintln!(
                    "Standard '{slug}' is declared in apss.toml but not registered in this binary"
                );
                eprintln!("Run 'apss install' to rebuild with the correct standards.");
                return 1;
            }
        };

        if args.len() < 2 {
            eprintln!("Usage: apss run {slug} <command> [args...]");
            eprintln!("\nAvailable commands:");
            for cmd in entry.handler.commands() {
                eprintln!("  {:<20} {}", cmd.name, cmd.description);
            }
            return 3;
        }

        let command = &args[1];
        let cmd_args = &args[2..];

        entry.handler.execute(command, cmd_args, &resolved.config)
    }

    fn print_usage(&self) {
        eprintln!("APSS — Agent Paradise Standards System");
        eprintln!();
        eprintln!("Usage: apss <command>");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  run <standard> <cmd>   Run a standard's command");
        eprintln!("  list                   List available standards");
        eprintln!("  help                   Show this help");
    }

    fn print_standards(&self) {
        eprintln!("Available standards:");
        for (slug, standard) in &self.config.standards {
            if !standard.enabled {
                continue;
            }
            let registered = if self.entries.iter().any(|e| e.info.slug == *slug) {
                ""
            } else {
                " (not registered)"
            };
            eprintln!(
                "  {:<20} {} v{}{}",
                slug, standard.id, standard.version_req, registered
            );
        }
    }
}

impl StandardRegistry for ProjectRunner {
    fn register(&mut self, standard: RegisteredStandard, handler: Box<dyn CommandHandler>) {
        self.entries.push(RegistryEntry {
            info: standard,
            handler,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ProjectInfo, ToolConfig};
    use crate::resolution::ResolvedStandard;
    use std::collections::BTreeMap;

    struct MockHandler;

    impl CommandHandler for MockHandler {
        fn execute(&self, command: &str, _args: &[String], _config: &toml::Value) -> i32 {
            match command {
                "analyze" => 0,
                _ => 1,
            }
        }

        fn commands(&self) -> Vec<CommandInfo> {
            vec![CommandInfo {
                name: "analyze".to_string(),
                description: "Analyze code".to_string(),
                usage: "analyze <path>".to_string(),
            }]
        }
    }

    fn test_config() -> ResolvedProjectConfig {
        ResolvedProjectConfig {
            project: ProjectInfo {
                name: "test".to_string(),
                apss_version: "v1".to_string(),
            },
            standards: BTreeMap::from([(
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
            )]),
            tool: ToolConfig::default(),
            source_files: vec![],
        }
    }

    #[test]
    fn test_runner_dispatch() {
        let mut runner = ProjectRunner::from_resolved(test_config());
        runner.register(
            RegisteredStandard {
                id: "APS-V1-0001".to_string(),
                slug: "topology".to_string(),
                name: "Code Topology".to_string(),
                description: "Topology analysis".to_string(),
                version: "1.0.0".to_string(),
                commands: vec!["analyze".to_string()],
            },
            Box::new(MockHandler),
        );

        let args: Vec<String> = ["run", "topology", "analyze", "."]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let exit = runner.run(&args);
        assert_eq!(exit, 0);
    }

    #[test]
    fn test_runner_unknown_standard() {
        let runner = ProjectRunner::from_resolved(test_config());
        let args: Vec<String> = ["run", "unknown", "analyze"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let exit = runner.run(&args);
        assert_eq!(exit, 1);
    }

    #[test]
    fn test_runner_list() {
        let runner = ProjectRunner::from_resolved(test_config());
        let args: Vec<String> = ["list"].iter().map(|s| s.to_string()).collect();
        let exit = runner.run(&args);
        assert_eq!(exit, 0);
    }
}
