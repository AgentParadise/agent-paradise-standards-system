//! Composed CLI for the Documentation and Context Engineering standard
//! (APS-V1-0003, ADR-0002, issue #68).
//!
//! This module hosts the command implementations that back `apss run
//! documentation <command>` in composed consumer binaries and
//! `apss-dev run documentation|docs|doc <command>` in the development CLI.
//! `documentation` is the canonical slug and the only one a composed binary
//! resolves; `docs` and `doc` are dev-CLI aliases. Both routes dispatch through
//! [`DocumentationCommandHandler`], which implements
//! [`apss_core::registry::CommandHandler`].
//!
//! Impedance notes:
//! - `CommandHandler::execute` receives no repo root, so the handler resolves
//!   `repo_root = std::env::current_dir()`.
//! - verbose output is env-driven: the handler reads `APSS_VERBOSE=1` once and
//!   passes a plain `bool` to the internal command functions. The doc commands
//!   do not currently branch on verbose, but the transport is preserved for
//!   parity with the topology handler.
//! - command functions return `i32` (0 success, 1 error, 3 usage).
//! - substandard validators (AD01/PV01/RT01) are crate-internal modules gated
//!   behind their cargo features; when a feature is off, that substandard's
//!   validation is skipped rather than failing, mirroring how code-topology
//!   gates its optional viz backends.

use apss_core::registry::{CommandHandler, CommandInfo};

use crate::config::{ApssConfig, ConfigError, DocsConfig};

/// Handler that backs `run documentation <command>` in composed binaries.
/// The `docs`/`doc` spellings are accepted only by the development CLI
/// `apss-dev`; a composed consumer binary matches on the canonical slug.
pub struct DocumentationCommandHandler;

impl DocumentationCommandHandler {
    /// Create a new handler instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DocumentationCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandHandler for DocumentationCommandHandler {
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

/// Load docs config from a custom path (for the `--config` flag).
///
/// Mirrors [`crate::config::load_config`] but works against an arbitrary path
/// so operators can point `apss run documentation validate` at a non-default
/// `apss.yaml`. YAML format per the unified-config brief (2026-06-04); other
/// top-level sections owned by other standards are tolerated and ignored.
fn load_docs_config(path: &str) -> Result<ApssConfig, ConfigError> {
    let path_buf = std::path::PathBuf::from(path);
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadError {
        path: path_buf.clone(),
        source: e,
    })?;
    let root: serde_yaml::Value =
        serde_yaml::from_str(&content).map_err(|e| ConfigError::ParseError {
            path: path_buf.clone(),
            source: e,
        })?;
    let docs_value = root.get("docs").cloned().unwrap_or(serde_yaml::Value::Null);
    let docs: DocsConfig =
        serde_yaml::from_value(docs_value).map_err(|e| ConfigError::ParseError {
            path: path_buf,
            source: e,
        })?;
    Ok(ApssConfig { docs })
}

/// Dispatch a documentation command to its implementation.
fn dispatch(command: &str, args: &[String], repo_root: &std::path::Path, verbose: bool) -> i32 {
    match command {
        "--help" | "-h" | "help" => {
            print_help();
            0
        }
        "validate" => validate(args, repo_root, verbose),
        "index" => index(args, repo_root, verbose),
        other => {
            eprintln!("Error: Unknown docs command '{other}'");
            eprintln!("Use 'apss-dev run documentation --help' for available commands.");
            3
        }
    }
}

/// Validate documentation structure, ADRs, and indexes.
fn validate(args: &[String], repo_root: &std::path::Path, _verbose: bool) -> i32 {
    let mut positional_path: Option<&str> = None;
    let mut json_output = false;
    let mut config_path: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_output = true;
                i += 1;
            }
            "--config" if i + 1 < args.len() => {
                config_path = Some(&args[i + 1]);
                i += 2;
            }
            arg if !arg.starts_with('-') && positional_path.is_none() => {
                positional_path = Some(arg);
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    let path = positional_path.unwrap_or(".");
    let target = if std::path::Path::new(path).is_absolute() {
        std::path::PathBuf::from(path)
    } else {
        repo_root.join(path)
    };

    let (validator, docs_config) = if let Some(cfg) = config_path {
        // Resolve relative --config paths against the target directory.
        let cfg_path = if std::path::Path::new(cfg).is_absolute() {
            std::path::PathBuf::from(cfg)
        } else {
            target.join(cfg)
        };
        match load_docs_config(cfg_path.to_str().unwrap_or(cfg)) {
            Ok(config) => {
                let dc = config.docs.clone();
                (crate::DocValidator::with_config(&target, config.docs), dc)
            }
            Err(e) => {
                eprintln!("Error loading config: {e}");
                return 1;
            }
        }
    } else {
        match crate::DocValidator::load(&target) {
            Ok(v) => {
                let dc = v.config().clone();
                (v, dc)
            }
            Err(e) => {
                eprintln!("Error: {e}");
                return 1;
            }
        }
    };

    // `mut` is needed whenever any substandard feature is on (each merges into
    // it). With all three off, no merge happens, so the binding is never
    // mutated: allow that case rather than failing the no-default-features
    // build under `-D warnings`.
    #[cfg_attr(
        not(any(feature = "AD01", feature = "PV01", feature = "RT01")),
        allow(unused_mut)
    )]
    let mut diagnostics = validator.validate();

    // AD01: Architecture Decision Records substandard. Gated behind the AD01
    // cargo feature so a --no-default-features build skips ADR validation
    // rather than failing to compile.
    #[cfg(feature = "AD01")]
    {
        let adr_validator = if config_path.is_some() {
            crate::substandards::adr::AdrValidator::with_config(&target, docs_config.clone())
        } else {
            match crate::substandards::adr::AdrValidator::load(&target) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Error loading ADR validator: {e}");
                    return 1;
                }
            }
        };
        diagnostics.merge(adr_validator.validate());
    }
    // Silence the unused-variable warning when AD01 is disabled; the config is
    // only consumed by the ADR validator.
    let _ = &docs_config;

    // PV01: Purpose and Vision (North Star) substandard. Scaffold returns an
    // empty diagnostic set today; wired in so the full doc type registry is
    // honoured as soon as the validator body lands.
    #[cfg(feature = "PV01")]
    {
        diagnostics.merge(crate::substandards::purpose_and_vision::validate(&target));
    }

    // RT01: Retrospectives substandard. Same scaffold pattern as PV01.
    #[cfg(feature = "RT01")]
    {
        diagnostics.merge(crate::substandards::retrospectives::validate(&target));
    }

    if json_output {
        match diagnostics.to_json() {
            Ok(json) => println!("{json}"),
            Err(e) => {
                eprintln!("Error serializing JSON: {e}");
                return 1;
            }
        }
    } else if diagnostics.is_empty() {
        println!("Documentation validation passed.");
    } else {
        println!("{diagnostics}");
    }

    // Per the APS-V1-0003 install contract (spec section 9.2), warnings MUST
    // NOT block. Here we only fail on errors.
    if diagnostics.has_errors() { 1 } else { 0 }
}

