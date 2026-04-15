use documentation::config::IndexConfig;
use documentation::index::{
    find_index_section, generate_index, update_readme_index, validate_index, IndexIssue,
};
use std::fs;
use tempfile::tempdir;

fn default_index_config() -> IndexConfig {
    IndexConfig::default()
}

#[test]
fn test_generate_index_with_frontmatter() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("ADR_001_auth.md"),
        "---\nname: \"Auth Strategy\"\ndescription: \"How we handle auth\"\n---\n\nContent.",
    )
    .unwrap();
    fs::write(
        dir.path().join("ADR_002_db.md"),
        "---\nname: \"Database Choice\"\ndescription: \"Why PostgreSQL\"\n---\n\nContent.",
    )
    .unwrap();

    let config = default_index_config();
    let result = generate_index(dir.path(), &config).unwrap();

    assert_eq!(result.entries.len(), 2);
    assert!(result.markdown.contains("## Index"));
    assert!(result.markdown.contains("Auth Strategy"));
    assert!(result.markdown.contains("How we handle auth"));
    assert!(result.markdown.contains("Database Choice"));
    assert!(result.markdown.contains("Why PostgreSQL"));
}

#[test]
fn test_generate_index_excludes_structural_files() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("README.md"), "# Readme").unwrap();
    fs::write(dir.path().join("CLAUDE.md"), "# Claude").unwrap();
    fs::write(dir.path().join("AGENTS.md"), "# Agents").unwrap();
    fs::write(
        dir.path().join("ADR_001_test.md"),
        "---\nname: \"Test\"\ndescription: \"A test\"\n---\n",
    )
    .unwrap();

    let config = default_index_config();
    let result = generate_index(dir.path(), &config).unwrap();

    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].filename, "ADR_001_test.md");
}

#[test]
fn test_generate_index_no_frontmatter_uses_filename() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("notes.md"), "# Just a heading\n\nContent.").unwrap();

    let config = default_index_config();
    let result = generate_index(dir.path(), &config).unwrap();

    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].name, "notes");
    assert_eq!(result.entries[0].description, "");
}

#[test]
fn test_generate_index_empty_directory() {
    let dir = tempdir().unwrap();

    let config = default_index_config();
    let result = generate_index(dir.path(), &config).unwrap();

    assert_eq!(result.entries.len(), 0);
    assert!(result.markdown.contains("No documents found"));
}

#[test]
fn test_find_index_section() {
    let content = "# Title\n\nIntro.\n\n## Index\n\n| Doc | Desc |\n|---|---|\n\n## Other\n\nMore.";
    let (start, end) = find_index_section(content).unwrap();
    let section = &content[start..end];
    assert!(section.starts_with("## Index"));
    assert!(!section.contains("## Other"));
}

#[test]
fn test_find_index_section_at_end() {
    let content = "# Title\n\n## Index\n\n| Doc | Desc |\n|---|---|";
    let (start, end) = find_index_section(content).unwrap();
    assert_eq!(end, content.len());
    assert!(content[start..end].starts_with("## Index"));
}

#[test]
fn test_find_index_section_missing() {
    let content = "# Title\n\n## Other Section\n\nContent.";
    assert!(find_index_section(content).is_none());
}

#[test]
fn test_validate_index_missing() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("doc.md"),
        "---\nname: \"Doc\"\ndescription: \"A doc\"\n---\n",
    )
    .unwrap();

    let readme_content = "# Title\n\nNo index here.";
    let config = default_index_config();
    let result = validate_index(readme_content, dir.path(), &config).unwrap();

    assert!(!result.is_valid);
    assert_eq!(result.reason, IndexIssue::Missing);
}

#[test]
fn test_validate_index_stale() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("doc.md"),
        "---\nname: \"Doc\"\ndescription: \"A doc\"\n---\n",
    )
    .unwrap();

    let readme_content = "# Title\n\n## Index\n\n| Document | Description |\n|----------|-------------|\n| [old_doc](old_doc.md) | Old |\n";
    let config = default_index_config();
    let result = validate_index(readme_content, dir.path(), &config).unwrap();

    assert!(!result.is_valid);
    assert_eq!(result.reason, IndexIssue::Stale);
}

#[test]
fn test_update_readme_index() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("doc.md"),
        "---\nname: \"My Doc\"\ndescription: \"A document\"\n---\n",
    )
    .unwrap();
    let readme_path = dir.path().join("README.md");
    fs::write(&readme_path, "# Title\n\nIntro.\n\n## Other\n\nMore.").unwrap();

    let config = default_index_config();
    update_readme_index(&readme_path, dir.path(), &config).unwrap();

    let content = fs::read_to_string(&readme_path).unwrap();
    assert!(content.contains("## Index"));
    assert!(content.contains("My Doc"));
    assert!(content.contains("# Title")); // preserved
}

#[test]
fn test_update_readme_index_replaces_existing() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("doc.md"),
        "---\nname: \"New Doc\"\ndescription: \"Updated\"\n---\n",
    )
    .unwrap();
    let readme_path = dir.path().join("README.md");
    fs::write(
        &readme_path,
        "# Title\n\n## Index\n\n| Document | Description |\n|----------|-------------|\n| [old](old.md) | Old |\n\n## Footer\n\nEnd.",
    )
    .unwrap();

    let config = default_index_config();
    update_readme_index(&readme_path, dir.path(), &config).unwrap();

    let content = fs::read_to_string(&readme_path).unwrap();
    assert!(content.contains("New Doc"));
    assert!(!content.contains("old.md"));
    assert!(content.contains("## Footer")); // preserved
}
