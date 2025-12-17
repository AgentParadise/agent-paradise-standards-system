//! APS CLI
//!
//! Command-line interface for APS validation and scaffolding.
//!
//! # Usage
//!
//! ## Consumer Commands (adopt standards)
//!
//! ```bash
//! # Initialize .aps/ in current directory
//! aps init --name my-project
//!
//! # Add a standard to the manifest
//! aps add code-topology@0.1.0
//!
//! # List adopted standards
//! aps list
//!
//! # Sync artifacts from adopted standards
//! aps sync
//!
//! # Check compliance
//! aps check
//! ```
//!
//! ## Author Commands (create standards)
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
//! # List all V1 packages
//! aps v1 list
//! ```

use aps_core::discovery::{PackageType, count_packages, discover_v1_packages, find_package_by_id};
use aps_core::versioning::BumpPart;
use aps_core::{
    StandardContext, TemplateEngine, bump_version, generate_all_views, generate_index, get_version,
    init_manifest, load_manifest, promote_experiment, write_index,
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
    // ========================================================================
    // Consumer Commands (for adopting standards in downstream projects)
    // ========================================================================
    /// Initialize .aps/ manifest in current directory
    Init {
        /// Project name (defaults to directory name)
        #[arg(long)]
        name: Option<String>,
    },

    /// Add a standard to the manifest
    Add {
        /// Standard slug with optional version (e.g., code-topology@0.1.0)
        standard: String,
        /// Source URI override
        #[arg(long)]
        source: Option<String>,
    },

    /// List adopted standards (consumer context)
    #[command(name = "list")]
    ConsumerList,

    /// Sync artifacts from adopted standards
    Sync {
        /// Force regeneration even if up to date
        #[arg(long)]
        force: bool,
    },

    /// Check compliance with adopted standards
    Check,

    /// Check for available standard updates
    Update {
        /// Show what would be updated without making changes
        #[arg(long)]
        dry_run: bool,
    },

    // ========================================================================
    // Author Commands (for creating/maintaining standards)
    // ========================================================================
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
        // ====================================================================
        // Consumer Commands
        // ====================================================================
        Commands::Init { name } => {
            let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let project_name = name.unwrap_or_else(|| {
                cwd.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("my-project")
                    .to_string()
            });

            match init_manifest(&cwd, &project_name) {
                Ok(manifest_path) => {
                    println!("✅ Created {}", manifest_path.display());
                    println!("✅ Created .aps/index.json");
                    println!();
                    println!("Next steps:");
                    println!("  aps add code-topology@0.1.0");
                    println!("  aps sync");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error initializing manifest: {e}");
                    ExitCode::FAILURE
                }
            }
        }

        Commands::Add { standard, source } => {
            let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            // Parse standard@version format
            let (slug, version) = if let Some((s, v)) = standard.split_once('@') {
                (s.to_string(), v.to_string())
            } else {
                (standard.clone(), "0.1.0".to_string())
            };

            // Load existing manifest
            let manifest = match load_manifest(&cwd) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error: {e}");
                    return ExitCode::FAILURE;
                }
            };

            // Check if already added
            if manifest.standards.contains_key(&slug) {
                eprintln!("Standard '{slug}' is already in the manifest.");
                eprintln!("To update, edit .aps/manifest.toml directly.");
                return ExitCode::FAILURE;
            }

            // Determine standard ID from slug (simplified mapping)
            let id = match slug.as_str() {
                "code-topology" => "EXP-V1-0001",
                "consumer-sdk" => "EXP-V1-0002",
                _ => {
                    eprintln!("Unknown standard slug: {slug}");
                    eprintln!("Available standards: code-topology, consumer-sdk");
                    return ExitCode::FAILURE;
                }
            };

            let source_uri = source.unwrap_or_else(|| {
                "github:AgentParadise/agent-paradise-standards-system".to_string()
            });

            // Read and update manifest file
            let manifest_path = cwd.join(".aps/manifest.toml");
            let mut content = std::fs::read_to_string(&manifest_path).unwrap_or_default();

            // Append the new standard
            let addition = format!(
                r#"
[standards.{slug}]
id = "{id}"
version = "{version}"
source = "{source_uri}"
"#
            );
            content.push_str(&addition);

            if let Err(e) = std::fs::write(&manifest_path, content) {
                eprintln!("Error writing manifest: {e}");
                return ExitCode::FAILURE;
            }

            println!("✅ Added {slug}@{version} to .aps/manifest.toml");
            println!();
            println!("Run 'aps sync' to generate artifacts.");
            ExitCode::SUCCESS
        }

        Commands::ConsumerList => {
            let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            let manifest = match load_manifest(&cwd) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error: {e}");
                    return ExitCode::FAILURE;
                }
            };

            if manifest.standards.is_empty() {
                println!("No standards adopted yet.");
                println!();
                println!("Add a standard with: aps add code-topology@0.1.0");
                return ExitCode::SUCCESS;
            }

            println!("Adopted Standards:");
            for (slug, standard_ref) in &manifest.standards {
                let artifacts = if standard_ref.artifacts.is_empty() {
                    String::new()
                } else {
                    format!(" → {}", standard_ref.artifacts.join(", "))
                };
                println!(
                    "  {:<16} {:12} {:8}{}",
                    slug, standard_ref.id, standard_ref.version, artifacts
                );
            }

            if !manifest.substandards.is_empty() {
                println!();
                println!("Enabled Substandards:");
                for (key, config) in &manifest.substandards {
                    if config.enabled {
                        println!("  {key}");
                    }
                }
            }

            ExitCode::SUCCESS
        }

        Commands::Sync { force: _ } => {
            let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            let manifest = match load_manifest(&cwd) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error: {e}");
                    return ExitCode::FAILURE;
                }
            };

            if manifest.standards.is_empty() {
                println!("No standards to sync.");
                println!("Add a standard first: aps add code-topology@0.1.0");
                return ExitCode::SUCCESS;
            }

            // Generate index
            match generate_index(&cwd, &manifest) {
                Ok(index) => match write_index(&cwd, &index) {
                    Ok(path) => {
                        println!("✅ Generated {}", path.display());
                    }
                    Err(e) => {
                        eprintln!("Error writing index: {e}");
                        return ExitCode::FAILURE;
                    }
                },
                Err(e) => {
                    eprintln!("Error generating index: {e}");
                    return ExitCode::FAILURE;
                }
            }

            // TODO: Invoke standard-specific sync hooks
            for (slug, standard_ref) in &manifest.standards {
                println!(
                    "Syncing {slug}@{}... (artifact generation not yet implemented)",
                    standard_ref.version
                );
            }

            println!();
            println!("✅ Sync complete");
            ExitCode::SUCCESS
        }

        Commands::Check => {
            let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            let manifest = match load_manifest(&cwd) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error: {e}");
                    return ExitCode::FAILURE;
                }
            };

            if manifest.standards.is_empty() {
                println!("No standards to check.");
                return ExitCode::SUCCESS;
            }

            println!("Checking compliance...");
            println!();

            let mut warnings = 0;
            let errors = 0;

            for (slug, standard_ref) in &manifest.standards {
                println!("{slug}@{}:", standard_ref.version);

                // Check artifacts exist
                for artifact in &standard_ref.artifacts {
                    let path = cwd.join(artifact);
                    if path.exists() {
                        println!("  ✅ {artifact} exists");
                    } else {
                        println!("  ⚠️  {artifact} missing");
                        warnings += 1;
                    }
                }

                if standard_ref.artifacts.is_empty() {
                    println!("  (no artifacts to check)");
                }
            }

            // Check index exists
            let index_path = cwd.join(".aps/index.json");
            if index_path.exists() {
                println!();
                println!("✅ .aps/index.json exists");
            } else {
                println!();
                println!("⚠️  .aps/index.json missing (run 'aps sync')");
                warnings += 1;
            }

            println!();
            if errors > 0 {
                println!("{warnings} warning(s), {errors} error(s)");
                ExitCode::FAILURE
            } else if warnings > 0 {
                println!("{warnings} warning(s), 0 errors");
                ExitCode::SUCCESS
            } else {
                println!("✅ All checks passed");
                ExitCode::SUCCESS
            }
        }

        Commands::Update { dry_run } => {
            let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            let manifest = match load_manifest(&cwd) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Error: {e}");
                    return ExitCode::FAILURE;
                }
            };

            if manifest.standards.is_empty() {
                println!("No standards to update.");
                return ExitCode::SUCCESS;
            }

            println!("Checking for updates...");
            if dry_run {
                println!("(dry run - no changes will be made)");
            }
            println!();

            // TODO: Actually check for updates from sources
            for (slug, standard_ref) in &manifest.standards {
                // Placeholder: just report current version
                println!("  {slug}  {} (up to date)", standard_ref.version);
            }

            println!();
            println!("Update checking not yet implemented.");
            println!("Edit .aps/manifest.toml manually to update versions.");
            ExitCode::SUCCESS
        }

        // ====================================================================
        // Author Commands (V1)
        // ====================================================================
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
