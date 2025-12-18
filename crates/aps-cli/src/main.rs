//! APS CLI
//!
//! Command-line interface for APS validation and scaffolding.
//!
//! # Usage
//!
//! ```bash
//! # Run a standard's CLI
//! aps run topology analyze .
//! aps run topology validate .topology/
//! aps run --list
//!
//! # Validate the entire V1 repo structure
//! aps v1 validate repo
//!
//! # Validate a specific standard
//! aps v1 validate standard APS-V1-0000
//!
//! # Create a new standard
//! aps v1 create standard my-new-standard
//!
//! # List all packages
//! aps v1 list
//! ```

use aps_core::discovery::{PackageType, count_packages, discover_v1_packages, find_package_by_id};
use aps_core::versioning::BumpPart;
use aps_core::{
    StandardContext, TemplateEngine, bump_version, generate_all_views, get_version,
    promote_experiment,
};
use aps_v1_0000_meta::{MetaStandard, Standard};
use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "aps")]
#[command(version, about = "Agent Paradise Standards System CLI")]
#[command(propagate_version = true)]
#[command(after_help = "Use 'aps v1 --help' for V1 standards operations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: human (default) or json
    #[arg(long, default_value = "human", global = true)]
    format: OutputFormat,

    /// Enable/disable colors (auto-detected by default)
    #[arg(long, global = true)]
    color: Option<bool>,

    /// Enable verbose output for debugging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Clone, Copy, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Human,
    Json,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Run a standard's CLI commands
    Run {
        /// Standard slug or ID (e.g., "topology", "EXP-V1-0001")
        #[arg(required_unless_present = "list")]
        standard: Option<String>,

        /// Command and arguments for the standard
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,

        /// List available standards
        #[arg(long)]
        list: bool,
    },

    /// V1 standards operations (authoring)
    V1 {
        #[command(subcommand)]
        command: V1Commands,
    },
}

#[derive(clap::Subcommand)]
enum V1Commands {
    /// Validate standards, substandards, or experiments
    Validate {
        #[command(subcommand)]
        target: ValidateTarget,
    },
    /// Create new standards, substandards, or experiments
    Create {
        #[command(subcommand)]
        target: CreateTarget,
    },
    /// Promote an experiment to an official standard
    Promote {
        /// Experiment ID to promote (e.g., EXP-V1-0001)
        experiment_id: String,
        /// Optional target standard ID (otherwise auto-allocated)
        #[arg(long)]
        target_id: Option<String>,
    },
    /// Generate derived views (registry.json, INDEX.md)
    Generate {
        #[command(subcommand)]
        target: GenerateTarget,
    },
    /// Bump version of a standard, substandard, or experiment
    Version {
        #[command(subcommand)]
        action: VersionAction,
    },
    /// List all V1 packages
    List,
}

#[derive(clap::Subcommand)]
enum GenerateTarget {
    /// Generate all derived views
    Views,
}

