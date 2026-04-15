use documentation::config::DocsConfig;
use documentation_adr::AdrValidator;
use documentation_adr::error_codes;
use std::fs;
use tempfile::tempdir;

const ADR_CONTEXT_GUIDANCE: &str = "\
Files that implement an ADR should reference it in a comment block at the top of the file.\n\
Example: `// Implements ADR-001-security`\n\
This keeps agents and developers in context when making updates.\n";

fn setup_valid_adr_dir(root: &std::path::Path) {
    let adr_dir = root.join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();
    fs::write(
        adr_dir.join("ADR-001-initial-architecture.md"),
        "---\nname: \"Initial Architecture\"\ndescription: \"Foundational system design\"\nstatus: accepted\n---\n\n# Content\n",
    )
    .unwrap();
    fs::write(
        adr_dir.join("ADR-002-auth-strategy.md"),
        "---\nname: \"Auth Strategy\"\ndescription: \"Authentication approach\"\nstatus: accepted\n---\n\n# Content\n",
    )
    .unwrap();
    fs::write(adr_dir.join("CLAUDE.md"), ADR_CONTEXT_GUIDANCE).unwrap();
    fs::write(adr_dir.join("AGENTS.md"), ADR_CONTEXT_GUIDANCE).unwrap();
}

#[test]
fn test_valid_adrs_pass() {
    let dir = tempdir().unwrap();
    setup_valid_adr_dir(dir.path());

    let validator = AdrValidator::with_config(dir.path(), DocsConfig::default());
    let diagnostics = validator.validate();

    assert!(
        !diagnostics.has_errors(),
        "Expected no errors but got: {:?}",
        diagnostics.errors().collect::<Vec<_>>()
    );
}

#[test]
fn test_missing_adr_directory() {
    let dir = tempdir().unwrap();

    let validator = AdrValidator::with_config(dir.path(), DocsConfig::default());
    let diagnostics = validator.validate();

    assert!(diagnostics.has_errors());
    assert!(
        diagnostics
            .errors()
            .any(|d| d.code == error_codes::MISSING_ADR_DIR)
    );
}

#[test]
fn test_invalid_adr_naming() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();

    // Invalid: wrong naming pattern (underscores instead of hyphens)
    fs::write(
        adr_dir.join("bad-name.md"),
        "---\nname: \"Bad\"\ndescription: \"Bad name\"\nstatus: accepted\n---\n",
    )
    .unwrap();

    let validator = AdrValidator::with_config(dir.path(), DocsConfig::default());
    let diagnostics = validator.validate();

    assert!(
        diagnostics
            .errors()
            .any(|d| d.code == error_codes::INVALID_ADR_NAMING)
    );
}

#[test]
fn test_missing_frontmatter() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();

    fs::write(
        adr_dir.join("ADR-001-no-frontmatter.md"),
        "# Just a heading\n\nNo front matter block.",
    )
    .unwrap();

    let validator = AdrValidator::with_config(dir.path(), DocsConfig::default());
    let diagnostics = validator.validate();

    assert!(
        diagnostics
            .errors()
            .any(|d| d.code == error_codes::MISSING_ADR_FRONTMATTER)
    );
}

#[test]
fn test_incomplete_frontmatter() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();

    // Has name but no description
    fs::write(
        adr_dir.join("ADR-001-partial.md"),
        "---\nname: \"Has Name\"\n---\n\nContent.",
    )
    .unwrap();

    let validator = AdrValidator::with_config(dir.path(), DocsConfig::default());
    let diagnostics = validator.validate();

    assert!(
        diagnostics
            .errors()
            .any(|d| d.code == error_codes::MISSING_ADR_FRONTMATTER)
    );
}

#[test]
fn test_required_keyword_missing() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();

    let mut config = DocsConfig::default();
    config.adr.required_adr_keywords = vec!["security".to_string()];

    let validator = AdrValidator::with_config(dir.path(), config);
    let diagnostics = validator.validate();

    assert!(
        diagnostics
            .errors()
            .any(|d| d.code == error_codes::MISSING_REQUIRED_ADR)
    );
}

#[test]
fn test_required_keyword_satisfied() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();

    fs::write(
        adr_dir.join("ADR-001-security.md"),
        "---\nname: \"Security Architecture\"\ndescription: \"Security patterns\"\nstatus: accepted\n---\n",
    )
    .unwrap();

    let mut config = DocsConfig::default();
    config.adr.required_adr_keywords = vec!["security".to_string()];

    let validator = AdrValidator::with_config(dir.path(), config);
    let diagnostics = validator.validate();

    assert!(
        !diagnostics
            .errors()
            .any(|d| d.code == error_codes::MISSING_REQUIRED_ADR)
    );
}

