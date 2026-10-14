use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use workspace_standard::LaunchManifest;

const SCHEMA: &str = include_str!("../schemas/workspace-launch.schema.json");
const EXAMPLE: &str = include_str!("../examples/minimal/workspace-launch.json");
const SCHEMA_BASE: &str = "https://standards.agentparadise.dev/experimental/v1/workspace/";
const TERMINAL_STATES: [&str; 5] = ["succeeded", "failed", "cancelled", "timed_out", "lost"];

#[test]
fn minimal_example_conforms_to_schema_and_rust_contract() {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let instance: Value = serde_json::from_str(EXAMPLE).unwrap();
    assert!(validator.is_valid(&instance));

    let manifest: LaunchManifest = serde_json::from_value(instance).unwrap();
    manifest.validate().unwrap();
    assert_eq!(
        serde_json::to_value(&manifest).unwrap()["schema"],
        "apss.workspace-launch/v1"
    );
}

#[test]
fn schema_rejects_unknown_fields() {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let mut instance: Value = serde_json::from_str(EXAMPLE).unwrap();
    instance["provider_secret"] = Value::String("must-not-cross-boundary".into());
    assert!(!validator.is_valid(&instance));
}

/// Every schema the experiment ships, registered by `$id` so cross-file
/// `$ref`s resolve locally. This is the 0.2.0 validator: one entry point that
/// dispatches any workspace document on its `schema` discriminator.
struct Contract {
    schemas: BTreeMap<String, Value>,
}

impl Contract {
    fn load() -> Self {
        let schemas = json_files(&root().join("schemas"))
            .into_iter()
            .map(|path| {
                let name = path.file_name().unwrap().to_string_lossy().into_owned();
                (name, read_json(&path))
            })
            .collect();
        Self { schemas }
    }

    /// Validator for `target`, a schema file name optionally followed by a
    /// `#/$defs/...` pointer.
    fn validator(&self, target: &str) -> jsonschema::Validator {
        let mut options = jsonschema::options();
        for (name, schema) in &self.schemas {
            options.with_resource(
                format!("{SCHEMA_BASE}{name}"),
                jsonschema::Resource::from_contents(schema.clone()).unwrap(),
            );
        }
        options
            .build(&json!({ "$ref": format!("{SCHEMA_BASE}{target}") }))
            .unwrap_or_else(|error| panic!("schema {target} does not compile: {error}"))
    }

    /// `apss.<name>/v1` is defined by `schemas/<name>.schema.json`. A document
    /// without a known discriminator is not a workspace document.
    fn accepts(&self, document: &Value) -> bool {
        let Some(name) = document["schema"]
            .as_str()
            .and_then(|id| id.strip_prefix("apss."))
            .and_then(|id| id.strip_suffix("/v1"))
        else {
            return false;
        };
        let file = format!("{name}.schema.json");
        self.schemas.contains_key(&file) && self.validator(&file).is_valid(document)
    }
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(path: &Path) -> Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("{} is not JSON: {error}", path.display()))
}

fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("cannot list {}: {error}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    files
}

/// `fixtures/valid/<name>/*.json`, each paired with `<name>.schema.json`.
fn success_fixtures() -> Vec<(String, PathBuf)> {
    let mut fixtures = Vec::new();
    for dir in std::fs::read_dir(root().join("fixtures/valid")).unwrap() {
        let dir = dir.unwrap().path();
        let schema = format!("{}.schema.json", dir.file_name().unwrap().to_string_lossy());
        for file in json_files(&dir) {
            fixtures.push((schema.clone(), file));
        }
    }
    fixtures.sort();
    fixtures
}

#[test]
fn every_new_schema_has_success_fixtures() {
    let contract = Contract::load();
    let covered: Vec<_> = success_fixtures().into_iter().map(|(s, _)| s).collect();
    let new_schemas: Vec<_> = contract
        .schemas
        .keys()
        .filter(|name| *name != "workspace-launch.schema.json")
        .collect();
    assert_eq!(new_schemas.len(), 10);
    for schema in new_schemas {
        assert!(covered.contains(schema), "{schema} has no success fixture");
    }
}