#[derive(clap::Subcommand)]
enum VersionAction {
    /// Bump version (major, minor, or patch)
    Bump {
        /// Package ID to version
        id: String,
        /// Version part to bump: major, minor, or patch
        #[arg(value_enum)]
        part: VersionPart,
    },
    /// Show current version of a package
    Show {
        /// Package ID
        id: String,
    },
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum VersionPart {
    Major,
    Minor,
    Patch,
}

#[derive(clap::Subcommand)]
enum ValidateTarget {
    /// Validate the entire repository structure
    Repo,
    /// Validate a specific standard by ID
    Standard {
        /// Standard ID (e.g., APS-V1-0000)
        id: String,
    },
    /// Validate a specific substandard by ID
    Substandard {
        /// Substandard ID (e.g., APS-V1-0002.GH01)
        id: String,
    },
    /// Validate a specific experiment by ID
    Experiment {
        /// Experiment ID (e.g., EXP-V1-0001)
        id: String,
    },
}

#[derive(clap::Subcommand)]
enum CreateTarget {
    /// Create a new standard
    Standard {
        /// Slug for the new standard (kebab-case)
        slug: String,
        /// Human-readable name
        #[arg(long)]
        name: Option<String>,
    },
    /// Create a new substandard
    Substandard {
        /// Parent standard ID
        parent_id: String,
        /// Profile identifier (e.g., GH01)
        profile: String,
    },
    /// Create a new experiment
    Experiment {
        /// Slug for the new experiment (kebab-case)
        slug: String,
        /// Human-readable name
        #[arg(long)]
        name: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Determine repo root (current directory or CARGO_MANIFEST_DIR for development)
    let repo_root = find_repo_root().unwrap_or_else(|| {
        eprintln!("Error: Could not find repository root");
        std::process::exit(1);
    });

    match cli.command {
        Commands::Run {
            standard,
            args,
            list,
        } => {
            if list {
                // List available standards
                println!("Available Standards:\n");
                println!("  topology (EXP-V1-0001) v0.1.0");
                println!("    Code Topology - architectural metrics and visualization");
                println!("    Commands: analyze, validate, diff, report");
                println!();
                println!("Use 'aps run <slug> --help' for command details.");
                return ExitCode::SUCCESS;
            }

            let slug = standard.unwrap_or_default();
            if slug.is_empty() {
                eprintln!(
                    "Error: Standard slug required. Use 'aps run --list' to see available standards."
                );
                return ExitCode::FAILURE;
            }

            // Dispatch to standard CLI
            match resolve_standard(&slug) {
                Some(info) => dispatch_standard_cli(&info, &args, &repo_root, cli.verbose),
                None => {
                    eprintln!("Error: Unknown standard '{slug}'");
                    eprintln!("Use 'aps run --list' to see available standards.");
                    ExitCode::FAILURE
                }
            }
        }

        Commands::V1 { command } => match command {
            V1Commands::Validate { target } => {
                let meta = MetaStandard::new();
                let diagnostics = match target {
                    ValidateTarget::Repo => {
                        println!("Validating V1 repository at: {}", repo_root.display());
                        meta.validate_repo(&repo_root)
                    }
                    ValidateTarget::Standard { id } => {
                        if let Some(pkg) = find_package_by_id(&repo_root, &id) {
                            println!("Validating standard: {} at {}", id, pkg.path.display());
                            meta.validate_package(&pkg.path)
                        } else {
                            eprintln!("Error: Standard '{id}' not found");
                            return ExitCode::FAILURE;
                        }
                    }
                    ValidateTarget::Substandard { id } => {
                        if let Some(pkg) = find_package_by_id(&repo_root, &id) {
                            println!("Validating substandard: {} at {}", id, pkg.path.display());
                            meta.validate_package(&pkg.path)
                        } else {
                            eprintln!("Error: Substandard '{id}' not found");
                            return ExitCode::FAILURE;
                        }
                    }
                    ValidateTarget::Experiment { id } => {
                        if let Some(pkg) = find_package_by_id(&repo_root, &id) {
                            println!("Validating experiment: {} at {}", id, pkg.path.display());
                            meta.validate_package(&pkg.path)
                        } else {
                            eprintln!("Error: Experiment '{id}' not found");
                            return ExitCode::FAILURE;
                        }
                    }
                };

                // Output results
                match cli.format {
                    OutputFormat::Human => {
                        if diagnostics.is_empty() {
                            println!("\n✓ Validation passed with no issues");
                        } else {
                            println!("\n{diagnostics}");
                        }
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            diagnostics
                                .to_json()
                                .unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
                        );
                    }
                }

                match diagnostics.exit_code() {
                    0 => ExitCode::SUCCESS,
                    _ => ExitCode::FAILURE,
                }
            }
            V1Commands::Create { target } => match target {
                CreateTarget::Standard { slug, name } => {
                    let name = name.unwrap_or_else(|| slug_to_name(&slug));
                    let id = allocate_next_standard_id(&repo_root);

                    println!("Creating new standard:");
                    println!("  ID:   {id}");
                    println!("  Name: {name}");
                    println!("  Slug: {slug}");

                    let output_dir = repo_root.join(format!("standards/v1/{id}-{slug}"));

                    if output_dir.exists() {
                        eprintln!("Error: Directory already exists: {}", output_dir.display());
                        return ExitCode::FAILURE;
                    }

                    let engine = TemplateEngine::new();
                    let context = StandardContext::new(&id, &name, &slug);

                    // Find the template skeleton
                    let skeleton_dir =
                        repo_root.join("standards/v1/APS-V1-0000-meta/templates/standard/skeleton");

                    match engine.render_skeleton(&skeleton_dir, &output_dir, &context) {
                        Ok(files) => {
                            println!("\n✓ Created {} files:", files.len());
                            for file in &files {
                                if let Ok(rel) = file.strip_prefix(&repo_root) {
                                    println!("  {}", rel.display());
                                }
                            }
                            println!(
                                "\nNext steps:\n  1. Add to Cargo.toml workspace members\n  2. Implement the Standard trait\n  3. Run: aps v1 validate standard {id}"
                            );
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("Error creating standard: {e}");
                            ExitCode::FAILURE
                        }
                    }
                }
                CreateTarget::Substandard { parent_id, profile } => {
                    // Find the parent standard
                    let parent = find_package_by_id(&repo_root, &parent_id);
                    if parent.is_none() {
                        eprintln!("Error: Parent standard '{parent_id}' not found");
                        return ExitCode::FAILURE;
                    }
                    let parent = parent.unwrap();

                    let id = format!("{parent_id}.{profile}");
                    let name = format!("{profile} Profile");
                    let slug = format!(
                        "{}-{}",
                        parent_id.to_lowercase().replace('-', "_"),
                        profile.to_lowercase()
                    );

                    println!("Creating new substandard:");
                    println!("  ID:     {id}");
                    println!("  Name:   {name}");
                    println!("  Parent: {parent_id}");

                    let output_dir = parent.path.join("substandards").join(&slug);

                    if output_dir.exists() {
                        eprintln!("Error: Directory already exists: {}", output_dir.display());
                        return ExitCode::FAILURE;
                    }

                    let engine = TemplateEngine::new();
                    let context = aps_core::SubstandardContext::new(&id, &name, &slug, &parent_id);

                    let skeleton_dir = repo_root
                        .join("standards/v1/APS-V1-0000-meta/templates/substandard/skeleton");

                    match engine.render_skeleton(&skeleton_dir, &output_dir, &context) {
                        Ok(files) => {
                            println!("\n✓ Created {} files:", files.len());
                            for file in &files {
                                if let Ok(rel) = file.strip_prefix(&repo_root) {
                                    println!("  {}", rel.display());
                                }
                            }
                            println!(
                                "\nNext steps:\n  1. Add to Cargo.toml workspace members\n  2. Implement the profile-specific logic\n  3. Run: aps v1 validate substandard {id}"
                            );
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("Error creating substandard: {e}");
                            ExitCode::FAILURE
                        }
                    }
                }
                CreateTarget::Experiment { slug, name } => {
                    let name = name.unwrap_or_else(|| slug_to_name(&slug));
                    let id = allocate_next_experiment_id(&repo_root);

                    println!("Creating new experiment:");
                    println!("  ID:   {id}");
                    println!("  Name: {name}");
                    println!("  Slug: {slug}");

                    let output_dir =
                        repo_root.join(format!("standards-experimental/v1/{id}-{slug}"));

                    if output_dir.exists() {
                        eprintln!("Error: Directory already exists: {}", output_dir.display());
                        return ExitCode::FAILURE;
                    }

                    let engine = TemplateEngine::new();
                    let context = aps_core::ExperimentContext::new(&id, &name, &slug);

                    let skeleton_dir = repo_root
                        .join("standards/v1/APS-V1-0000-meta/templates/experiment/skeleton");

                    match engine.render_skeleton(&skeleton_dir, &output_dir, &context) {
                        Ok(files) => {
                            println!("\n✓ Created {} files:", files.len());
                            for file in &files {
                                if let Ok(rel) = file.strip_prefix(&repo_root) {
                                    println!("  {}", rel.display());
                                }
                            }
                            println!(
                                "\nNext steps:\n  1. Add to Cargo.toml workspace members\n  2. Iterate on the experiment\n  3. When ready, use: aps v1 promote {id}"
                            );
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("Error creating experiment: {e}");
                            ExitCode::FAILURE
                        }
                    }
                }
            },
            V1Commands::Promote {
                experiment_id,
                target_id,
            } => {
                println!("Promoting experiment: {experiment_id}");

                match promote_experiment(&repo_root, &experiment_id, target_id.as_deref()) {
                    Ok(result) => {
                        println!("\n✓ Promotion successful!");
                        println!("  From: {}", result.experiment_id);
                        println!("  To:   {}", result.standard_id);
                        println!("  Path: {}", result.new_path.display());
                        println!("\n  Migrated {} files", result.migrated_files.len());
                        println!("\nNext steps:");
                        println!("  1. Add to Cargo.toml workspace members");
                        println!("  2. Remove the old experiment from workspace");
                        println!("  3. Run: aps v1 validate standard {}", result.standard_id);
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("Error promoting experiment: {e}");
                        ExitCode::FAILURE
                    }
                }
            }
            V1Commands::Generate { target } => match target {
                GenerateTarget::Views => {
                    println!("Generating derived views...");

                    match generate_all_views(&repo_root) {
                        Ok(files) => {
                            println!("\n✓ Generated {} files:", files.len());
                            for file in &files {
                                if let Ok(rel) = file.strip_prefix(&repo_root) {
                                    println!("  {}", rel.display());
                                }
                            }
                            println!(
                                "\nNote: These files are derived views, not authoritative.\nThe filesystem is the source of truth."
                            );
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("Error generating views: {e}");
                            ExitCode::FAILURE
                        }
                    }
                }
            },
            V1Commands::Version { action } => match action {
                VersionAction::Show { id } => match get_version(&repo_root, &id) {
                    Ok(version) => {
                        println!("{id}: {version}");
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        ExitCode::FAILURE
                    }
                },
                VersionAction::Bump { id, part } => {
                    let bump_part = match part {
                        VersionPart::Major => BumpPart::Major,
                        VersionPart::Minor => BumpPart::Minor,
                        VersionPart::Patch => BumpPart::Patch,
                    };

                    match bump_version(&repo_root, &id, bump_part) {
                        Ok(result) => {
                            println!("✓ Version bumped:");
                            println!("  Package: {}", result.id);
                            println!("  {} → {}", result.old_version, result.new_version);
                            ExitCode::SUCCESS
                        }
                        Err(e) => {
                            eprintln!("Error bumping version: {e}");
                            ExitCode::FAILURE
                        }
                    }
                }
            },
            V1Commands::List => {
                let packages = discover_v1_packages(&repo_root);
                let (standards, substandards, experiments) = count_packages(&repo_root);

                println!("V1 Packages ({} total):", packages.len());
                println!("  Standards:    {standards}");
                println!("  Substandards: {substandards}");
                println!("  Experiments:  {experiments}");
                println!();

                if !packages.is_empty() {
                    println!("Packages:");
                    for pkg in &packages {
                        let type_label = match pkg.package_type {
                            PackageType::Standard => "standard",
                            PackageType::Substandard => "substandard",
                            PackageType::Experiment => "experiment",
                        };
                        let name = pkg
                            .path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");
                        println!("  [{type_label:^11}] {name}");
                    }
                }

                ExitCode::SUCCESS
            }
        },
    }
}

/// Find the repository root by looking for Cargo.toml with workspace config.
fn find_repo_root() -> Option<PathBuf> {
    // First try current directory
    let cwd = env::current_dir().ok()?;

    // Walk up looking for a Cargo.toml with [workspace]
    let mut current = cwd.as_path();
    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    return Some(current.to_path_buf());
                }
            }
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    // Fallback to current directory
    Some(cwd)
}

