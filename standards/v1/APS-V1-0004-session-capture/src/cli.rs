//! CLI commands for APS-V1-0004 (APS-V1-0000.CL01).
//!
//! The standard is definitional, so these commands do not manage a project.
//! They expose the two operations a consumer most often needs to perform against
//! the standard itself:
//!
//! - `validate` answers "is this envelope conformant?" It runs the I-JSON check
//!   on the original text (which is the only point duplicate member names are
//!   still visible), then the crate's structural validation. It deliberately
//!   does NOT run the JSON Schema: `SessionEnvelope` deserialization already
//!   enforces the schema's required fields and types, and a test asserts the two
//!   layers agree, so running both here would report the same failure twice.
//! - `hash` computes the canonical `content_hash` (section 4.2.3), so a store
//!   implementer can compare their computation against the reference one. The
//!   hash is the most interoperability-sensitive rule in the standard, and a
//!   store that derives it differently silently fails to deduplicate.
//!
//! Both commands validate before acting. `hash` in particular must not print a
//! digest for a non-conformant envelope: that digest would look authoritative
//! while describing something the standard rejects.

use apss_core::registry::{CommandHandler, CommandInfo};

use crate::{
    SessionEnvelope,
    content_hash::{content_hash_for, parse_ijson},
};

/// Commands this standard registers.
pub const COMMAND_NAMES: [&str; 2] = ["validate", "hash"];

/// Exit codes, per APS-V1-0000.CL01.
mod exit {
    pub const SUCCESS: i32 = 0;
    pub const ERROR: i32 = 1;
    pub const USAGE: i32 = 3;
}

/// Dispatches this standard's commands.
#[derive(Debug, Default)]
pub struct SessionCaptureCommandHandler;

impl SessionCaptureCommandHandler {
    /// Create a handler.
    pub fn new() -> Self {
        Self
    }
}

/// Read the envelopes in a file, accepting either a single envelope or a batch
/// body (`{ "envelopes": [...] }`, section 5.1).
fn envelopes_in(path: &str) -> Result<Vec<serde_json::Value>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;

    // I-JSON first, on the original text. Duplicate member names are invisible
    // after parsing, so this cannot be deferred (section 4.2.3).
    let value = parse_ijson(&text).map_err(|e| format!("{path}: {e}"))?;

    if value.is_array() {
        return Err(format!(
            "{path}: a batch body must be wrapped as {{ \"envelopes\": [...] }}, \
             not a bare array (section 5.1)"
        ));
    }

    // Distinguish a batch from a single envelope by shape, not by key presence:
    // the schema permits unknown top-level fields, so a lone envelope carrying an
    // extension field named `envelopes` would otherwise be misread as a batch.
    let looks_like_batch = value.get("envelopes").is_some() && value.get("raw").is_none();
    if looks_like_batch {
        let array = value["envelopes"]
            .as_array()
            .ok_or_else(|| format!("{path}: `envelopes` must be an array (section 5.1)"))?;
        if array.is_empty() {
            return Err(format!(
                "{path}: a batch must carry one or more envelopes (section 5.1)"
            ));
        }
        return Ok(array.clone());
    }
    Ok(vec![value])
}

fn parse_envelope(value: &serde_json::Value, index: usize) -> Result<SessionEnvelope, String> {
    serde_json::from_value(value.clone()).map_err(|e| format!("envelope[{index}]: {e}"))
}

