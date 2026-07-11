//! Composed CLI for the Code Topology standard (ADR-0002, issue #68).
//!
//! This module hosts the command implementations that back `apss run
//! code-topology <command>` in composed consumer binaries and `apss-dev run
//! topology <command>` in the development CLI. Both routes dispatch through
//! [`TopologyCommandHandler`], which implements
//! [`apss_core::registry::CommandHandler`].
//!
//! Impedance notes:
//! - `CommandHandler::execute` receives no repo root, so the handler resolves
//!   `repo_root = std::env::current_dir()`.
//! - verbose output is env-driven: the handler reads `APSS_VERBOSE=1` once and
//!   passes a plain `bool` to the internal command functions.
//! - command functions return `i32` (0 success, 1 error, 2 warning, 3 usage,
//!   5 unavailable feature).

mod analyze;
mod diff;
mod health;
mod report;
pub mod sensors;
mod validate;
mod viz;
pub mod vsa_config;

use apss_core::registry::{CommandHandler, CommandInfo};

/// Handler that backs `run code-topology <command>` in composed binaries.
pub struct TopologyCommandHandler;

impl TopologyCommandHandler {
    /// Create a new handler instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TopologyCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHandler for TopologyCommandHandler {
    fn execute(&self, command: &str, args: &[String], _config: &toml::Value) -> i32 {
        // repo_root: the dev CLI runs commands relative to the invocation
        // directory, so resolving the current directory preserves behavior.
        let repo_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        // verbose: env-driven so the trait boundary stays signature-stable.
        let verbose = std::env::var("APSS_VERBOSE").is_ok_and(|v| v == "1");

        dispatch(command, args, &repo_root, verbose)
    }

    fn commands(&self) -> Vec<CommandInfo> {
        command_infos()
    }
}

/// Dispatch a topology command to its implementation.
fn dispatch(command: &str, args: &[String], repo_root: &std::path::Path, verbose: bool) -> i32 {
    match command {
        "--help" | "-h" | "help" => {
            print_help();
            0
        }
        "analyze" => {
            let path = args.first().map(|s| s.as_str()).unwrap_or(".");
            let output = args
                .iter()
                .position(|a| a == "--output")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str())
                .unwrap_or(".topology");
            let language = args
                .iter()
                .position(|a| a == "--language")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str());

            analyze::topology_analyze(path, output, language, repo_root, verbose)
        }
        "validate" => {
            let path = args.first().map(|s| s.as_str()).unwrap_or(".topology");
            validate::topology_validate(path, verbose)
        }
        "diff" => {
            if args.len() < 2 {
                eprintln!("Error: diff requires two paths");
                eprintln!("Usage: apss-dev run topology diff <base> <target> [--format json]");
                return 1;
            }
            let format = args
                .iter()
                .position(|a| a == "--format")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str())
                .unwrap_or("text");
            diff::topology_diff(&args[0], &args[1], format, verbose)
        }
        "check" => {
            let diff_file = args.first().map(|s| s.as_str());
            let config = args
                .iter()
                .position(|a| a == "--config")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str());
            diff::topology_check(diff_file, config, verbose)
        }
        "comment" => {
            let diff_file = args.first().map(|s| s.as_str());
            let config = args
                .iter()
                .position(|a| a == "--config")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str());
            diff::topology_comment(diff_file, config, verbose)
        }
        "report" => {
            let path = args.first().map(|s| s.as_str()).unwrap_or(".topology");
            report::topology_report(path, verbose)
        }
        "sensors" => sensors::topology_sensors(args, repo_root, verbose),
        "viz" | "3d" | "visualize" => {
            // Parse options first
            let viz_type = args
                .iter()
                .position(|a| a == "--type" || a == "-t")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str())
                .unwrap_or("3d");
            let output = args
                .iter()
                .position(|a| a == "--output" || a == "-o")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str());

            // Get path: first non-option argument that's not a value of --type or --output
            let type_value_idx = args
                .iter()
                .position(|a| a == "--type" || a == "-t")
                .map(|i| i + 1);
            let output_value_idx = args
                .iter()
                .position(|a| a == "--output" || a == "-o")
                .map(|i| i + 1);

            let path = args
                .iter()
                .enumerate()
                .find(|(i, a)| {
                    !a.starts_with('-')
                        && Some(*i) != type_value_idx
                        && Some(*i) != output_value_idx
                })
                .map(|(_, s)| s.as_str())
                .unwrap_or(".topology");

            viz::topology_viz(path, viz_type, output, verbose)
        }
        other => {
            eprintln!("Error: Unknown topology command '{other}'");
            eprintln!("Use 'apss-dev run topology --help' for available commands.");
            3
        }
    }
}

