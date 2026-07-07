//! APS CLI
//!
//! Command-line interface for APS validation and scaffolding.
//!
//! # Usage
//!
//! ```bash
//! # Run a standard's CLI
//! apss-dev run topology analyze .
//! apss-dev run topology validate .topology/
//! apss-dev run --list
//!
//! # Validate the entire V1 repo structure
//! apss-dev v1 validate repo
//!
//! # Validate a specific standard
//! apss-dev v1 validate standard APS-V1-0000
//!
//! # Create a new standard
//! apss-dev v1 create standard my-new-standard
//!
//! # List all packages
//! apss-dev v1 list
//! ```

mod cli_exemptions;

use aps_v1_0000_meta::{MetaStandard, Standard};
use apss_core::discovery::{
    PackageMetadata, PackageType, count_packages, discover_v1_packages, find_package_by_id,
};
use apss_core::versioning::BumpPart;
use apss_core::{
    Diagnostic, Diagnostics, StandardContext, TemplateEngine, bump_version, generate_all_views,
    get_version, promote_experiment,
};
use clap::Parser;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "apss-dev")]
#[command(version, about = "Agent Paradise Standards System CLI")]
#[command(propagate_version = true)]
#[command(after_help = "Use 'apss-dev v1 --help' for V1 standards operations")]
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
    /// Create a local APSS bundle for a standard or substandard
    Bundle {
        /// Package ID to bundle, for example APS-V1-0001 or APS-V1-0000.DI01
        id: String,
        /// Output directory for bundle directories
        #[arg(long, default_value = "target/apss-bundles")]
        output: PathBuf,
    },
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
    /// Validate an apss.yaml project configuration file (CF01)
    Config {
        /// Path to apss.yaml. If omitted, searches upward from the current directory.
        path: Option<PathBuf>,
    },
    /// Validate standard crates for distribution compliance (DI01)
    Distribution,
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
                // Build the list from the registered standards themselves so it
                // can never drift from register() metadata.
                println!("Available Standards:\n");
                let mut collector = apss_core::registry::CollectorRegistry::new();
                code_topology::register(&mut collector);
                architecture_fitness::register(&mut collector);
                documentation::register(&mut collector);
                agent_recipe::register(&mut collector);
                for (info, _handler) in collector.entries() {
                    println!("  {} ({}) v{}", info.slug, info.id, info.version);
                    println!("    {}", info.description);
                    if !info.commands.is_empty() {
                        println!("    Commands: {}", info.commands.join(", "));
                    }
                    println!();
                }
                println!("Use 'apss-dev run <slug> --help' for command details.");
                return ExitCode::SUCCESS;
            }

            let slug = standard.unwrap_or_default();
            if slug.is_empty() {
                eprintln!(
                    "Error: Standard slug required. Use 'apss-dev run --list' to see available standards."
                );
                return ExitCode::FAILURE;
            }

            // Dispatch to standard CLI
            match resolve_standard(&slug) {
                Some(info) => dispatch_standard_cli(&info, &args, cli.verbose),
                None => {
                    eprintln!("Error: Unknown standard '{slug}'");
                    eprintln!("Use 'apss-dev run --list' to see available standards.");
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
                    ValidateTarget::Config { path } => {
                        let config_path = match path {
                            Some(path) => path,
                            None => match apss_core::config::find_project_config(
                                &std::env::current_dir().unwrap_or_else(|_| repo_root.clone()),
                            ) {
                                Some(path) => path,
                                None => {
                                    eprintln!("Error: No apss.yaml found");
                                    return ExitCode::FAILURE;
                                }
                            },
                        };
                        println!("Validating project config: {}", config_path.display());
                        apss_project_config::validate_project_config(&config_path)
                    }
                    ValidateTarget::Distribution => {
                        println!(
                            "Validating distribution compliance for all standards in: {}",
                            repo_root.display()
                        );
                        let packages = discover_v1_packages(&repo_root);
                        let mut all_diags = Diagnostics::new();
                        for package in &packages {
                            // Merged substandards (ADR-0002) are not separately
                            // published: their code lives in the parent crate as a
                            // feature module, so they carry `substandard.toml` and
                            // `docs/` but no `Cargo.toml`. Skip publish and
                            // release-readiness checks for them; the parent crate is
                            // validated on its own iteration.
                            let is_merged_substandard =
                                package.path.join("substandard.toml").exists()
                                    && !package.path.join("Cargo.toml").exists();
                            if is_merged_substandard {
                                continue;
                            }

                            let mut pkg_diags =
                                apss_distribution::validate_publishable_standard(&package.path);
                            pkg_diags.merge(apss_distribution::validate_release_readiness(
                                &package.path,
                            ));
                            // Parity poka-yoke (ADR-0002 / DI01): a standard crate's
                            // [features] keys must equal its substandard codes.
                            // Standards without a substandards/ dir are skipped by
                            // the validator itself.
                            if package.path.join("substandards").is_dir() {
                                pkg_diags.merge(
                                    apss_distribution::validate_substandard_feature_parity(
                                        &package.path,
                                    ),
                                );
                            }
                            if !pkg_diags.is_empty() {
                                all_diags.push(Diagnostic::info(
                                    "DI_CHECKING",
                                    format!("Checking: {}", package.path.display()),
                                ));
                                all_diags.merge(pkg_diags);
                            }
                        }

                        // CL01 poka-yoke (issue #69, ADR-0002): every linked standard must
                        // actually register CLI commands. Silence is never a pass.
                        let mut collector = apss_core::registry::CollectorRegistry::new();
                        code_topology::register(&mut collector);
                        architecture_fitness::register(&mut collector);
                        documentation::register(&mut collector);
                        agent_recipe::register(&mut collector);

                        let package_dirs: Vec<std::path::PathBuf> =
                            packages.iter().map(|p| p.path.clone()).collect();
                        let exempt = cli_exemptions::collect_cli_exemptions(&package_dirs);

                        all_diags.merge(apss_core::registry::validate_registered_commands(
                            collector.entries(),
                            &exempt,
                        ));

                        all_diags
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
                                "\nNext steps:\n  1. Add to Cargo.toml workspace members\n  2. Implement the Standard trait\n  3. Run: apss-dev v1 validate standard {id}"
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
                    let context = apss_core::SubstandardContext::new(&id, &name, &slug, &parent_id);

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
                                "\nNext steps:\n  1. Add to Cargo.toml workspace members\n  2. Implement the profile-specific logic\n  3. Run: apss-dev v1 validate substandard {id}"
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
                    let context = apss_core::ExperimentContext::new(&id, &name, &slug);

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
                                "\nNext steps:\n  1. Add to Cargo.toml workspace members\n  2. Iterate on the experiment\n  3. When ready, use: apss-dev v1 promote {id}"
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
                        println!(
                            "  3. Run: apss-dev v1 validate standard {}",
                            result.standard_id
                        );
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
            V1Commands::Bundle { id, output } => {
                match create_local_bundle(&repo_root, &id, &output) {
                    Ok(bundle_dir) => {
                        println!("Created APSS bundle: {}", bundle_dir.display());
                        println!(
                            "Install locally with: apss install --bundle-dir {}",
                            bundle_dir.display()
                        );
                        ExitCode::SUCCESS
                    }
                    Err(error) => {
                        eprintln!("Error creating APSS bundle: {error}");
                        ExitCode::FAILURE
                    }
                }
            }
        },
    }
}