fn run_validate(args: &[String]) -> i32 {
    let Some(path) = args.first() else {
        eprintln!("usage: validate <file.json>");
        return exit::USAGE;
    };

    let envelopes = match envelopes_in(path) {
        Ok(envelopes) => envelopes,
        Err(message) => {
            eprintln!("error: {message}");
            return exit::ERROR;
        }
    };

    let mut failures = 0usize;
    for (index, value) in envelopes.iter().enumerate() {
        match parse_envelope(value, index) {
            Ok(envelope) => match envelope.validate() {
                Ok(()) => {
                    let state = if envelope.content_hash.is_some() {
                        "stored"
                    } else {
                        "in flight"
                    };
                    println!("envelope[{index}] {} ok ({state})", envelope.session_id);
                }
                Err(error) => {
                    failures += 1;
                    eprintln!("envelope[{index}] invalid: {error}");
                }
            },
            Err(message) => {
                failures += 1;
                eprintln!("error: {message}");
            }
        }
    }

    if failures == 0 {
        println!("{} envelope(s) conform to APS-V1-0004", envelopes.len());
        exit::SUCCESS
    } else {
        eprintln!("{failures} of {} envelope(s) failed", envelopes.len());
        exit::ERROR
    }
}

fn run_hash(args: &[String]) -> i32 {
    let Some(path) = args.first() else {
        eprintln!("usage: hash <file.json>");
        return exit::USAGE;
    };

    let envelopes = match envelopes_in(path) {
        Ok(envelopes) => envelopes,
        Err(message) => {
            eprintln!("error: {message}");
            return exit::ERROR;
        }
    };

    let mut failures = 0usize;
    for (index, value) in envelopes.iter().enumerate() {
        match parse_envelope(value, index).and_then(|envelope| {
            // Never print a digest for a non-conformant envelope: it would look
            // authoritative while describing something the standard rejects.
            envelope
                .validate()
                .map_err(|e| format!("envelope[{index}] invalid, refusing to hash: {e}"))?;
            content_hash_for(&envelope)
                .map(|hash| (envelope.session_id, hash))
                .map_err(|e| format!("envelope[{index}]: {e}"))
        }) {
            Ok((session_id, hash)) => println!("{session_id}\t{hash}"),
            Err(message) => {
                failures += 1;
                eprintln!("error: {message}");
            }
        }
    }

    if failures == 0 {
        exit::SUCCESS
    } else {
        exit::ERROR
    }
}

impl CommandHandler for SessionCaptureCommandHandler {
    fn execute(&self, command: &str, args: &[String], _config: &toml::Value) -> i32 {
        match command {
            "validate" => run_validate(args),
            "hash" => run_hash(args),
            other => {
                eprintln!("unknown command `{other}`; expected one of: validate, hash");
                exit::USAGE
            }
        }
    }

