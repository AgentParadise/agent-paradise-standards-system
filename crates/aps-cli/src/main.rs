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
            println!("    report <path>      Generate human-readable report");
            println!();
            println!("OPTIONS:");
            println!("    --output <dir>     Output directory (default: .topology)");
            println!("    --format <fmt>     Output format: json, text (default: text)");
            println!("    --help             Show this help message");
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

            topology_analyze(path, output, repo_root, verbose)
        }
        "validate" => {
            let path = args.first().map(|s| s.as_str()).unwrap_or(".topology");
            topology_validate(path, verbose)
        }
        "diff" => {
            if args.len() < 2 {
                eprintln!("Error: diff requires two paths");
                eprintln!("Usage: aps run topology diff <base> <target>");
                return ExitCode::FAILURE;
            }
            topology_diff(&args[0], &args[1], verbose)
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
    _repo_root: &std::path::Path,
    verbose: bool,
) -> ExitCode {
    use code_topology_rust_adapter::RustAdapter;
    use std::path::Path;

    let project_path = Path::new(path);
    let output_path = Path::new(output);

    if verbose {
        println!("Analyzing: {}", project_path.display());
        println!("Output:    {}", output_path.display());
    }

    let adapter = RustAdapter::new();

    match adapter.analyze(project_path) {
        Ok(result) => {
            println!(
                "✓ Analyzed {} functions in {} modules",
                result.functions.len(),
                result.modules.len()
            );

            match result.write_artifacts(output_path) {
                Ok(()) => {
                    println!("✓ Wrote artifacts to {}", output_path.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error writing artifacts: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(e) => {
            eprintln!("Error analyzing project: {e}");
            ExitCode::FAILURE
        }
    }
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
fn topology_diff(base: &str, target: &str, _verbose: bool) -> ExitCode {
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

    // Load and compare function counts (simplified diff)
    let base_funcs = base_path.join("metrics/functions.json");
    let target_funcs = target_path.join("metrics/functions.json");

    let base_count = count_functions(&base_funcs);
    let target_count = count_functions(&target_funcs);

    println!("Topology Diff: {base} → {target}");
    println!();
    println!(
        "  Functions: {} → {} ({:+})",
        base_count,
        target_count,
        target_count as i64 - base_count as i64
    );

    // TODO: Add detailed metric comparison

    if target_count >= base_count {
        println!();
        println!("✓ No degradation detected");
        ExitCode::SUCCESS
    } else {
        println!();
        println!("⚠ Complexity reduced (review recommended)");
        ExitCode::from(2) // Warning exit code
    }
}

/// Count functions in a functions.json file.
fn count_functions(path: &std::path::Path) -> usize {
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(funcs) = json.get("functions").and_then(|f| f.as_array()) {
                return funcs.len();
            }
        }
    }
    0
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