/// Generate or check README indexes from front matter.
fn index(args: &[String], repo_root: &std::path::Path, _verbose: bool) -> i32 {
    let mut positional_path: Option<&str> = None;
    let mut write_mode = false;
    let mut config_path: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--write" => {
                write_mode = true;
                i += 1;
            }
            "--config" if i + 1 < args.len() => {
                config_path = Some(&args[i + 1]);
                i += 2;
            }
            arg if !arg.starts_with('-') && positional_path.is_none() => {
                positional_path = Some(arg);
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    let path = positional_path.unwrap_or(".");
    let target = if std::path::Path::new(path).is_absolute() {
        std::path::PathBuf::from(path)
    } else {
        repo_root.join(path)
    };

    let validator = if let Some(cfg) = config_path {
        let cfg_path = if std::path::Path::new(cfg).is_absolute() {
            std::path::PathBuf::from(cfg)
        } else {
            target.join(cfg)
        };
        match load_docs_config(cfg_path.to_str().unwrap_or(cfg)) {
            Ok(config) => crate::DocValidator::with_config(&target, config.docs),
            Err(e) => {
                eprintln!("Error loading config: {e}");
                return 1;
            }
        }
    } else {
        match crate::DocValidator::load(&target) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error: {e}");
                return 1;
            }
        }
    };

    if write_mode {
        match validator.write_indexes() {
            Ok(count) => {
                println!("Updated {count} README.md file(s) with generated indexes.");
                0
            }
            Err(e) => {
                eprintln!("Error writing indexes: {e}");
                1
            }
        }
    } else {
        match validator.generate_indexes() {
            Ok(indexes) => {
                if indexes.is_empty() {
                    // `generate_indexes` returns one entry per traversed
                    // directory, so empty means the docs root itself was not
                    // found (or was empty). Say that, instead of implying there
                    // was no frontmatter to read.
                    println!(
                        "Docs root not found or contains no directories under {}.",
                        target.display(),
                    );
                    println!("Create the docs root or configure docs.root in apss.yaml.");
                } else {
                    for idx in &indexes {
                        println!("--- {} ---", idx.dir.display());
                        println!("{}", idx.markdown);
                    }
                    println!("{} directory index(es) generated (dry run).", indexes.len());
                    println!("Use --write to update README.md files.");
                }
                0
            }
            Err(e) => {
                eprintln!("Error generating indexes: {e}");
                1
            }
        }
    }
}

/// Print the documentation CLI help text.
fn print_help() {
    println!("{} ({}) v{}", crate::NAME, crate::ID, crate::VERSION);
    println!();
    println!("USAGE:");
    println!("    apss-dev run documentation <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    validate [path]    Validate documentation structure, ADRs, and indexes");
    println!("    index [path]       Generate or check README indexes from front matter");
    println!();
    println!("OPTIONS:");
    println!("    --config <file>    Path to apss.yaml (default: <path>/apss.yaml)");
    println!("    --write            Write generated indexes into README.md files (index only)");
    println!("    --json             Output validation results as JSON (validate only)");
    println!("    --help             Show this help message");
}

/// The command list returned by `commands()` and used by `register()`.
fn command_infos() -> Vec<CommandInfo> {
    vec![
        CommandInfo {
            name: "validate".to_string(),
            description: "Validate documentation structure, ADRs, and indexes".to_string(),
            usage: "validate [path]".to_string(),
        },
        CommandInfo {
            name: "index".to_string(),
            description: "Generate or check README indexes from front matter".to_string(),
            usage: "index [path]".to_string(),
        },
    ]
}

/// Command names registered by `register()`; kept in sync with [`command_infos`].
pub(crate) const COMMAND_NAMES: [&str; 2] = ["validate", "index"];