fn create_local_bundle(
    repo_root: &Path,
    id: &str,
    output_dir: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut package = find_package_by_id(repo_root, id).ok_or_else(|| {
        format!("package '{id}' not found in standards/v1 or standards-experimental/v1")
    })?;
    let metadata = package.load_metadata()?.clone();
    let bundle_name = format!(
        "{}-{}-{}.apss",
        metadata.id(),
        metadata_slug(&metadata),
        metadata.version()
    );
    let bundle_dir = output_dir.join(bundle_name);

    if bundle_dir.exists() {
        fs::remove_dir_all(&bundle_dir)?;
    }
    fs::create_dir_all(&bundle_dir)?;

    let package_relative = package.path.strip_prefix(repo_root)?.to_path_buf();
    let package_output = bundle_dir.join(&package_relative);
    copy_dir_filtered(&package.path, &package_output)?;

    let core_relative = PathBuf::from("crates/apss-core");
    let core_source = repo_root.join(&core_relative);
    let core_output = bundle_dir.join(&core_relative);
    copy_dir_filtered(&core_source, &core_output)?;

    let mut workspace_members = vec![core_relative, package_relative.clone()];
    workspace_members.extend(discover_cargo_members(&package.path, repo_root)?);
    workspace_members.sort();
    workspace_members.dedup();

    let workspace_manifest = workspace_manifest_with_members(repo_root, &workspace_members)?;
    fs::write(bundle_dir.join("Cargo.toml"), workspace_manifest)?;

    let bundle_manifest = bundle_manifest(&metadata, &package, &package_relative);
    fs::write(bundle_dir.join("bundle.toml"), bundle_manifest)?;

    Ok(bundle_dir)
}

