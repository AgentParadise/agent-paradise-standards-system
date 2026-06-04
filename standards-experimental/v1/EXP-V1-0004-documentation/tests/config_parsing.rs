//! Integration tests for the EXP-V1-0004 config loader.
//!
//! Per the unified-config brief (2026-06-04), configuration lives in
//! `apss.yaml` at the project root under the `docs` section. These tests
//! exercise the YAML loader, the `disable` flag semantics, partial config
//! defaults, and tolerance of unrelated top-level sections owned by other
//! standards.

use documentation::config::{ApssConfig, CONFIG_FILENAME, load_config};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_missing_config_returns_defaults() {
    let dir = tempdir().unwrap();
    let config = load_config(dir.path()).unwrap();

    assert!(!config.docs.disable);
    assert_eq!(config.docs.root, "docs");
    assert!(!config.docs.adr.disable);
    assert_eq!(config.docs.adr.directory, "adrs");
    assert!(!config.docs.readme.disable);
    assert!(!config.docs.root_context.disable);
}

#[test]
fn test_full_config_parsing() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join(CONFIG_FILENAME),
        r#"
docs:
  disable: false
  root: documentation
  adr:
    disable: false
    directory: decisions
    naming_pattern: "DEC_\\d{3}_[a-z]+\\.md"
    required_adr_keywords:
      - init
  readme:
    disable: true
    max_depth: 3
    exclude_dirs:
      - build
  root_context:
    disable: false
    docs_reference_pattern: documentation/
  backlinking:
    disable: true
"#,
    )
    .unwrap();

    let config = load_config(dir.path()).unwrap();

    assert_eq!(config.docs.root, "documentation");
    assert_eq!(config.docs.adr.directory, "decisions");
    assert_eq!(config.docs.adr.naming_pattern, "DEC_\\d{3}_[a-z]+\\.md");
    assert_eq!(config.docs.adr.required_adr_keywords, vec!["init"]);
    assert!(config.docs.backlinking.disable);
    assert!(config.docs.readme.disable);
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
    fs::write(
        dir.path().join(CONFIG_FILENAME),
        r#"
docs:
  root: my-docs
"#,
    )
    .unwrap();

    let config = load_config(dir.path()).unwrap();

    assert_eq!(config.docs.root, "my-docs");
    // All other fields should be defaults.
    assert!(!config.docs.adr.disable);
    assert_eq!(config.docs.adr.directory, "adrs");
    assert!(!config.docs.readme.disable);
}

#[test]
fn test_invalid_yaml_returns_error() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join(CONFIG_FILENAME),
        "this is not valid yaml: : :",
    )
    .unwrap();

    let result = load_config(dir.path());
    assert!(result.is_err());
}

#[test]
fn test_default_config_struct() {
    let config = ApssConfig::default();
    assert!(!config.docs.disable);
    assert_eq!(config.docs.root, "docs");
    assert!(!config.docs.index.disable);
    assert!(config.docs.index.auto_generate);
    assert!(config.docs.context_files.require_claude_md);
    assert!(config.docs.context_files.require_agents_md);
}

#[test]
fn test_docs_disabled() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join(CONFIG_FILENAME),
        r#"
docs:
  disable: true
"#,
    )
    .unwrap();

    let config = load_config(dir.path()).unwrap();
    assert!(config.docs.disable);
}

#[test]
fn test_other_standards_sections_are_tolerated() {
    // The meta-standard owns `apss.yaml`. Other standards (fitness, topology,
    // ...) register their own top-level sections. The docs loader must
    // ignore those sections rather than fail the whole load.
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join(CONFIG_FILENAME),
        r#"
fitness:
  disable: false
  threshold: 42
topology:
  disable: true
docs:
  root: project-docs
"#,
    )
    .unwrap();

    let config = load_config(dir.path()).unwrap();
    assert_eq!(config.docs.root, "project-docs");
    // Defaults for everything we did not explicitly set.
    assert!(!config.docs.disable);
    assert!(!config.docs.adr.disable);
}

#[test]
fn test_missing_docs_section_returns_defaults() {
    // A project may activate other standards without configuring docs.
    // An apss.yaml with no `docs:` key MUST produce the docs defaults.
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join(CONFIG_FILENAME),
        r#"
fitness:
  disable: false
"#,
    )
    .unwrap();

    let config = load_config(dir.path()).unwrap();
    assert!(!config.docs.disable);
    assert_eq!(config.docs.root, "docs");
}
