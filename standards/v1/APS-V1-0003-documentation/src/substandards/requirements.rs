//! Requirements (APS-V1-0003.RQ01) substandard.
//!
//! RQ01 is an OPTIONAL doc type. A system MAY keep one requirements document
//! stating what the system must be and do. The validator never asks a project
//! to create one: it validates the structure of the documents that exist, and
//! checks presence only for the locations a project declares in
//! `docs.requirements.locations` (declaring a location is the opt-in).
//!
//! The normative contract lives in
//! `substandards/RQ01-requirements/docs/01_spec.md`.

use crate::config::{DocsConfig, resolve_docs_root};
use crate::frontmatter::{FrontMatter, parse_frontmatter};
use apss_core::{Diagnostic, Diagnostics};
use regex::Regex;
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

/// Diagnostic codes emitted by the Requirements substandard.
///
/// See `docs/01_spec.md` Section 11 for descriptions.
pub mod error_codes {
    pub const DOCUMENT_MISSING: &str = "RQ01-document-missing";
    pub const ABSOLUTE_LOCATION: &str = "RQ01-absolute-location";
    pub const LOCATION_OUT_OF_TREE: &str = "RQ01-location-out-of-tree";
    pub const FRONTMATTER_MISSING: &str = "RQ01-frontmatter-missing";
    pub const FRONTMATTER_FIELD_MISSING: &str = "RQ01-frontmatter-field-missing";
    pub const INVALID_STATUS: &str = "RQ01-invalid-status";
    pub const SUPERSEDED_WITHOUT_POINTER: &str = "RQ01-superseded-without-pointer";
    pub const INVALID_VERSION: &str = "RQ01-invalid-version";
    pub const INVALID_DATE: &str = "RQ01-invalid-date";
    pub const SERVES_UNRESOLVED: &str = "RQ01-serves-unresolved";
    pub const MISSING_SECTION: &str = "RQ01-missing-section";
    pub const NO_REQUIREMENTS: &str = "RQ01-no-requirements";
    pub const REQUIREMENT_ID_INVALID: &str = "RQ01-requirement-id-invalid";
    pub const REQUIREMENT_ID_DUPLICATE: &str = "RQ01-requirement-id-duplicate";
    pub const REQUIREMENT_OUTSIDE_PILLAR: &str = "RQ01-requirement-outside-pillar";
    pub const REQUIREMENT_PILLAR_MISMATCH: &str = "RQ01-requirement-pillar-mismatch";
    pub const REQUIREMENT_TEXT_MISSING: &str = "RQ01-requirement-text-missing";
    pub const MEASURE_MISSING: &str = "RQ01-measure-missing";
    pub const CHECK_STATUS_MISSING: &str = "RQ01-check-status-missing";
    pub const INVALID_CHECK_STATUS: &str = "RQ01-invalid-check-status";
    pub const CHECK_UNNAMED: &str = "RQ01-check-unnamed";
    pub const NO_ROLES: &str = "RQ01-no-roles";
    pub const NO_USER_STORIES: &str = "RQ01-no-user-stories";
    pub const STORY_ROLE_UNKNOWN: &str = "RQ01-story-role-unknown";
    pub const CHANGELOG_EMPTY: &str = "RQ01-changelog-empty";
    pub const CHANGELOG_ENTRY_INVALID: &str = "RQ01-changelog-entry-invalid";
    pub const CHANGELOG_VERSION_MISMATCH: &str = "RQ01-changelog-version-mismatch";
}

/// Allowed lifecycle status values (parent spec Section 8.1; requirements
/// use `accepted` for "in force", like ADRs, because the owner ratifies them).
pub const ALLOWED_STATUSES: &[&str] = &["proposed", "accepted", "deprecated", "superseded"];

/// Allowed values for a requirement's check status.
pub const CHECK_STATUSES: &[&str] = &["checked", "partial", "unchecked", "later"];

/// Required frontmatter fields.
pub const REQUIRED_FIELDS: &[&str] = &[
    "name",
    "description",
    "owner",
    "status",
    "version",
    "last_reviewed",
    "serves",
];

