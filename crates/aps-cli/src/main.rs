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

mod vsa_config;

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
                println!("    Commands: analyze, validate, diff, report, viz");
                println!();
                println!("  fitness (EXP-V1-0003) v0.1.0");
                println!(
                    "    Architecture Fitness Functions - declarative architectural assertions"
                );
                println!("    Commands: validate");
                println!();
                println!("  docs (EXP-V1-0004) v0.1.0");
                println!(
                    "    Documentation and Context Engineering - ADR enforcement, README indexes"
                );
                println!("    Commands: validate, index");
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
        "fitness" | "fitness-functions" | "exp-v1-0003" => Some(StandardCliInfo {
            id: "EXP-V1-0003",
            slug: "fitness",
            name: "Architecture Fitness Functions",
            version: "0.1.0",
        }),
        "docs" | "documentation" | "doc" | "exp-v1-0004" => Some(StandardCliInfo {
            id: "EXP-V1-0004",
            slug: "docs",
            name: "Documentation and Context Engineering",
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
        "fitness" => dispatch_fitness(command, cmd_args, repo_root, verbose),
        "docs" => dispatch_docs(command, cmd_args, repo_root, verbose),
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
            println!("    viz <path>         Generate visualizations from .topology/");
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
            println!("VIZ OPTIONS:");
            println!("    --type <type>      Visualization type:");
            println!(
                "                       3d       - 3D force-directed coupling graph (default)"
            );
            println!("                       codecity - 3D city metaphor (buildings = modules)");
            println!("                       clusters - 2D package relationship graph");
            println!("                       vsa      - Vertical Slice Architecture matrix");
            println!("                       all      - Generate all visualizations");
            println!("    --output <path>    Output file/directory");
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

            topology_viz(path, viz_type, output, verbose)
        }
        _ => {
            eprintln!("Error: Unknown topology command '{command}'");
            eprintln!("Use 'aps run topology --help' for available commands.");
            ExitCode::FAILURE
        }
    }
}

/// Dispatch fitness function commands.
fn dispatch_fitness(
    command: &str,
    args: &[String],
    repo_root: &std::path::Path,
    _verbose: bool,
) -> ExitCode {
    match command {
        "--help" | "-h" | "help" => {
            println!("Architecture Fitness Functions (EXP-V1-0003) v0.1.0");
            println!();
            println!("USAGE:");
            println!("    aps run fitness <COMMAND> [OPTIONS]");
            println!();
            println!("COMMANDS:");
            println!("    validate <path>    Validate fitness rules against topology artifacts");
            println!();
            println!("OPTIONS:");
            println!("    --config <file>    Path to fitness.toml (default: ./fitness.toml)");
            println!("    --report <file>    Write JSON report to file");
            println!("    --help             Show this help message");
            ExitCode::SUCCESS
        }
        "validate" => {
            // Parse flags and positional args separately to avoid
            // `--config custom.toml .` misinterpreting "--config" as the path
            let mut positional_path: Option<&str> = None;
            let mut config_path: Option<std::path::PathBuf> = None;
            let mut report_path: Option<&String> = None;
            let mut i = 0;
            while i < args.len() {
                match args[i].as_str() {
                    "--config" => {
                        config_path = args.get(i + 1).map(std::path::PathBuf::from);
                        i += 2;
                    }
                    "--report" => {
                        report_path = args.get(i + 1);
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

            // Resolve --config relative to target repo, not CWD
            let config_path = config_path.map(|p| if p.is_absolute() { p } else { target.join(p) });

            let validator =
                match fitness_functions::FitnessValidator::load(&target, config_path.as_deref()) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        return ExitCode::FAILURE;
                    }
                };

            let report = match validator.validate() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error during validation: {e}");
                    return ExitCode::FAILURE;
                }
            };

            // Print human-readable summary
            println!("Fitness Validation Report");
            println!("========================\n");
            for result in &report.results {
                let status_icon = match result.status {
                    fitness_functions::RuleStatus::Pass => "PASS",
                    fitness_functions::RuleStatus::Fail => "FAIL",
                    fitness_functions::RuleStatus::Warn => "WARN",
                    fitness_functions::RuleStatus::Skip => "SKIP",
                };
                println!(
                    "  [{status_icon}] {} ({})",
                    result.rule_name, result.rule_id
                );
                for v in &result.violations {
                    let exc = if v.excepted { " (excepted)" } else { "" };
                    println!(
                        "         {} = {} (threshold: {} {:?}){exc}",
                        v.entity, v.actual, v.threshold, v.direction
                    );
                }
            }

            if !report.stale_exceptions.is_empty() {
                println!("\nStale Exceptions:");
                for s in &report.stale_exceptions {
                    println!("  {} [{}]: {:?}", s.entity, s.rule_id, s.reason);
                }
            }

            println!(
                "\nSummary: {} passed, {} failed, {} warned, {} violations ({} excepted), {} stale exceptions",
                report.summary.passed,
                report.summary.failed,
                report.summary.warned,
                report.summary.total_violations,
                report.summary.excepted_violations,
                report.summary.stale_exceptions,
            );

            // Write JSON report if requested
            if let Some(report_file) = report_path {
                match serde_json::to_string_pretty(&report) {
                    Ok(json) => {
                        if let Err(e) = std::fs::write(report_file, json) {
                            eprintln!("Error writing report: {e}");
                        } else {
                            println!("\nReport written to: {report_file}");
                        }
                    }
                    Err(e) => eprintln!("Error serializing report: {e}"),
                }
            }

            if fitness_functions::FitnessValidator::has_failures(&report) {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        other => {
            eprintln!("Error: Unknown fitness command '{other}'");
            eprintln!("Use 'aps run fitness --help' for available commands.");
            ExitCode::FAILURE
        }
    }
}

/// Load docs config from a custom path (for `--config` flag).
fn load_docs_config(
    path: &str,
) -> Result<documentation::config::ApssConfig, documentation::config::ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        documentation::config::ConfigError::ReadError {
            path: std::path::PathBuf::from(path),
            source: e,
        }
    })?;
    toml::from_str(&content).map_err(|e| documentation::config::ConfigError::ParseError {
        path: std::path::PathBuf::from(path),
        source: e,
    })
}

