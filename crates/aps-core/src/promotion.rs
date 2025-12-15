//! Promotion engine for graduating experiments to standards.
//!
//! Handles the workflow of promoting an experimental standard (EXP-V1-XXXX)
//! to an official standard (APS-V1-XXXX).

use crate::discovery::{PackageType, discover_v1_packages};
use crate::metadata::parse_experiment_metadata;
use std::fs;
use std::path::{Path, PathBuf};

/// Result of a promotion operation.
#[derive(Debug)]
pub struct PromotionResult {
    /// The original experiment ID.
    pub experiment_id: String,
    /// The new standard ID.
    pub standard_id: String,
    /// Path to the new standard package.
    pub new_path: PathBuf,
    /// Files that were migrated.
    pub migrated_files: Vec<PathBuf>,
}

/// Errors that can occur during promotion.
#[derive(Debug, thiserror::Error)]
pub enum PromotionError {
    /// Experiment not found.
    #[error("experiment not found: {0}")]
    ExperimentNotFound(String),

    /// Target standard ID already exists.
    #[error("standard ID already exists: {0}")]
    StandardIdExists(String),

    /// Metadata parsing error.
    #[error("metadata error: {0}")]
    Metadata(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Validation failed.
    #[error("validation failed: {0}")]
    ValidationFailed(String),
}

/// Promote an experiment to an official standard.
///
/// # Arguments
///
/// * `repo_root` - Path to the repository root
/// * `experiment_id` - The experiment ID (e.g., "EXP-V1-0001")
/// * `target_standard_id` - Optional specific standard ID, otherwise auto-allocated
///
/// # Returns
///
/// A `PromotionResult` with details about the promotion.
pub fn promote_experiment(
    repo_root: &Path,
    experiment_id: &str,
    target_standard_id: Option<&str>,
) -> Result<PromotionResult, PromotionError> {
    // Find the experiment
    let packages = discover_v1_packages(repo_root);
    let experiment = packages
        .iter()
        .find(|p| {
            p.package_type == PackageType::Experiment
                && p.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|name| name.starts_with(experiment_id))
        })
        .ok_or_else(|| PromotionError::ExperimentNotFound(experiment_id.to_string()))?;

    // Parse experiment metadata
    let exp_metadata = parse_experiment_metadata(&experiment.path.join("experiment.toml"))
        .map_err(|e| {
            PromotionError::Metadata(format!("failed to parse experiment metadata: {}", e))
        })?;

    // Determine the target standard ID
    let standard_id = match target_standard_id {
        Some(id) => id.to_string(),
        None => allocate_next_standard_id(repo_root),
    };

    // Check if target already exists
    let existing = packages.iter().any(|p| {
        p.package_type == PackageType::Standard
            && p.path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name.starts_with(&standard_id))
    });

    if existing {
        return Err(PromotionError::StandardIdExists(standard_id));
    }

    // Create the new standard directory
    let slug = &exp_metadata.experiment.slug;
    let new_dir_name = format!("{}-{}", standard_id, slug);
    let new_path = repo_root.join("standards/v1").join(&new_dir_name);

    // Copy the experiment directory
    copy_dir_all(&experiment.path, &new_path)?;

    let mut migrated_files = Vec::new();

    // Convert experiment.toml to standard.toml
    let exp_toml_path = new_path.join("experiment.toml");
    let std_toml_path = new_path.join("standard.toml");

    if exp_toml_path.exists() {
        let standard_toml = generate_standard_toml(
            &standard_id,
            &exp_metadata.experiment.name,
            slug,
            &exp_metadata.experiment.version,
            &exp_metadata.experiment.category,
            &exp_metadata.ownership.maintainers,
            experiment_id,
        );

        fs::write(&std_toml_path, standard_toml)?;
        fs::remove_file(&exp_toml_path)?;
        migrated_files.push(std_toml_path);
    }

    // Update Cargo.toml package name
    let cargo_toml_path = new_path.join("Cargo.toml");
    if cargo_toml_path.exists() {
        let content = fs::read_to_string(&cargo_toml_path)?;
        // Update the package name to match the new standard
        let new_crate_name = slug.replace('-', "_");
        let updated = content
            .lines()
            .map(|line| {
                if line.starts_with("name = ") {
                    format!("name = \"{}\"", new_crate_name)
                } else if line.contains("(Experimental)") {
                    line.replace("(Experimental)", "")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&cargo_toml_path, updated)?;
        migrated_files.push(cargo_toml_path);
    }

    // Update spec doc to remove experimental warnings
    let spec_path = new_path.join("docs/01_spec.md");
    if spec_path.exists() {
        let content = fs::read_to_string(&spec_path)?;
        let updated = content
            .replace("(Experimental Specification)", "(Canonical Specification)")
            .replace("**Status**: Experimental", "**Status**: Active")
            .replace(
                "⚠️ **EXPERIMENTAL**: This standard is in incubation and may change significantly before promotion.\n\n---",
                "",
            );

        // Also update the ID in the header
        let updated = updated.replace(experiment_id, &standard_id);

        fs::write(&spec_path, updated)?;
        migrated_files.push(spec_path);
    }

    Ok(PromotionResult {
        experiment_id: experiment_id.to_string(),
        standard_id,
        new_path,
        migrated_files,
    })
}

/// Allocate the next available standard ID.
fn allocate_next_standard_id(repo_root: &Path) -> String {
    let packages = discover_v1_packages(repo_root);

    let max_id = packages
        .iter()
        .filter(|p| p.package_type == PackageType::Standard)
        .filter_map(|p| {
            p.path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|name| {
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

/// Generate a standard.toml from experiment metadata.
fn generate_standard_toml(
    id: &str,
    name: &str,
    slug: &str,
    version: &str,
    category: &str,
    maintainers: &[String],
    promoted_from: &str,
) -> String {
    let maintainers_str = maintainers
        .iter()
        .map(|m| format!("\"{}\"", m))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"schema = "aps.standard/v1"

[standard]
id = "{id}"
name = "{name}"
slug = "{slug}"
version = "{version}"
category = "{category}"
status = "active"

[aps]
aps_major = "v1"
backwards_compatible_major_required = true

[ownership]
maintainers = [{maintainers_str}]

# Promotion metadata
[promotion]
promoted_from = "{promoted_from}"
promoted_at = "{date}"
"#,
        date = chrono_lite_date()
    )
}

/// Get current date in YYYY-MM-DD format (without external deps).
pub fn chrono_lite_date() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Simple date calculation (not accounting for leap seconds, etc.)
    let days = secs / 86400;
    let mut year = 1970u32;
    let mut remaining_days = days as u32;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [u32; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for days_in_month in days_in_months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }

    let day = remaining_days + 1;

    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Recursively copy a directory.
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrono_lite_date() {
        let date = chrono_lite_date();
        // Should be YYYY-MM-DD format
        assert_eq!(date.len(), 10);
        assert_eq!(&date[4..5], "-");
        assert_eq!(&date[7..8], "-");
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(1900));
    }

    #[test]
    fn test_generate_standard_toml() {
        let toml = generate_standard_toml(
            "APS-V1-0001",
            "Test Standard",
            "test-standard",
            "1.0.0",
            "governance",
            &["Alice".to_string(), "Bob".to_string()],
            "EXP-V1-0001",
        );

        assert!(toml.contains("APS-V1-0001"));
        assert!(toml.contains("Test Standard"));
        assert!(toml.contains("promoted_from = \"EXP-V1-0001\""));
        assert!(toml.contains("\"Alice\", \"Bob\""));
    }
}
