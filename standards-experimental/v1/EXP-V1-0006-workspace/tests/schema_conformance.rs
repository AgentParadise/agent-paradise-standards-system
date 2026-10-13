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

/// Replays one scripted scenario: every expected response satisfies its
/// schema, and the runtime rules schemas cannot express hold across steps.
fn check_scenario(contract: &Contract, path: &Path) {
    let scenario = read_json(path);
    let name = path.display();
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
    for (index, step) in scenario["steps"].as_array().unwrap().iter().enumerate() {
        let at = format!("{name} step {index}");
        let call = step["call"].as_str().unwrap();
        let document = &step["expect"]["document"];
        let schema = step["expect"]["schema"].as_str().unwrap();
        assert!(
            contract.validator(schema).is_valid(document),
            "{at}: {schema}"
        );
        if document.get("schema").is_some() {
            assert!(contract.accepts(document), "{at}: discriminator dispatch");
        }

        let now = step["launches"].as_u64().unwrap();
        assert!(now >= launches, "{at}: launches never decrease");
        let is_error = document.get("kind").is_some();
        if call == "start" {
            let spec = &step["args"]["spec"];
            let key = &spec["operation_key"];
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
                None => {}
            }
        } else {
            assert_eq!(now, launches, "{at}: only start launches");
        }
        launches = now;

        if let Some(state) = document["state"].as_str() {
            if let Some(previous) = &terminal {
                assert_eq!(state, previous, "{at}: terminal states are sticky");
            } else if is_terminal(state) {
                terminal = Some(state.to_string());
            }
        }
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
            assert_eq!(
                document["cursor"], step["args"]["cursor"],
                "{at}: a page starts at the requested cursor"
            );
            let lost = check_page_accounting(document, &at);
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
            }
        }
    }
}

#[test]
fn behaviour_fixtures_replay_with_expected_states() {
    let contract = Contract::load();
    let scenarios = json_files(&root().join("fixtures/behaviour"));
    assert!(scenarios.len() >= 4);
    for path in scenarios {
        check_scenario(&contract, &path);
    }
}
