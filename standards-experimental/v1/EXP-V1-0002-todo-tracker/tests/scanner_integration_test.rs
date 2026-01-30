//! Integration tests for the TODO/FIXME scanner.

use std::path::Path;
use todo_tracker::{Scanner, TrackerConfig};

const SAMPLE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/sample-repos");

#[test]
fn test_scan_rust_sample() {
    let config = TrackerConfig::default();
    let scanner = Scanner::new(config).expect("Failed to create scanner");

    let rust_dir = Path::new(SAMPLE_DIR).join("rust");
    let result = scanner.scan_repo(&rust_dir).expect("Failed to scan");

    // Should find 6 TODO/FIXME items in sample.rs
    assert!(
        result.items.len() >= 5,
        "Expected at least 5 items, found {}",
        result.items.len()
    );
    assert_eq!(result.files_scanned, 1);

    // Check for specific items
    let with_issues: Vec<_> = result
        .items
        .iter()
        .filter(|item| item.issue.is_some())
        .collect();
    let without_issues: Vec<_> = result
        .items
        .iter()
        .filter(|item| item.issue.is_none())
        .collect();

    assert!(
        with_issues.len() >= 4,
        "Expected at least 4 items with issues"
    );
    assert!(
        !without_issues.is_empty(),
        "Expected at least 1 item without issue"
    );

    // Validate specific TODO
    let todo_123 = result
        .items
        .iter()
        .find(|item| item.issue.as_ref().map(|i| i.number) == Some(123));
    assert!(todo_123.is_some(), "Should find TODO(#123)");
    let todo = todo_123.unwrap();
    assert_eq!(todo.tag, "TODO");
    assert!(todo.description.contains("integration tests"));
}

#[test]
fn test_scan_typescript_sample() {
    let config = TrackerConfig::default();
    let scanner = Scanner::new(config).expect("Failed to create scanner");

    let ts_dir = Path::new(SAMPLE_DIR).join("typescript");
    let result = scanner.scan_repo(&ts_dir).expect("Failed to scan");

    // Should find TODO/FIXME items in sample.ts
    assert!(
        result.items.len() >= 4,
        "Expected at least 4 items, found {}",
        result.items.len()
    );
    assert_eq!(result.files_scanned, 1);

    // Check for FIXME(#606)
    let fixme_606 = result
        .items
        .iter()
        .find(|item| item.issue.as_ref().map(|i| i.number) == Some(606));
    assert!(fixme_606.is_some(), "Should find FIXME(#606)");
    let fixme = fixme_606.unwrap();
    assert_eq!(fixme.tag, "FIXME");
    assert!(fixme.description.contains("Race condition"));
}

#[test]
fn test_scan_python_sample() {
    let config = TrackerConfig::default();
    let scanner = Scanner::new(config).expect("Failed to create scanner");

    let py_dir = Path::new(SAMPLE_DIR).join("python");
    let result = scanner.scan_repo(&py_dir).expect("Failed to scan");

    // Should find TODO/FIXME items in sample.py
    assert!(
        result.items.len() >= 5,
        "Expected at least 5 items, found {}",
        result.items.len()
    );
    assert_eq!(result.files_scanned, 1);

    // Check for TODO(#1111)
    let todo_1111 = result
        .items
        .iter()
        .find(|item| item.issue.as_ref().map(|i| i.number) == Some(1111));
    assert!(todo_1111.is_some(), "Should find TODO(#1111)");
    assert_eq!(todo_1111.unwrap().tag, "TODO");
}

#[test]
fn test_scan_all_samples() {
    let config = TrackerConfig::default();
    let scanner = Scanner::new(config).expect("Failed to create scanner");

    let sample_dir = Path::new(SAMPLE_DIR);
    let result = scanner.scan_repo(sample_dir).expect("Failed to scan");

    // Should find items across all three languages
    assert!(result.items.len() >= 14, "Expected at least 14 total items");
    assert!(
        result.files_scanned >= 3,
        "Expected at least 3 files scanned"
    );

    // Validate item structure
    for item in &result.items {
        assert!(!item.id.is_empty(), "Item ID should not be empty");
        assert!(!item.tag.is_empty(), "Tag should not be empty");
        assert!(!item.file.is_empty(), "File path should not be empty");
        assert!(item.line > 0, "Line number should be positive");
        assert!(!item.text.is_empty(), "Text should not be empty");
        assert!(
            !item.description.is_empty(),
            "Description should not be empty"
        );
    }
}

#[test]
fn test_tracked_vs_untracked() {
    let config = TrackerConfig::default();
    let scanner = Scanner::new(config).expect("Failed to create scanner");

    let sample_dir = Path::new(SAMPLE_DIR);
    let result = scanner.scan_repo(sample_dir).expect("Failed to scan");

    let tracked_count = result.items.iter().filter(|item| item.is_tracked()).count();
    let untracked_count = result
        .items
        .iter()
        .filter(|item| !item.is_tracked())
        .count();

    assert!(tracked_count > 0, "Should have some tracked items");
    assert!(untracked_count > 0, "Should have some untracked items");
    assert_eq!(tracked_count + untracked_count, result.items.len());
}

#[test]
fn test_tag_distribution() {
    let config = TrackerConfig::default();
    let scanner = Scanner::new(config).expect("Failed to create scanner");

    let sample_dir = Path::new(SAMPLE_DIR);
    let result = scanner.scan_repo(sample_dir).expect("Failed to scan");

    let todo_count = result
        .items
        .iter()
        .filter(|item| item.tag == "TODO")
        .count();
    let fixme_count = result
        .items
        .iter()
        .filter(|item| item.tag == "FIXME")
        .count();

    assert!(todo_count > 0, "Should have TODO items");
    assert!(fixme_count > 0, "Should have FIXME items");
}
