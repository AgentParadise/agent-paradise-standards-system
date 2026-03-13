//! Tests for fitness.toml and fitness-exceptions.toml deserialization.

use fitness_functions::{ExceptionSet, FitnessConfig, Severity, ThresholdRule};

#[test]
fn parse_minimal_config() {
    let toml_str = r#"
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-cc"
name = "Max Cyclomatic Complexity"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 15
scope = "function"
"#;
    let config: FitnessConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.config.topology_dir, ".topology");
    assert_eq!(config.config.exceptions, "fitness-exceptions.toml");
    assert_eq!(config.config.severity_default, Severity::Error);
    assert_eq!(config.rules.threshold.len(), 1);
    assert!(config.rules.dependency.is_empty());
}

#[test]
fn parse_full_config() {
    let toml_str = r#"
[config]
topology_dir = ".topology"
exceptions = "custom-exceptions.toml"
severity_default = "warning"

[[rules.threshold]]
id = "max-cc"
name = "Max Cyclomatic Complexity"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 15
scope = "function"
severity = "error"
exclude = ["**/test_*", "**/tests/**"]

[[rules.threshold]]
id = "max-loc"
name = "Max Lines of Code"
source = "metrics/file_metrics.json"
field = "lines_of_code"
max = 500
scope = "file"

[[rules.threshold]]
id = "min-instability"
name = "Min Instability"
source = "metrics/coupling.json"
field = "instability"
min = 0.1
max = 0.9
scope = "module"

[[rules.dependency]]
id = "no-circular"
name = "No Circular Dependencies"
type = "forbidden"
from = { path = "src/**" }
to = { path = "src/**" }
circular = true
severity = "error"
"#;
    let config: FitnessConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.config.exceptions, "custom-exceptions.toml");
    assert_eq!(config.config.severity_default, Severity::Warning);
    assert_eq!(config.rules.threshold.len(), 3);
    assert_eq!(config.rules.dependency.len(), 1);

    // Check threshold rule fields
    let cc = &config.rules.threshold[0];
    assert_eq!(cc.id, "max-cc");
    assert_eq!(cc.max, Some(15.0));
    assert_eq!(cc.min, None);
    assert_eq!(cc.severity, Some(Severity::Error));
    assert_eq!(cc.exclude, vec!["**/test_*", "**/tests/**"]);

    // Check min/max rule
    let inst = &config.rules.threshold[2];
    assert_eq!(inst.min, Some(0.1));
    assert_eq!(inst.max, Some(0.9));

    // Check dependency rule
    let dep = &config.rules.dependency[0];
    assert_eq!(dep.rule_type, "forbidden");
    assert_eq!(dep.from.path, "src/**");
    assert!(dep.circular);
}

#[test]
fn parse_exceptions() {
    let toml_str = r##"
[max-cc."src/engine.py::execute"]
value = 42
issue = "#138"

[max-cc."src/setup.py::configure"]
value = 28
issue = "#185"

[max-loc."src/setup.py"]
value = 2284
issue = "#185"
"##;
    let set: ExceptionSet = toml::from_str(toml_str).unwrap();

    // Check max-cc exceptions
    let exc = set.get("max-cc", "src/engine.py::execute").unwrap();
    assert_eq!(exc.value, Some(42.0));
    assert_eq!(exc.issue, "#138");

    let exc2 = set.get("max-cc", "src/setup.py::configure").unwrap();
    assert_eq!(exc2.value, Some(28.0));
    assert_eq!(exc2.issue, "#185");

    // Check max-loc exception
    let exc3 = set.get("max-loc", "src/setup.py").unwrap();
    assert_eq!(exc3.value, Some(2284.0));

    // Check missing
    assert!(set.get("max-cc", "nonexistent").is_none());
    assert!(set.get("nonexistent", "src/engine.py::execute").is_none());
}

#[test]
fn parse_exceptions_with_targets() {
    let toml_str = r##"
[no-circular."src/api"]
targets = ["src/domain", "src/infra"]
issue = "#200"
"##;
    let set: ExceptionSet = toml::from_str(toml_str).unwrap();
    let exc = set.get("no-circular", "src/api").unwrap();
    assert_eq!(exc.targets.as_ref().unwrap().len(), 2);
    assert_eq!(exc.issue, "#200");
}

#[test]
fn rule_validate_rejects_no_bounds() {
    let rule = ThresholdRule {
        id: "bad".to_string(),
        name: "Bad Rule".to_string(),
        source: "metrics/test.json".to_string(),
        field: "value".to_string(),
        max: None,
        min: None,
        scope: "function".to_string(),
        severity: None,
        exclude: vec![],
    };
    let err = rule.validate().unwrap_err();
    assert!(err.contains("at least one of"));
}

#[test]
fn effective_severity_uses_override() {
    let rule = ThresholdRule {
        id: "test".to_string(),
        name: "Test".to_string(),
        source: "m.json".to_string(),
        field: "v".to_string(),
        max: Some(10.0),
        min: None,
        scope: "function".to_string(),
        severity: Some(Severity::Warning),
        exclude: vec![],
    };
    assert_eq!(rule.effective_severity(Severity::Error), Severity::Warning);
}

#[test]
fn effective_severity_falls_back_to_default() {
    let rule = ThresholdRule {
        id: "test".to_string(),
        name: "Test".to_string(),
        source: "m.json".to_string(),
        field: "v".to_string(),
        max: Some(10.0),
        min: None,
        scope: "function".to_string(),
        severity: None,
        exclude: vec![],
    };
    assert_eq!(rule.effective_severity(Severity::Warning), Severity::Warning);
    assert_eq!(rule.effective_severity(Severity::Error), Severity::Error);
}