/// Required H2 sections, in order.
pub const REQUIRED_SECTIONS: &[&str] = &[
    "Vision",
    "Roles",
    "Requirements",
    "User Stories",
    "Later and Out of Scope",
    "Change Log",
];

/// Default location, docs-root-relative (resolves to `docs/requirements.md`).
pub const DEFAULT_LOCATION: &str = "requirements.md";

/// Validate every requirements document the project has.
///
/// - `docs.disable` or `docs.requirements.disable`: nothing is checked.
/// - `docs.requirements.locations` absent: the default location is checked
///   only if a file exists there. A project without one passes.
/// - `docs.requirements.locations` set: each listed document MUST exist.
pub fn validate(repo_root: &Path, docs: &DocsConfig) -> Diagnostics {
    let mut diags = Diagnostics::new();
    if docs.disable || docs.requirements.disable {
        return diags;
    }
    let docs_root = resolve_docs_root(repo_root, docs);
    let (locations, declared) = match &docs.requirements.locations {
        Some(list) => (list.clone(), true),
        None => (vec![DEFAULT_LOCATION.to_string()], false),
    };

    for location in &locations {
        let Some(path) = resolve_location(&docs_root, location, &mut diags) else {
            continue;
        };
        if !path.is_file() {
            if declared {
                diags.push(document_missing_diagnostic(&path));
            }
            continue;
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => validate_document(&path, &content, &mut diags),
            Err(e) => diags.push(
                Diagnostic::error(
                    error_codes::DOCUMENT_MISSING,
                    format!("Cannot read requirements document: {e}"),
                )
                .with_path(&path),
            ),
        }
    }
    diags
}

/// Resolve a docs-root-relative location, rejecting absolute and escaping paths.
fn resolve_location(docs_root: &Path, location: &str, diags: &mut Diagnostics) -> Option<PathBuf> {
    let rel = Path::new(location);
    if location.starts_with('/') || rel.is_absolute() {
        diags.push(
            Diagnostic::error(
                error_codes::ABSOLUTE_LOCATION,
                format!("docs.requirements location '{location}' is absolute"),
            )
            .with_hint("Locations are docs-root-relative; drop the leading '/'"),
        );
        return None;
    }
    if rel.components().any(|c| matches!(c, Component::ParentDir)) {
        diags.push(
            Diagnostic::error(
                error_codes::LOCATION_OUT_OF_TREE,
                format!("docs.requirements location '{location}' escapes the docs root"),
            )
            .with_hint("Remove '..' segments; locations resolve under docs.root"),
        );
        return None;
    }
    Some(docs_root.join(rel))
}

/// Validate one requirements document's content. Public so tooling (and the
/// tests) can check a document without a repository around it.
pub fn validate_document(path: &Path, content: &str, diags: &mut Diagnostics) {
    match parse_frontmatter(content) {
        Ok(Some(fm)) => validate_frontmatter(path, &fm, diags),
        Ok(None) | Err(_) => diags.push(
            Diagnostic::error(
                error_codes::FRONTMATTER_MISSING,
                "Requirements document has no closed frontmatter block",
            )
            .with_path(path)
            .with_hint(format!(
                "Start the file with a '---' block holding: {}",
                REQUIRED_FIELDS.join(", ")
            )),
        ),
    }

    let sections = split_sections(content);
    for required in REQUIRED_SECTIONS {
        if !sections
            .iter()
            .any(|s| s.title.eq_ignore_ascii_case(required))
        {
            diags.push(
                Diagnostic::error(
                    error_codes::MISSING_SECTION,
                    format!("Requirements document missing '## {required}' section"),
                )
                .with_path(path),
            );
        }
    }

    let body = |title: &str| {
        sections
            .iter()
            .find(|s| s.title.eq_ignore_ascii_case(title))
            .map(|s| s.lines.as_slice())
    };

    if let Some(lines) = body("Requirements") {
        validate_requirements(path, lines, diags);
    }
    let roles = body("Roles").map(parse_roles).unwrap_or_default();
    if body("Roles").is_some() && roles.is_empty() {
        diags.push(
            Diagnostic::error(error_codes::NO_ROLES, "'## Roles' lists no roles")
                .with_path(path)
                .with_hint("List each role as a bullet: '- **Role**: who they are'"),
        );
    }
    if let Some(lines) = body("User Stories") {
        validate_stories(path, lines, &roles, diags);
    }
    if let Some(lines) = body("Change Log") {
        let fm_version = parse_frontmatter(content)
            .ok()
            .flatten()
            .and_then(|fm| fm.get("version").map(str::to_string));
        validate_changelog(path, lines, fm_version.as_deref(), diags);
    }
}