#[test]
fn every_success_fixture_validates_and_round_trips() {
    let contract = Contract::load();
    for (schema, path) in success_fixtures() {
        let document = read_json(&path);
        assert!(
            contract.validator(&schema).is_valid(&document),
            "{} must satisfy {schema}",
            path.display()
        );
        assert!(
            contract.accepts(&document),
            "{} must be accepted when dispatched on its discriminator",
            path.display()
        );
        let reparsed: Value =
            serde_json::from_str(&serde_json::to_string(&document).unwrap()).unwrap();
        assert_eq!(reparsed, document, "{} must round-trip", path.display());
        assert!(contract.accepts(&reparsed));
    }
}

/// A failure fixture is a success fixture plus a minimal mutation, so the
/// rejection can only come from the mutation.
fn apply_failure_fixture(fixture: &Value) -> (String, Value, Value) {
    let base = fixture["base"].as_str().unwrap();
    let schema = format!(
        "{}.schema.json",
        Path::new(base)
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
    );
    let original = read_json(&root().join("fixtures").join(base));
    let mut document = original.clone();
    for edit in fixture["set"].as_array().unwrap() {
        let pointer = edit["path"].as_str().unwrap();
        let (parent, key) = pointer.rsplit_once('/').unwrap();
        let target = document
            .pointer_mut(parent)
            .unwrap_or_else(|| panic!("{base}: no parent at {pointer}"));
        match target {
            Value::Object(map) => {
                map.insert(key.to_string(), edit["value"].clone());
            }
            Value::Array(items) => items[key.parse::<usize>().unwrap()] = edit["value"].clone(),
            _ => panic!("{base}: {parent} is not a container"),
        }
    }
    for pointer in fixture["remove"].as_array().unwrap() {
        let (parent, key) = pointer.as_str().unwrap().rsplit_once('/').unwrap();
        let removed = document
            .pointer_mut(parent)
            .and_then(Value::as_object_mut)
            .and_then(|map| map.remove(key));
        assert!(removed.is_some(), "{base}: nothing to remove at {pointer}");
    }
    (schema, original, document)
}

#[test]
fn every_failure_fixture_is_rejected() {
    let contract = Contract::load();
    let fixtures = json_files(&root().join("fixtures/invalid"));
    assert!(!fixtures.is_empty());
    for path in fixtures {
        let (schema, original, document) = apply_failure_fixture(&read_json(&path));
        let validator = contract.validator(&schema);
        assert!(
            validator.is_valid(&original),
            "{}: base must be valid",
            path.display()
        );
        assert_ne!(
            original,
            document,
            "{}: mutation is a no-op",
            path.display()
        );
        assert!(
            !validator.is_valid(&document),
            "{} must be rejected by {schema}",
            path.display()
        );
        assert!(!contract.accepts(&document), "{}", path.display());
    }
}

#[test]
fn required_failure_cases_are_present() {
    // The exit tests the plan names, by fixture.
    for name in [
        "unknown-field-operation",
        "succeeded-with-primary-failure",
        "gap-lost-bytes-zero",
        "complete-on-running",
        "credential-value-on-ref",
        "credential-value-on-binding",
        "credential-value-in-environment",
        "credential-token-in-environment",
        "credential-value-in-operation-metadata",
        "credential-value-in-status-metadata",
        "credential-value-in-result-metadata",
        "credential-value-in-result-process-metadata",
        "descriptor-missing-enforceable-limits",
        "error-kind-not-in-closed-set",
    ] {
        let path = root().join(format!("fixtures/invalid/{name}.json"));
        assert!(path.exists(), "missing failure fixture {name}");
    }
}

fn object_pointers(value: &Value, pointer: String, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            out.push(pointer.clone());
            for (key, child) in map {
                object_pointers(child, format!("{pointer}/{key}"), out);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                object_pointers(child, format!("{pointer}/{index}"), out);
            }
        }
        _ => {}
    }
}

