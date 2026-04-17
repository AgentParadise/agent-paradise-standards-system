//! Tests for fitness.toml and fitness-exceptions.toml deserialization.

use architecture_fitness::{
    DimensionCode, ExceptionSet, FitnessConfig, FitnessValidator, Severity, ThresholdRule,
};
use std::fs;
use tempfile::TempDir;

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
        dimension: None,
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
        dimension: None,
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
        dimension: None,
        source: "m.json".to_string(),
        field: "v".to_string(),
        max: Some(10.0),
        min: None,
        scope: "function".to_string(),
        severity: None,
        exclude: vec![],
    };
    assert_eq!(
        rule.effective_severity(Severity::Warning),
        Severity::Warning
    );
    assert_eq!(rule.effective_severity(Severity::Error), Severity::Error);
}

// ─── Wave 2: Config Extension Tests ────────────────────────────────────────

#[test]
fn parse_v2_config_with_dimensions_and_system_fitness() {
    let toml_str = r#"
[config]
topology_dir = ".topology"

[dimensions]
AC01 = false
PF01 = false
AV01 = false

[system_fitness]
enabled = true
min_score = 0.7

[system_fitness.weights]
MT01 = 0.25
MD01 = 0.25
ST01 = 0.20
SC01 = 0.15
LG01 = 0.15

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
dimension = "MT01"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 10
scope = "function"

[[rules.dependency]]
id = "no-circular"
name = "No Circular Deps"
dimension = "MD01"
type = "forbidden"
from = { path = "src/**" }
to = { path = "src/**" }
circular = true

[[rules.structural]]
id = "layer-enforcement"
name = "Controllers Must Not Import Repos"
dimension = "ST01"
pattern = "forbidden_import"
from = { path = "**/controllers/**" }
to = { path = "**/repositories/**" }
"#;
    let config: FitnessConfig = toml::from_str(toml_str).unwrap();

    // Dimensions
    assert!(config.dimensions.mt01);
    assert!(config.dimensions.md01);
    assert!(!config.dimensions.ac01);
    assert!(!config.dimensions.pf01);
    assert!(config.dimensions.is_enabled(DimensionCode::MT01));
    assert!(!config.dimensions.is_enabled(DimensionCode::AC01));

    // System fitness
    assert!(config.system_fitness.enabled);
    assert_eq!(config.system_fitness.min_score, 0.7);
    assert_eq!(config.system_fitness.weights.len(), 5);
    assert_eq!(config.system_fitness.weights["MT01"], 0.25);

    // Threshold rule with dimension
    assert_eq!(
        config.rules.threshold[0].dimension.as_deref(),
        Some("MT01")
    );

    // Dependency rule with dimension
    assert_eq!(
        config.rules.dependency[0].dimension.as_deref(),
        Some("MD01")
    );

    // Structural rule
    assert_eq!(config.rules.structural.len(), 1);
    assert_eq!(config.rules.structural[0].pattern, "forbidden_import");
}

#[test]
fn dimension_disabled_without_reason_fails() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    fs::create_dir_all(root.join(".topology")).unwrap();
    fs::write(
        root.join("fitness.toml"),
        r#"
[config]
topology_dir = ".topology"

[dimensions]
MT01 = false
"#,
    )
    .unwrap();

    let result = FitnessValidator::load(root, None);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("MT01"),
        "error should mention the dimension: {err}"
    );
}

#[test]
fn dimension_disabled_with_reason_passes() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    fs::create_dir_all(root.join(".topology")).unwrap();
    fs::write(
        root.join("fitness.toml"),
        r#"
[config]
topology_dir = ".topology"

[dimensions]
MT01 = false

[dimensions.reasons]
MT01 = "Pure library, complexity managed externally"
"#,
    )
    .unwrap();

    let result = FitnessValidator::load(root, None);
    assert!(result.is_ok());
}

#[test]
fn weight_validation_rejects_bad_sum() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    fs::create_dir_all(root.join(".topology")).unwrap();
    fs::write(
        root.join("fitness.toml"),
        r#"
[config]
topology_dir = ".topology"

[system_fitness.weights]
MT01 = 0.5
MD01 = 0.3
"#,
    )
    .unwrap();

    let result = FitnessValidator::load(root, None);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("weights"),
        "error should mention weights: {err}"
    );
}

#[test]
fn weight_validation_accepts_correct_sum() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    fs::create_dir_all(root.join(".topology")).unwrap();
    fs::write(
        root.join("fitness.toml"),
        r#"
[config]
topology_dir = ".topology"

[system_fitness.weights]
MT01 = 0.5
MD01 = 0.5
"#,
    )
    .unwrap();

    let result = FitnessValidator::load(root, None);
    assert!(result.is_ok());
}