fn validate_frontmatter(path: &Path, fm: &FrontMatter, diags: &mut Diagnostics) {
    for field in REQUIRED_FIELDS {
        if fm.get(field).is_none_or(|v| v.trim().is_empty()) {
            diags.push(
                Diagnostic::error(
                    error_codes::FRONTMATTER_FIELD_MISSING,
                    format!("Requirements frontmatter missing '{field}'"),
                )
                .with_path(path),
            );
        }
    }
    if let Some(status) = fm.get("status").filter(|s| !s.is_empty()) {
        if !ALLOWED_STATUSES.contains(&status) {
            diags.push(
                Diagnostic::error(
                    error_codes::INVALID_STATUS,
                    format!("Invalid requirements status '{status}'"),
                )
                .with_path(path)
                .with_hint(format!(
                    "Use one of: {} (parent spec Section 8.1; a draft is 'proposed')",
                    ALLOWED_STATUSES.join(", ")
                )),
            );
        }
        if status == "superseded" && fm.get("superseded_by").is_none_or(|v| v.is_empty()) {
            diags.push(
                Diagnostic::error(
                    error_codes::SUPERSEDED_WITHOUT_POINTER,
                    "status: superseded requires a 'superseded_by' field",
                )
                .with_path(path),
            );
        }
    }
    if let Some(version) = fm.get("version").filter(|s| !s.is_empty()) {
        if !semver_re().is_match(version) {
            diags.push(
                Diagnostic::error(
                    error_codes::INVALID_VERSION,
                    format!("Requirements version '{version}' is not MAJOR.MINOR.PATCH"),
                )
                .with_path(path),
            );
        }
    }
    if let Some(date) = fm.get("last_reviewed").filter(|s| !s.is_empty()) {
        if !date_re().is_match(date) {
            diags.push(
                Diagnostic::error(
                    error_codes::INVALID_DATE,
                    format!("last_reviewed '{date}' is not a YYYY-MM-DD date"),
                )
                .with_path(path),
            );
        }
    }
    if let Some(serves) = fm.get("serves").filter(|s| !s.is_empty()) {
        let base = path.parent().unwrap_or(Path::new("."));
        if !base.join(serves).is_file() {
            diags.push(
                Diagnostic::error(
                    error_codes::SERVES_UNRESOLVED,
                    format!("'serves: {serves}' does not resolve to a file"),
                )
                .with_path(path)
                .with_hint(
                    "Point 'serves' at the North Star (PV01) document, relative to this file",
                ),
            );
        }
    }
}

/// One `## ` section: its title and the lines up to the next `## `.
struct Section {
    title: String,
    lines: Vec<String>,
}

fn split_sections(content: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    // The open fence's character and length (CommonMark fenced code blocks).
    let mut fence: Option<(char, usize)> = None;
    for line in content.lines() {
        // Fenced content is illustration, never structure: no check reads it.
        if let Some((ch, len)) = fence {
            if closes_fence(line, ch, len) {
                fence = None;
            }
            continue;
        }
        if let Some(open) = opening_fence(line) {
            fence = Some(open);
            continue;
        }
        if let Some(title) = line.strip_prefix("## ") {
            sections.push(Section {
                title: title.trim().to_string(),
                lines: Vec::new(),
            });
            continue;
        }
        if let Some(current) = sections.last_mut() {
            current.lines.push(line.to_string());
        }
    }
    sections
}

