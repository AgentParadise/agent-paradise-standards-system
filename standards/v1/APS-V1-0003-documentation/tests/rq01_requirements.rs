//! Validator tests for the Requirements substandard (APS-V1-0003.RQ01).
//!
//! RQ01 is optional: a repo without a requirements document passes, a
//! declared location must exist, and an existing document must conform.

#![cfg(feature = "RQ01")]

use documentation::config::{DocsConfig, load_config};
use documentation::substandards::requirements::{DEFAULT_LOCATION, error_codes, validate};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

const EXAMPLE: &str = include_str!("../examples/requirements.md");

fn codes(repo: &Path, docs: &DocsConfig) -> Vec<String> {
    validate(repo, docs)
        .iter()
        .map(|d| d.code.clone())
        .collect()
}

/// A repo with docs/north-star.md and the given requirements doc.
fn repo_with(doc: &str) -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    let docs = dir.path().join("docs");
    fs::create_dir_all(&docs).unwrap();
    fs::write(docs.join("north-star.md"), "---\nname: x\n---\n").unwrap();
    fs::write(docs.join(DEFAULT_LOCATION), doc).unwrap();
    dir
}

#[test]
fn repo_without_requirements_passes() {
    let dir = tempdir().unwrap();
    assert!(codes(dir.path(), &DocsConfig::default()).is_empty());
}

#[test]
fn worked_example_is_valid() {
    let dir = repo_with(EXAMPLE);
    let report = validate(dir.path(), &DocsConfig::default());
    assert_eq!(report.len(), 0, "example must validate cleanly: {report}");
}

#[test]
fn declared_location_must_exist() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("apss.yaml"),
        "docs:\n  requirements:\n    locations:\n      - requirements.md\n      - systems/billing/requirements.md\n",
    )
    .unwrap();
    let docs = load_config(dir.path()).unwrap().docs;
    let found = codes(dir.path(), &docs);
    assert_eq!(
        found,
        vec![error_codes::DOCUMENT_MISSING, error_codes::DOCUMENT_MISSING]
    );
}

#[test]
fn per_system_location_validates() {
    let dir = repo_with(EXAMPLE);
    let sys = dir.path().join("docs/systems/notes");
    fs::create_dir_all(&sys).unwrap();
    fs::write(
        sys.join("requirements.md"),
        EXAMPLE.replace("serves: north-star.md", "serves: ../../north-star.md"),
    )
    .unwrap();
    let mut docs = DocsConfig::default();
    docs.requirements.locations = Some(vec![
        "requirements.md".into(),
        "systems/notes/requirements.md".into(),
    ]);
    assert!(codes(dir.path(), &docs).is_empty());
}

#[test]
fn absolute_and_escaping_locations_are_rejected() {
    let dir = tempdir().unwrap();
    let mut docs = DocsConfig::default();
    docs.requirements.locations = Some(vec!["/etc/req.md".into(), "../req.md".into()]);
    assert_eq!(
        codes(dir.path(), &docs),
        vec![
            error_codes::ABSOLUTE_LOCATION,
            error_codes::LOCATION_OUT_OF_TREE
        ]
    );
}

#[test]
fn disable_skips_everything() {
    let dir = repo_with("no frontmatter, no sections");
    let mut docs = DocsConfig::default();
    docs.requirements.disable = true;
    assert!(codes(dir.path(), &docs).is_empty());
}

#[test]
fn broken_document_reports_each_rule() {
    let doc = EXAMPLE
        .replace("status: accepted", "status: draft")
        .replace("last_reviewed: 2026-11-01", "last_reviewed: soon")
        .replace("| P1.2 |", "| P1.1 |")
        .replace(
            "| Top-5 hit rate on the labelled query set is at least 90%. |",
            "|  |",
        )
        .replace("| unchecked |", "|  |")
        .replace("| checked: `test-share-acl` |", "| checked |")
        .replace("| later |", "| someday |")
        .replace("- As a Reader,", "- As a Visitor,")
        .replace("- 1.0.0 (2026-09-15)", "- v1 (Sept)");
    let dir = repo_with(&doc);
    let found = codes(dir.path(), &DocsConfig::default());
    for expected in [
        error_codes::INVALID_STATUS,
        error_codes::INVALID_DATE,
        error_codes::REQUIREMENT_ID_DUPLICATE,
        error_codes::MEASURE_MISSING,
        error_codes::CHECK_STATUS_MISSING,
        error_codes::CHECK_UNNAMED,
        error_codes::INVALID_CHECK_STATUS,
        error_codes::STORY_ROLE_UNKNOWN,
        error_codes::CHANGELOG_ENTRY_INVALID,
    ] {
        assert!(
            found.iter().any(|c| c == expected),
            "missing {expected} in {found:?}"
        );
    }
}