/// Every object node, typed objects and free-form string maps alike, must
/// refuse a smuggled credential value field.
#[test]
fn no_object_admits_a_credential_value_field() {
    let contract = Contract::load();
    for (schema, path) in success_fixtures() {
        let validator = contract.validator(&schema);
        let document = read_json(&path);
        let mut pointers = Vec::new();
        object_pointers(&document, String::new(), &mut pointers);
        for pointer in pointers {
            for field in [
                "value", "secret", "token", "password", "GH_TOKEN", "api_key",
            ] {
                let mut mutated = document.clone();
                let target = mutated.pointer_mut(&pointer).unwrap();
                target[field] = json!("ghp_exampleSecretValue");
                assert!(
                    !validator.is_valid(&mutated),
                    "{} at '{pointer}' admits `{field}`",
                    path.display()
                );
            }
        }
    }
}

/// Launch/v1 documents: the shipped example plus `fixtures/launch-v1`.
fn launch_v1_fixtures() -> Vec<(PathBuf, Value)> {
    let fixtures = root().join("fixtures/launch-v1");
    let mut paths = vec![root().join("examples/minimal/workspace-launch.json")];
    paths.extend(json_files(&fixtures.join("valid")));
    paths.extend(json_files(&fixtures.join("invalid")));
    paths
        .into_iter()
        .map(|path| {
            let document = read_json(&path);
            (path, document)
        })
        .collect()
}

#[test]
fn launch_v1_verdicts_are_identical_under_0_1_0_and_0_2_0() {
    let frozen = read_json(&root().join("fixtures/launch-v1/workspace-launch-0.1.0.schema.json"));
    let v0_1_0 = jsonschema::validator_for(&frozen).unwrap();
    let v0_2_0 = Contract::load();
    let mut accepted = 0;
    let mut rejected = 0;
    for (path, document) in launch_v1_fixtures() {
        let expected = v0_1_0.is_valid(&document);
        assert_eq!(
            v0_2_0.accepts(&document),
            expected,
            "{}: 0.2.0 verdict differs from 0.1.0",
            path.display()
        );
        let in_valid_dir = !path.components().any(|c| c.as_os_str() == "invalid");
        assert_eq!(expected, in_valid_dir, "{} is misfiled", path.display());
        if expected {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }
    assert!(accepted >= 2 && rejected >= 5);
}

fn is_terminal(state: &str) -> bool {
    TERMINAL_STATES.contains(&state)
}

/// The fixture provider's cursor encoding, `g<generation>:<byte offset>`.
/// Portable consumers treat cursors as opaque; only this oracle reads them.
fn parse_cursor(cursor: &Value, at: &str) -> (u64, u64) {
    let text = cursor.as_str().unwrap();
    text.strip_prefix('g')
        .and_then(|rest| rest.split_once(':'))
        .and_then(|(generation, offset)| Some((generation.parse().ok()?, offset.parse().ok()?)))
        .unwrap_or_else(|| panic!("{at}: cursor {text} is not g<generation>:<offset>"))
}

/// Decoded length of a padded base64 string the schema has already accepted.
fn decoded_len(data_b64: &str) -> u64 {
    let padding = data_b64.bytes().rev().take_while(|b| *b == b'=').count();
    (data_b64.len() / 4 * 3 - padding) as u64
}

/// Every byte from `cursor` to `next_cursor` is either returned in
/// `data_b64` or declared lost in `gap`. Returns the bytes the page declares
/// lost, `None` when the extent is unknown.
fn check_page_accounting(page: &Value, at: &str) -> Option<u64> {
    let (generation, offset) = parse_cursor(&page["cursor"], at);
    let (next_generation, next) = parse_cursor(&page["next_cursor"], at);
    assert_eq!(
        next_generation, generation,
        "{at}: a page stays in its generation"
    );
    let returned = decoded_len(page["data_b64"].as_str().unwrap());
    let lost = match page.get("gap") {
        None => Some(0),
        Some(gap) => gap["lost_bytes"].as_u64(),
    };
    match lost {
        Some(lost) => assert_eq!(
            next,
            offset + lost + returned,
            "{at}: bytes between cursors are neither returned nor declared lost"
        ),
        None => assert!(
            next >= offset + returned,
            "{at}: next cursor precedes the data"
        ),
    }
    lost
}

#[test]
fn every_observation_fixture_accounts_for_every_byte() {
    let pages = json_files(&root().join("fixtures/valid/workspace-observation"));
    assert!(
        pages
            .iter()
            .any(|path| read_json(path).get("gap").is_some())
    );
    for path in pages {
        check_page_accounting(&read_json(&path), &path.display().to_string());
    }
}

/// Seeded output state for a scenario: `produced_bytes` written in
/// `generation`, of which the descriptor's `retention_bytes` are retained.
struct SeededOutput {
    generation: u64,
    retained_from: u64,
    produced: u64,
}

/// Lifecycle hazards (spec 5.1.11). Every id needs a behaviour fixture and a
/// MUST clause in the spec naming it.
const HAZARDS: [&str; 7] = ["LH-1", "LH-2", "LH-3", "LH-4", "LH-5", "LH-6", "LH-7"];
/// Conditions a provider harness injects before a step (`given`). Closed so a
/// misspelled condition fails here instead of being silently not injected.
const CONDITIONS: [&str; 9] = [
    "detached_descendant",
    "caller_cancelled_during_confirmation",
    "launch_header_not_seen",
    "destroyed_during_credential_resolution",
    "workspace_destroyed",
    "backend_stuck_removing",
    "timeout_elapsed",
    "observation_gap",
    "fifo_in_workspace",
];
/// Conditions under which the provider must hold no operation record once the
/// call returns, so the step must declare `operation_records: 0`.
const DESTROYED: [&str; 2] = [
    "destroyed_during_credential_resolution",
    "workspace_destroyed",
];
/// Calls a step may make: the operation contract plus the harness file calls
/// (`file_exists`, `read_file`) that LH-7 drives against a named path.
const CALLS: [&str; 8] = [
    "start",
    "status",
    "cancel",
    "observe",
    "result",
    "finalize",
    "file_exists",
    "read_file",
];
/// Outcome schemas outside `schemas/`: fixture-level shapes for harness calls
/// the contract does not define.
const FIXTURE_SCHEMA_PREFIX: &str = "fixtures/";
/// Upper bound on any `settles_within_seconds`: a bound larger than this is
/// not a settlement bound.
const MAX_SETTLE_SECONDS: f64 = 300.0;

/// The responses a step accepts: its single `expect`, or each of
/// `expect_one_of` when the contract allows more than one correct outcome.
fn step_outcomes<'a>(step: &'a Value, at: &str) -> Vec<&'a Value> {
    match (step.get("expect"), step.get("expect_one_of")) {
        (Some(expect), None) => vec![expect],
        (None, Some(Value::Array(outcomes))) if !outcomes.is_empty() => outcomes.iter().collect(),
        _ => panic!("{at}: exactly one of expect or a non-empty expect_one_of"),
    }
}

