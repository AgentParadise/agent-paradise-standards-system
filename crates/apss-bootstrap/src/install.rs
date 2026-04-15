//! `apss install` command implementation.

use aps_core::config::{self, CONFIG_FILENAME};
use aps_core::lockfile::{self, LOCKFILE_FILENAME, LockedPackage, Lockfile};
use aps_core::resolution;
use apss_distribution::codegen;
use clap::Args;
use std::path::Path;
use std::process::Command;

#[derive(Args)]
pub struct InstallArgs {
    /// Fail if lockfile would change (for CI).
    #[arg(long)]
    locked: bool,

    /// Update a specific standard to latest compatible version.
    #[arg(long)]
    update: Option<String>,

    /// Update all standards to latest compatible versions.
    #[arg(long)]
    update_all: bool,

    /// Use only cached crates (no network).
    #[arg(long)]
    offline: bool,
}

pub fn run(args: InstallArgs) -> i32 {
    // 1. Find and parse config
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to get current directory: {e}");
            return 1;
        }
    };

    let config_path = match config::find_project_config(&cwd) {
        Some(p) => p,
        None => {
            eprintln!("No {CONFIG_FILENAME} found. Run 'apss init' first.");
            return 1;
        }
    };

    let project_config = match config::parse_project_config(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to parse {}: {e}", config_path.display());
            return 1;
        }
    };

    // 2. Validate config
    let diags = apss_project_config::validate_project_config(&config_path);
    if diags.has_errors() {
        eprintln!("{diags}");
        eprintln!("\nFix configuration errors before installing.");
        return 1;
    }

    let project_root = config_path.parent().unwrap_or(Path::new("."));
    let resolved = resolution::resolve_single(project_config, config_path.clone());

    // 3. Generate/update lockfile
    let lockfile_path = project_root.join(LOCKFILE_FILENAME);
    let lockfile = generate_lockfile(&resolved);

    if args.locked && lockfile_path.exists() {
        let existing = match lockfile::parse_lockfile(&lockfile_path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to parse existing lockfile: {e}");
                return 1;
            }
        };

        // Compare package lists
        let existing_ids: Vec<&str> = existing.packages.iter().map(|p| p.id.as_str()).collect();
        let new_ids: Vec<&str> = lockfile.packages.iter().map(|p| p.id.as_str()).collect();

        if existing_ids != new_ids {
            eprintln!("Lockfile would change but --locked was specified.");
            eprintln!("Run 'apss install' without --locked to update.");
            return 1;
        }
    }

    if let Err(e) = lockfile::write_lockfile(&lockfile_path, &lockfile) {
        eprintln!("Failed to write lockfile: {e}");
        return 1;
    }
    println!("Updated {LOCKFILE_FILENAME}");

    // 4. Generate build crate
    let build_dir = project_root.join(apss_distribution::BUILD_DIR);
    match codegen::generate_build_crate(&resolved, &build_dir) {
        Ok(files) => {
            println!("Generated {} files in {}", files.len(), build_dir.display());
        }
        Err(e) => {
            eprintln!("Failed to generate build crate: {e}");
            return 1;
        }
    }

    // 5. Build composed binary
    let bin_dir = project_root.join(resolved.tool.bin_dir.as_str());

    if let Err(e) = std::fs::create_dir_all(&bin_dir) {
        eprintln!("Failed to create bin directory: {e}");
        return 1;
    }

    println!("Building composed binary...");

    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(build_dir.join("Cargo.toml"));

    if args.offline {
        cargo_cmd.arg("--offline");
    }

    match cargo_cmd.status() {
        Ok(status) if status.success() => {
            println!("Build succeeded.");
        }
        Ok(status) => {
            eprintln!(
                "Build failed with exit code: {}",
                status.code().unwrap_or(1)
            );
            eprintln!("\nThis may happen if the standard crates haven't been published yet.");
            eprintln!(
                "Check that the declared standards are available on the configured registry."
            );
            return 1;
        }
        Err(e) => {
            eprintln!("Failed to run cargo build: {e}");
            return 1;
        }
    }

    // 6. Copy binary to bin_dir
    let target_binary = build_dir
        .join("target")
        .join("release")
        .join(apss_distribution::BIN_NAME);
    let output_binary = bin_dir.join(apss_distribution::BIN_NAME);

    if target_binary.exists() {
        if let Err(e) = std::fs::copy(&target_binary, &output_binary) {
            eprintln!("Failed to copy binary: {e}");
            return 1;
        }
        println!("Installed: {}", output_binary.display());
    } else {
        eprintln!(
            "Warning: expected binary at {} not found",
            target_binary.display()
        );
    }

    println!("\nDone! Run 'apss run <standard> <command>' to use your standards.");
    0
}

fn generate_lockfile(config: &resolution::ResolvedProjectConfig) -> Lockfile {
    let mut lockfile = Lockfile::new(env!("CARGO_PKG_VERSION").to_string());

    for (slug, standard) in &config.standards {
        if !standard.enabled {
            continue;
        }

        lockfile.packages.push(LockedPackage {
            id: standard.id.clone(),
            slug: slug.clone(),
            crate_name: standard.crate_name.clone(),
            version: standard.version_req.clone(), // placeholder until real resolution
            checksum: String::new(),               // populated during actual resolution
            source: format!("registry+{}", config.tool.registry),
            substandards: vec![],
        });
    }

    lockfile
}
