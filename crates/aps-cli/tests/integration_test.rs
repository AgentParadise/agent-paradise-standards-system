//! End-to-end integration tests for the APS CLI.
//!
//! These tests verify the complete workflow of creating, validating,
//! and managing standards.

use std::fs;
use std::process::Command;

/// Get the path to the compiled aps binary.
fn aps_binary() -> std::path::PathBuf {
    // During tests, the binary is in target/debug/aps
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove deps
    path.push("aps");
    path
}

/// Create a temporary test repository with minimal structure.
fn create_test_repo(temp_dir: &std::path::Path) {
    // Create workspace Cargo.toml
    let cargo_toml = r#"
[workspace]
resolver = "2"
members = []

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
aps-core = { path = "../../crates/aps-core" }
"#;
    fs::write(temp_dir.join("Cargo.toml"), cargo_toml).unwrap();

    // Create standards directories
    fs::create_dir_all(temp_dir.join("standards/v1")).unwrap();
    fs::create_dir_all(temp_dir.join("standards-experimental/v1")).unwrap();
}

#[test]
fn test_cli_help() {
    let output = Command::new(aps_binary())
        .arg("--help")
        .output()
        .expect("Failed to execute aps");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Agent Paradise Standards System CLI"));
    assert!(stdout.contains("v1"));
}

#[test]
fn test_cli_v1_help() {
    let output = Command::new(aps_binary())
        .args(["v1", "--help"])
        .output()
        .expect("Failed to execute aps v1");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("validate"));
    assert!(stdout.contains("create"));
    assert!(stdout.contains("promote"));
    assert!(stdout.contains("version"));
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("list"));
}

#[test]
fn test_cli_list_empty_repo() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_repo(temp_dir.path());

    let output = Command::new(aps_binary())
        .args(["v1", "list"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute aps v1 list");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0 total"));
}

#[test]
fn test_cli_validate_repo_empty() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_repo(temp_dir.path());

    let output = Command::new(aps_binary())
        .args(["v1", "validate", "repo"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute aps v1 validate repo");

    // Empty repo with proper structure should pass
    assert!(output.status.success());
}

#[test]
fn test_version_show_not_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_repo(temp_dir.path());

    let output = Command::new(aps_binary())
        .args(["v1", "version", "show", "APS-V1-9999"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute aps v1 version show");

    // Should fail because package doesn't exist
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found"));
}