/// Replays one scripted scenario: every expected response satisfies its
/// schema, and the runtime rules schemas cannot express hold across steps.
/// Returns the scenario's hazard id, if it encodes one.
fn check_scenario(contract: &Contract, path: &Path) -> Option<String> {
    let scenario = read_json(path);
    let name = path.display();
    let hazard = scenario["hazard"].as_str().map(|hazard| {
        assert!(HAZARDS.contains(&hazard), "{name}: unknown hazard {hazard}");
        hazard.to_string()
    });
    let descriptor = scenario["provider"].as_str().map(|provider| {
        let descriptor = read_json(&root().join("fixtures").join(provider));
        assert!(contract.accepts(&descriptor), "{name}: provider descriptor");
        descriptor
    });
    let integrity = descriptor
        .as_ref()
        .map(|descriptor| descriptor["observation"]["integrity"].as_str().unwrap());
    let output = scenario.get("output").map(|output| {
        let retention = descriptor.as_ref().expect("seeded output needs a provider")["observation"]
            ["retention_bytes"]
            .as_u64()
            .unwrap();
        let produced = output["produced_bytes"].as_u64().unwrap();
        SeededOutput {
            generation: output["generation"].as_u64().unwrap(),
            retained_from: produced.saturating_sub(retention),
            produced,
        }
    });
    let mut launches = 0;
    let mut started: Vec<(Value, Value)> = Vec::new();
    let mut terminal: Option<String> = None;
    let mut injected = false;
    let mut loss_seen = false;
    let mut stuck_drained = false;
    let mut fifo_given = false;
    let mut fifo_calls: Vec<&str> = Vec::new();
    for (index, step) in scenario["steps"].as_array().unwrap().iter().enumerate() {
        let at = format!("{name} step {index}");
        let call = step["call"].as_str().unwrap();
        assert!(CALLS.contains(&call), "{at}: unknown call {call}");
        let outcomes = step_outcomes(step, &at);
        for outcome in &outcomes {
            let document = &outcome["document"];
            let schema = outcome["schema"].as_str().unwrap();
            let valid = match schema.strip_prefix(FIXTURE_SCHEMA_PREFIX) {
                Some(file) => {
                    jsonschema::validator_for(&read_json(&root().join("fixtures").join(file)))
                        .unwrap()
                        .is_valid(document)
                }
                None => contract.validator(schema).is_valid(document),
            };
            assert!(valid, "{at}: {schema}");
            if document.get("schema").is_some() {
                assert!(contract.accepts(document), "{at}: discriminator dispatch");
            }
            // A confirmed cancel and a live descendant cannot coexist; under a
            // hazard the outcome must say which one it is.
            let cancelled = document["state"] == "cancelled";
            match outcome.get("descendant_alive").map(Value::as_bool) {
                Some(Some(true)) => assert!(!cancelled, "{at}: cancelled while a descendant lives"),
                Some(Some(false)) => {}
                Some(None) => panic!("{at}: descendant_alive is a boolean"),
                None => assert!(
                    !(cancelled && hazard.is_some()),
                    "{at}: a confirmed cancel under a hazard must declare descendant_alive false"
                ),
            }
        }

        let given = step.get("given").map(|given| given.as_array().unwrap());
        let bound = step.get("settles_within_seconds").map(|bound| {
            let bound = bound.as_f64().expect("settles_within_seconds is a number");
            assert!(
                bound > 0.0 && bound <= MAX_SETTLE_SECONDS,
                "{at}: settlement bound {bound} outside (0, {MAX_SETTLE_SECONDS}]"
            );
            bound
        });
        if let Some(given) = given {
            assert!(
                !given.is_empty(),
                "{at}: given names at least one condition"
            );
            for condition in given {
                let condition = condition.as_str().unwrap();
                assert!(
                    CONDITIONS.contains(&condition),
                    "{at}: unknown condition {condition}"
                );
            }
            assert!(
                bound.is_some(),
                "{at}: an injected condition needs a settlement bound"
            );
            injected = true;
        }
        let given: Vec<&str> = given
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();

        // LH-4: a destroyed workspace holds no operation record. The harness
        // counts the provider's records (idempotency and registry entries) for
        // the workspace once the call, and any resolver it blocked on, returns.
        match step.get("operation_records").map(Value::as_u64) {
            Some(Some(records)) => assert!(
                records == 0 || !given.iter().any(|c| DESTROYED.contains(c)),
                "{at}: a destroyed workspace retains {records} operation record(s)"
            ),
            Some(None) => panic!("{at}: operation_records is a count"),
            None => assert!(
                !given.iter().any(|c| DESTROYED.contains(c)),
                "{at}: after destroy the step must declare operation_records 0"
            ),
        }

        // LH-5: a drain repeats observe, following next_cursor, until a page
        // completes or the call fails. The bound covers the whole loop.
        let drains = match step.get("repeat_until_complete") {
            None => false,
            Some(Value::Bool(true)) => {
                assert_eq!(call, "observe", "{at}: only observe repeats");
                assert!(bound.is_some(), "{at}: a drain needs a total bound");
                for outcome in &outcomes {
                    let document = &outcome["document"];
                    assert!(
                        document["complete"] == true || document.get("kind").is_some(),
                        "{at}: a drain ends complete or with a typed error"
                    );
                }
                true
            }
            Some(_) => panic!("{at}: repeat_until_complete is true when present"),
        };
        if given.contains(&"backend_stuck_removing") && !stuck_drained {
            // A status call first could repair settlement the drain must reach
            // on its own.
            assert!(
                drains && given.contains(&"timeout_elapsed"),
                "{at}: a stuck backend is first met by a timeout drain"
            );
            stuck_drained = true;
        }

        // LH-7: the file boundary itself is driven, against a named path.
        fifo_given |= given.contains(&"fifo_in_workspace");
        if call == "file_exists" || call == "read_file" {
            assert!(
                step["args"]["workspace"].is_object() && step["args"]["path"].is_string(),
                "{at}: a file call names its workspace and path"
            );
            if given.contains(&"fifo_in_workspace") {
                fifo_calls.push(call);
            }
        }
        if call == "read_file" {
            for outcome in &outcomes {
                assert!(
                    outcome["document"].get("kind").is_some(),
                    "{at}: reading a non-regular file is a typed error, not data"
                );
            }
        }

        let now = step["launches"].as_u64().unwrap();
        assert!(now >= launches, "{at}: launches never decrease");
        if call == "start" {
            let spec = &step["args"]["spec"];
            let key = &spec["operation_key"];
            let document = &outcomes[0]["document"];
            let is_error = outcomes
                .iter()
                .all(|outcome| outcome["document"].get("kind").is_some());
            assert!(
                outcomes.len() == 1 || is_error,
                "{at}: a start with alternative outcomes may only fail"
            );
            match started
                .iter()
                .find(|(earlier, _)| &earlier["operation_key"] == key)
            {
                Some((earlier, earlier_ref)) if earlier == spec => {
                    assert_eq!(document, earlier_ref, "{at}: same key, same spec, same ref");
                    assert_eq!(now, launches, "{at}: an idempotent start launches nothing");
                }
                Some(_) => {
                    assert_eq!(document["kind"], "conflict", "{at}: same key, new spec");
                    assert_eq!(now, launches, "{at}: a conflict launches nothing");
                }
                None if !is_error => {
                    assert_eq!(now, launches + 1, "{at}: a new start launches once");
                    started.push((spec.clone(), document.clone()));
                }
                None => assert_eq!(now, launches, "{at}: a failed start launches nothing"),
            }
        } else {
            assert_eq!(now, launches, "{at}: only start launches");
        }
        launches = now;

        // Operation states only: a finalization outcome has its own `state`
        // and no operation ref.
        let states: Vec<&str> = outcomes
            .iter()
            .map(|outcome| &outcome["document"])
            .filter(|document| document.get("ref").is_some())
            .filter_map(|document| document["state"].as_str())
            .collect();
        if let Some(previous) = &terminal {
            for state in &states {
                assert_eq!(state, previous, "{at}: terminal states are sticky");
            }
        } else if let [state] = states[..] {
            // Only an unambiguous outcome fixes the terminal state.
            if outcomes.len() == 1 && is_terminal(state) {
                terminal = Some(state.to_string());
            }
        }

        for outcome in &outcomes {
            let document = &outcome["document"];
            if let Some(gap) = document.get("gap") {
                match integrity {
                    Some("exact") => assert!(gap["lost_bytes"].is_u64(), "{at}: exact gap"),
                    Some(_) => {}
                    None => panic!("{at}: a gap needs the provider descriptor"),
                }
                assert!(
                    output.is_some(),
                    "{at}: a gap needs seeded output to derive it"
                );
            }
            if call == "observe" && document.get("data_b64").is_some() {
                // A drain's final page starts wherever the loop reached.
                if !drains {
                    assert_eq!(
                        document["cursor"], step["args"]["cursor"],
                        "{at}: a page starts at the requested cursor"
                    );
                }
                let lost = check_page_accounting(document, &at);
                // LH-6: loss persists to completion, whichever page saw it.
                loss_seen |= lost != Some(0);
                if document.get("loss_detected").is_some() {
                    assert!(loss_seen, "{at}: loss_detected without a loss");
                }
                if loss_seen && document["complete"] == true {
                    assert_eq!(
                        document["loss_detected"], true,
                        "{at}: completion after a loss must carry loss_detected"
                    );
                }
                if let Some(output) = &output {
                    let (generation, offset) = parse_cursor(&document["cursor"], &at);
                    let (_, next) = parse_cursor(&document["next_cursor"], &at);
                    assert_eq!(generation, output.generation, "{at}: seeded generation");
                    assert_eq!(
                        lost,
                        Some(output.retained_from.saturating_sub(offset)),
                        "{at}: loss must follow from seeded retention"
                    );
                    assert!(
                        next <= output.produced,
                        "{at}: page runs past produced output"
                    );
                    if document["complete"] == true {
                        assert_eq!(
                            next, output.produced,
                            "{at}: completion before the terminal offset truncates output"
                        );
                    }
                }
            }
        }
    }
    if hazard.is_some() {
        assert!(injected, "{name}: a hazard scenario injects its condition");
    }
    if fifo_given {
        for call in ["file_exists", "read_file"] {
            assert!(
                fifo_calls.contains(&call),
                "{name}: the FIFO must be met by {call}"
            );
        }
    }
    hazard
}

