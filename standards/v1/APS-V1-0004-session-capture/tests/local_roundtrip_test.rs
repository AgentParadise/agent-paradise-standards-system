//! Local conformance harness: the producer round trip (spec section 6.4.4) run
//! against REAL harness session files on the machine executing the test.
//!
//! These tests are `#[ignore]`d because they depend on a populated `~/.claude`
//! or `~/.codex`, which CI does not have. Run them deliberately:
//!
//! ```bash
//! cargo test -p apss-v1-0004-session-capture -- --ignored --nocapture
//! ```
//!
//! What they prove, end to end and without a running store:
//!
//! 1. A real transcript can be wrapped into a schema-valid envelope.
//! 2. The reconstitution registry resolves a target path for it.
//! 3. Writing `raw` back reproduces the original file BYTE FOR BYTE.
//!
//! Step 3 is the executable form of "raw is preserved verbatim" (section 4.3),
//! which is otherwise only a prose MUST.

use session_capture::reconstitution::{Registry, resolve_within_root};
use session_capture::{SCS_VERSION, SessionEnvelope, session_envelope_schema};
use std::path::{Path, PathBuf};

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").expect("HOME must be set"))
}

/// Find one real session file under `root` matching `extension`, deepest-first
/// so date-partitioned layouts are handled.
fn find_one_session(root: &Path, extension: &str) -> Option<PathBuf> {
    fn walk(dir: &Path, extension: &str, out: &mut Vec<PathBuf>, depth: usize) {
        if depth > 6 || out.len() > 200 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, extension, out, depth + 1);
            } else if path.extension().and_then(|e| e.to_str()) == Some(extension) {
                out.push(path);
            }
        }
    }
    let mut found = Vec::new();
    walk(root, extension, &mut found, 0);

    // Only consider real session transcripts: files whose name yields a valid
    // session id. This excludes incidental JSONL in the tree (subagent journals,
    // tool logs), which would make the round trip pass without exercising an
    // actually resumable session.
    found.retain(|p| {
        p.file_stem()
            .and_then(|s| s.to_str())
            .map(|stem| is_uuid_like(&extract_session_id(stem)))
            .unwrap_or(false)
    });

    // Prefer a small file so the test stays fast.
    found.sort_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(u64::MAX));
    found.into_iter().find(|p| {
        std::fs::metadata(p)
            .map(|m| m.len() > 0 && m.len() < 5_000_000)
            .unwrap_or(false)
    })
}