/// Convert a slug to a human-readable name.
fn slug_to_name(slug: &str) -> String {
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Allocate the next available standard ID.
fn allocate_next_standard_id(repo_root: &std::path::Path) -> String {
    let packages = discover_v1_packages(repo_root);

    let max_id = packages
        .iter()
        .filter(|p| p.package_type == PackageType::Standard)
        .filter_map(|p| {
            p.path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|name| {
                    // Parse "APS-V1-XXXX-slug" to extract XXXX
                    if name.starts_with("APS-V1-") {
                        name[7..11].parse::<u32>().ok()
                    } else {
                        None
                    }
                })
        })
        .max()
        .unwrap_or(0);

    format!("APS-V1-{:04}", max_id + 1)
}

/// Allocate the next available experiment ID.
fn allocate_next_experiment_id(repo_root: &std::path::Path) -> String {
    let packages = discover_v1_packages(repo_root);

    let max_id = packages
        .iter()
        .filter(|p| p.package_type == PackageType::Experiment)
        .filter_map(|p| {
            p.path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|name| {
                    // Parse "EXP-V1-XXXX-slug" to extract XXXX
                    if name.starts_with("EXP-V1-") {
                        name[7..11].parse::<u32>().ok()
                    } else {
                        None
                    }
                })
        })
        .max()
        .unwrap_or(0);

    format!("EXP-V1-{:04}", max_id + 1)
}

// ============================================================================
// Standard CLI Dispatch
// ============================================================================

/// Information about a registered standard.
#[allow(dead_code)]
struct StandardCliInfo {
    id: &'static str,
    slug: &'static str,
    name: &'static str,
    version: &'static str,
}

/// Resolve a slug to standard info.
fn resolve_standard(slug: &str) -> Option<StandardCliInfo> {
    match slug.to_lowercase().as_str() {
        "topology" | "topo" | "code-topology" | "exp-v1-0001" => Some(StandardCliInfo {
            id: "EXP-V1-0001",
            slug: "topology",
            name: "Code Topology",
            version: "0.1.0",
        }),
        _ => None,
    }
}

/// Dispatch to a standard's CLI.
fn dispatch_standard_cli(
    info: &StandardCliInfo,
    args: &[String],
    repo_root: &std::path::Path,
    verbose: bool,
) -> ExitCode {
    let command = args.first().map(|s| s.as_str()).unwrap_or("--help");
    let cmd_args = if args.len() > 1 { &args[1..] } else { &[] };

    match info.slug {
        "topology" => dispatch_topology(command, cmd_args, repo_root, verbose),
        _ => {
            eprintln!("Error: Standard '{}' CLI not implemented", info.slug);
            ExitCode::FAILURE
        }
    }
}

