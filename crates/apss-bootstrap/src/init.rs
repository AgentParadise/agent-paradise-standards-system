//! `apss init` command implementation.

use aps_core::config::{CONFIG_FILENAME, PROJECT_SCHEMA};
use clap::Args;
use std::path::Path;

#[derive(Args)]
pub struct InitArgs {
    /// Add a standard (format: slug@version or slug).
    /// Can be specified multiple times.
    #[arg(long = "standard", short = 's')]
    standards: Vec<String>,

    /// Initialize in an existing repo (don't create .gitignore entries).
    #[arg(long)]
    existing: bool,
}

pub fn run(args: InitArgs) -> i32 {
    let config_path = Path::new(CONFIG_FILENAME);

    if config_path.exists() {
        eprintln!("{CONFIG_FILENAME} already exists. Use 'apss install' to update.");
        return 1;
    }

    // Build config content
    let mut content = String::new();
    content.push_str(&format!("schema = \"{PROJECT_SCHEMA}\"\n\n"));
    content.push_str("[project]\n");

    // Try to derive project name from directory
    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my-project".to_string());

    content.push_str(&format!("name = \"{project_name}\"\n"));
    content.push_str("apss_version = \"v1\"\n");

    // Add declared standards
    if !args.standards.is_empty() {
        content.push('\n');
        for spec in &args.standards {
            let (slug, version) = parse_standard_spec(spec);
            content.push_str(&format!("\n[standards.{slug}]\n"));
            content.push_str("id = \"APS-V1-XXXX\"  # FIXME: Replace with correct standard ID (e.g. APS-V1-0001)\n");
            content.push_str(&format!("version = \"{version}\"\n"));
        }
    }

    // Write config
    if let Err(e) = std::fs::write(config_path, &content) {
        eprintln!("Failed to write {CONFIG_FILENAME}: {e}");
        return 1;
    }

    println!("Created {CONFIG_FILENAME}");

    // Create .apss directory
    if let Err(e) = std::fs::create_dir_all(".apss/bin") {
        eprintln!("Failed to create .apss/ directory: {e}");
        return 1;
    }

    // Add .gitignore entries if not --existing
    if !args.existing {
        let gitignore_path = Path::new(".gitignore");
        let gitignore_entries = "\n# APSS build artifacts\n.apss/build/\n.apss/bin/\n";

        if gitignore_path.exists() {
            let existing = std::fs::read_to_string(gitignore_path).unwrap_or_default();
            if !existing.contains(".apss/") {
                if let Err(e) =
                    std::fs::write(gitignore_path, format!("{existing}{gitignore_entries}"))
                {
                    eprintln!("Warning: failed to update .gitignore: {e}");
                }
            }
        } else if let Err(e) = std::fs::write(gitignore_path, gitignore_entries.trim_start()) {
            eprintln!("Warning: failed to create .gitignore: {e}");
        }
    }

    println!();
    println!("Next steps:");
    println!("  1. Edit {CONFIG_FILENAME} to configure your standards");
    println!("  2. Run 'apss install' to build the project CLI");
    println!("  3. Run 'apss run <standard> <command>' to use it");

    0
}

fn parse_standard_spec(spec: &str) -> (String, String) {
    if let Some((slug, version)) = spec.split_once('@') {
        (slug.to_string(), version.to_string())
    } else {
        (spec.to_string(), ">=0.1.0".to_string())
    }
}
