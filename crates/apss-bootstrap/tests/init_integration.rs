//! Integration tests for `apss init`.
//!
//! The `apss` binary exposes `init` only via `main.rs`'s private `mod init;`,
//! so these tests spawn the real binary rather than calling `init::run`
//! directly. That also exercises the CLI dispatch path end-to-end.

use std::process::Command;

const APSS_BIN: &str = env!("CARGO_BIN_EXE_apss");

#[test]
fn test_init_creates_expected_layout() {
    let temp = tempfile::tempdir().unwrap();

    let status = Command::new(APSS_BIN)
        .arg("init")
        .current_dir(temp.path())
        .status()
        .expect("failed to invoke apss init");
    assert!(status.success(), "apss init exited non-zero: {status}");

    // APSS.yaml exists and contains expected fields
    let apss_yaml = std::fs::read_to_string(temp.path().join("APSS.yaml")).unwrap();
    assert!(
        apss_yaml.contains("schema: apss.project/v1"),
        "missing schema line:\n{apss_yaml}"
    );
    assert!(
        apss_yaml.contains("apss_version: v1"),
        "missing apss_version line:\n{apss_yaml}"
    );
    assert!(
        apss_yaml.contains("hooks:"),
        "missing hooks section:\n{apss_yaml}"
    );
    assert!(
        apss_yaml.contains("pre_commit: true"),
        "missing default pre_commit hook setting:\n{apss_yaml}"
    );

    // .apss/bin is generated; .apss/config is intentionally not created
    // because configuration is user-owned and lives outside .apss/.
    assert!(
        temp.path().join(".apss/bin").is_dir(),
        ".apss/bin is not a directory"
    );
    assert!(
        !temp.path().join(".apss/config").exists(),
        ".apss/config should not be created by apss init"
    );

    // .apss/.gitignore holds only build artifacts, not configs
    let apss_gitignore = std::fs::read_to_string(temp.path().join(".apss/.gitignore")).unwrap();
    assert_eq!(apss_gitignore, "build/\nbin/\n");

    // Regression guard for removed `--existing` behavior: root .gitignore
    // must not be created when one wasn't there already.
    assert!(
        !temp.path().join(".gitignore").exists(),
        "apss init should not create a root .gitignore"
    );
}

#[test]
fn test_init_preserves_existing_root_gitignore() {
    let temp = tempfile::tempdir().unwrap();
    let sentinel = "# user content\ntarget/\n";
    std::fs::write(temp.path().join(".gitignore"), sentinel).unwrap();

    let status = Command::new(APSS_BIN)
        .arg("init")
        .current_dir(temp.path())
        .status()
        .expect("failed to invoke apss init");
    assert!(status.success(), "apss init exited non-zero: {status}");

    let after = std::fs::read_to_string(temp.path().join(".gitignore")).unwrap();
    assert_eq!(after, sentinel, "root .gitignore was modified by apss init");
}

#[test]
fn test_init_with_standard_flag() {
    let temp = tempfile::tempdir().unwrap();

    let status = Command::new(APSS_BIN)
        .args(["init", "--standard", "code-topology@>=1.0.0"])
        .current_dir(temp.path())
        .status()
        .expect("failed to invoke apss init --standard");
    assert!(status.success(), "apss init exited non-zero: {status}");

    let apss_yaml = std::fs::read_to_string(temp.path().join("APSS.yaml")).unwrap();
    assert!(
        apss_yaml.contains("  code-topology:"),
        "missing standards.code-topology section:\n{apss_yaml}"
    );
    assert!(
        apss_yaml.contains("version: \">=1.0.0\""),
        "missing version requirement:\n{apss_yaml}"
    );
    assert!(
        apss_yaml.contains("FIXME"),
        "missing FIXME placeholder hint for standard id:\n{apss_yaml}"
    );
}