/// Dispatch topology commands.
fn dispatch_topology(
    command: &str,
    args: &[String],
    repo_root: &std::path::Path,
    verbose: bool,
) -> ExitCode {
    match command {
        "--help" | "-h" | "help" => {
            println!("Code Topology (EXP-V1-0001) v0.1.0");
            println!();
            println!("USAGE:");
            println!("    aps run topology <COMMAND> [OPTIONS]");
            println!();
            println!("COMMANDS:");
            println!("    analyze <path>     Analyze codebase and generate .topology/");
            println!("    validate <path>    Validate existing .topology/ artifacts");
            println!("    diff <a> <b>       Compare two topology snapshots");
            println!("    check <diff.json>  Check diff against thresholds");
            println!("    comment <diff.json> Generate PR comment markdown");
            println!("    report <path>      Generate human-readable report");
            println!();
            println!("OPTIONS:");
            println!("    --output <dir>     Output directory (default: .topology)");
            println!(
                "    --language <lang>  Filter by language: rust, python (default: auto-detect)"
            );
            println!("    --format <fmt>     Output format: json, text (default: text)");
            println!("    --config <file>    Config file for thresholds");
            println!("    --help             Show this help message");
            println!();
            println!("SUPPORTED LANGUAGES:");
            println!("    rust       .rs");
            println!("    python     .py, .pyi");
            ExitCode::SUCCESS
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

            topology_analyze(path, output, language, repo_root, verbose)
        }
        "validate" => {
            let path = args.first().map(|s| s.as_str()).unwrap_or(".topology");
            topology_validate(path, verbose)
        }
        "diff" => {
            if args.len() < 2 {
                eprintln!("Error: diff requires two paths");
                eprintln!("Usage: aps run topology diff <base> <target> [--format json]");
                return ExitCode::FAILURE;
            }
            let format = args
                .iter()
                .position(|a| a == "--format")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str())
                .unwrap_or("text");
            topology_diff(&args[0], &args[1], format, verbose)
        }
        "check" => {
            let diff_file = args.first().map(|s| s.as_str());
            let config = args
                .iter()
                .position(|a| a == "--config")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str());
            topology_check(diff_file, config, verbose)
        }
        "comment" => {
            let diff_file = args.first().map(|s| s.as_str());
            let config = args
                .iter()
                .position(|a| a == "--config")
                .and_then(|i| args.get(i + 1))
                .map(|s| s.as_str());
            topology_comment(diff_file, config, verbose)
        }
        "report" => {
            let path = args.first().map(|s| s.as_str()).unwrap_or(".topology");
            topology_report(path, verbose)
        }
        _ => {
            eprintln!("Error: Unknown topology command '{command}'");
            eprintln!("Use 'aps run topology --help' for available commands.");
            ExitCode::FAILURE
        }
    }
}