/// Dispatch documentation standard commands.
fn dispatch_docs(
    command: &str,
    args: &[String],
    repo_root: &std::path::Path,
    _verbose: bool,
) -> ExitCode {
    match command {
        "--help" | "-h" | "help" => {
            println!("Documentation and Context Engineering (EXP-V1-0004) v0.1.0");
            println!();
            println!("USAGE:");
            println!("    aps run docs <COMMAND> [OPTIONS]");
            println!();
            println!("COMMANDS:");
            println!("    validate [path]    Validate documentation structure, ADRs, and indexes");
            println!("    index [path]       Generate or check README indexes from front matter");
            println!();
            println!("OPTIONS:");
            println!(
                "    --config <file>    Path to .apss/config.toml (default: <path>/.apss/config.toml)"
            );
            println!(
                "    --write            Write generated indexes into README.md files (index only)"
            );
            println!("    --json             Output validation results as JSON");
            println!("    --help             Show this help message");
            ExitCode::SUCCESS
        }
        "validate" => {
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
                match load_docs_config(cfg) {
                    Ok(config) => {
                        let dc = config.docs.clone();
                        (
                            documentation::DocValidator::with_config(&target, config.docs),
                            dc,
                        )
                    }
                    Err(e) => {
                        eprintln!("Error loading config: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            } else {
                match documentation::DocValidator::load(&target) {
                    Ok(v) => {
                        let dc = v.config().clone();
                        (v, dc)
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            };

            let mut diagnostics = validator.validate();

            // ADR substandard (EXP-V1-0004.ADR01)
            let adr_validator = if config_path.is_some() {
                documentation_adr::AdrValidator::with_config(&target, docs_config)
            } else {
                match documentation_adr::AdrValidator::load(&target) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error loading ADR validator: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            };
            diagnostics.merge(adr_validator.validate());

            if json_output {
                match diagnostics.to_json() {
                    Ok(json) => println!("{json}"),
                    Err(e) => {
                        eprintln!("Error serializing JSON: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            } else if diagnostics.is_empty() {
                println!("Documentation validation passed.");
            } else {
                println!("{diagnostics}");
            }

            match diagnostics.exit_code() {
                0 => ExitCode::SUCCESS,
                _ => ExitCode::FAILURE,
            }
        }
        "index" => {
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
                match load_docs_config(cfg) {
                    Ok(config) => documentation::DocValidator::with_config(&target, config.docs),
                    Err(e) => {
                        eprintln!("Error loading config: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            } else {
                match documentation::DocValidator::load(&target) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            };

            if write_mode {
                match validator.write_indexes() {
                    Ok(count) => {
                        println!("Updated {count} README.md file(s) with generated indexes.");
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("Error writing indexes: {e}");
                        ExitCode::FAILURE
                    }
                }
            } else {
                match validator.generate_indexes() {
                    Ok(indexes) => {
                        if indexes.is_empty() {
                            println!("No documents with front matter found.");
                        } else {
                            for idx in &indexes {
                                println!("--- {} ---", idx.dir.display());
                                println!("{}", idx.markdown);
                            }
                            println!("{} directory index(es) generated (dry run).", indexes.len());
                            println!("Use --write to update README.md files.");
                        }
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("Error generating indexes: {e}");
                        ExitCode::FAILURE
                    }
                }
            }
        }
        other => {
            eprintln!("Error: Unknown docs command '{other}'");
            eprintln!("Use 'aps run docs --help' for available commands.");
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
    use code_topology::adapter::grammars::{
        PythonGrammar, RustGrammar, TsxGrammar, TypeScriptGrammar,
    };
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
    registry.register(Box::new(TypeScriptGrammar::new()));
    registry.register(Box::new(TsxGrammar::new()));

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
            // Skip hidden dirs, test dirs, and common non-source dirs
            !name.starts_with('.')
                && name != "target"
                && name != "node_modules"
                && name != "__pycache__"
                && name != "tests"
                && !name.ends_with("_test.rs")
                && !name.starts_with("test_")
                && !name.ends_with("_test.py")
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

    // Analyze all files - extract functions, imports, types, AND calls
    let mut all_functions = Vec::new();
    let mut all_imports: Vec<code_topology::ImportInfo> = Vec::new();
    let mut all_types: Vec<code_topology::TypeInfo> = Vec::new();
    let mut all_calls: Vec<code_topology::CallInfo> = Vec::new();
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

            // Extract types for abstractness calculation
            if let Ok(types) = adapter.extract_types(&source, file_path) {
                all_types.extend(types);
            }

            // Extract calls for call coupling analysis
            if let Ok(calls) = adapter.extract_calls(&source, file_path) {
                all_calls.extend(calls);
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
    if let Err(e) = write_topology_artifacts(
        output_path,
        &all_functions,
        &all_imports,
        &all_types,
        &all_calls,
        &files_by_lang,
    ) {
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
    types: &[code_topology::TypeInfo],
    calls: &[code_topology::CallInfo],
    files_by_lang: &std::collections::HashMap<String, Vec<std::path::PathBuf>>,
) -> std::io::Result<()> {
    use std::collections::{HashMap, HashSet};
    use std::fs;

    // Create directories
    fs::create_dir_all(output_path)?;
    fs::create_dir_all(output_path.join("metrics"))?;
    fs::create_dir_all(output_path.join("graphs"))?;

    // Deduplicate functions — tree-sitter queries can match the same function
    // multiple times (e.g. a class method matches both the function pattern
    // and the method-in-class pattern).  Keep the first occurrence per
    // (file_path, start_line) pair.
    let mut seen_functions: HashSet<(std::path::PathBuf, u32)> = HashSet::new();
    let functions: Vec<_> = functions
        .iter()
        .filter(|(func, _)| seen_functions.insert((func.file_path.clone(), func.start_line)))
        .collect();

    // Group functions by module
    let mut modules: HashMap<
        String,
        Vec<&&(code_topology::FunctionInfo, code_topology::FunctionMetrics)>,
    > = HashMap::new();
    for func_with_metrics in &functions {
        modules
            .entry(func_with_metrics.0.module.clone())
            .or_default()
            .push(func_with_metrics);
    }

    // Group types by module for abstractness calculation
    // Map module -> (abstract_count, total_count)
    let mut module_types: HashMap<String, (u32, u32)> = HashMap::new();
    for type_info in types {
        let entry = module_types
            .entry(type_info.module.clone())
            .or_insert((0, 0));
        entry.1 += 1; // total count
        if type_info.is_abstract {
            entry.0 += 1; // abstract count
        }
    }

    // Build dependency graph from imports
    // Map module -> set of modules it depends on (efferent coupling)
    let mut efferent: HashMap<String, HashSet<String>> = HashMap::new();
    // Map module -> set of modules that depend on it (afferent coupling)
    let mut afferent: HashMap<String, HashSet<String>> = HashMap::new();
    // Map (from, to) -> list of imports with full details (for weighted coupling calculation)
    let mut import_edges: HashMap<(String, String), Vec<code_topology::ImportInfo>> =
        HashMap::new();

    // Initialize all modules
    for module in modules.keys() {
        efferent.entry(module.clone()).or_default();
        afferent.entry(module.clone()).or_default();
    }

    // Process imports to build coupling with weighted scoring
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
            let matches = import_path.contains(to_module.split("::").last().unwrap_or(to_module))
                || to_module.contains(import_path)
                || import_path.split("::").any(|part| to_module.contains(part));

            if matches && from_module != to_module {
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
                // Track the full import for weighted coupling calculation
                import_edges
                    .entry((from_module.clone(), to_module.clone()))
                    .or_default()
                    .push(import.clone());
            }
        }
    }

    // Write manifest.toml
    let mut languages: Vec<&str> = files_by_lang.keys().map(|s| s.as_str()).collect();
    languages.sort();
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
        "functions": functions.iter().map(|&(func, metrics)| {
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
            let total_cc: u32 = funcs.iter().map(|&&(_, m)| m.cyclomatic_complexity).sum();
            let total_cog: u32 = funcs.iter().map(|&&(_, m)| m.cognitive_complexity).sum();
            let total_loc: u32 = funcs.iter().map(|&&(_, m)| m.total_lines).sum();
            let count = funcs.len() as f64;

            // Unique files
            let unique_files: HashSet<_> = funcs.iter()
                .map(|&&(f, _)| f.file_path.clone())
                .collect();

            // Per-module languages (derived from qualified_name prefix)
            let module_languages: Vec<&str> = {
                let mut langs: HashSet<&str> = HashSet::new();
                for &&(f, _) in funcs {
                    if let Some(lang) = f.qualified_name.split(':').next() {
                        langs.insert(lang);
                    }
                }
                let mut v: Vec<&str> = langs.into_iter().collect();
                v.sort();
                v
            };

            // Martin metrics
            let ca = afferent.get(module_id).map(|s| s.len()).unwrap_or(0) as u32;
            let ce = efferent.get(module_id).map(|s| s.len()).unwrap_or(0) as u32;
            let instability = if ca + ce > 0 {
                ce as f64 / (ca + ce) as f64
            } else {
                0.5 // Default when no coupling
            };

            // Calculate abstractness from type analysis
            let (abstract_count, total_types) = module_types
                .get(module_id)
                .copied()
                .unwrap_or((0, 0));
            let abstractness = if total_types > 0 {
                abstract_count as f64 / total_types as f64
            } else {
                0.0 // No types = not abstract
            };

            let distance = (instability + abstractness - 1.0).abs();

            serde_json::json!({
                "id": module_id,
                "name": module_id.split("::").last().unwrap_or(module_id),
                "path": format!("{}/", module_id.replace("::", "/")),
                "languages": module_languages,
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

    // =========================================================================
    // M1: Write dependencies.json (dependency graph with edges)
    // =========================================================================
    let dependency_nodes: Vec<serde_json::Value> = modules
        .keys()
        .map(|id| {
            serde_json::json!({
                "id": id,
                "type": "module"
            })
        })
        .collect();

    let dependency_edges: Vec<serde_json::Value> = import_edges
        .iter()
        .map(|((from, to), imports)| {
            serde_json::json!({
                "from": from,
                "to": to,
                "imports": imports,
                "weight": imports.len()
            })
        })
        .collect();

    let total_internal_edges = dependency_edges.len();
    let total_external_imports = imports.iter().filter(|i| i.is_external).count();

    let dependencies_json = serde_json::json!({
        "schema_version": "1.0.0",
        "nodes": dependency_nodes,
        "edges": dependency_edges,
        "metadata": {
            "total_nodes": modules.len(),
            "total_edges": total_internal_edges,
            "external_imports": total_external_imports
        }
    });
    fs::write(
        output_path.join("graphs/dependencies.json"),
        serde_json::to_string_pretty(&dependencies_json).unwrap(),
    )?;

    // =========================================================================
    // M2: Build coupling matrix with REAL values (not hardcoded 0.5)
    // =========================================================================
    let module_names: Vec<&str> = modules.keys().map(|s| s.as_str()).collect();
    let n = module_names.len();
    let mut matrix = vec![vec![0.0; n]; n];

    // Create index map
    let module_index: HashMap<&str, usize> = module_names
        .iter()
        .enumerate()
        .map(|(i, &name)| (name, i))
        .collect();

    // Fill diagonal with 1.0 (self-coupling)
    for (i, row) in matrix.iter_mut().enumerate() {
        row[i] = 1.0;
    }

    // =========================================================================
    // COMPOSITE COUPLING CALCULATION (v2.0)
    // Uses weighted import coupling with logarithmic percentile normalization
    // =========================================================================

    // Step 1: Calculate raw weighted import coupling
    // Weight by import kind: wildcard=0.3, multi=0.7/symbol, single=1.0, module=0.5
    let mut raw_coupling: HashMap<(usize, usize), f64> = HashMap::new();

    for ((from, to), imports_list) in &import_edges {
        if let (Some(&from_idx), Some(&to_idx)) = (
            module_index.get(from.as_str()),
            module_index.get(to.as_str()),
        ) {
            let mut weighted_score = 0.0;
            for import in imports_list {
                let base_weight = import.kind.weight();
                // For multi-imports, weight per symbol but cap total contribution
                // to avoid single large imports dominating the score
                let symbol_count = if import.symbols.is_empty() {
                    1.0
                } else {
                    import.symbols.len() as f64
                };
                // Cap at 3.0 to prevent outliers (e.g., `use foo::{a,b,c,d,e,f,g}`)
                let import_score = (base_weight * symbol_count).min(3.0);
                weighted_score += import_score;
            }
            raw_coupling.insert((from_idx, to_idx), weighted_score);
        }
    }

    // Step 2: Logarithmic percentile normalization
    // This produces a smooth distribution instead of discrete buckets
    fn logarithmic_percentile_normalize(
        values: &HashMap<(usize, usize), f64>,
    ) -> HashMap<(usize, usize), f64> {
        if values.is_empty() {
            return HashMap::new();
        }

        // Apply log transform to handle outliers
        let log_values: Vec<((usize, usize), f64)> =
            values.iter().map(|(&k, &v)| (k, (v + 1.0).ln())).collect();

        // Sort by log value to compute percentile ranks
        let mut sorted_values: Vec<f64> = log_values.iter().map(|(_, v)| *v).collect();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Compute percentile rank for each value
        let n = sorted_values.len();
        if n <= 1 {
            // Single value or empty: all get 1.0
            return log_values.iter().map(|(k, _)| (*k, 1.0)).collect();
        }
        log_values
            .iter()
            .map(|(k, log_v)| {
                // Count values strictly less than current (0-indexed rank)
                let rank = sorted_values.iter().filter(|&&v| v < *log_v).count() as f64;
                // Use (rank) / (n-1) to get proper 0.0 to 1.0 range
                let percentile = rank / (n - 1) as f64;
                (*k, percentile)
            })
            .collect()
    }

    // Step 3: Store raw import coupling for components breakdown (normalized by max)
    let mut import_coupling_matrix = vec![vec![0.0; n]; n];
    let max_import_raw = raw_coupling.values().cloned().fold(1.0_f64, f64::max);
    for ((from_idx, to_idx), raw_score) in &raw_coupling {
        import_coupling_matrix[*from_idx][*to_idx] = raw_score / max_import_raw;
    }

    // =========================================================================
    // CALL COUPLING CALCULATION
    // Count cross-module function calls
    // =========================================================================
    let mut call_edges: HashMap<(String, String), usize> = HashMap::new();
    for call in calls {
        let caller_module = &call.caller;
        let callee = &call.callee;

        // Try to resolve callee to a module with stricter matching
        for to_module in modules.keys() {
            if caller_module == to_module {
                continue; // Skip self-references
            }

            let to_name = to_module.split("::").last().unwrap_or(to_module);

            // Match if:
            // 1. Qualified call: callee starts with module path (e.g., "discovery::find_packages")
            // 2. Direct module call: callee equals module name (e.g., "discovery")
            // 3. Function in module: callee contains "::" and first part matches module
            let is_qualified_call =
                callee.starts_with(to_module) || callee.starts_with(&format!("{to_name}::"));
            let is_module_reference = callee == to_name || callee == to_module;
            let is_namespaced_call = callee.contains("::") && {
                let parts: Vec<&str> = callee.split("::").collect();
                parts.first().is_some_and(|first| *first == to_name)
            };

            if is_qualified_call || is_module_reference || is_namespaced_call {
                *call_edges
                    .entry((caller_module.clone(), to_module.clone()))
                    .or_insert(0) += 1;
            }
        }
    }

    // Build call coupling matrix
    let mut call_coupling_matrix = vec![vec![0.0; n]; n];
    let mut raw_call_coupling: HashMap<(usize, usize), f64> = HashMap::new();
    for ((from, to), count) in &call_edges {
        if let (Some(&from_idx), Some(&to_idx)) = (
            module_index.get(from.as_str()),
            module_index.get(to.as_str()),
        ) {
            raw_call_coupling.insert((from_idx, to_idx), *count as f64);
        }
    }

    // Normalize call coupling
    let max_call_raw = raw_call_coupling.values().cloned().fold(1.0_f64, f64::max);
    for ((from_idx, to_idx), raw_score) in &raw_call_coupling {
        call_coupling_matrix[*from_idx][*to_idx] = raw_score / max_call_raw;
    }

    // =========================================================================
    // TYPE COUPLING CALCULATION
    // Track type references between modules
    // =========================================================================
    let mut type_edges: HashMap<(String, String), usize> = HashMap::new();

    // Build a map of type name -> defining module
    let mut type_to_module: HashMap<String, String> = HashMap::new();
    for type_info in types {
        type_to_module.insert(type_info.name.clone(), type_info.module.clone());
    }

    // For each function, check if it uses types from other modules
    // This is a simplified approach - we look for type names in the same module's functions
    for &(func, _) in &functions {
        let func_module = &func.module;
        // Check for type usages - simplified: count types defined in other modules
        for (type_name, defining_module) in &type_to_module {
            if func_module != defining_module && func.qualified_name.contains(type_name) {
                *type_edges
                    .entry((func_module.clone(), defining_module.clone()))
                    .or_insert(0) += 1;
            }
        }
    }

    // Build type coupling matrix
    let mut type_coupling_matrix = vec![vec![0.0; n]; n];
    let mut raw_type_coupling: HashMap<(usize, usize), f64> = HashMap::new();
    for ((from, to), count) in &type_edges {
        if let (Some(&from_idx), Some(&to_idx)) = (
            module_index.get(from.as_str()),
            module_index.get(to.as_str()),
        ) {
            raw_type_coupling.insert((from_idx, to_idx), *count as f64);
        }
    }

    // Normalize type coupling
    let max_type_raw = raw_type_coupling.values().cloned().fold(1.0_f64, f64::max);
    for ((from_idx, to_idx), raw_score) in &raw_type_coupling {
        type_coupling_matrix[*from_idx][*to_idx] = raw_score / max_type_raw;
    }

    // =========================================================================
    // COMPOSITE SCORE
    // Combine all coupling components with weights
    // =========================================================================
    const IMPORT_WEIGHT: f64 = 0.40; // Increased since we have fewer components
    const CALL_WEIGHT: f64 = 0.35;
    const TYPE_WEIGHT: f64 = 0.25;

    // Combine all raw couplings for composite percentile normalization
    let mut composite_raw: HashMap<(usize, usize), f64> = HashMap::new();
    for i in 0..n {
        for j in 0..n {
            if i != j {
                let import_score = import_coupling_matrix[i][j];
                let call_score = call_coupling_matrix[i][j];
                let type_score = type_coupling_matrix[i][j];

                let composite = IMPORT_WEIGHT * import_score
                    + CALL_WEIGHT * call_score
                    + TYPE_WEIGHT * type_score;

                if composite > 0.0 {
                    composite_raw.insert((i, j), composite);
                }
            }
        }
    }

    // Re-normalize the composite scores using percentile ranking
    let normalized_composite = logarithmic_percentile_normalize(&composite_raw);

    // Fill the final matrix with composite normalized values
    for ((from_idx, to_idx), strength) in &normalized_composite {
        matrix[*from_idx][*to_idx] = *strength;
    }

    let coupling_json = serde_json::json!({
        "schema_version": "2.0.0",
        "metric": "composite_coupling",
        "description": "Composite coupling strength combining import, call, and type coupling (0-1). Directional: matrix[i][j] = strength of module i depending on module j.",
        "modules": module_names,
        "matrix": matrix,
        "components": {
            "import_coupling": {
                "weight": IMPORT_WEIGHT,
                "description": "Weighted import statement dependencies (wildcard=0.3, multi=0.7, single=1.0)",
                "matrix": import_coupling_matrix
            },
            "call_coupling": {
                "weight": CALL_WEIGHT,
                "description": "Cross-module function call count",
                "matrix": call_coupling_matrix,
                "total_calls": calls.len()
            },
            "type_coupling": {
                "weight": TYPE_WEIGHT,
                "description": "Type references between modules",
                "matrix": type_coupling_matrix,
                "total_types": types.len()
            }
        },
        "metadata": {
            "normalization": "logarithmic_percentile",
            "directional": true,
            "total_import_edges": import_edges.len(),
            "total_call_edges": call_edges.len(),
            "total_type_edges": type_edges.len(),
            "weights": {
                "wildcard": 0.3,
                "multi_per_symbol": 0.7,
                "single": 1.0,
                "module": 0.5
            }
        }
    });
    fs::write(
        output_path.join("graphs/coupling-matrix.json"),
        serde_json::to_string_pretty(&coupling_json).unwrap(),
    )?;

    // =========================================================================
    // M4: Slice Independence Score (SIS) for Vertical Slice Architecture
    // =========================================================================

    // Detect slices from first-level module path segment
    // e.g., "aef.core.events" -> slice "aef.core"
    //       "crates::aps-cli::src::main" -> slice "crates::aps-cli"
    fn get_slice_id(module_id: &str) -> String {
        // Split by the appropriate separator and take first two segments.
        // Path-like IDs (containing '/') use '/' — this avoids splitting inside
        // Next.js catch-all routes like [[...slug]] where '.' is literal.
        let separator = if module_id.contains('/') {
            "/"
        } else if module_id.contains("::") {
            "::"
        } else {
            "."
        };
        let parts: Vec<&str> = module_id.split(separator).collect();
        if parts.len() >= 2 {
            format!("{}{}{}", parts[0], separator, parts[1])
        } else {
            parts[0].to_string()
        }
    }

    // Group modules by slice
    let mut slices: HashMap<String, Vec<String>> = HashMap::new();
    for module_id in modules.keys() {
        let slice_id = get_slice_id(module_id);
        slices.entry(slice_id).or_default().push(module_id.clone());
    }

    // Calculate SIS for each slice
    // SIS = internal_imports / (internal_imports + external_imports)
    let slices_json: Vec<serde_json::Value> = slices
        .iter()
        .map(|(slice_id, slice_modules)| {
            let slice_module_set: HashSet<&str> =
                slice_modules.iter().map(|s| s.as_str()).collect();

            let mut internal_imports = 0u32;
            let mut cross_slice_imports = 0u32;
            let mut outbound_slices: HashSet<String> = HashSet::new();
            let mut inbound_slices: HashSet<String> = HashSet::new();

            // Count imports for modules in this slice
            for module in slice_modules {
                // Outbound: modules this slice depends on
                if let Some(deps) = efferent.get(module) {
                    for dep in deps {
                        let dep_slice = get_slice_id(dep);
                        if dep_slice == *slice_id {
                            internal_imports += 1;
                        } else {
                            cross_slice_imports += 1;
                            outbound_slices.insert(dep_slice);
                        }
                    }
                }

                // Inbound: modules that depend on this slice
                if let Some(dependents) = afferent.get(module) {
                    for dependent in dependents {
                        if !slice_module_set.contains(dependent.as_str()) {
                            let dependent_slice = get_slice_id(dependent);
                            inbound_slices.insert(dependent_slice);
                        }
                    }
                }
            }

            // Unique slice counts (more meaningful than edge counts)
            let inbound_coupling = inbound_slices.len() as u32;
            let outbound_coupling = outbound_slices.len() as u32;

            let total_imports = internal_imports + cross_slice_imports;
            let sis = if total_imports > 0 {
                internal_imports as f64 / total_imports as f64
            } else {
                1.0 // No imports = fully independent
            };

            serde_json::json!({
                "id": slice_id,
                "modules": slice_modules,
                "metrics": {
                    "module_count": slice_modules.len(),
                    "internal_imports": internal_imports,
                    "cross_slice_imports": cross_slice_imports,
                    "sis": sis,
                    "inbound_coupling": inbound_coupling,
                    "outbound_coupling": outbound_coupling
                }
            })
        })
        .collect();

    let slices_output = serde_json::json!({
        "schema_version": "1.0.0",
        "description": "Slice Independence Score (SIS) for Vertical Slice Architecture analysis. SIS = internal_imports / total_imports. Higher = more isolated.",
        "slices": slices_json,
        "metadata": {
            "total_slices": slices.len(),
            "slice_detection": "first_two_path_segments"
        }
    });
    fs::write(
        output_path.join("metrics/slices.json"),
        serde_json::to_string_pretty(&slices_output).unwrap(),
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
        "graphs/dependencies.json",
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

/// Calculate health score for a module (0.0 to 1.0)
fn calculate_health(
    function_count: u32,
    total_cyclomatic: u32,
    total_cognitive: u32,
    lines_of_code: u32,
    ca: u32,
    ce: u32,
) -> f64 {
    let mut scores = Vec::new();

    let func_count = function_count.max(1) as f64;

    // 1. Complexity per function (ideal: 3-8, bad: >15)
    let avg_cc = total_cyclomatic as f64 / func_count;
    let cc_score = if avg_cc > 5.0 {
        (1.0 - (avg_cc - 5.0) / 15.0).max(0.0)
    } else {
        1.0
    };
    scores.push(cc_score);

    // 2. Cognitive load per function (ideal: <10, bad: >30)
    let avg_cog = total_cognitive as f64 / func_count;
    let cog_score = (1.0 - avg_cog / 30.0).max(0.0);
    scores.push(cog_score);

    // 3. LOC per function (ideal: 10-50, bad: >100)
    let loc_per_func = lines_of_code as f64 / func_count;
    let loc_score = if loc_per_func > 50.0 {
        (1.0 - (loc_per_func - 50.0) / 100.0).max(0.0)
    } else {
        1.0
    };
    scores.push(loc_score);

    // 4. Coupling balance (isolated or over-coupled is bad)
    let total_coupling = ca + ce;
    let coupling_score = if total_coupling == 0 {
        0.6 // Isolated
    } else if total_coupling > 20 {
        (1.0 - (total_coupling as f64 - 10.0) / 30.0).max(0.2)
    } else {
        1.0
    };
    scores.push(coupling_score);

    // 5. Module size (ideal: 5-30 functions)
    let size_score = if function_count < 2 {
        0.5
    } else if function_count > 50 {
        (1.0 - (function_count as f64 - 30.0) / 70.0).max(0.3)
    } else {
        1.0
    };
    scores.push(size_score);

    scores.iter().sum::<f64>() / scores.len() as f64
}

/// Convert health score (0.0-1.0) to hex color
fn health_to_color(health: f64) -> &'static str {
    match health {
        h if h >= 0.80 => "#00ff88", // Excellent
        h if h >= 0.65 => "#44dd77", // Good
        h if h >= 0.50 => "#88cc55", // OK
        h if h >= 0.35 => "#ddaa33", // Warning
        h if h >= 0.20 => "#ff7744", // Poor
        _ => "#ff3333",              // Critical
    }
}

/// Get health label from score
fn health_label(health: f64) -> &'static str {
    match health {
        h if h >= 0.80 => "Excellent",
        h if h >= 0.65 => "Good",
        h if h >= 0.50 => "OK",
        h if h >= 0.35 => "Warning",
        h if h >= 0.20 => "Poor",
        _ => "Critical",
    }
}

/// Detect architectural layer from module path
fn detect_layer(module_path: &str) -> &'static str {
    let path_lower = module_path.to_lowercase();

    // Check patterns in order of specificity - includes Rust patterns
    let patterns: [(&str, &[&str]); 6] = [
        // Entry points / handlers
        (
            "handlers",
            &[
                "handler",
                "controller",
                "api",
                "routes",
                "endpoint",
                "view",
                "main",
                "cli",
                "bin",
                "cmd",
            ],
        ),
        // Business logic
        (
            "services",
            &[
                "service",
                "usecase",
                "application",
                "interactor",
                "core",
                "engine",
                "processor",
                "worker",
            ],
        ),
        // Domain models and types
        (
            "models",
            &[
                "model", "entity", "domain", "schema", "types", "struct", "metadata", "config",
            ],
        ),
        // Data access
        (
            "data",
            &[
                "repository",
                "repo",
                "data",
                "store",
                "db",
                "persistence",
                "storage",
                "discovery",
            ],
        ),
        // Utilities and helpers
        (
            "utils",
            &[
                "util", "helper", "common", "shared", "lib", "support", "tools", "ext",
            ],
        ),
        // Adapters and integrations (Rust-specific)
        (
            "adapters",
            &[
                "adapter",
                "grammars",
                "queries",
                "parser",
                "lexer",
                "projector",
                "renderer",
                "visitor",
            ],
        ),
    ];

    for (layer, keywords) in patterns {
        for keyword in keywords.iter() {
            if path_lower.contains(keyword) {
                return layer;
            }
        }
    }

    // Fallback: Check Rust directory patterns
    if path_lower.contains("examples") {
        return "examples";
    }
    if path_lower.contains("tests") || path_lower.contains("test_") {
        return "tests";
    }
    if path_lower.contains("src") && !path_lower.contains("adapter") {
        return "core";
    }

    "other"
}

/// Generate a placeholder HTML page when no vsa.yaml is found.
fn generate_vsa_placeholder() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>VSA Visualization — No Configuration</title>
<style>
  body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; background: #1a1a2e; color: #ccc; display: flex; justify-content: center; align-items: center; min-height: 100vh; margin: 0; }
  .card { background: #16213e; border: 1px solid #0f3460; border-radius: 12px; padding: 48px; max-width: 560px; text-align: center; }
  h1 { color: #e94560; font-size: 1.5em; margin-bottom: 16px; }
  p { line-height: 1.6; margin: 8px 0; }
  code { background: #0f3460; padding: 2px 8px; border-radius: 4px; font-size: 0.9em; }
  pre { background: #0f3460; padding: 16px; border-radius: 8px; text-align: left; overflow-x: auto; font-size: 0.85em; margin-top: 24px; }
</style>
</head>
<body>
<div class="card">
  <h1>No VSA Configuration Found</h1>
  <p>The VSA (Vertical Slice Architecture) visualization requires a <code>vsa.yaml</code> file in your repository root to identify which bounded contexts to display.</p>
  <p>Without this file, all modules would appear as vertical slices — which is misleading for non-VSA packages.</p>
  <pre>
# vsa.yaml (version 1)
version: 1
root: ./path/to/contexts
language: python

contexts:
  orchestration:
    description: "Workflow execution"
  artifacts:
    description: "Artifact storage"</pre>
  <p style="margin-top: 24px; font-size: 0.9em; color: #888;">See the Event Sourcing Platform docs for the full <code>vsa.yaml</code> specification.</p>
</div>
</body>
</html>"#.to_string()
}

/// Get slice (top-level package) from module ID
/// For Rust: crates::foo -> "crates::foo", standards-experimental::v1::NAME -> "NAME"
fn get_slice_from_id(module_id: &str) -> String {
    // Handle Rust-style paths with ::
    if module_id.contains("::") {
        let parts: Vec<&str> = module_id.split("::").collect();

        // For standards-experimental, use the standard name as slice
        if parts.len() >= 3 && parts[0] == "standards-experimental" {
            return parts[2].to_string(); // e.g., "EXP-V1-0001-code-topology"
        }

        // For crates, use crate name
        if parts.len() >= 2 && parts[0] == "crates" {
            return parts[1].to_string(); // e.g., "aps-core"
        }

        // Default: first two segments
        if parts.len() >= 2 {
            return format!("{}::{}", parts[0], parts[1]);
        }
        return parts.first().unwrap_or(&module_id).to_string();
    }

    // Handle path-like IDs (containing '/') — split on '/' to avoid breaking
    // Next.js catch-all routes like [[...slug]] where '.' is literal.
    let separator = if module_id.contains('/') { "/" } else { "." };
    let parts: Vec<&str> = module_id.split(separator).collect();
    if parts.len() >= 2 {
        format!("{}{}{}", parts[0], separator, parts[1])
    } else {
        parts.first().unwrap_or(&module_id).to_string()
    }
}

/// Generate visualization from topology artifacts.
fn topology_viz(path: &str, viz_type: &str, output: Option<&str>, verbose: bool) -> ExitCode {
    use code_topology::{
        CouplingMatrixData, CouplingMatrixFile, MartinMetrics, ModuleMetrics, ModuleRecord,
        OutputFormat, Projector, Topology,
    };
    use code_topology_3d::ForceDirectedProjector;
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    let topology_path = Path::new(path);
    let modules_path = topology_path.join("metrics/modules.json");
    let coupling_path = topology_path.join("graphs/coupling-matrix.json");

    // Check for required artifacts
    if !modules_path.exists() {
        eprintln!("Error: No modules.json found at {}", modules_path.display());
        eprintln!("Run 'aps run topology analyze' first.");
        return ExitCode::FAILURE;
    }

    if !coupling_path.exists() {
        eprintln!(
            "Error: No coupling-matrix.json found at {}",
            coupling_path.display()
        );
        eprintln!("Run 'aps run topology analyze' first.");
        return ExitCode::FAILURE;
    }

    if verbose {
        println!("Loading topology from: {}", topology_path.display());
    }

    // Load VSA config if present (look in repo root, i.e. parent of .topology/)
    let repo_root = topology_path.parent().unwrap_or(Path::new("."));
    let vsa_config = match vsa_config::VsaConfig::load(repo_root) {
        Ok(Some(config)) => {
            if verbose {
                println!(
                    "  Found vsa.yaml (v{}) — root: {}",
                    config.version,
                    config.normalized_root()
                );
                if let Some(names) = config.contexts.as_ref() {
                    println!(
                        "  Contexts: {}",
                        names.keys().cloned().collect::<Vec<_>>().join(", ")
                    );
                }
            }
            Some(config)
        }
        Ok(None) => {
            if verbose {
                println!("  No vsa.yaml found (VSA viz will show placeholder)");
            }
            None
        }
        Err(e) => {
            eprintln!("Warning: {e}");
            eprintln!("VSA config load/validation failed; VSA viz will show placeholder.");
            None
        }
    };

    // Load coupling matrix
    let coupling_content = match fs::read_to_string(&coupling_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading coupling matrix: {e}");
            return ExitCode::FAILURE;
        }
    };

    let matrix_file: CouplingMatrixFile = match serde_json::from_str(&coupling_content) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error parsing coupling matrix: {e}");
            return ExitCode::FAILURE;
        }
    };

    if verbose {
        println!(
            "  Loaded {} modules from coupling matrix",
            matrix_file.modules.len()
        );
    }

    // Load module metrics
    let modules_content = match fs::read_to_string(&modules_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading modules: {e}");
            return ExitCode::FAILURE;
        }
    };

    #[derive(serde::Deserialize)]
    struct ModulesFile {
        modules: Vec<ModuleRecord>,
    }

    let modules_file: ModulesFile = match serde_json::from_str(&modules_content) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error parsing modules: {e}");
            return ExitCode::FAILURE;
        }
    };

    if verbose {
        println!("  Loaded {} module metrics", modules_file.modules.len());
    }

    // Build topology for 3D viz (used by 3d type)
    let mut topology = Topology {
        languages: vec!["rust".to_string()],
        ..Default::default()
    };

    // Convert coupling matrix to internal format
    let positions = matrix_file.layout.as_ref().map(|l| l.positions.clone());
    topology.coupling_matrix = Some(CouplingMatrixData {
        modules: matrix_file.modules.clone(),
        values: matrix_file.matrix.clone(),
        positions,
    });

    // Build enriched module data for visualizations
    #[derive(serde::Serialize)]
    struct VizModule {
        id: String,
        name: String,
        path: String,
        slice: String,
        layer: String,
        function_count: u32,
        total_cyclomatic: u32,
        total_cognitive: u32,
        lines_of_code: u32,
        ca: u32,
        ce: u32,
        health: f64,
        color: String,
        health_label: String,
    }

    let mut viz_modules: Vec<VizModule> = Vec::new();

    for record in &modules_file.modules {
        let health = calculate_health(
            record.metrics.function_count,
            record.metrics.total_cyclomatic,
            record.metrics.total_cognitive,
            record.metrics.lines_of_code,
            record.metrics.martin.ca,
            record.metrics.martin.ce,
        );

        viz_modules.push(VizModule {
            id: record.id.clone(),
            name: record.name.clone(),
            path: record.path.clone(),
            slice: get_slice_from_id(&record.id),
            layer: detect_layer(&record.path).to_string(),
            function_count: record.metrics.function_count,
            total_cyclomatic: record.metrics.total_cyclomatic,
            total_cognitive: record.metrics.total_cognitive,
            lines_of_code: record.metrics.lines_of_code,
            ca: record.metrics.martin.ca,
            ce: record.metrics.martin.ce,
            health,
            color: health_to_color(health).to_string(),
            health_label: health_label(health).to_string(),
        });

        // Also add to topology for 3D viz
        topology.modules.push(ModuleMetrics {
            id: record.id.clone(),
            name: record.name.clone(),
            path: PathBuf::from(&record.path),
            languages: record.languages.clone(),
            file_count: record.metrics.file_count,
            function_count: record.metrics.function_count,
            total_cyclomatic: record.metrics.total_cyclomatic,
            avg_cyclomatic: record.metrics.avg_cyclomatic,
            total_cognitive: record.metrics.total_cognitive,
            avg_cognitive: record.metrics.avg_cognitive,
            lines_of_code: record.metrics.lines_of_code,
            martin: MartinMetrics {
                ca: record.metrics.martin.ca,
                ce: record.metrics.martin.ce,
                instability: record.metrics.martin.instability,
                abstractness: record.metrics.martin.abstractness,
                distance_from_main_sequence: record.metrics.martin.distance_from_main_sequence,
            },
        });
    }

    // Determine which visualizations to generate
    let viz_types: Vec<&str> = match viz_type {
        "all" => vec!["3d", "codecity", "clusters", "vsa"],
        t => vec![t],
    };

    // Create viz output directory if generating multiple
    let viz_dir = topology_path.join("viz");
    if viz_type == "all" {
        if let Err(e) = fs::create_dir_all(&viz_dir) {
            eprintln!("Error creating viz directory: {e}");
            return ExitCode::FAILURE;
        }
    }

    let mut generated_files: Vec<String> = Vec::new();

    for vtype in &viz_types {
        let (html_content, output_path): (String, PathBuf) = match *vtype {
            "3d" => {
                let projector = ForceDirectedProjector::new();
                if verbose {
                    println!("Rendering 3D force-directed visualization...");
                }
                match projector.render(&topology, OutputFormat::Html, None) {
                    Ok(html_bytes) => {
                        let html = String::from_utf8_lossy(&html_bytes).to_string();
                        let out = if viz_type == "all" {
                            viz_dir.join("topology-3d.html")
                        } else {
                            PathBuf::from(output.unwrap_or("topology-3d.html"))
                        };
                        (html, out)
                    }
                    Err(e) => {
                        eprintln!("Error rendering 3D visualization: {}", e.message);
                        return ExitCode::FAILURE;
                    }
                }
            }
            "codecity" => {
                if verbose {
                    println!("Rendering CodeCity visualization...");
                }
                let modules_json = serde_json::to_string_pretty(&viz_modules).unwrap_or_default();
                let coupling_json = serde_json::to_string_pretty(&matrix_file).unwrap_or_default();
                let html = code_topology_viz::codecity::generate(&modules_json, &coupling_json);
                let out = if viz_type == "all" {
                    viz_dir.join("codecity.html")
                } else {
                    PathBuf::from(output.unwrap_or("codecity.html"))
                };
                (html, out)
            }
            "clusters" => {
                if verbose {
                    println!("Rendering Package Clusters visualization...");
                }
                let modules_json = serde_json::to_string_pretty(&viz_modules).unwrap_or_default();
                let coupling_json = serde_json::to_string_pretty(&matrix_file).unwrap_or_default();
                let html = code_topology_viz::clusters::generate(&modules_json, &coupling_json);
                let out = if viz_type == "all" {
                    viz_dir.join("clusters.html")
                } else {
                    PathBuf::from(output.unwrap_or("clusters.html"))
                };
                (html, out)
            }
            "vsa" => {
                if verbose {
                    println!("Rendering VSA diagram...");
                }
                let out = if viz_type == "all" {
                    viz_dir.join("vsa.html")
                } else {
                    PathBuf::from(output.unwrap_or("vsa.html"))
                };

                let html = if let Some(ref vsa_cfg) = vsa_config {
                    // Filter to only modules under the VSA root and fix slice names
                    let vsa_modules: Vec<serde_json::Value> = viz_modules
                        .iter()
                        .filter_map(|m| {
                            let path = &m.path;
                            let id = &m.id;
                            // Check if module is under the VSA root
                            if !vsa_cfg.contains_path(path) && !vsa_cfg.contains_path(id) {
                                return None;
                            }
                            // Extract context name as the slice
                            let context = vsa_cfg
                                .extract_context(path)
                                .or_else(|| vsa_cfg.extract_context(id))?;
                            // If v1 config has explicit contexts, only include listed ones
                            if !vsa_cfg.is_context_allowed(&context) {
                                return None;
                            }
                            // Re-serialize with the correct slice and layer names
                            let mut val = serde_json::to_value(m).ok()?;
                            val["slice"] = serde_json::Value::String(context);
                            // Override layer from directory structure instead of keyword matching
                            if let Some(layer) = vsa_cfg
                                .extract_layer(path)
                                .or_else(|| vsa_cfg.extract_layer(id))
                            {
                                val["layer"] = serde_json::Value::String(layer);
                            }
                            Some(val)
                        })
                        .collect();

                    if verbose {
                        println!(
                            "  VSA: {} of {} modules matched config",
                            vsa_modules.len(),
                            viz_modules.len()
                        );
                    }
                    let modules_json =
                        serde_json::to_string_pretty(&vsa_modules).unwrap_or_default();
                    code_topology_viz::vsa::generate(&modules_json)
                } else {
                    // No vsa.yaml — render placeholder
                    generate_vsa_placeholder()
                };

                (html, out)
            }
            unknown => {
                eprintln!("Error: Unknown visualization type '{unknown}'");
                eprintln!("Available types: 3d, codecity, clusters, vsa, all");
                return ExitCode::FAILURE;
            }
        };

        if let Err(e) = fs::write(&output_path, &html_content) {
            eprintln!("Error writing {}: {e}", output_path.display());
            return ExitCode::FAILURE;
        }
        generated_files.push(output_path.display().to_string());
    }

    // Generate index if --all
    if viz_type == "all" {
        if verbose {
            println!("Generating index...");
        }

        // Calculate summary stats
        let total_modules = viz_modules.len();
        let mut slices: HashMap<String, u32> = HashMap::new();
        let mut total_health = 0.0;
        for m in &viz_modules {
            *slices.entry(m.slice.clone()).or_insert(0) += 1;
            total_health += m.health;
        }
        let avg_health = if total_modules > 0 {
            total_health / total_modules as f64
        } else {
            0.0
        };

        // Derive repo name from topology path or current directory
        let repo_name = topology_path
            .canonicalize()
            .ok()
            .and_then(|p| {
                // Go up from .topology to the repo root
                let repo_root = if p.ends_with(".topology") || p.ends_with(".topology/") {
                    p.parent()
                } else {
                    Some(p.as_path())
                };
                repo_root
                    .and_then(|r| r.file_name())
                    .map(|n| n.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "Project".to_string());

        let index_html =
            code_topology_viz::index::generate(&repo_name, total_modules, slices.len(), avg_health);
        let index_path = viz_dir.join("index.html");
        if let Err(e) = fs::write(&index_path, &index_html) {
            eprintln!("Error writing index: {e}");
            return ExitCode::FAILURE;
        }
        generated_files.push(index_path.display().to_string());
    }

    // Print results
    println!("✓ Generated visualizations:");
    for file in &generated_files {
        println!("  {file}");
    }
    // Auto-open in browser
    let open_path = if viz_type == "all" {
        viz_dir.join("index.html")
    } else {
        PathBuf::from(generated_files.first().unwrap_or(&String::new()))
    };

    println!();
    println!("Opening in browser: {}", open_path.display());

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&open_path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&open_path)
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", &open_path.display().to_string()])
            .spawn();
    }

    ExitCode::SUCCESS
}