    fn commands(&self) -> Vec<CommandInfo> {
        vec![
            CommandInfo {
                name: "validate".to_string(),
                description: "Validate session envelopes against APS-V1-0004".to_string(),
                usage: "validate <file.json>".to_string(),
            },
            CommandInfo {
                name: "hash".to_string(),
                description: "Compute the canonical content_hash for captured content".to_string(),
                usage: "hash <file.json>".to_string(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_EXAMPLE: &str = "../examples/claude-source-envelope.json";
    const BATCH_EXAMPLE: &str = "../examples/workflow-exporter-batch.json";

    fn example(relative: &str) -> String {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join(relative)
            .to_string_lossy()
            .into_owned()
    }

    fn handler() -> SessionCaptureCommandHandler {
        SessionCaptureCommandHandler::new()
    }

    fn empty_config() -> toml::Value {
        toml::Value::Table(toml::map::Map::new())
    }

    #[test]
    fn registered_command_names_match_the_handler() {
        let listed: Vec<String> = handler().commands().into_iter().map(|c| c.name).collect();
        assert_eq!(listed, COMMAND_NAMES);
    }

    #[test]
    fn validate_accepts_the_shipped_examples() {
        for path in [SOURCE_EXAMPLE, BATCH_EXAMPLE] {
            assert_eq!(
                handler().execute("validate", &[example(path)], &empty_config()),
                exit::SUCCESS,
                "{path} must validate"
            );
        }
    }

    #[test]
    fn hash_succeeds_for_the_shipped_examples() {
        for path in [SOURCE_EXAMPLE, BATCH_EXAMPLE] {
            assert_eq!(
                handler().execute("hash", &[example(path)], &empty_config()),
                exit::SUCCESS
            );
        }
    }

    #[test]
    fn a_bare_array_batch_is_rejected_with_a_pointer_to_the_wrapper() {
        let path = std::env::temp_dir().join("apss-scs-cli-bare-array.json");
        std::fs::write(&path, b"[]").unwrap();
        let arg = path.to_string_lossy().into_owned();
        assert_eq!(
            handler().execute("validate", &[arg], &empty_config()),
            exit::ERROR,
            "section 5.1 forbids a bare top-level array"
        );
        let _ = std::fs::remove_file(&path);
    }

    fn with_temp_file(name: &str, contents: &str, run: impl Fn(String) -> i32) -> i32 {
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, contents).unwrap();
        let code = run(path.to_string_lossy().into_owned());
        let _ = std::fs::remove_file(&path);
        code
    }

    /// Duplicate member names are invisible after parsing, so the CLI must run
    /// the I-JSON check on the original text (spec 4.2.3).
    #[test]
    fn duplicate_member_names_are_rejected() {
        let code = with_temp_file("apss-scs-cli-dupe.json", r#"{"a":1,"a":2}"#, |arg| {
            handler().execute("validate", &[arg], &empty_config())
        });
        assert_eq!(code, exit::ERROR);
    }

    /// `hash` must refuse a non-conformant envelope rather than print a digest
    /// that looks authoritative for something the standard rejects.
    #[test]
    fn hash_refuses_a_non_conformant_envelope() {
        // Valid JSON and a parseable envelope, but `raw` is a scalar.
        let bad = r#"{
          "scs_version": "1.0",
          "origin": { "host": "h", "environment": "local" },
          "agent": "ClaudeCode",
          "source_format": "claude-jsonl-v1",
          "session_id": "abc-123",
          "started_at": "2026-05-02T14:03:11Z",
          "last_activity_at": "2026-05-02T15:20:44Z",
          "raw": 42
        }"#;
        let code = with_temp_file("apss-scs-cli-badraw.json", bad, |arg| {
            handler().execute("hash", &[arg], &empty_config())
        });
        assert_eq!(code, exit::ERROR);
    }

    #[test]
    fn an_empty_batch_is_rejected() {
        let code = with_temp_file(
            "apss-scs-cli-empty-batch.json",
            r#"{"envelopes":[]}"#,
            |arg| handler().execute("validate", &[arg], &empty_config()),
        );
        assert_eq!(
            code,
            exit::ERROR,
            "section 5.1 requires one or more envelopes"
        );
    }

    /// The schema permits unknown top-level fields, so a single envelope that
    /// happens to carry one named `envelopes` must not be misread as a batch.
    #[test]
    fn a_lone_envelope_with_an_envelopes_extension_field_is_not_a_batch() {
        let envelope = r#"{
          "scs_version": "1.0",
          "origin": { "host": "h", "environment": "local" },
          "agent": "ClaudeCode",
          "source_format": "claude-jsonl-v1",
          "session_id": "abc-123",
          "started_at": "2026-05-02T14:03:11Z",
          "last_activity_at": "2026-05-02T15:20:44Z",
          "raw": "transcript",
          "envelopes": "an extension field from some later minor"
        }"#;
        let code = with_temp_file("apss-scs-cli-ambiguous.json", envelope, |arg| {
            handler().execute("validate", &[arg], &empty_config())
        });
        assert_eq!(code, exit::SUCCESS);
    }

    #[test]
    fn missing_argument_is_a_usage_error_not_a_failure() {
        assert_eq!(
            handler().execute("validate", &[], &empty_config()),
            exit::USAGE
        );
        assert_eq!(handler().execute("hash", &[], &empty_config()), exit::USAGE);
    }

    #[test]
    fn an_unknown_command_is_a_usage_error() {
        assert_eq!(handler().execute("nope", &[], &empty_config()), exit::USAGE);
    }
}