/// Analyze a codebase and generate .topology/ artifacts.
fn topology_analyze(
    path: &str,
    output: &str,
    language_filter: Option<&str>,
    _repo_root: &std::path::Path,
    verbose: bool,
) -> ExitCode {
    use code_topology::LanguageAdapter;
    use code_topology::adapter::grammars::{PythonGrammar, RustGrammar};
    use code_topology::adapter::{GrammarRegistry, TreeSitterAdapter};
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use walkdir::WalkDir;

    let project_path = Path::new(path);
    let output_path = Path::new(output);

    if verbose {
        println!("Analyzing: {}", project_path.display());
        println!("Output:    {}", output_path.display());
        if let Some(lang) = language_filter {
            println!("Language:  {lang}");
        }
    }

    // Create grammar registry with available grammars
    let mut registry = GrammarRegistry::new();
    registry.register(Box::new(RustGrammar::new()));
    registry.register(Box::new(PythonGrammar::new()));

    let adapter = TreeSitterAdapter::new(registry);

    // Collect files to analyze
    let mut files_by_lang: HashMap<String, Vec<std::path::PathBuf>> = HashMap::new();

    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            // Allow the root entry even if it's "."
            if e.depth() == 0 {
                return true;
            }
            // Skip hidden dirs and common non-source dirs
            !name.starts_with('.')
                && name != "target"
                && name != "node_modules"
                && name != "__pycache__"
                && name != "venv"
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let file_path = entry.path();

        // Check if we have a grammar for this file
        if let Some(grammar) = adapter.registry().get_for_path(file_path) {
            let lang = grammar.language_id();

            // Apply language filter if specified
            if let Some(filter) = language_filter {
                if lang != filter {
                    continue;
                }
            }

            files_by_lang
                .entry(lang.to_string())
                .or_default()
                .push(file_path.to_path_buf());
        }
    }

    if files_by_lang.is_empty() {
        let msg = if let Some(lang) = language_filter {
            format!("No {lang} files found in {}", project_path.display())
        } else {
            format!(
                "No supported source files found in {}",
                project_path.display()
            )
        };
        eprintln!("Error: {msg}");
        eprintln!("Supported: .rs (Rust), .py/.pyi (Python)");
        return ExitCode::FAILURE;
    }

    // Print summary
    let total_files: usize = files_by_lang.values().map(|v| v.len()).sum();
    println!("Found {total_files} source file(s):");
    for (lang, files) in &files_by_lang {
        println!("  {lang}: {} files", files.len());
    }

    // Analyze all files - extract functions AND imports
    let mut all_functions = Vec::new();
    let mut all_imports: Vec<code_topology::ImportInfo> = Vec::new();
    let mut errors = 0;

    for (lang, files) in &files_by_lang {
        if verbose {
            println!("Analyzing {lang} files...");
        }

        for file_path in files {
            let source = match fs::read_to_string(file_path) {
                Ok(s) => s,
                Err(e) => {
                    if verbose {
                        eprintln!("  Warning: Could not read {}: {e}", file_path.display());
                    }
                    errors += 1;
                    continue;
                }
            };

            // Extract imports for coupling analysis
            if let Ok(imports) = adapter.extract_imports(&source, file_path) {
                all_imports.extend(imports);
            }

            match adapter.extract_functions(&source, file_path) {
                Ok(functions) => {
                    for func in functions {
                        // Compute metrics for each function
                        match adapter.compute_metrics(&source, &func) {
                            Ok(metrics) => {
                                all_functions.push((func, metrics));
                            }
                            Err(e) => {
                                if verbose {
                                    eprintln!(
                                        "  Warning: Could not compute metrics for {}: {e}",
                                        func.name
                                    );
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    if verbose {
                        eprintln!("  Warning: Could not parse {}: {e}", file_path.display());
                    }
                    errors += 1;
                }
            }
        }
    }

    println!(
        "✓ Analyzed {} functions ({}  warnings)",
        all_functions.len(),
        errors
    );

    // Write artifacts
    if let Err(e) =
        write_topology_artifacts(output_path, &all_functions, &all_imports, &files_by_lang)
    {
        eprintln!("Error writing artifacts: {e}");
        return ExitCode::FAILURE;
    }

    println!("✓ Wrote artifacts to {}", output_path.display());
    ExitCode::SUCCESS
}

/// Write topology artifacts to disk.
fn write_topology_artifacts(
    output_path: &std::path::Path,
    functions: &[(code_topology::FunctionInfo, code_topology::FunctionMetrics)],
    imports: &[code_topology::ImportInfo],
    files_by_lang: &std::collections::HashMap<String, Vec<std::path::PathBuf>>,
) -> std::io::Result<()> {
    use std::collections::{HashMap, HashSet};
    use std::fs;

    // Create directories
    fs::create_dir_all(output_path)?;
    fs::create_dir_all(output_path.join("metrics"))?;
    fs::create_dir_all(output_path.join("graphs"))?;

    // Group functions by module
    let mut modules: HashMap<
        String,
        Vec<&(code_topology::FunctionInfo, code_topology::FunctionMetrics)>,
    > = HashMap::new();
    for func_with_metrics in functions {
        modules
            .entry(func_with_metrics.0.module.clone())
            .or_default()
            .push(func_with_metrics);
    }

    // Build dependency graph from imports
    // Map module -> set of modules it depends on (efferent coupling)
    let mut efferent: HashMap<String, HashSet<String>> = HashMap::new();
    // Map module -> set of modules that depend on it (afferent coupling)
    let mut afferent: HashMap<String, HashSet<String>> = HashMap::new();

    // Initialize all modules
    for module in modules.keys() {
        efferent.entry(module.clone()).or_default();
        afferent.entry(module.clone()).or_default();
    }

    // Process imports to build coupling
    for import in imports {
        let from_module = &import.from_module;

        // Skip external imports
        if import.is_external {
            continue;
        }

        // Try to resolve the import path to a known module
        let import_path = &import.import_path;

        // Find which module this import refers to
        for to_module in modules.keys() {
            // Check if the import path matches or is contained in the module
            if import_path.contains(to_module.split("::").last().unwrap_or(to_module))
                || to_module.contains(import_path)
                || import_path
                    .split("::")
                    .any(|part| to_module.contains(part))
            {
                if from_module != to_module {
                    // from_module depends on to_module
                    efferent
                        .entry(from_module.clone())
                        .or_default()
                        .insert(to_module.clone());
                    // to_module is depended upon by from_module
                    afferent
                        .entry(to_module.clone())
                        .or_default()
                        .insert(from_module.clone());
                }
            }
        }
    }

    // Write manifest.toml
    let languages: Vec<&str> = files_by_lang.keys().map(|s| s.as_str()).collect();
    let total_files: usize = files_by_lang.values().map(|v| v.len()).sum();
    let total_deps: usize = efferent.values().map(|s| s.len()).sum();
    let manifest = format!(
        r#"[topology]
version = "0.1.0"
generated_at = "{}"
generator = "aps-cli"
generator_version = "0.1.0"

[analysis]
root = "."
languages = {:?}
total_files = {}
total_functions = {}
total_modules = {}
total_dependencies = {}
"#,
        chrono_lite_now(),
        languages,
        total_files,
        functions.len(),
        modules.len(),
        total_deps
    );
    fs::write(output_path.join("manifest.toml"), manifest)?;

    // Write functions.json
    let functions_json = serde_json::json!({
        "schema_version": "1.0.0",
        "functions": functions.iter().map(|(func, metrics)| {
            serde_json::json!({
                "id": func.qualified_name,
                "name": func.name,
                "module": func.module,
                "file": func.file_path.to_string_lossy(),
                "line": func.start_line,
                "metrics": {
                    "cyclomatic": metrics.cyclomatic_complexity,
                    "cognitive": metrics.cognitive_complexity,
                    "halstead": {
                        "vocabulary": metrics.halstead.vocabulary,
                        "length": metrics.halstead.length,
                        "volume": metrics.halstead.volume,
                        "difficulty": metrics.halstead.difficulty,
                        "effort": metrics.halstead.effort
                    },
                    "loc": metrics.total_lines
                }
            })
        }).collect::<Vec<_>>()
    });
    fs::write(
        output_path.join("metrics/functions.json"),
        serde_json::to_string_pretty(&functions_json).unwrap(),
    )?;

    // Write modules.json with real Martin metrics
    let modules_json = serde_json::json!({
        "schema_version": "1.0.0",
        "modules": modules.iter().map(|(module_id, funcs)| {
            let total_cc: u32 = funcs.iter().map(|(_, m)| m.cyclomatic_complexity).sum();
            let total_cog: u32 = funcs.iter().map(|(_, m)| m.cognitive_complexity).sum();
            let total_loc: u32 = funcs.iter().map(|(_, m)| m.total_lines).sum();
            let count = funcs.len() as f64;

            // Unique files
            let unique_files: HashSet<_> = funcs.iter()
                .map(|(f, _)| f.file_path.clone())
                .collect();

            // Martin metrics
            let ca = afferent.get(module_id).map(|s| s.len()).unwrap_or(0) as u32;
            let ce = efferent.get(module_id).map(|s| s.len()).unwrap_or(0) as u32;
            let instability = if ca + ce > 0 {
                ce as f64 / (ca + ce) as f64
            } else {
                0.5 // Default when no coupling
            };
            let abstractness = 0.0; // TODO: Would need type analysis
            let distance = (instability + abstractness - 1.0).abs();

            serde_json::json!({
                "id": module_id,
                "name": module_id.split("::").last().unwrap_or(module_id),
                "path": format!("{}/", module_id.replace("::", "/")),
                "languages": languages,
                "metrics": {
                    "file_count": unique_files.len(),
                    "function_count": funcs.len(),
                    "total_cyclomatic": total_cc,
                    "avg_cyclomatic": if count > 0.0 { total_cc as f64 / count } else { 0.0 },
                    "total_cognitive": total_cog,
                    "avg_cognitive": if count > 0.0 { total_cog as f64 / count } else { 0.0 },
                    "lines_of_code": total_loc,
                    "martin": {
                        "ca": ca,
                        "ce": ce,
                        "instability": instability,
                        "abstractness": abstractness,
                        "distance_from_main_sequence": distance
                    }
                }
            })
        }).collect::<Vec<_>>()
    });
    fs::write(
        output_path.join("metrics/modules.json"),
        serde_json::to_string_pretty(&modules_json).unwrap(),
    )?;

    // Build coupling matrix from import dependencies
    let module_names: Vec<&str> = modules.keys().map(|s| s.as_str()).collect();
    let n = module_names.len();
    let mut matrix = vec![vec![0.0; n]; n];

    // Create index map
    let module_index: HashMap<&str, usize> = module_names
        .iter()
        .enumerate()
        .map(|(i, &name)| (name, i))
        .collect();

    // Fill matrix with coupling values
    for (i, row) in matrix.iter_mut().enumerate() {
        row[i] = 1.0; // Self-coupling is 1.0
    }

    // Add actual coupling from dependencies
    for (from, deps) in &efferent {
        if let Some(&from_idx) = module_index.get(from.as_str()) {
            for to in deps {
                if let Some(&to_idx) = module_index.get(to.as_str()) {
                    // Coupling strength of 0.5 for each import dependency
                    matrix[from_idx][to_idx] = 0.5;
                    matrix[to_idx][from_idx] = 0.5; // Symmetric
                }
            }
        }
    }

    let coupling_json = serde_json::json!({
        "schema_version": "1.0.0",
        "metric": "import_coupling",
        "description": "Normalized coupling strength between modules (0-1)",
        "modules": module_names,
        "matrix": matrix
    });
    fs::write(
        output_path.join("graphs/coupling-matrix.json"),
        serde_json::to_string_pretty(&coupling_json).unwrap(),
    )?;

    Ok(())
}

/// Validate existing .topology/ artifacts.
fn topology_validate(path: &str, _verbose: bool) -> ExitCode {
    use std::path::Path;

    let topology_path = Path::new(path);

    // Check required files exist
    let required = [
        "manifest.toml",
        "metrics/functions.json",
        "metrics/modules.json",
        "graphs/coupling-matrix.json",
    ];

    let mut errors = 0;
    for file in required {
        let file_path = topology_path.join(file);
        if file_path.exists() {
            println!("✓ {file}");
        } else {
            println!("✗ {file} (missing)");
            errors += 1;
        }
    }

    if errors > 0 {
        println!();
        println!("{errors} error(s) found. Run 'aps run topology analyze' to generate artifacts.");
        ExitCode::FAILURE
    } else {
        println!();
        println!("✓ All required artifacts present");
        ExitCode::SUCCESS
    }
}

/// Compare two topology snapshots.
fn topology_diff(base: &str, target: &str, format: &str, _verbose: bool) -> ExitCode {
    use std::path::Path;

    let base_path = Path::new(base);
    let target_path = Path::new(target);

    // Check both paths exist
    if !base_path.exists() {
        eprintln!("Error: Base path does not exist: {base}");
        return ExitCode::FAILURE;
    }
    if !target_path.exists() {
        eprintln!("Error: Target path does not exist: {target}");
        return ExitCode::FAILURE;
    }

    // Load metrics from both snapshots
    let base_metrics = load_topology_metrics(base_path);
    let target_metrics = load_topology_metrics(target_path);

    // Compute diff
    let diff = compute_topology_diff(base, target, &base_metrics, &target_metrics);

    if format == "json" {
        // Output JSON format matching proto/diff.proto schema
        match serde_json::to_string_pretty(&diff) {
            Ok(json) => {
                println!("{json}");
                match diff.status.as_str() {
                    "success" => ExitCode::SUCCESS,
                    "error" => ExitCode::FAILURE,
                    _ => ExitCode::from(2), // warning
                }
            }
            Err(e) => {
                eprintln!("Error serializing diff: {e}");
                ExitCode::FAILURE
            }
        }
    } else {
        // Human-readable text format
        println!("Topology Diff: {base} → {target}");
        println!();
        println!(
            "  Functions: {} → {} ({:+})",
            base_metrics.function_count,
            target_metrics.function_count,
            target_metrics.function_count as i64 - base_metrics.function_count as i64
        );
        println!(
            "  Total CC:  {} → {} ({:+})",
            base_metrics.total_cyclomatic,
            target_metrics.total_cyclomatic,
            target_metrics.total_cyclomatic as i64 - base_metrics.total_cyclomatic as i64
        );
        println!(
            "  Avg CC:    {:.1} → {:.1} ({:+.1})",
            base_metrics.avg_cyclomatic,
            target_metrics.avg_cyclomatic,
            target_metrics.avg_cyclomatic - base_metrics.avg_cyclomatic
        );

        if !diff.hotspots.is_empty() {
            println!();
            println!("Hotspots:");
            for hotspot in &diff.hotspots {
                println!("  ⚠ {} - {}", hotspot.id, hotspot.reason);
            }
        }

        println!();
        match diff.status.as_str() {
            "success" => {
                println!("✓ No degradation detected");
                ExitCode::SUCCESS
            }
            "error" => {
                println!("✗ Quality gate failed");
                ExitCode::FAILURE
            }
            _ => {
                println!("⚠ Warnings detected (review recommended)");
                ExitCode::from(2)
            }
        }
    }
}

/// Aggregated topology metrics for comparison.
#[derive(Default)]
struct TopologyMetrics {
    function_count: usize,
    total_cyclomatic: u64,
    avg_cyclomatic: f64,
    total_cognitive: u64,
    avg_cognitive: f64,
    lines_of_code: u64,
}

/// Load topology metrics from a .topology/ directory.
fn load_topology_metrics(path: &std::path::Path) -> TopologyMetrics {
    let mut metrics = TopologyMetrics::default();

    // Load functions.json
    let funcs_path = path.join("metrics/functions.json");
    if let Ok(content) = std::fs::read_to_string(&funcs_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(funcs) = json.get("functions").and_then(|f| f.as_array()) {
                metrics.function_count = funcs.len();

                let mut total_cc = 0u64;
                let mut total_cog = 0u64;
                let mut total_loc = 0u64;

                for func in funcs {
                    if let Some(m) = func.get("metrics") {
                        total_cc += m
                            .get("cyclomatic_complexity")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        total_cog += m
                            .get("cognitive_complexity")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        total_loc += m.get("lines_of_code").and_then(|v| v.as_u64()).unwrap_or(0);
                    }
                }

                metrics.total_cyclomatic = total_cc;
                metrics.total_cognitive = total_cog;
                metrics.lines_of_code = total_loc;

                if metrics.function_count > 0 {
                    metrics.avg_cyclomatic = total_cc as f64 / metrics.function_count as f64;
                    metrics.avg_cognitive = total_cog as f64 / metrics.function_count as f64;
                }
            }
        }
    }

    metrics
}

/// Diff output matching proto/diff.proto schema.
#[derive(serde::Serialize)]
struct TopologyDiff {
    schema_version: String,
    status: String,
    timestamp: String,
    base: DiffRef,
    target: DiffRef,
    summary: DiffSummary,
    metrics: MetricDeltas,
    hotspots: Vec<DiffHotspot>,
    violations: Vec<ThresholdViolation>,
}

#[derive(serde::Serialize)]
struct DiffRef {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    git_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commit: Option<String>,
}

#[derive(serde::Serialize)]
struct DiffSummary {
    functions_added: u32,
    functions_removed: u32,
    functions_modified: u32,
    modules_added: u32,
    modules_removed: u32,
    modules_modified: u32,
}

#[derive(serde::Serialize)]
struct MetricDeltas {
    total_cyclomatic: MetricDelta,
    avg_cyclomatic: MetricDelta,
    total_cognitive: MetricDelta,
    avg_cognitive: MetricDelta,
    lines_of_code: MetricDelta,
    function_count: MetricDelta,
}

#[derive(serde::Serialize)]
struct MetricDelta {
    base: f64,
    target: f64,
    delta: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    percent_change: Option<f64>,
}

impl MetricDelta {
    fn new(base: f64, target: f64) -> Self {
        let delta = target - base;
        let percent_change = if base > 0.0 {
            Some((delta / base) * 100.0)
        } else {
            None
        };
        Self {
            base,
            target,
            delta,
            percent_change,
        }
    }
}

#[derive(serde::Serialize)]
struct DiffHotspot {
    id: String,
    #[serde(rename = "type")]
    hotspot_type: String,
    reason: String,
    severity: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggestion: Option<String>,
}

#[derive(serde::Serialize)]
struct ThresholdViolation {
    threshold: String,
    value: f64,
    limit: f64,
    severity: String,
    message: String,
}

/// Compute a topology diff between two snapshots.
fn compute_topology_diff(
    base_path: &str,
    target_path: &str,
    base: &TopologyMetrics,
    target: &TopologyMetrics,
) -> TopologyDiff {
    let mut hotspots = Vec::new();
    let mut violations = Vec::new();

    // Check for significant complexity increases
    let cc_delta = target.avg_cyclomatic - base.avg_cyclomatic;
    if cc_delta > 2.0 {
        hotspots.push(DiffHotspot {
            id: "aggregate".to_string(),
            hotspot_type: "INCREASED_COMPLEXITY".to_string(),
            reason: format!(
                "Average cyclomatic complexity increased by {:.1} ({:.0}%)",
                cc_delta,
                if base.avg_cyclomatic > 0.0 {
                    (cc_delta / base.avg_cyclomatic) * 100.0
                } else {
                    0.0
                }
            ),
            severity: if cc_delta > 5.0 { 3 } else { 2 },
            suggestion: Some("Review new functions for complexity".to_string()),
        });
    }

    // Determine status based on metrics
    let status = if cc_delta > 5.0 || (target.avg_cyclomatic > 15.0 && cc_delta > 0.0) {
        "error"
    } else if cc_delta > 2.0 || !hotspots.is_empty() {
        "warning"
    } else {
        "success"
    };

    // Add threshold violation if significant
    if cc_delta > 2.0 {
        violations.push(ThresholdViolation {
            threshold: "avg_cyclomatic_delta".to_string(),
            value: cc_delta,
            limit: 2.0,
            severity: if cc_delta > 5.0 {
                "ERROR".to_string()
            } else {
                "WARNING".to_string()
            },
            message: format!(
                "Average cyclomatic complexity increased by {cc_delta:.1}, exceeds threshold"
            ),
        });
    }

    // Compute function changes (simplified - just counts)
    let func_diff = target.function_count as i32 - base.function_count as i32;
    let (added, removed) = if func_diff >= 0 {
        (func_diff as u32, 0)
    } else {
        (0, (-func_diff) as u32)
    };

    TopologyDiff {
        schema_version: "1.0.0".to_string(),
        status: status.to_string(),
        timestamp: chrono_lite_now(),
        base: DiffRef {
            path: base_path.to_string(),
            git_ref: None,
            commit: None,
        },
        target: DiffRef {
            path: target_path.to_string(),
            git_ref: None,
            commit: None,
        },
        summary: DiffSummary {
            functions_added: added,
            functions_removed: removed,
            functions_modified: 0, // Would need function-level tracking
            modules_added: 0,
            modules_removed: 0,
            modules_modified: 0,
        },
        metrics: MetricDeltas {
            total_cyclomatic: MetricDelta::new(
                base.total_cyclomatic as f64,
                target.total_cyclomatic as f64,
            ),
            avg_cyclomatic: MetricDelta::new(base.avg_cyclomatic, target.avg_cyclomatic),
            total_cognitive: MetricDelta::new(
                base.total_cognitive as f64,
                target.total_cognitive as f64,
            ),
            avg_cognitive: MetricDelta::new(base.avg_cognitive, target.avg_cognitive),
            lines_of_code: MetricDelta::new(base.lines_of_code as f64, target.lines_of_code as f64),
            function_count: MetricDelta::new(
                base.function_count as f64,
                target.function_count as f64,
            ),
        },
        hotspots,
        violations,
    }
}

/// Simple timestamp without chrono dependency.
fn chrono_lite_now() -> String {
    // Use a fixed format - in production would use actual time
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Approximate ISO 8601 (good enough for now)
    format!(
        "2025-12-17T{:02}:{:02}:{:02}Z",
        (secs / 3600) % 24,
        (secs / 60) % 60,
        secs % 60
    )
}

/// Check a diff against thresholds.
fn topology_check(diff_file: Option<&str>, config: Option<&str>, _verbose: bool) -> ExitCode {
    let diff_path = match diff_file {
        Some(p) => p,
        None => {
            eprintln!("Error: diff file required");
            eprintln!("Usage: aps run topology check <diff.json> [--config <file>]");
            return ExitCode::FAILURE;
        }
    };

    // Load the diff
    let diff_content = match std::fs::read_to_string(diff_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading diff file: {e}");
            return ExitCode::FAILURE;
        }
    };

    let diff: serde_json::Value = match serde_json::from_str(&diff_content) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error parsing diff JSON: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Load thresholds from config (or use defaults)
    let thresholds = load_thresholds(config);

    // Check violations
    let mut errors = 0;
    let mut warnings = 0;

    // Check avg_cyclomatic delta
    if let Some(delta) = diff
        .get("metrics")
        .and_then(|m| m.get("avg_cyclomatic"))
        .and_then(|d| d.get("delta"))
        .and_then(|v| v.as_f64())
    {
        if delta > thresholds.max_cc_delta_error {
            println!(
                "✗ ERROR: avg_cyclomatic increased by {delta:.1} (limit: {})",
                thresholds.max_cc_delta_error
            );
            errors += 1;
        } else if delta > thresholds.max_cc_delta_warning {
            println!(
                "⚠ WARNING: avg_cyclomatic increased by {delta:.1} (limit: {})",
                thresholds.max_cc_delta_warning
            );
            warnings += 1;
        }
    }

    // Check if any existing violations
    if let Some(violations) = diff.get("violations").and_then(|v| v.as_array()) {
        for v in violations {
            let severity = v
                .get("severity")
                .and_then(|s| s.as_str())
                .unwrap_or("WARNING");
            let message = v
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown violation");
            if severity == "ERROR" {
                println!("✗ ERROR: {message}");
                errors += 1;
            } else {
                println!("⚠ WARNING: {message}");
                warnings += 1;
            }
        }
    }

    // Summary
    println!();
    if errors > 0 {
        println!("✗ Check failed: {errors} error(s), {warnings} warning(s)");
        ExitCode::FAILURE
    } else if warnings > 0 {
        println!("⚠ Check passed with warnings: {warnings} warning(s)");
        ExitCode::from(2)
    } else {
        println!("✓ All checks passed");
        ExitCode::SUCCESS
    }
}

/// Threshold configuration.
struct Thresholds {
    max_cc_delta_warning: f64,
    max_cc_delta_error: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            max_cc_delta_warning: 2.0,
            max_cc_delta_error: 5.0,
        }
    }
}