/// Print the topology CLI help text.
fn print_help() {
    println!("Code Topology (EXP-V1-0001) v0.1.0");
    println!();
    println!("USAGE:");
    println!("    apss-dev run topology <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    analyze <path>     Analyze codebase and generate .topology/");
    println!("    validate <path>    Validate existing .topology/ artifacts");
    println!("    diff <a> <b>       Compare two topology snapshots");
    println!("    check <diff.json>  Check diff against thresholds");
    println!("    comment <diff.json> Generate PR comment markdown");
    println!("    report <path>      Generate human-readable report");
    println!("    sensors <path>     Emit agent-queryable JSON sensor signals from .topology/");
    println!("    viz <path>         Generate visualizations from .topology/");
    println!();
    println!("OPTIONS:");
    println!("    --output <dir>     Output directory (default: .topology)");
    println!("    --language <lang>  Filter by language: rust, python (default: auto-detect)");
    println!("    --format <fmt>     Output format: json, text (default: text)");
    println!("    --config <file>    Config file for thresholds");
    println!("    --help             Show this help message");
    println!();
    println!("VIZ OPTIONS:");
    println!("    --type <type>      Visualization type:");
    println!("                       3d       - 3D force-directed coupling graph (default)");
    println!("                       codecity - 3D city metaphor (buildings = modules)");
    println!("                       clusters - 2D package relationship graph");
    println!("                       vsa      - Vertical Slice Architecture matrix");
    println!("                       all      - Generate all visualizations");
    println!("    --output <path>    Output file/directory");
    println!();
    println!("SUPPORTED LANGUAGES:");
    println!("    rust       .rs");
    println!("    python     .py, .pyi");
}

/// The command list returned by `commands()` and used by `register()`.
fn command_infos() -> Vec<CommandInfo> {
    vec![
        CommandInfo {
            name: "analyze".to_string(),
            description: "Analyze codebase and generate .topology/".to_string(),
            usage: "analyze <path>".to_string(),
        },
        CommandInfo {
            name: "validate".to_string(),
            description: "Validate existing .topology/ artifacts".to_string(),
            usage: "validate <path>".to_string(),
        },
        CommandInfo {
            name: "diff".to_string(),
            description: "Compare two topology snapshots".to_string(),
            usage: "diff <a> <b>".to_string(),
        },
        CommandInfo {
            name: "check".to_string(),
            description: "Check diff against thresholds".to_string(),
            usage: "check <diff.json>".to_string(),
        },
        CommandInfo {
            name: "comment".to_string(),
            description: "Generate PR comment markdown".to_string(),
            usage: "comment <diff.json>".to_string(),
        },
        CommandInfo {
            name: "report".to_string(),
            description: "Generate human-readable report".to_string(),
            usage: "report <path>".to_string(),
        },
        CommandInfo {
            name: "sensors".to_string(),
            description: "Emit agent-queryable JSON sensor signals from .topology/".to_string(),
            usage:
                "sensors <path> [--by MODE] [--top N] [--format FMT] [--output FILE] [--persist]"
                    .to_string(),
        },
        CommandInfo {
            name: "viz".to_string(),
            description: "Generate visualizations from .topology/".to_string(),
            usage: "viz <path>".to_string(),
        },
    ]
}

/// Command names registered by `register()`; kept in sync with [`command_infos`].
pub(crate) const COMMAND_NAMES: [&str; 8] = [
    "analyze", "validate", "diff", "check", "comment", "report", "sensors", "viz",
];