#[test]
fn test_required_keyword_any_number_satisfies() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();

    // ADR-042-security.md should satisfy keyword "security" regardless of number
    fs::write(
        adr_dir.join("ADR-042-security.md"),
        "---\nname: \"Security v2\"\ndescription: \"Updated security\"\nstatus: accepted\n---\n",
    )
    .unwrap();

    let mut config = DocsConfig::default();
    config.adr.required_adr_keywords = vec!["security".to_string()];

    let validator = AdrValidator::with_config(dir.path(), config);
    let diagnostics = validator.validate();

    assert!(
        !diagnostics
            .errors()
            .any(|d| d.code == error_codes::MISSING_REQUIRED_ADR)
    );
}

#[test]
fn test_adr_disabled() {
    let dir = tempdir().unwrap();
    // No ADR directory at all

    let mut config = DocsConfig::default();
    config.adr.enabled = false;

    let validator = AdrValidator::with_config(dir.path(), config);
    let diagnostics = validator.validate();

    // Should produce no diagnostics when disabled
    assert!(!diagnostics.has_errors());
    assert!(!diagnostics.has_warnings());
}

#[test]
fn test_custom_naming_pattern() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();

    fs::write(
        adr_dir.join("DEC-01-test.md"),
        "---\nname: \"Test\"\ndescription: \"Custom\"\nstatus: accepted\n---\n",
    )
    .unwrap();

    let mut config = DocsConfig::default();
    config.adr.naming_pattern = r"DEC-\d{2}-[a-z]+\.md".to_string();

    let validator = AdrValidator::with_config(dir.path(), config);
    let diagnostics = validator.validate();

    assert!(
        !diagnostics
            .errors()
            .any(|d| d.code == error_codes::INVALID_ADR_NAMING)
    );
}

#[test]
fn test_readme_in_adr_dir_not_flagged() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();

    // Structural files should be excluded from naming validation
    fs::write(adr_dir.join("README.md"), "# ADRs\n\n## Index\n").unwrap();
    fs::write(adr_dir.join("CLAUDE.md"), ADR_CONTEXT_GUIDANCE).unwrap();
    fs::write(adr_dir.join("AGENTS.md"), ADR_CONTEXT_GUIDANCE).unwrap();
    fs::write(
        adr_dir.join("ADR-001-init.md"),
        "---\nname: \"Init\"\ndescription: \"Initial\"\nstatus: accepted\n---\n",
    )
    .unwrap();

    let validator = AdrValidator::with_config(dir.path(), DocsConfig::default());
    let diagnostics = validator.validate();

    assert!(
        !diagnostics.has_errors(),
        "Structural files should not trigger naming errors: {:?}",
        diagnostics.errors().collect::<Vec<_>>()
    );
}

#[test]
fn test_multiple_required_keywords() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();

    fs::write(
        adr_dir.join("ADR-001-security.md"),
        "---\nname: \"Security\"\ndescription: \"Security patterns\"\nstatus: accepted\n---\n",
    )
    .unwrap();
    // "testing" keyword NOT satisfied

    let mut config = DocsConfig::default();
    config.adr.required_adr_keywords = vec!["security".to_string(), "testing".to_string()];

    let validator = AdrValidator::with_config(dir.path(), config);
    let diagnostics = validator.validate();

    let missing_adr_errors: Vec<_> = diagnostics
        .errors()
        .filter(|d| d.code == error_codes::MISSING_REQUIRED_ADR)
        .collect();

    // Should have exactly one error (for "testing"), not for "security"
    assert_eq!(missing_adr_errors.len(), 1);
    assert!(missing_adr_errors[0].message.contains("testing"));
}

// ─── Context file tests (ADR01-007/008) ──────────────────────────────────

#[test]
fn test_missing_adr_context_files() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();
    fs::write(
        adr_dir.join("ADR-001-init.md"),
        "---\nname: \"Init\"\ndescription: \"Initial\"\nstatus: accepted\n---\n",
    )
    .unwrap();

    let validator = AdrValidator::with_config(dir.path(), DocsConfig::default());
    let diagnostics = validator.validate();

    let context_errors: Vec<_> = diagnostics
        .errors()
        .filter(|d| d.code == error_codes::MISSING_ADR_CONTEXT_FILE)
        .collect();

    // Both CLAUDE.md and AGENTS.md missing
    assert_eq!(context_errors.len(), 2);
}