/// Load thresholds from config file or use defaults.
fn load_thresholds(config: Option<&str>) -> Thresholds {
    if let Some(config_path) = config {
        if let Ok(content) = std::fs::read_to_string(config_path) {
            // Simple TOML parsing for thresholds
            let mut thresholds = Thresholds::default();
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("max_cyclomatic_warning") {
                    if let Some(val) = line.split('=').nth(1) {
                        if let Ok(v) = val.trim().parse::<f64>() {
                            thresholds.max_cc_delta_warning = v;
                        }
                    }
                } else if line.starts_with("max_cyclomatic_failure") {
                    if let Some(val) = line.split('=').nth(1) {
                        if let Ok(v) = val.trim().parse::<f64>() {
                            thresholds.max_cc_delta_error = v;
                        }
                    }
                }
            }
            return thresholds;
        }
    }
    Thresholds::default()
}

/// Generate a PR comment from a diff.
fn topology_comment(diff_file: Option<&str>, _config: Option<&str>, _verbose: bool) -> ExitCode {
    let diff_path = match diff_file {
        Some(p) => p,
        None => {
            eprintln!("Error: diff file required");
            eprintln!("Usage: aps run topology comment <diff.json>");
            return ExitCode::FAILURE;
        }
    };

    // Load the diff
    let diff_content = match std::fs::read_to_string(diff_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading diff file: {e}");
            return ExitCode::FAILURE;
        }
    };

    let diff: serde_json::Value = match serde_json::from_str(&diff_content) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error parsing diff JSON: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Generate markdown comment
    let status = diff
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");
    let status_emoji = match status {
        "success" => "✅",
        "warning" => "⚠️",
        "error" => "❌",
        _ => "❓",
    };

    println!("## 🔍 Topology Analysis {status_emoji}");
    println!();

    // Metrics table
    println!("### Metrics");
    println!();
    println!("| Metric | Base | Target | Δ |");
    println!("|--------|------|--------|---|");

    if let Some(metrics) = diff.get("metrics") {
        print_metric_row(metrics, "function_count", "Functions");
        print_metric_row(metrics, "total_cyclomatic", "Total CC");
        print_metric_row(metrics, "avg_cyclomatic", "Avg CC");
        print_metric_row(metrics, "total_cognitive", "Total Cognitive");
        print_metric_row(metrics, "lines_of_code", "Lines of Code");
    }

    // Hotspots
    if let Some(hotspots) = diff.get("hotspots").and_then(|h| h.as_array()) {
        if !hotspots.is_empty() {
            println!();
            println!("### ⚠️ Hotspots");
            println!();
            for hotspot in hotspots {
                let id = hotspot.get("id").and_then(|i| i.as_str()).unwrap_or("?");
                let reason = hotspot
                    .get("reason")
                    .and_then(|r| r.as_str())
                    .unwrap_or("?");
                let suggestion = hotspot.get("suggestion").and_then(|s| s.as_str());
                println!("- **{id}**: {reason}");
                if let Some(s) = suggestion {
                    println!("  - 💡 {s}");
                }
            }
        }
    }

    // Violations
    if let Some(violations) = diff.get("violations").and_then(|v| v.as_array()) {
        if !violations.is_empty() {
            println!();
            println!("### Threshold Violations");
            println!();
            for v in violations {
                let severity = v
                    .get("severity")
                    .and_then(|s| s.as_str())
                    .unwrap_or("WARNING");
                let message = v.get("message").and_then(|m| m.as_str()).unwrap_or("?");
                let emoji = if severity == "ERROR" { "❌" } else { "⚠️" };
                println!("- {emoji} {message}");
            }
        }
    }

    // Footer
    println!();
    println!("---");
    println!(
        "*Generated by [APS Topology](https://github.com/AgentParadise/agent-paradise-standards-system) (EXP-V1-0001)*"
    );

    ExitCode::SUCCESS
}

