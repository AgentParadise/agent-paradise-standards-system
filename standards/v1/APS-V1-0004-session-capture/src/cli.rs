//! CLI commands for APS-V1-0004 (APS-V1-0000.CL01).
//!
//! The standard is definitional, so these commands do not manage a project.
//! They expose the two operations a consumer most often needs to perform against
//! the standard itself:
//!
//! - `validate` answers "is this envelope conformant?" against both the JSON
//!   Schema's structural rules and the crate's validation, so a producer can
//!   check its output without writing a test harness.
//! - `hash` computes the canonical `content_hash` (section 4.2.3), so a store
//!   implementer can compare their computation against the reference one. The
//!   hash is the most interoperability-sensitive rule in the standard, and a
//!   store that derives it differently silently fails to deduplicate.

use apss_core::registry::{CommandHandler, CommandInfo};

use crate::{SessionEnvelope, content_hash::content_hash_for};

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
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{path} is not valid JSON: {e}"))?;

    if let Some(envelopes) = value.get("envelopes") {
        let array = envelopes
            .as_array()
            .ok_or_else(|| format!("{path}: `envelopes` must be an array (section 5.1)"))?;
        return Ok(array.clone());
    }
    if value.is_array() {
        return Err(format!(
            "{path}: a batch body must be wrapped as {{ \"envelopes\": [...] }}, \
             not a bare array (section 5.1)"
        ));
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