fn metadata_slug(metadata: &PackageMetadata) -> &str {
    match metadata {
        PackageMetadata::Standard(metadata) => &metadata.standard.slug,
        PackageMetadata::Substandard(metadata) => &metadata.substandard.slug,
        PackageMetadata::Experiment(metadata) => &metadata.experiment.slug,
    }
}

fn metadata_kind(metadata: &PackageMetadata) -> &'static str {
    match metadata {
        PackageMetadata::Standard(_) => "standard",
        PackageMetadata::Substandard(_) => "substandard",
        PackageMetadata::Experiment(_) => "experiment",
    }
}

fn bundle_manifest(
    metadata: &PackageMetadata,
    package: &apss_core::discovery::DiscoveredPackage,
    package_relative: &Path,
) -> String {
    format!(
        r#"schema = "apss.bundle/v1"
id = "{}"
name = "{}"
slug = "{}"
version = "{}"
kind = "{}"
metadata_file = "{}"

[source]
package_path = "{}"
repository = "{}"

[payload]
metadata = "{}"
docs = "docs"
implementation = "."
"#,
        metadata.id(),
        escape_toml_string(metadata.name()),
        metadata_slug(metadata),
        metadata.version(),
        metadata_kind(metadata),
        package.metadata_file,
        escape_toml_string(&package_relative.display().to_string()),
        env!("CARGO_PKG_REPOSITORY"),
        package.metadata_file
    )
}

fn discover_cargo_members(
    package_path: &Path,
    repo_root: &Path,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut members = Vec::new();
    for entry in walkdir::WalkDir::new(package_path)
        .into_iter()
        .filter_entry(|entry| !should_skip_bundle_dir(entry.path()))
    {
        let entry = entry?;
        if entry.file_type().is_dir() && should_skip_bundle_dir(entry.path()) {
            continue;
        }
        if entry.file_type().is_file() && entry.file_name() == "Cargo.toml" {
            let Some(parent) = entry.path().parent() else {
                continue;
            };
            let relative = parent.strip_prefix(repo_root)?.to_path_buf();
            members.push(relative);
        }
    }
    Ok(members)
}

fn workspace_manifest_with_members(
    repo_root: &Path,
    members: &[PathBuf],
) -> Result<String, Box<dyn std::error::Error>> {
    let source = fs::read_to_string(repo_root.join("Cargo.toml"))?;
    let members_start = source.find("members = [").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "workspace manifest does not contain members list",
        )
    })?;
    let list_start = members_start + "members = [".len();
    let list_end = source[list_start..]
        .find(']')
        .map(|offset| list_start + offset)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "workspace manifest members list is unterminated",
            )
        })?;

    let mut replacement = String::from("members = [\n");
    for member in members {
        replacement.push_str(&format!("    \"{}\",\n", escape_toml_path(member)));
    }
    replacement.push(']');

    let mut output = String::new();
    output.push_str(&source[..members_start]);
    output.push_str(&replacement);
    output.push_str(&source[list_end + 1..]);
    Ok(output)
}

fn copy_dir_filtered(source: &Path, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for entry in walkdir::WalkDir::new(source)
        .into_iter()
        .filter_entry(|entry| !should_skip_bundle_dir(entry.path()))
    {
        let entry = entry?;
        let path = entry.path();
        let relative = path.strip_prefix(source)?;
        let output_path = destination.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&output_path)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &output_path)?;
        }
    }
    Ok(())
}

fn should_skip_bundle_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(
        name,
        ".git" | ".apss" | "target" | "node_modules" | ".cargo" | "tmp" | "temporary"
    )
}

fn escape_toml_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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
        "architecture-fitness" | "fitness" | "fitness-functions" | "aps-v1-0002" => {
            Some(StandardCliInfo {
                id: architecture_fitness::ID,
                slug: "architecture-fitness",
                name: architecture_fitness::NAME,
                version: architecture_fitness::VERSION,
            })
        }
        "docs" | "doc" | "documentation" | "aps-v1-0003" => Some(StandardCliInfo {
            id: documentation::ID,
            slug: "documentation",
            name: documentation::NAME,
            version: documentation::VERSION,
        }),
        "agent-recipe" | "recipe" | "exp-v1-0005" => Some(StandardCliInfo {
            id: agent_recipe::ID,
            slug: "agent-recipe",
            name: agent_recipe::NAME,
            version: agent_recipe::VERSION,
        }),
        _ => None,
    }
}