/// Print a metric row for the comment table.
fn print_metric_row(metrics: &serde_json::Value, key: &str, label: &str) {
    if let Some(m) = metrics.get(key) {
        let base = m.get("base").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let target = m.get("target").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let delta = m.get("delta").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let delta_str = if delta >= 0.0 {
            format!("+{delta:.1}")
        } else {
            format!("{delta:.1}")
        };

        println!("| {label} | {base:.1} | {target:.1} | {delta_str} |");
    }
}

/// Generate a human-readable topology report.
fn topology_report(path: &str, _verbose: bool) -> ExitCode {
    use std::path::Path;

    let topology_path = Path::new(path);
    let modules_path = topology_path.join("metrics/modules.json");

    if !modules_path.exists() {
        eprintln!("Error: No topology artifacts found at {path}");
        eprintln!("Run 'aps run topology analyze' first.");
        return ExitCode::FAILURE;
    }

    // Load modules and generate report
    if let Ok(content) = std::fs::read_to_string(&modules_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(modules) = json.get("modules").and_then(|m| m.as_array()) {
                println!("# Code Topology Report");
                println!();
                println!("## Modules ({})", modules.len());
                println!();
                println!("| Module | Functions | Avg CC | Instability |");
                println!("|--------|-----------|--------|-------------|");

                for module in modules {
                    let id = module.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                    let metrics = module.get("metrics");
                    let func_count = metrics
                        .and_then(|m| m.get("function_count"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let avg_cc = metrics
                        .and_then(|m| m.get("avg_cyclomatic"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let instability = metrics
                        .and_then(|m| m.get("martin"))
                        .and_then(|m| m.get("instability"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);

                    println!("| {id} | {func_count} | {avg_cc:.1} | {instability:.2} |");
                }

                return ExitCode::SUCCESS;
            }
        }
    }

    eprintln!("Error: Could not parse modules.json");
    ExitCode::FAILURE
}