/// The fence a line opens: at most three spaces of indent, then three or more
/// backticks or tildes (a backtick fence's info string has no backtick).
fn opening_fence(line: &str) -> Option<(char, usize)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return None;
    }
    let rest = &line[indent..];
    let ch = rest.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let len = rest.chars().take_while(|&c| c == ch).count();
    if len < 3 || (ch == '`' && rest[len..].contains('`')) {
        return None;
    }
    Some((ch, len))
}

/// The line closes a fence of `ch` opened with `len` characters: at most three
/// spaces of indent, at least `len` of `ch`, then only whitespace.
fn closes_fence(line: &str, ch: char, len: usize) -> bool {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 3 {
        return false;
    }
    let rest = &line[indent..];
    let run = rest.chars().take_while(|&c| c == ch).count();
    run >= len
        && rest[run * ch.len_utf8()..]
            .chars()
            .all(|c| c == ' ' || c == '\t')
}

/// Split a Markdown table row into trimmed cells, honouring `\|` escapes.
fn split_row(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&'|') => {
                cur.push('|');
                chars.next();
            }
            '|' => cells.push(std::mem::take(&mut cur).trim().to_string()),
            _ => cur.push(c),
        }
    }
    cells.push(cur.trim().to_string());
    cells
}

fn is_separator_row(cells: &[String]) -> bool {
    cells
        .iter()
        .all(|c| !c.is_empty() && c.chars().all(|ch| matches!(ch, '-' | ':' | ' ')))
}

fn is_header_row(cells: &[String]) -> bool {
    cells.first().is_some_and(|c| c.eq_ignore_ascii_case("id"))
}

fn validate_requirements(path: &Path, lines: &[String], diags: &mut Diagnostics) {
    let id_re = Regex::new(r"^[A-Z][A-Z0-9]*\d\.\d+$").expect("static regex");
    let mut seen: HashSet<String> = HashSet::new();
    let mut count = 0usize;
    // The code of the enclosing `### <CODE>: <name>` pillar heading.
    let mut pillar: Option<String> = None;

    for line in lines {
        if let Some(heading) = line.strip_prefix("### ") {
            pillar = pillar_code(heading);
            continue;
        }
        if !line.trim_start().starts_with('|') {
            continue;
        }
        let cells = split_row(line);
        if is_header_row(&cells) || is_separator_row(&cells) {
            continue;
        }
        count += 1;
        let cell = |i: usize| cells.get(i).map(|s| s.trim_matches('`')).unwrap_or("");
        let id = cell(0);
        let label = if id.is_empty() { "<no id>" } else { id };

        if !id_re.is_match(id) {
            diags.push(
                Diagnostic::error(
                    error_codes::REQUIREMENT_ID_INVALID,
                    format!("Requirement id '{label}' is not of the form <PILLAR><n>.<m>"),
                )
                .with_path(path)
                .with_hint("Use a stable id such as 'P3.5' (pillar P3, requirement 5)"),
            );
        } else if let Some(code) = id_pillar(id) {
            match pillar.as_deref() {
                None => diags.push(
                    Diagnostic::error(
                        error_codes::REQUIREMENT_OUTSIDE_PILLAR,
                        format!("Requirement '{id}' is under no '### <PILLAR>: <name>' heading"),
                    )
                    .with_path(path)
                    .with_hint(format!("Put it under a '### {code}: <name>' heading")),
                ),
                Some(current) if current != code => diags.push(
                    Diagnostic::error(
                        error_codes::REQUIREMENT_PILLAR_MISMATCH,
                        format!("Requirement '{id}' sits under pillar '{current}'"),
                    )
                    .with_path(path)
                    .with_hint(format!("Move it under '### {code}: <name>' or renumber it")),
                ),
                Some(_) => {}
            }
        }
        if id_re.is_match(id) && !seen.insert(id.to_string()) {
            diags.push(
                Diagnostic::error(
                    error_codes::REQUIREMENT_ID_DUPLICATE,
                    format!("Requirement id '{id}' is used more than once"),
                )
                .with_path(path)
                .with_hint("Ids are stable: never reuse one, even for a removed requirement"),
            );
        }
        if cell(1).is_empty() {
            diags.push(
                Diagnostic::error(
                    error_codes::REQUIREMENT_TEXT_MISSING,
                    format!("Requirement '{label}' has no requirement text"),
                )
                .with_path(path),
            );
        }
        if cell(2).is_empty() {
            diags.push(
                Diagnostic::error(
                    error_codes::MEASURE_MISSING,
                    format!("Requirement '{label}' has no measure"),
                )
                .with_path(path)
                .with_hint("Say how you know it is met"),
            );
        }
        validate_check_cell(path, label, cells.get(3).map(String::as_str), diags);
    }

    if count == 0 {
        diags.push(
            Diagnostic::error(
                error_codes::NO_REQUIREMENTS,
                "'## Requirements' holds no requirement rows",
            )
            .with_path(path)
            .with_hint("Add a table: | ID | Requirement | Measure | Check |"),
        );
    }
}