#[test]
fn test_adr_context_files_present_with_guidance() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();
    fs::write(
        adr_dir.join("ADR-001-init.md"),
        "---\nname: \"Init\"\ndescription: \"Initial\"\nstatus: accepted\n---\n",
    )
    .unwrap();
    fs::write(adr_dir.join("CLAUDE.md"), ADR_CONTEXT_GUIDANCE).unwrap();
    fs::write(adr_dir.join("AGENTS.md"), ADR_CONTEXT_GUIDANCE).unwrap();

    let validator = AdrValidator::with_config(dir.path(), DocsConfig::default());
    let diagnostics = validator.validate();

    assert!(
        !diagnostics
            .errors()
            .any(|d| d.code == error_codes::MISSING_ADR_CONTEXT_FILE),
        "Context files are present — no missing-file errors expected"
    );
    assert!(
        !diagnostics
            .warnings()
            .any(|d| d.code == error_codes::ADR_CONTEXT_MISSING_GUIDANCE),
        "Context files have guidance — no missing-guidance warnings expected"
    );
}

#[test]
fn test_adr_context_files_without_guidance() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();
    fs::write(
        adr_dir.join("ADR-001-init.md"),
        "---\nname: \"Init\"\ndescription: \"Initial\"\nstatus: accepted\n---\n",
    )
    .unwrap();

    // Files exist but contain no ADR referencing guidance
    fs::write(
        adr_dir.join("CLAUDE.md"),
        "# ADR Context\n\nSome generic text.",
    )
    .unwrap();
    fs::write(
        adr_dir.join("AGENTS.md"),
        "# ADR Agents\n\nSome generic text.",
    )
    .unwrap();

    let validator = AdrValidator::with_config(dir.path(), DocsConfig::default());
    let diagnostics = validator.validate();

    // No missing-file errors
    assert!(
        !diagnostics
            .errors()
            .any(|d| d.code == error_codes::MISSING_ADR_CONTEXT_FILE),
    );

    // But should warn about missing guidance
    let guidance_warnings: Vec<_> = diagnostics
        .warnings()
        .filter(|d| d.code == error_codes::ADR_CONTEXT_MISSING_GUIDANCE)
        .collect();
    assert_eq!(guidance_warnings.len(), 2);
}

#[test]
fn test_partial_adr_context_files() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();
    fs::write(
        adr_dir.join("ADR-001-init.md"),
        "---\nname: \"Init\"\ndescription: \"Initial\"\nstatus: accepted\n---\n",
    )
    .unwrap();

    // Only CLAUDE.md present
    fs::write(adr_dir.join("CLAUDE.md"), ADR_CONTEXT_GUIDANCE).unwrap();

    let validator = AdrValidator::with_config(dir.path(), DocsConfig::default());
    let diagnostics = validator.validate();

    // AGENTS.md missing
    let context_errors: Vec<_> = diagnostics
        .errors()
        .filter(|d| d.code == error_codes::MISSING_ADR_CONTEXT_FILE)
        .collect();
    assert_eq!(context_errors.len(), 1);
    assert!(context_errors[0].message.contains("AGENTS.md"));
}

// ─── Fixture-based integration tests ─────────────────────────────────────

fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn fixture_valid_repo_passes() {
    let validator = AdrValidator::with_config(&fixture_path("valid_repo"), DocsConfig::default());
    let diagnostics = validator.validate();

    assert!(
        !diagnostics.has_errors(),
        "Valid fixture should pass, got errors: {:?}",
        diagnostics.errors().collect::<Vec<_>>()
    );
    assert!(
        !diagnostics.has_warnings(),
        "Valid fixture should have no warnings, got: {:?}",
        diagnostics.warnings().collect::<Vec<_>>()
    );
}

#[test]
fn fixture_invalid_naming_catches_bad_filenames() {
    let validator =
        AdrValidator::with_config(&fixture_path("invalid_naming"), DocsConfig::default());
    let diagnostics = validator.validate();

    let naming_errors: Vec<_> = diagnostics
        .errors()
        .filter(|d| d.code == error_codes::INVALID_ADR_NAMING)
        .collect();

    // adr_001_bad_underscores.md and no-number.md should both fail
    assert_eq!(
        naming_errors.len(),
        2,
        "Expected 2 naming errors, got {}: {:?}",
        naming_errors.len(),
        naming_errors
    );
}