#[test]
fn missing_frontmatter_fields_sections_and_stories() {
    let doc = "---\nname: R\ndescription: d\nstatus: accepted\nversion: 1.0.0\nlast_reviewed: 2026-01-01\nserves: nowhere.md\n---\n\n## Vision\n\nx\n\n## User Stories\n\nnone yet\n";
    let dir = repo_with(doc);
    let found = codes(dir.path(), &DocsConfig::default());
    for expected in [
        error_codes::FRONTMATTER_FIELD_MISSING, // owner
        error_codes::SERVES_UNRESOLVED,
        error_codes::MISSING_SECTION,
        error_codes::NO_USER_STORIES,
    ] {
        assert!(
            found.iter().any(|c| c == expected),
            "missing {expected} in {found:?}"
        );
    }
}

#[test]
fn changelog_version_must_track_frontmatter() {
    let doc = EXAMPLE.replace("version: 1.1.0", "version: 1.2.0");
    let dir = repo_with(&doc);
    let report = validate(dir.path(), &DocsConfig::default());
    let found: Vec<_> = report.iter().map(|d| d.code.as_str()).collect();
    assert_eq!(found, vec![error_codes::CHANGELOG_VERSION_MISMATCH]);
    assert!(!report.has_errors(), "version drift is a warning");
}

#[test]
fn fenced_requirements_neither_satisfy_nor_fail() {
    // A requirements table wrapped in a fence is not a requirement.
    let start = EXAMPLE.find("### P1: Capture").unwrap();
    let end = EXAMPLE.find("## User Stories").unwrap();
    let mut doc = EXAMPLE.to_string();
    doc.replace_range(
        start..end,
        &format!("```markdown\n{}```\n\n", &EXAMPLE[start..end]),
    );
    let found = codes(repo_with(&doc).path(), &DocsConfig::default());
    assert!(
        found.iter().any(|c| c == error_codes::NO_REQUIREMENTS),
        "{found:?}"
    );

    // A fenced illustrative duplicate row fails nothing.
    let doc = EXAMPLE.replacen(
        "### P2: Find",
        "```markdown\n| P1.1 | x | y | unchecked |\n```\n\n### P2: Find",
        1,
    );
    assert!(codes(repo_with(&doc).path(), &DocsConfig::default()).is_empty());
}

#[test]
fn tilde_and_long_fences_hide_their_content_and_close_only_on_their_own_kind() {
    // A tilde fence hides an illustrative duplicate row.
    let doc = EXAMPLE.replacen(
        "### P2: Find",
        "~~~markdown\n| P1.1 | x | y | unchecked |\n~~~\n\n### P2: Find",
        1,
    );
    assert!(codes(repo_with(&doc).path(), &DocsConfig::default()).is_empty());

    // A four-backtick example holding a three-backtick line and a heading:
    // the inner line does not close it, and the real sections after it count.
    let doc = EXAMPLE.replacen(
        "### P2: Find",
        "````markdown\n```\n## Change Log\n| P1.1 | x | y | unchecked |\n```\n````\n\n### P2: Find",
        1,
    );
    assert!(codes(repo_with(&doc).path(), &DocsConfig::default()).is_empty());

    // Only ASCII spaces or tabs may follow a closing fence: a no-break space
    // leaves the fence open, so the heading after it stays hidden.
    let doc = EXAMPLE.replacen(
        "### P2: Find",
        "~~~\n~~~\u{00A0}\n## Change Log\n~~~\n\n### P2: Find",
        1,
    );
    assert!(codes(repo_with(&doc).path(), &DocsConfig::default()).is_empty());
}

#[test]
fn requirements_must_sit_under_their_pillar() {
    let doc = EXAMPLE.replace("### P1: Capture", "Capture rows:");
    let found = codes(repo_with(&doc).path(), &DocsConfig::default());
    assert!(
        found
            .iter()
            .any(|c| c == error_codes::REQUIREMENT_OUTSIDE_PILLAR),
        "{found:?}"
    );

    let doc = EXAMPLE.replace("### P2: Find", "### P9: Find");
    let found = codes(repo_with(&doc).path(), &DocsConfig::default());
    assert!(
        found
            .iter()
            .any(|c| c == error_codes::REQUIREMENT_PILLAR_MISMATCH),
        "{found:?}"
    );
}

#[test]
fn changelog_entry_needs_the_colon() {
    let doc = EXAMPLE.replace("- 1.0.0 (2026-09-15): First", "- 1.0.0 (2026-09-15) First");
    let found = codes(repo_with(&doc).path(), &DocsConfig::default());
    assert!(
        found
            .iter()
            .any(|c| c == error_codes::CHANGELOG_ENTRY_INVALID),
        "{found:?}"
    );
}

#[test]
fn error_codes_use_human_readable_scheme() {
    for code in [
        error_codes::DOCUMENT_MISSING,
        error_codes::FRONTMATTER_FIELD_MISSING,
        error_codes::REQUIREMENT_ID_DUPLICATE,
        error_codes::CHANGELOG_VERSION_MISMATCH,
    ] {
        let suffix = code.strip_prefix("RQ01-").expect("RQ01- prefix");
        assert!(!suffix.contains(char::is_uppercase), "{code}");
    }
}