/// The pillar code of a `### <CODE>: <name>` heading: the text before the
/// colon, or the first word when there is none.
fn pillar_code(heading: &str) -> Option<String> {
    let heading = heading.trim();
    let code = heading
        .split_once(':')
        .map_or_else(
            || heading.split_whitespace().next().unwrap_or(""),
            |(c, _)| c,
        )
        .trim();
    (!code.is_empty()).then(|| code.to_string())
}

/// The pillar a requirement id names: the part before its `.` (`P3` of `P3.5`).
fn id_pillar(id: &str) -> Option<&str> {
    id.split_once('.').map(|(code, _)| code)
}

/// The check cell is `<status>` or `<status>: <check name>`.
fn validate_check_cell(path: &Path, id: &str, cell: Option<&str>, diags: &mut Diagnostics) {
    let cell = cell.unwrap_or("").trim();
    if cell.is_empty() {
        diags.push(
            Diagnostic::error(
                error_codes::CHECK_STATUS_MISSING,
                format!("Requirement '{id}' has no check status"),
            )
            .with_path(path)
            .with_hint(format!("Use one of: {}", CHECK_STATUSES.join(", "))),
        );
        return;
    }
    let (status, name) = match cell.split_once(':') {
        Some((s, n)) => (s.trim(), n.trim()),
        None => (cell, ""),
    };
    let status = status.to_ascii_lowercase();
    if !CHECK_STATUSES.contains(&status.as_str()) {
        diags.push(
            Diagnostic::error(
                error_codes::INVALID_CHECK_STATUS,
                format!("Requirement '{id}' has check status '{status}'"),
            )
            .with_path(path)
            .with_hint(format!("Use one of: {}", CHECK_STATUSES.join(", "))),
        );
        return;
    }
    if status == "checked" && name.is_empty() {
        diags.push(
            Diagnostic::error(
                error_codes::CHECK_UNNAMED,
                format!("Requirement '{id}' is 'checked' but names no check"),
            )
            .with_path(path)
            .with_hint("Write 'checked: <test, gate or command>'"),
        );
    }
}

/// Roles are top-level bullets: `- **Role**: ...`, `- Role: ...`, or `- Role`.
fn parse_roles(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .filter_map(|l| l.strip_prefix("- ").or_else(|| l.strip_prefix("* ")))
        .map(|item| {
            let item = item.trim();
            let head = item.split(':').next().unwrap_or(item);
            let head = head.split(" - ").next().unwrap_or(head);
            head.replace("**", "").trim().to_string()
        })
        .filter(|r| !r.is_empty())
        .collect()
}

