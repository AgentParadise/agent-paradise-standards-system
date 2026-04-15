use documentation::config::DocsConfig;
use documentation::context::validate_root_context;
use documentation::error_codes;
use documentation::readme::validate_readmes;
use documentation::DocValidator;
use aps_core::Diagnostics;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_missing_readme_in_docs_dir() {
    let dir = tempdir().unwrap();
    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    // No README.md

    let config = DocsConfig::default();
    let mut diagnostics = Diagnostics::new();
    validate_readmes(dir.path(), &config, &mut diagnostics);

    assert!(diagnostics
        .errors()
        .any(|d| d.code == error_codes::MISSING_README));
}

#[test]
fn test_readme_present_passes() {
    let dir = tempdir().unwrap();
    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(docs_dir.join("README.md"), "# Docs\n").unwrap();
    fs::write(docs_dir.join("CLAUDE.md"), "See README.md").unwrap();
    fs::write(docs_dir.join("AGENTS.md"), "See README.md").unwrap();

    let config = DocsConfig::default();
    let mut diagnostics = Diagnostics::new();
    validate_readmes(dir.path(), &config, &mut diagnostics);

    assert!(
        !diagnostics.has_errors(),
        "Unexpected errors: {:?}",
        diagnostics.errors().collect::<Vec<_>>()
    );
}

#[test]
fn test_missing_claude_md_warning() {
    let dir = tempdir().unwrap();
    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(docs_dir.join("README.md"), "# Docs\n").unwrap();
    fs::write(docs_dir.join("AGENTS.md"), "See README.md").unwrap();
    // No CLAUDE.md

    let config = DocsConfig::default();
    let mut diagnostics = Diagnostics::new();
    validate_readmes(dir.path(), &config, &mut diagnostics);

    assert!(diagnostics
        .warnings()
        .any(|d| d.code == error_codes::MISSING_CLAUDE_MD));
}

#[test]
fn test_missing_agents_md_warning() {
    let dir = tempdir().unwrap();
    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(docs_dir.join("README.md"), "# Docs\n").unwrap();
    fs::write(docs_dir.join("CLAUDE.md"), "See README.md").unwrap();
    // No AGENTS.md

    let config = DocsConfig::default();
    let mut diagnostics = Diagnostics::new();
    validate_readmes(dir.path(), &config, &mut diagnostics);

    assert!(diagnostics
        .warnings()
        .any(|d| d.code == error_codes::MISSING_AGENTS_MD));
}

#[test]
fn test_subdirectory_readme_enforcement() {
    let dir = tempdir().unwrap();
    let docs_dir = dir.path().join("docs");
    let sub_dir = docs_dir.join("guides");
    fs::create_dir_all(&sub_dir).unwrap();
    fs::write(docs_dir.join("README.md"), "# Docs\n").unwrap();
    fs::write(docs_dir.join("CLAUDE.md"), "context").unwrap();
    fs::write(docs_dir.join("AGENTS.md"), "context").unwrap();
    // sub_dir has no README

    let config = DocsConfig::default();
    let mut diagnostics = Diagnostics::new();
    validate_readmes(dir.path(), &config, &mut diagnostics);

    assert!(diagnostics
        .errors()
        .any(|d| d.code == error_codes::MISSING_README
            && d.message.contains("guides")));
}

#[test]
fn test_exclude_dirs_respected() {
    let dir = tempdir().unwrap();
    let docs_dir = dir.path().join("docs");
    let excluded = docs_dir.join("node_modules");
    fs::create_dir_all(&excluded).unwrap();
    fs::write(docs_dir.join("README.md"), "# Docs\n").unwrap();
    fs::write(docs_dir.join("CLAUDE.md"), "context").unwrap();
    fs::write(docs_dir.join("AGENTS.md"), "context").unwrap();
    // node_modules has no README, but should be excluded

    let config = DocsConfig::default();
    let mut diagnostics = Diagnostics::new();
    validate_readmes(dir.path(), &config, &mut diagnostics);

    // Should NOT report missing README for node_modules
    assert!(!diagnostics
        .errors()
        .any(|d| d.message.contains("node_modules")));
}

#[test]
fn test_readme_disabled_skips_all() {
    let dir = tempdir().unwrap();
    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).unwrap();
    // No README at all

    let mut config = DocsConfig::default();
    config.readme.enabled = false;

    let mut diagnostics = Diagnostics::new();
    validate_readmes(dir.path(), &config, &mut diagnostics);

    assert!(!diagnostics.has_errors());
}

// ─── Root context tests ────────────────────────────────────────────────────

#[test]
fn test_root_context_missing_claude() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "# Agents").unwrap();

    let config = DocsConfig::default();
    let mut diagnostics = Diagnostics::new();
    validate_root_context(dir.path(), &config, &mut diagnostics);

    assert!(diagnostics
        .errors()
        .any(|d| d.code == error_codes::MISSING_ROOT_CLAUDE_MD));
}

#[test]
fn test_root_context_missing_agents() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("CLAUDE.md"), "# Claude\n\ndocs/ has docs").unwrap();

    let config = DocsConfig::default();
    let mut diagnostics = Diagnostics::new();
    validate_root_context(dir.path(), &config, &mut diagnostics);

    assert!(diagnostics
        .errors()
        .any(|d| d.code == error_codes::MISSING_ROOT_AGENTS_MD));
}

#[test]
fn test_root_context_missing_docs_reference() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("CLAUDE.md"), "# Claude\n\nNo reference here.").unwrap();
    fs::write(dir.path().join("AGENTS.md"), "# Agents").unwrap();

    let config = DocsConfig::default();
    let mut diagnostics = Diagnostics::new();
    validate_root_context(dir.path(), &config, &mut diagnostics);

    assert!(diagnostics
        .warnings()
        .any(|d| d.code == error_codes::MISSING_DOCS_REFERENCE));
}

#[test]
fn test_root_context_valid() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("CLAUDE.md"),
        "# Claude\n\nTechnical documentation is in docs/ following APSS standard.",
    )
    .unwrap();
    fs::write(dir.path().join("AGENTS.md"), "# Agents").unwrap();

    let config = DocsConfig::default();
    let mut diagnostics = Diagnostics::new();
    validate_root_context(dir.path(), &config, &mut diagnostics);

    assert!(!diagnostics.has_errors());
    assert!(!diagnostics.has_warnings());
}

// ─── Full validator integration ────────────────────────────────────────────

#[test]
fn test_full_validator_disabled() {
    let dir = tempdir().unwrap();

    let mut config = DocsConfig::default();
    config.enabled = false;

    let validator = DocValidator::with_config(dir.path(), config);
    let diagnostics = validator.validate();

    assert!(diagnostics.is_empty());
}