#[test]
fn fixture_missing_frontmatter_catches_both_cases() {
    let validator =
        AdrValidator::with_config(&fixture_path("missing_frontmatter"), DocsConfig::default());
    let diagnostics = validator.validate();

    let fm_errors: Vec<_> = diagnostics
        .errors()
        .filter(|d| d.code == error_codes::MISSING_ADR_FRONTMATTER)
        .collect();

    // ADR-001-no-frontmatter.md: no front matter at all (1 error)
    // ADR-002-partial.md: missing description (1 error)
    // Total: 2 frontmatter errors (status errors are counted separately as ADR01-011)
    assert!(
        fm_errors.len() >= 2,
        "Expected at least 2 front matter errors, got {}: {:?}",
        fm_errors.len(),
        fm_errors
    );
}

#[test]
fn fixture_missing_context_files() {
    let validator =
        AdrValidator::with_config(&fixture_path("missing_context"), DocsConfig::default());
    let diagnostics = validator.validate();

    let context_errors: Vec<_> = diagnostics
        .errors()
        .filter(|d| d.code == error_codes::MISSING_ADR_CONTEXT_FILE)
        .collect();

    // Neither CLAUDE.md nor AGENTS.md exists in this fixture
    assert_eq!(context_errors.len(), 2);
}

#[test]
fn fixture_no_guidance_warns() {
    let validator = AdrValidator::with_config(&fixture_path("no_guidance"), DocsConfig::default());
    let diagnostics = validator.validate();

    // Files exist so no missing-file errors
    assert!(
        !diagnostics
            .errors()
            .any(|d| d.code == error_codes::MISSING_ADR_CONTEXT_FILE),
    );

    // But both lack guidance keywords
    let guidance_warnings: Vec<_> = diagnostics
        .warnings()
        .filter(|d| d.code == error_codes::ADR_CONTEXT_MISSING_GUIDANCE)
        .collect();
    assert_eq!(guidance_warnings.len(), 2);
}

#[test]
fn fixture_dead_adr_references() {
    let validator =
        AdrValidator::with_config(&fixture_path("dead_references"), DocsConfig::default());
    let diagnostics = validator.validate();

    let dead_refs: Vec<_> = diagnostics
        .warnings()
        .filter(|d| d.code == error_codes::DEAD_ADR_REFERENCE)
        .collect();

    // src/main.rs references ADR-999-nonexistent which doesn't exist
    assert_eq!(
        dead_refs.len(),
        1,
        "Expected 1 dead reference warning, got {}: {:?}",
        dead_refs.len(),
        dead_refs
    );
    assert!(dead_refs[0].message.contains("ADR-999-nonexistent"));
}

#[test]
fn fixture_valid_repo_no_dead_references() {
    let validator = AdrValidator::with_config(&fixture_path("valid_repo"), DocsConfig::default());
    let diagnostics = validator.validate();

    assert!(
        !diagnostics
            .warnings()
            .any(|d| d.code == error_codes::DEAD_ADR_REFERENCE),
        "Valid fixture should have no dead ADR references"
    );
}

#[test]
fn fixture_missing_headers() {
    let validator =
        AdrValidator::with_config(&fixture_path("missing_headers"), DocsConfig::default());
    let diagnostics = validator.validate();

    let header_warnings: Vec<_> = diagnostics
        .warnings()
        .filter(|d| d.code == error_codes::MISSING_ADR_HEADER)
        .collect();

    // ADR-001-no-structure.md: missing all 3 headers
    // ADR-002-partial-structure.md: missing Decision + Consequences (has Context)
    assert_eq!(
        header_warnings.len(),
        5,
        "Expected 5 header warnings (3 + 2), got {}: {:?}",
        header_warnings.len(),
        header_warnings
    );
}

// ─── Tempdir tests for dead references ───────────────────────────────────