fn validate_stories(path: &Path, lines: &[String], roles: &[String], diags: &mut Diagnostics) {
    let story_re = Regex::new(r"(?i)\bas an? \*{0,2}([^,*]+?)\*{0,2}, I want .+?,? so that .+")
        .expect("static regex");
    let mut count = 0usize;
    for line in lines {
        let Some(caps) = story_re.captures(line) else {
            continue;
        };
        count += 1;
        let role = caps[1].trim();
        if !roles.is_empty() && !roles.iter().any(|r| r.eq_ignore_ascii_case(role)) {
            diags.push(
                Diagnostic::warning(
                    error_codes::STORY_ROLE_UNKNOWN,
                    format!("User story role '{role}' is not listed under '## Roles'"),
                )
                .with_path(path),
            );
        }
    }
    if count == 0 {
        diags.push(
            Diagnostic::error(
                error_codes::NO_USER_STORIES,
                "'## User Stories' holds no story",
            )
            .with_path(path)
            .with_hint("Write 'As a <role>, I want <goal>, so that <reason>.'"),
        );
    }
}

fn validate_changelog(
    path: &Path,
    lines: &[String],
    fm_version: Option<&str>,
    diags: &mut Diagnostics,
) {
    let entry_re =
        Regex::new(r"^- (\d+\.\d+\.\d+) \((\d{4}-\d{2}-\d{2})\): .+").expect("static regex");
    let mut first_version: Option<String> = None;
    let mut count = 0usize;
    for line in lines.iter().filter(|l| l.starts_with("- ")) {
        count += 1;
        match entry_re.captures(line) {
            Some(caps) => {
                if first_version.is_none() {
                    first_version = Some(caps[1].to_string());
                }
            }
            None => diags.push(
                Diagnostic::error(
                    error_codes::CHANGELOG_ENTRY_INVALID,
                    format!(
                        "Change log entry is not '- <version> (<YYYY-MM-DD>): <change>': {line}"
                    ),
                )
                .with_path(path),
            ),
        }
    }
    if count == 0 {
        diags.push(
            Diagnostic::error(
                error_codes::CHANGELOG_EMPTY,
                "'## Change Log' has no entries",
            )
            .with_path(path)
            .with_hint("Add '- 0.1.0 (YYYY-MM-DD): First draft.'"),
        );
    }
    if let (Some(first), Some(fm)) = (first_version, fm_version) {
        if first != fm {
            diags.push(
                Diagnostic::warning(
                    error_codes::CHANGELOG_VERSION_MISMATCH,
                    format!("Newest change log entry is {first} but frontmatter version is {fm}"),
                )
                .with_path(path)
                .with_hint("List the newest entry first and keep it equal to 'version'"),
            );
        }
    }
}

fn semver_re() -> Regex {
    Regex::new(r"^\d+\.\d+\.\d+$").expect("static regex")
}

fn date_re() -> Regex {
    Regex::new(r"^\d{4}-\d{2}-\d{2}$").expect("static regex")
}

/// The canonical `RQ01-document-missing` diagnostic.
pub fn document_missing_diagnostic(location: &Path) -> Diagnostic {
    Diagnostic::error(
        error_codes::DOCUMENT_MISSING,
        format!(
            "Requirements document declared but not found: {}",
            location.display()
        ),
    )
    .with_path(location)
    .with_hint(format!(
        "Create the file at '{}' or remove it from docs.requirements.locations in apss.yaml",
        location.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_statuses_are_lowercase_unique() {
        let mut seen = HashSet::new();
        for s in CHECK_STATUSES {
            assert_eq!(*s, s.to_lowercase());
            assert!(seen.insert(*s), "duplicate check status: {s}");
        }
    }

    #[test]
    fn split_row_honours_escaped_pipes() {
        let cells = split_row(r"| P1.1 | a \| b | m | checked: `x` |");
        assert_eq!(cells, vec!["P1.1", "a | b", "m", "checked: `x`"]);
    }

    #[test]
    fn parse_roles_reads_bold_and_plain_bullets() {
        let lines: Vec<String> = [
            "- **Maintainer**: keeps it running",
            "- Agent",
            "  - nested",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(parse_roles(&lines), vec!["Maintainer", "Agent"]);
    }
}
