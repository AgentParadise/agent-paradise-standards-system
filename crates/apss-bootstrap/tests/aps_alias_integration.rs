//! Integration tests for the deprecated `aps` alias binary.
//!
//! `aps` must behave exactly like `apss` except for one added deprecation
//! warning on stderr, so these tests spawn the real `aps` binary.

use std::process::Command;

const APS_BIN: &str = env!("CARGO_BIN_EXE_aps");

#[test]
fn test_aps_prints_deprecation_warning_on_stderr() {
    let output = Command::new(APS_BIN)
        .arg("--version")
        .output()
        .expect("failed to invoke aps --version");

    assert!(
        output.status.success(),
        "aps --version exited non-zero: {:?}",
        output.status
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("deprecated"),
        "expected a deprecation warning on stderr, got:\n{stderr}"
    );
    assert!(
        stderr.contains("apss"),
        "deprecation warning should point users at apss, got:\n{stderr}"
    );
}

#[test]
fn test_aps_status_delegates_to_same_cli_as_apss() {
    let temp = tempfile::tempdir().unwrap();

    let output = Command::new(APS_BIN)
        .arg("status")
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke aps status");

    assert_eq!(
        output.status.code(),
        Some(1),
        "aps status should fail like apss status does with no apss.yaml"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No apss.yaml found"),
        "expected the same missing-config message apss prints, got:\n{stderr}"
    );
    assert!(
        stderr.contains("deprecated"),
        "expected the deprecation warning alongside the normal command output, got:\n{stderr}"
    );
}
