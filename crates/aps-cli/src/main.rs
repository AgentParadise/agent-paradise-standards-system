//! APS CLI
//!
//! Command-line interface for APS validation and scaffolding.
//!
//! # Usage
//!
//! ```bash
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
use aps_core::{StandardContext, TemplateEngine, generate_all_views, promote_experiment};
use aps_v1_0000_meta::{MetaStandard, Standard};
use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "aps")]
#[command(version, about = "Agent Paradise Standards System CLI")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: human (default) or json
    #[arg(long, default_value = "human", global = true)]
    format: OutputFormat,

    /// Enable/disable colors (auto-detected by default)
    #[arg(long, global = true)]
    color: Option<bool>,
}

#[derive(Clone, Copy, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Human,
    Json,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// V1 standards operations
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
    /// List all V1 packages
    List,
}

#[derive(clap::Subcommand)]
enum GenerateTarget {
    /// Generate all derived views
    Views,
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
                            eprintln!("Error: Standard '{}' not found", id);
                            return ExitCode::FAILURE;
                        }
                    }
                    ValidateTarget::Substandard { id } => {
                        if let Some(pkg) = find_package_by_id(&repo_root, &id) {
                            println!("Validating substandard: {} at {}", id, pkg.path.display());
                            meta.validate_package(&pkg.path)
                        } else {
                            eprintln!("Error: Substandard '{}' not found", id);
                            return ExitCode::FAILURE;
                        }
                    }
                    ValidateTarget::Experiment { id } => {
                        if let Some(pkg) = find_package_by_id(&repo_root, &id) {
                            println!("Validating experiment: {} at {}", id, pkg.path.display());
                            meta.validate_package(&pkg.path)
                        } else {
                            eprintln!("Error: Experiment '{}' not found", id);
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
                            println!("\n{}", diagnostics);
                        }
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            diagnostics
                                .to_json()
                                .unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
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
                    println!("  ID:   {}", id);
                    println!("  Name: {}", name);
                    println!("  Slug: {}", slug);

                    let output_dir = repo_root.join(format!("standards/v1/{}-{}", id, slug));

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
                            eprintln!("Error creating standard: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                CreateTarget::Substandard { parent_id, profile } => {
                    println!("Creating new substandard: {}.{}", parent_id, profile);
                    eprintln!("⚠️  Substandard creation not yet implemented");
                    ExitCode::FAILURE
                }
                CreateTarget::Experiment { slug, name } => {
                    let name = name.unwrap_or_else(|| slug_to_name(&slug));
                    let id = allocate_next_experiment_id(&repo_root);

                    println!("Creating new experiment:");
                    println!("  ID:   {}", id);
                    println!("  Name: {}", name);
                    println!("  Slug: {}", slug);

                    let output_dir =
                        repo_root.join(format!("standards-experimental/v1/{}-{}", id, slug));

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
                            eprintln!("Error creating experiment: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
            },
            V1Commands::Promote {
                experiment_id,
                target_id,
            } => {
                println!("Promoting experiment: {}", experiment_id);

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
                        eprintln!("Error promoting experiment: {}", e);
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
                            eprintln!("Error generating views: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
            },
            V1Commands::List => {
                let packages = discover_v1_packages(&repo_root);
                let (standards, substandards, experiments) = count_packages(&repo_root);

                println!("V1 Packages ({} total):", packages.len());
                println!("  Standards:    {}", standards);
                println!("  Substandards: {}", substandards);
                println!("  Experiments:  {}", experiments);
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
                        println!("  [{:^11}] {}", type_label, name);
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