/// Whether a string is a canonical 8-4-4-4-12 hex UUID.
fn is_uuid_like(s: &str) -> bool {
    let lengths = [8usize, 4, 4, 4, 12];
    let parts: Vec<&str> = s.split('-').collect();
    parts.len() == 5
        && parts
            .iter()
            .zip(lengths)
            .all(|(p, n)| p.len() == n && p.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Recover the session id from a transcript filename.
///
/// Claude names the file for the session id directly. Codex prefixes it with
/// `rollout-` and the start timestamp, so the id is the trailing UUID. This
/// mirrors the `path_template` shapes in the reconstitution registry, and it is
/// exactly the kind of harness detail a Source must get right: taking the whole
/// stem as the id yields a value the registry correctly rejects.
fn extract_session_id(stem: &str) -> String {
    let parts: Vec<&str> = stem.split('-').collect();
    if parts.len() >= 5 {
        let tail = &parts[parts.len() - 5..];
        let candidate = tail.join("-");
        let lengths = [8usize, 4, 4, 4, 12];
        let is_uuid = tail
            .iter()
            .zip(lengths)
            .all(|(p, n)| p.len() == n && p.chars().all(|c| c.is_ascii_hexdigit()));
        if is_uuid {
            return candidate;
        }
    }
    stem.to_string()
}

/// Build an in-flight envelope wrapping a transcript file verbatim.
///
/// `raw` is the file's exact bytes as a JSON string. This is the representation
/// that makes byte-exact reconstitution possible; see the
/// `parsed_raw_does_not_round_trip_byte_exact` test for why parsing to an array
/// instead would forfeit that.
fn envelope_for(path: &Path, agent: &str, source_format: &str) -> (SessionEnvelope, String) {
    let bytes = std::fs::read(path).expect("session file must be readable");
    let contents = String::from_utf8(bytes).expect("transcript must be UTF-8");
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .expect("session file must have a stem");
    let session_id = extract_session_id(stem);

    let envelope = SessionEnvelope {
        scs_version: SCS_VERSION.to_string(),
        origin: session_capture::Origin {
            host: "local-conformance-harness".to_string(),
            environment: "local".to_string(),
            deployment: None,
        },
        agent: agent.to_string(),
        source_format: source_format.to_string(),
        session_id,
        parent_session_id: None,
        started_at: "2026-08-06T00:00:00Z".to_string(),
        last_activity_at: "2026-08-06T01:00:00Z".to_string(),
        // In flight: the store computes this (section 4.2.3).
        content_hash: None,
        metadata: None,
        raw: serde_json::Value::String(contents.clone()),
    };
    (envelope, contents)
}

fn assert_schema_valid(envelope: &SessionEnvelope) {
    let schema = session_envelope_schema();
    let validator = jsonschema::validator_for(&schema).expect("schema compiles");
    let instance = serde_json::to_value(envelope).expect("envelope serializes");
    if let Err(error) = validator.validate(&instance) {
        panic!("real-session envelope must satisfy the shipped schema: {error}");
    }
}

/// The producer round trip for one harness: wrap a real transcript, validate it,
/// resolve the target through `metadata.source_path` and the containment check,
/// write `raw` back at that relocated path, and compare bytes.
///
/// Panics rather than returning when no transcript is available. These tests are
/// `#[ignore]`d precisely so that running them is a deliberate act; a silent pass
/// on an empty machine would report success while proving nothing.
fn round_trip(agent: &str, source_format: &str, sessions_root: PathBuf, extension: &str) {
    assert!(
        sessions_root.exists(),
        "{agent}: {} does not exist. These tests assert against real transcripts; \
         run them on a machine that has used this harness.",
        sessions_root.display()
    );
    let session_path = find_one_session(&sessions_root, extension).unwrap_or_else(|| {
        panic!(
            "{agent}: no {extension} session transcripts under {}. \
             Nothing to round trip, so this is a failure rather than a pass.",
            sessions_root.display()
        )
    });
    eprintln!("{agent}: using real transcript {}", session_path.display());

    // 1. Capture into an envelope, and validate it as an in-flight envelope.
    let (envelope, original) = envelope_for(&session_path, agent, source_format);
    envelope
        .validate()
        .expect("in-flight envelope must validate");
    assert!(
        envelope.idempotency_key().is_none(),
        "an in-flight envelope has no store-computed hash yet"
    );
    assert_schema_valid(&envelope);

    // 2. Resolve restore knowledge from the registry, exactly as a Reconstitutor
    //    would, including the untrusted-input validation.
    let registry = Registry::shipped();
    let entry = registry
        .entry(source_format)
        .expect("registry must know this source_format");
    entry
        .validate_session_id(&envelope.session_id)
        .expect("a real session id must pass the registry's shape check");
    assert_eq!(entry.harness, agent);

    // 3. Reconstitute onto a DIFFERENT root, which is the cross-machine case:
    //    the harness root differs, so the target must be recomputed rather than
    //    reusing the captured absolute path (section 6.4.3).
    let target_root = std::env::temp_dir().join(format!("apss-scs-roundtrip-{source_format}"));
    let _ = std::fs::remove_dir_all(&target_root);
    std::fs::create_dir_all(&target_root).expect("temp target root");

    // The relative path under the harness root, exactly what `source_path`
    // carries. Deriving it here proves the value the example ships is the value
    // a Source would actually record.
    let relative = session_path
        .strip_prefix(&sessions_root)
        .expect("the transcript lives under the harness root")
        .to_string_lossy()
        .replace('\\', "/");

    // Resolve it the way a Reconstitutor must: through the containment check,
    // against the new root. This is the step that would reject a hostile
    // source_path, and it is what relocates the file.
    let target = resolve_within_root(&target_root, &relative)
        .expect("a real relative transcript path must resolve inside the new root");
    // Compare against the CANONICAL root: resolve_within_root canonicalizes, and
    // on macOS /tmp is itself a symlink to /private/tmp, so the uncanonicalized
    // path is not a prefix of the resolved one.
    let canonical_root = target_root
        .canonicalize()
        .expect("the new harness root exists");
    assert!(
        target.starts_with(&canonical_root),
        "the relocated target must sit under the new harness root: {} not under {}",
        target.display(),
        canonical_root.display()
    );
    assert_ne!(
        target, session_path,
        "reconstitution must write to the relocated path, not the captured one"
    );
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).expect("create relocated parent dirs");
    }

    let restored = envelope
        .raw
        .as_str()
        .expect("raw was captured as a verbatim string");
    std::fs::write(&target, restored).expect("reconstitution write");

    let written = std::fs::read(&target).expect("read back");
    let source = std::fs::read(&session_path).expect("read original");
    assert_eq!(
        written, source,
        "{agent}: reconstitute(capture(session)) must equal session byte for byte"
    );
    assert_eq!(restored, original);
    eprintln!(
        "{agent}: round trip OK, {} bytes byte-identical",
        source.len()
    );

    let _ = std::fs::remove_dir_all(&target_root);
}