#[test]
fn behaviour_fixtures_replay_with_expected_states() {
    let contract = Contract::load();
    let scenarios = json_files(&root().join("fixtures/behaviour"));
    assert!(scenarios.len() >= 14);
    for path in scenarios {
        check_scenario(&contract, &path);
    }
}

/// The cancel steps whose unconfirmed outcome (5.1.4) leaves the descendant
/// unknown: LH-1 step 1, LH-2 step 3 (the retried cancel), LH-3 step 1.
const UNCONFIRMED_CANCEL_STEPS: [(&str, usize); 3] = [
    ("hazard-double-fork-orphan.json", 1),
    ("hazard-interrupted-cancel.json", 3),
    ("hazard-cancel-before-launch-header.json", 1),
];

/// Whether a step accepts a provider's response, as a harness replays it:
/// some outcome's document equals the response, and that outcome's
/// `descendant_alive`, when present, equals what the harness observed.
fn step_accepts(step: &Value, response: &Value, descendant_alive: bool) -> bool {
    step_outcomes(step, "step").iter().any(|outcome| {
        outcome["document"] == *response
            && outcome
                .get("descendant_alive")
                .is_none_or(|pinned| pinned.as_bool() == Some(descendant_alive))
    })
}

#[test]
fn unconfirmed_cancel_accepts_either_descendant_state_and_cancelled_only_when_dead() {
    for (file, index) in UNCONFIRMED_CANCEL_STEPS {
        let scenario = read_json(&root().join("fixtures/behaviour").join(file));
        let step = &scenario["steps"][index];
        let at = format!("{file} step {index}");
        assert_eq!(step["call"], "cancel", "{at}");
        let outcomes = step_outcomes(step, &at);
        let find = |key: &str, value: &str| {
            outcomes
                .iter()
                .map(|outcome| &outcome["document"])
                .find(|document| document[key] == value)
                .unwrap_or_else(|| panic!("{at}: no {key} {value} alternative"))
        };
        let cancelled = find("state", "cancelled");
        let timeout = find("kind", "timeout");
        assert!(
            timeout.get("ref").is_some(),
            "{at}: timeout carries the ref"
        );

        // Accepted by 0.2.1, still accepted.
        assert!(
            step_accepts(step, cancelled, false),
            "{at}: confirmed cancel"
        );
        assert!(
            step_accepts(step, timeout, true),
            "{at}: timeout, descendant alive"
        );
        // New in 0.2.2: unconfirmed means unknown, not known alive.
        assert!(
            step_accepts(step, timeout, false),
            "{at}: timeout, descendant gone"
        );
        // Still rejected: a confirmed cancel while a descendant lives.
        assert!(
            !step_accepts(step, cancelled, true),
            "{at}: cancelled while alive"
        );
    }
}

#[test]
fn every_lifecycle_hazard_has_a_fixture_and_a_must_clause() {
    let contract = Contract::load();
    let covered: Vec<String> = json_files(&root().join("fixtures/behaviour"))
        .iter()
        .filter_map(|path| check_scenario(&contract, path))
        .collect();
    let spec = std::fs::read_to_string(root().join("docs/01_spec.md")).unwrap();
    for hazard in HAZARDS {
        assert!(
            covered.iter().any(|id| id == hazard),
            "{hazard}: no behaviour fixture encodes it"
        );
        let tag = format!("**{hazard}**");
        assert!(
            spec.split("\n- ")
                .any(|clause| clause.starts_with(&tag) && clause.contains("MUST")),
            "{hazard}: the spec names no MUST clause for it"
        );
    }
}
