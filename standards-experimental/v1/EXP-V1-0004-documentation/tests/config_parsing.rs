//! Integration tests for the EXP-V1-0004 config loader.
//!
//! Per the unified-config brief (2026-06-04), configuration lives in
//! `apss.yaml` at the project root under the `docs` section. These tests
//! exercise the YAML loader, the `disable` flag semantics, partial config
//! defaults, and tolerance of unrelated top-level sections owned by other
//! standards.

use apss_core::config::CONFIG_FILENAME;
use documentation::config::{ApssConfig, load_config};
use std::fs;
use tempfile::tempdir;

fn assert_docs_defaults(docs: &documentation::config::DocsConfig) {
    let expected = ApssConfig::default().docs;

    assert_eq!(docs.root, expected.root);
    assert!(!docs.disable);

    assert!(!docs.index.disable);
    assert_eq!(docs.index.auto_generate, expected.index.auto_generate);
    assert_eq!(
        docs.index.frontmatter_fields,
        expected.index.frontmatter_fields
    );

    assert!(!docs.readme.disable);
    assert_eq!(docs.readme.max_depth, expected.readme.max_depth);
    assert_eq!(docs.readme.exclude_dirs, expected.readme.exclude_dirs);

    assert!(!docs.root_context.disable);
    assert_eq!(
        docs.root_context.docs_reference_pattern,
        expected.root_context.docs_reference_pattern
    );

    assert!(!docs.adr.disable);
    assert_eq!(docs.adr.directory, expected.adr.directory);
    assert_eq!(docs.adr.naming_pattern, expected.adr.naming_pattern);
    assert_eq!(
        docs.adr.required_adr_keywords,
        expected.adr.required_adr_keywords
    );

    assert!(!docs.backlinking.disable);
    assert_eq!(docs.backlinking.scan, None);
    assert!(docs.backlinking.file_types.is_empty());

    assert!(!docs.north_star.disable);
    assert_eq!(docs.north_star.location, expected.north_star.location);

    assert!(!docs.retrospectives.disable);
    assert_eq!(
        docs.retrospectives.directory,
        expected.retrospectives.directory
    );
    assert_eq!(
        docs.retrospectives.naming_pattern,
        expected.retrospectives.naming_pattern
    );

    assert_eq!(
        docs.context_files.require_claude_md,
        expected.context_files.require_claude_md
    );
    assert_eq!(
        docs.context_files.require_agents_md,
        expected.context_files.require_agents_md
    );
}

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
  root: documentation
  adr:
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
    docs_reference_pattern: documentation/
  backlinking:
    disable: true
    scan:
      - "**/*.rs"
      - "notes/**/*.md"
"#,
    )
    .unwrap();

    let config = load_config(dir.path()).unwrap();

    assert_eq!(config.docs.root, "documentation");
    assert_eq!(config.docs.adr.directory, "decisions");
    assert_eq!(config.docs.adr.naming_pattern, "DEC_\\d{3}_[a-z]+\\.md");
    assert_eq!(config.docs.adr.required_adr_keywords, vec!["init"]);
    assert!(config.docs.backlinking.disable);
    assert_eq!(
        config.docs.backlinking.scan,
        Some(vec!["**/*.rs".to_string(), "notes/**/*.md".to_string()])
    );
    assert!(config.docs.backlinking.file_types.is_empty());
    assert!(config.docs.readme.disable);
    assert_eq!(config.docs.readme.max_depth, 3);
    assert_eq!(config.docs.readme.exclude_dirs, vec!["build"]);
    assert_eq!(
        config.docs.root_context.docs_reference_pattern,
        "documentation/"
    );
}

#[test]
fn test_load_config_uses_canonical_uppercase_filename() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join(CONFIG_FILENAME),
        r#"
docs:
  root: documentation
"#,
    )
    .unwrap();

    let config = load_config(dir.path()).unwrap();
    assert_eq!(config.docs.root, "documentation");
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
  threshold: 42
"#,
    )
    .unwrap();

    let config = load_config(dir.path()).unwrap();
    assert!(!config.docs.disable);
    assert_eq!(config.docs.root, "docs");
}

#[test]
fn test_empty_docs_block_matches_defaults() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join(CONFIG_FILENAME),
        r#"
docs: {}
"#,
    )
    .unwrap();
    let config = load_config(dir.path()).unwrap();

    assert_docs_defaults(&config.docs);
}

#[test]
fn test_absence_and_empty_docs_block_match() {
    let no_docs_dir = tempdir().unwrap();
    let no_docs = load_config(no_docs_dir.path()).unwrap();

    let no_docs_key_dir = tempdir().unwrap();
    fs::write(
        no_docs_key_dir.path().join(CONFIG_FILENAME),
        r#"
fitness:
  threshold: 42
"#,
    )
    .unwrap();
    let no_docs_key = load_config(no_docs_key_dir.path()).unwrap();

    let empty_docs_dir = tempdir().unwrap();
    fs::write(
        empty_docs_dir.path().join(CONFIG_FILENAME),
        r#"
docs: {}
"#,
    )
    .unwrap();
    let empty_docs = load_config(empty_docs_dir.path()).unwrap();

    assert_docs_defaults(&no_docs.docs);
    assert_docs_defaults(&no_docs_key.docs);
    assert_docs_defaults(&empty_docs.docs);
    assert_eq!(no_docs.docs.root, no_docs_key.docs.root);
    assert_eq!(no_docs.docs.root, empty_docs.docs.root);
}