#[test]
#[ignore = "requires a populated ~/.claude on this machine"]
fn claude_producer_round_trip_is_byte_exact() {
    round_trip(
        "ClaudeCode",
        "claude-jsonl-v1",
        home().join(".claude/projects"),
        "jsonl",
    );
}

#[test]
#[ignore = "requires a populated ~/.codex on this machine"]
fn codex_producer_round_trip_is_byte_exact() {
    round_trip(
        "Codex",
        "codex-turns-v1",
        home().join(".codex/sessions"),
        "jsonl",
    );
}

/// The registry's Claude slug rule must reproduce the directory names actually
/// present on this machine. This is what catches the registry going stale.
#[test]
#[ignore = "requires a populated ~/.claude on this machine"]
fn claude_slug_rule_reproduces_real_directory_names() {
    let projects = home().join(".claude/projects");
    if !projects.exists() {
        eprintln!("SKIP: {} does not exist", projects.display());
        return;
    }
    let registry = Registry::shipped();
    let entry = registry.entry("claude-jsonl-v1").unwrap();

    // Claude's slug is derived from an absolute cwd. Round-trip the derivation
    // against real directory names that correspond to paths still on disk.
    let mut checked = 0;
    for dir in std::fs::read_dir(&projects).unwrap().flatten() {
        let name = dir.file_name().to_string_lossy().to_string();
        if !name.starts_with('-') {
            continue;
        }
        // Reconstruct a plausible cwd and confirm the rule maps it back.
        let candidate = name.replacen('-', "/", 1).replace('-', "/");
        if !Path::new(&candidate).exists() {
            continue; // hyphenated or dotted names are not uniquely invertible
        }
        let derived = entry
            .slug_rule
            .slug_for(&candidate)
            .expect("claude derives a cwd slug");
        assert_eq!(
            derived, name,
            "registry slug rule must reproduce the real directory name"
        );
        checked += 1;
        if checked >= 5 {
            break;
        }
    }
    eprintln!("verified slug rule against {checked} real project directories");
    assert!(checked > 0, "expected at least one verifiable project dir");
}

/// Why `raw` must be captured as a verbatim string rather than a parsed array.
///
/// This is a conformance trap worth having a test for: a Source that parses a
/// JSONL transcript into an array of JSON objects and stores that has silently
/// forfeited byte-exact reconstitution, because re-serializing does not
/// reproduce the original formatting.
#[test]
#[ignore = "requires a populated ~/.claude on this machine"]
fn parsed_raw_does_not_round_trip_byte_exact() {
    let projects = home().join(".claude/projects");
    let Some(session_path) = find_one_session(&projects, "jsonl") else {
        eprintln!("SKIP: no Claude transcripts available");
        return;
    };
    let original = std::fs::read_to_string(&session_path).unwrap();

    // Parse each line, then re-serialize, the way a normalizing Source would.
    let reparsed: Vec<serde_json::Value> = original
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("each line is JSON"))
        .collect();
    let reserialized = reparsed
        .iter()
        .map(|v| serde_json::to_string(v).unwrap())
        .collect::<Vec<_>>()
        .join("\n");

    assert_ne!(
        reserialized, original,
        "if this ever passes, the trap is not real for this file; \
         verbatim-string capture is still the conformant choice"
    );
    eprintln!(
        "confirmed: parse-then-reserialize diverges ({} vs {} bytes), \
         so a normalizing Source cannot satisfy section 6.4.4",
        reserialized.len(),
        original.len()
    );
}