#[test]
fn test_dead_reference_in_source_file() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    let src_dir = dir.path().join("src");
    fs::create_dir_all(&adr_dir).unwrap();
    fs::create_dir_all(&src_dir).unwrap();

    fs::write(
        adr_dir.join("ADR-001-auth.md"),
        "---\nname: \"Auth\"\ndescription: \"Auth\"\nstatus: accepted\n---\n\n## Context\n\n## Decision\n\n## Consequences\n",
    )
    .unwrap();
    fs::write(
        adr_dir.join("CLAUDE.md"),
        "Reference ADR- identifiers in comment blocks.",
    )
    .unwrap();
    fs::write(
        adr_dir.join("AGENTS.md"),
        "Reference ADR- identifiers in comment blocks.",
    )
    .unwrap();

    // Source file referencing valid + invalid ADR
    fs::write(
        src_dir.join("handler.rs"),
        "// Implements ADR-001-auth\n// See also ADR-050-deleted-feature\nfn handle() {}\n",
    )
    .unwrap();

    let validator = AdrValidator::with_config(dir.path(), DocsConfig::default());
    let diagnostics = validator.validate();

    let dead_refs: Vec<_> = diagnostics
        .warnings()
        .filter(|d| d.code == error_codes::DEAD_ADR_REFERENCE)
        .collect();

    assert_eq!(dead_refs.len(), 1);
    assert!(dead_refs[0].message.contains("ADR-050-deleted-feature"));
}

#[test]
fn test_no_dead_references_when_backlinking_disabled() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    let src_dir = dir.path().join("src");
    fs::create_dir_all(&adr_dir).unwrap();
    fs::create_dir_all(&src_dir).unwrap();

    fs::write(
        adr_dir.join("ADR-001-auth.md"),
        "---\nname: \"Auth\"\ndescription: \"Auth\"\nstatus: accepted\n---\n\n## Context\n\n## Decision\n\n## Consequences\n",
    )
    .unwrap();
    fs::write(adr_dir.join("CLAUDE.md"), "Reference ADR- identifiers.").unwrap();
    fs::write(adr_dir.join("AGENTS.md"), "Reference ADR- identifiers.").unwrap();

    // Dead reference, but backlinking is disabled
    fs::write(src_dir.join("main.rs"), "// ADR-999-ghost\nfn main() {}\n").unwrap();

    let mut config = DocsConfig::default();
    config.adr.backlinking = false;

    let validator = AdrValidator::with_config(dir.path(), config);
    let diagnostics = validator.validate();

    assert!(
        !diagnostics
            .warnings()
            .any(|d| d.code == error_codes::DEAD_ADR_REFERENCE),
        "Dead reference scanning should be skipped when backlinking is disabled"
    );
}

// ─── Tempdir tests for required headers ──────────────────────────────────

#[test]
fn test_adr_with_all_headers_passes() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();

    fs::write(
        adr_dir.join("ADR-001-complete.md"),
        "---\nname: \"Complete\"\ndescription: \"Has all headers\"\nstatus: accepted\n---\n\n\
         # ADR-001\n\n## Context\n\nContext.\n\n## Decision\n\nDecision.\n\n## Consequences\n\nConsequences.\n",
    )
    .unwrap();
    fs::write(adr_dir.join("CLAUDE.md"), "Reference ADR- identifiers.").unwrap();
    fs::write(adr_dir.join("AGENTS.md"), "Reference ADR- identifiers.").unwrap();

    let validator = AdrValidator::with_config(dir.path(), DocsConfig::default());
    let diagnostics = validator.validate();

    assert!(
        !diagnostics
            .warnings()
            .any(|d| d.code == error_codes::MISSING_ADR_HEADER),
        "ADR with all required headers should not trigger warnings"
    );
}

#[test]
fn test_adr_missing_consequences_header() {
    let dir = tempdir().unwrap();
    let adr_dir = dir.path().join("docs").join("adrs");
    fs::create_dir_all(&adr_dir).unwrap();

    fs::write(
        adr_dir.join("ADR-001-incomplete.md"),
        "---\nname: \"Incomplete\"\ndescription: \"Missing consequences\"\nstatus: accepted\n---\n\n\
         # ADR-001\n\n## Context\n\nContext.\n\n## Decision\n\nDecision.\n",
    )
    .unwrap();
    fs::write(adr_dir.join("CLAUDE.md"), "Reference ADR- identifiers.").unwrap();
    fs::write(adr_dir.join("AGENTS.md"), "Reference ADR- identifiers.").unwrap();

    let validator = AdrValidator::with_config(dir.path(), DocsConfig::default());
    let diagnostics = validator.validate();

    let header_warnings: Vec<_> = diagnostics
        .warnings()
        .filter(|d| d.code == error_codes::MISSING_ADR_HEADER)
        .collect();

    assert_eq!(header_warnings.len(), 1);
    assert!(header_warnings[0].message.contains("## Consequences"));
}
