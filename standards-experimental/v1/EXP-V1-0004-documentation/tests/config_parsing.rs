use documentation::config::{load_config, ApssConfig};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_missing_config_returns_defaults() {
    let dir = tempdir().unwrap();
    let config = load_config(dir.path()).unwrap();

    assert!(config.docs.enabled);
    assert_eq!(config.docs.root, "docs");
    assert!(config.docs.adr.enabled);
    assert_eq!(config.docs.adr.directory, "adrs");
    assert!(config.docs.readme.enabled);
    assert!(config.docs.root_context.enabled);
}

#[test]
fn test_full_config_parsing() {
    let dir = tempdir().unwrap();
    let apss_dir = dir.path().join(".apss");
    fs::create_dir_all(&apss_dir).unwrap();
    fs::write(
        apss_dir.join("config.toml"),
        r#"
schema = "apss.config/v1"

[docs]
enabled = true
root = "documentation"

[docs.adr]
enabled = true
directory = "decisions"
naming_pattern = "DEC_\\d{3}_[a-z]+\\.md"
required_adr_keywords = ["init"]
backlinking = false

[docs.readme]
enabled = false
max_depth = 3
exclude_dirs = ["build"]

[docs.root_context]
enabled = true
docs_reference_pattern = "documentation/"
"#,
    )
    .unwrap();

    let config = load_config(dir.path()).unwrap();

    assert_eq!(config.docs.root, "documentation");
    assert_eq!(config.docs.adr.directory, "decisions");
    assert_eq!(
        config.docs.adr.naming_pattern,
        "DEC_\\d{3}_[a-z]+\\.md"
    );
    assert_eq!(config.docs.adr.required_adr_keywords, vec!["init"]);
    assert!(!config.docs.adr.backlinking);
    assert!(!config.docs.readme.enabled);
    assert_eq!(config.docs.readme.max_depth, 3);
    assert_eq!(config.docs.readme.exclude_dirs, vec!["build"]);
    assert_eq!(
        config.docs.root_context.docs_reference_pattern,
        "documentation/"
    );
}

#[test]
fn test_partial_config_fills_defaults() {
    let dir = tempdir().unwrap();
    let apss_dir = dir.path().join(".apss");
    fs::create_dir_all(&apss_dir).unwrap();
    fs::write(
        apss_dir.join("config.toml"),
        r#"
[docs]
root = "my-docs"
"#,
    )
    .unwrap();

    let config = load_config(dir.path()).unwrap();

    assert_eq!(config.docs.root, "my-docs");
    // All other fields should be defaults
    assert!(config.docs.adr.enabled);
    assert_eq!(config.docs.adr.directory, "adrs");
    assert!(config.docs.readme.enabled);
}

#[test]
fn test_invalid_toml_returns_error() {
    let dir = tempdir().unwrap();
    let apss_dir = dir.path().join(".apss");
    fs::create_dir_all(&apss_dir).unwrap();
    fs::write(apss_dir.join("config.toml"), "this is not valid toml [[[").unwrap();

    let result = load_config(dir.path());
    assert!(result.is_err());
}

#[test]
fn test_default_config_struct() {
    let config = ApssConfig::default();
    assert!(config.docs.enabled);
    assert_eq!(config.docs.root, "docs");
    assert!(config.docs.index.enabled);
    assert!(config.docs.index.auto_generate);
    assert!(config.docs.context_files.require_claude_md);
    assert!(config.docs.context_files.require_agents_md);
}

#[test]
fn test_docs_disabled() {
    let dir = tempdir().unwrap();
    let apss_dir = dir.path().join(".apss");
    fs::create_dir_all(&apss_dir).unwrap();
    fs::write(
        apss_dir.join("config.toml"),
        r#"
[docs]
enabled = false
"#,
    )
    .unwrap();

    let config = load_config(dir.path()).unwrap();
    assert!(!config.docs.enabled);
}