#[test]
fn parse_adapter_config() {
    let toml_str = r#"
[config]
topology_dir = ".topology"

[[adapters]]
id = "cargo-audit"
dimension = "SC01"
command = "cargo audit --json"
output = "adapters/sc01/vulnerabilities.json"
normalizer = "builtin:cargo-audit"

[[adapters]]
id = "vsa"
dimension = "MD01"
input = ".vsa/analysis.json"
output = "adapters/md01/vsa-slices.json"
normalizer = "builtin:vsa"
"#;
    let config: FitnessConfig = toml::from_str(toml_str).unwrap();

    assert_eq!(config.adapters.len(), 2);
    assert_eq!(config.adapters[0].id, "cargo-audit");
    assert_eq!(config.adapters[0].dimension, "SC01");
    assert_eq!(
        config.adapters[0].command.as_deref(),
        Some("cargo audit --json")
    );
    assert!(config.adapters[0].input.is_none());
    assert_eq!(config.adapters[0].normalizer, "builtin:cargo-audit");

    assert_eq!(config.adapters[1].id, "vsa");
    assert!(config.adapters[1].command.is_none());
    assert_eq!(
        config.adapters[1].input.as_deref(),
        Some(".vsa/analysis.json")
    );
}

#[test]
fn dimension_code_properties() {
    assert!(DimensionCode::MT01.is_default_enabled());
    assert!(!DimensionCode::AC01.is_default_enabled());
    assert_eq!(DimensionCode::MT01.name(), "Maintainability");
    assert_eq!(DimensionCode::MT01.as_str(), "MT01");
    assert_eq!(DimensionCode::from_str("MT01"), Some(DimensionCode::MT01));
    assert_eq!(DimensionCode::from_str("INVALID"), None);
}

// ─── Wave 6: Structural + Adapter Tests ────────────────────────────────────

#[test]
fn structural_rules_skip_gracefully() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    fs::create_dir_all(root.join(".topology")).unwrap();
    fs::write(
        root.join("fitness.toml"),
        r#"
[config]
topology_dir = ".topology"

[[rules.structural]]
id = "layer-enforcement"
name = "Controllers Must Not Import Repos"
dimension = "ST01"
pattern = "forbidden_import"
from = { path = "**/controllers/**" }
to = { path = "**/repositories/**" }
"#,
    )
    .unwrap();

    let validator = FitnessValidator::load(root, None).unwrap();
    let report = validator.validate().unwrap();

    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].status, architecture_fitness::RuleStatus::Skip);
    assert_eq!(report.results[0].dimension.as_deref(), Some("ST01"));
}

#[test]
fn structural_rule_defaults_dimension_to_st01() {
    let toml_str = r#"
[config]
topology_dir = ".topology"

[[rules.structural]]
id = "test-rule"
name = "Test Structural Rule"
pattern = "some_pattern"
"#;
    let config: FitnessConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.rules.structural.len(), 1);
    assert_eq!(config.rules.structural[0].dimension, None);
    // The dimension defaults to ST01 during evaluation, not parsing
}

#[test]
fn unknown_normalizer_rejected() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    fs::create_dir_all(root.join(".topology")).unwrap();
    fs::write(
        root.join("fitness.toml"),
        r#"
[config]
topology_dir = ".topology"

[[adapters]]
id = "bad-adapter"
dimension = "SC01"
command = "some-command"
output = "adapters/sc01/output.json"
normalizer = "invalid:format"
"#,
    )
    .unwrap();

    let result = FitnessValidator::load(root, None);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("normalizer"),
        "error should mention normalizer: {err}"
    );
}

#[test]
fn valid_normalizer_accepted() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    fs::create_dir_all(root.join(".topology")).unwrap();
    fs::write(
        root.join("fitness.toml"),
        r#"
[config]
topology_dir = ".topology"

[[adapters]]
id = "cargo-audit"
dimension = "SC01"
command = "cargo audit --json"
output = "adapters/sc01/vulnerabilities.json"
normalizer = "builtin:cargo-audit"

[[adapters]]
id = "custom"
dimension = "LG01"
input = "raw/licenses.json"
output = "adapters/lg01/licenses.json"
normalizer = "script:./normalize_licenses.py"
"#,
    )
    .unwrap();

    let result = FitnessValidator::load(root, None);
    assert!(result.is_ok());
}