/// Dispatch to a standard's CLI.
fn dispatch_standard_cli(info: &StandardCliInfo, args: &[String], verbose: bool) -> ExitCode {
    let command = args.first().map(|s| s.as_str()).unwrap_or("--help");
    let cmd_args = if args.len() > 1 { &args[1..] } else { &[] };

    match info.slug {
        "topology" => dispatch_topology(command, cmd_args, verbose),
        "architecture-fitness" => dispatch_architecture_fitness(command, cmd_args, verbose),
        "documentation" => dispatch_documentation(command, cmd_args, verbose),
        "agent-recipe" => dispatch_agent_recipe(command, cmd_args),
        _ => {
            eprintln!("Error: Standard '{}' CLI not implemented", info.slug);
            ExitCode::FAILURE
        }
    }
}

/// Dispatch topology commands through the standard's own command handler.
///
/// The 3,300 LOC of topology command logic now live in the code-topology crate
/// behind `code_topology::cli::TopologyCommandHandler` (ADR-0002, issue #68).
/// aps-cli delegates here: it sets `APSS_VERBOSE` so the env-driven verbose flag
/// survives the trait boundary, then converts the handler's `i32` exit code into
/// an `ExitCode`.
fn dispatch_topology(command: &str, args: &[String], verbose: bool) -> ExitCode {
    use apss_core::registry::CommandHandler;

    if verbose {
        // SAFETY: aps-cli is single-threaded at this point (clap has parsed
        // args and we have not spawned any threads), so setting an env var is
        // safe here.
        unsafe {
            std::env::set_var("APSS_VERBOSE", "1");
        }
    }

    let handler = code_topology::cli::TopologyCommandHandler::new();
    let code = handler.execute(command, args, &toml::Value::Table(Default::default()));
    ExitCode::from(code as u8)
}

/// Dispatch architecture-fitness commands through the standard's own command
/// handler.
///
/// The fitness validate logic lives in the architecture-fitness crate behind
/// `architecture_fitness::cli::FitnessCommandHandler` (APS-V1-0002). aps-cli
/// delegates here: it sets `APSS_VERBOSE` so the env-driven verbose flag
/// survives the trait boundary, then converts the handler's `i32` exit code
/// into an `ExitCode`.
fn dispatch_architecture_fitness(command: &str, args: &[String], verbose: bool) -> ExitCode {
    use apss_core::registry::CommandHandler;

    if verbose {
        // SAFETY: aps-cli is single-threaded at this point (clap has parsed
        // args and we have not spawned any threads), so setting an env var is
        // safe here.
        unsafe {
            std::env::set_var("APSS_VERBOSE", "1");
        }
    }

    let handler = architecture_fitness::cli::FitnessCommandHandler::new();
    let code = handler.execute(command, args, &toml::Value::Table(Default::default()));
    ExitCode::from(code as u8)
}

/// Dispatch documentation commands through the standard's own command handler.
///
/// The doc validate/index logic now lives in the documentation crate behind
/// `documentation::cli::DocumentationCommandHandler` (APS-V1-0003, ADR-0002,
/// issue #68). aps-cli delegates here: it sets `APSS_VERBOSE` so the env-driven
/// verbose flag survives the trait boundary, then converts the handler's `i32`
/// exit code into an `ExitCode`.
fn dispatch_documentation(command: &str, args: &[String], verbose: bool) -> ExitCode {
    use apss_core::registry::CommandHandler;

    if verbose {
        // SAFETY: aps-cli is single-threaded at this point (clap has parsed
        // args and we have not spawned any threads), so setting an env var is
        // safe here.
        unsafe {
            std::env::set_var("APSS_VERBOSE", "1");
        }
    }

    let handler = documentation::cli::DocumentationCommandHandler::new();
    let code = handler.execute(command, args, &toml::Value::Table(Default::default()));
    ExitCode::from(code as u8)
}

/// Dispatch agent-recipe commands through the standard's own command handler.
///
/// The recipe-directory validate logic lives in the agent-recipe crate behind
/// `agent_recipe::cli::AgentRecipeCommandHandler` (EXP-V1-0005). aps-cli
/// delegates here and converts the handler's `i32` exit code into an
/// `ExitCode`. This experimental standard has no verbose output, so the flag
/// is not threaded through.
fn dispatch_agent_recipe(command: &str, args: &[String]) -> ExitCode {
    use apss_core::registry::CommandHandler;

    let handler = agent_recipe::cli::AgentRecipeCommandHandler::new();
    let code = handler.execute(command, args, &toml::Value::Table(Default::default()));
    ExitCode::from(code as u8)
}
