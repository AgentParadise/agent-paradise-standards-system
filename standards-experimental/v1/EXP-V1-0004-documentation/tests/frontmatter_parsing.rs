use documentation::frontmatter::{parse_frontmatter, parse_frontmatter_from_file};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_valid_frontmatter() {
    let content = r#"---
name: "Test Document"
description: "A test document for parsing"
status: accepted
---

# Body content here
"#;
    let fm = parse_frontmatter(content).unwrap().unwrap();
    assert_eq!(fm.name(), Some("Test Document"));
    assert_eq!(fm.description(), Some("A test document for parsing"));
    assert_eq!(fm.get("status"), Some("accepted"));
}

#[test]
fn test_no_frontmatter() {
    let content = "# Just a heading\n\nSome content.";
    let fm = parse_frontmatter(content).unwrap();
    assert!(fm.is_none());
}

#[test]
fn test_empty_frontmatter() {
    let content = "---\n---\n\n# Content";
    let fm = parse_frontmatter(content).unwrap().unwrap();
    assert!(fm.fields.is_empty());
}

#[test]
fn test_unclosed_frontmatter() {
    let content = "---\nname: test\n# No closing delimiter";
    let result = parse_frontmatter(content);
    assert!(result.is_err());
}

#[test]
fn test_quoted_values() {
    let content = r#"---
name: "Quoted Name"
single: 'Single Quoted'
unquoted: plain value
---
"#;
    let fm = parse_frontmatter(content).unwrap().unwrap();
    assert_eq!(fm.get("name"), Some("Quoted Name"));
    assert_eq!(fm.get("single"), Some("Single Quoted"));
    assert_eq!(fm.get("unquoted"), Some("plain value"));
}

#[test]
fn test_comments_in_frontmatter() {
    let content = r#"---
name: "Test"
# This is a comment
description: "Description"
---
"#;
    let fm = parse_frontmatter(content).unwrap().unwrap();
    assert_eq!(fm.name(), Some("Test"));
    assert_eq!(fm.description(), Some("Description"));
    assert_eq!(fm.fields.len(), 2);
}

#[test]
fn test_frontmatter_from_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.md");
    fs::write(
        &path,
        r#"---
name: "File Test"
description: "From a file"
---

Content.
"#,
    )
    .unwrap();

    let fm = parse_frontmatter_from_file(&path).unwrap().unwrap();
    assert_eq!(fm.name(), Some("File Test"));
    assert_eq!(fm.description(), Some("From a file"));
}

#[test]
fn test_frontmatter_from_missing_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nonexistent.md");
    let result = parse_frontmatter_from_file(&path);
    assert!(result.is_err());
}

#[test]
fn test_leading_whitespace() {
    let content = "  \n---\nname: test\n---\n";
    let fm = parse_frontmatter(content).unwrap().unwrap();
    assert_eq!(fm.get("name"), Some("test"));
}
