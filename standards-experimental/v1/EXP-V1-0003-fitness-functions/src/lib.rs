//! EXP-V1-0003 — Architecture Fitness Functions
//!
//! Declarative architecture fitness functions: automated assertions on architectural
//! properties that run in CI and fail on violations. This is the assertion layer on
//! top of APS-V1-0001's measurement layer.
//!
//! ⚠️ EXPERIMENTAL: This standard is in incubation and may change significantly.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ─── Error Codes ────────────────────────────────────────────────────────────

/// Error codes for fitness function validation.
pub mod error_codes {
    /// No `fitness.toml` found at specified path.
    pub const MISSING_FITNESS_TOML: &str = "MISSING_FITNESS_TOML";
    /// Rule definition is malformed or missing required fields.
    pub const INVALID_RULE: &str = "INVALID_RULE";
    /// Configured `topology_dir` does not exist.
    pub const MISSING_TOPOLOGY_DIR: &str = "MISSING_TOPOLOGY_DIR";
    /// Exception is missing required `issue` field.
    pub const MISSING_ISSUE_REF: &str = "MISSING_ISSUE_REF";
    /// Exception references entity that no longer violates.
    pub const STALE_EXCEPTION: &str = "STALE_EXCEPTION";
    /// Metric value exceeds rule threshold.
    pub const THRESHOLD_EXCEEDED: &str = "THRESHOLD_EXCEEDED";
}

// ─── Severity ───────────────────────────────────────────────────────────────

/// Rule severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

impl Default for Severity {
    fn default() -> Self {
        Self::Error
    }
}

// ─── Fitness Config (fitness.toml) ──────────────────────────────────────────

/// Top-level fitness configuration deserialized from `fitness.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct FitnessConfig {
    pub config: ConfigSection,
    #[serde(default)]
    pub rules: RulesSection,
}

/// The `[config]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfigSection {
    pub topology_dir: String,
    #[serde(default = "default_exceptions_path")]
    pub exceptions: String,
    #[serde(default)]
    pub severity_default: Severity,
}

fn default_exceptions_path() -> String {
    "fitness-exceptions.toml".to_string()
}

/// The `[rules]` section containing rule arrays.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RulesSection {
    #[serde(default)]
    pub threshold: Vec<ThresholdRule>,
    #[serde(default)]
    pub dependency: Vec<DependencyRule>,
}

/// A threshold rule: asserts a metric value per entity is within bounds.
#[derive(Debug, Clone, Deserialize)]
pub struct ThresholdRule {
    pub id: String,
    pub name: String,
    /// Path to topology artifact relative to `topology_dir`.
    pub source: String,
    /// JSON field to evaluate.
    pub field: String,
    /// Upper bound — violation if value > max.
    pub max: Option<f64>,
    /// Lower bound — violation if value < min.
    pub min: Option<f64>,
    /// Entity granularity: "module", "file", "function".
    pub scope: String,
    /// Override default severity.
    pub severity: Option<Severity>,
    /// Glob patterns for entities to exclude.
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl ThresholdRule {
    /// Effective severity (rule override or config default).
    pub fn effective_severity(&self, default: Severity) -> Severity {
        self.severity.unwrap_or(default)
    }

    /// Validate that the rule definition is well-formed.
    pub fn validate(&self) -> Result<(), String> {
        if self.max.is_none() && self.min.is_none() {
            return Err(format!(
                "Rule '{}': at least one of `max` or `min` must be specified",
                self.id
            ));
        }
        Ok(())
    }
}

/// A dependency rule: asserts constraints on the import/coupling graph.
/// Parsed but not evaluated in v0.1.0.
#[derive(Debug, Clone, Deserialize)]
pub struct DependencyRule {
    pub id: String,
    pub name: String,
    /// "forbidden", "allowed", or "required".
    #[serde(rename = "type")]
    pub rule_type: String,
    pub from: PathMatcher,
    pub to: PathMatcher,
    #[serde(default)]
    pub circular: bool,
    pub severity: Option<Severity>,
}

/// Path matcher for dependency rules.
#[derive(Debug, Clone, Deserialize)]
pub struct PathMatcher {
    pub path: String,
    pub path_not: Option<String>,
}

// ─── Exception Set (fitness-exceptions.toml) ────────────────────────────────

/// Exception set deserialized from `fitness-exceptions.toml`.
///
/// Structure: `HashMap<RuleId, HashMap<EntityPath, Exception>>`
///
/// In TOML this looks like:
/// ```toml
/// [max-cyclomatic."src/engine.py::execute"]
/// value = 42
/// issue = "#138"
/// ```
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct ExceptionSet {
    pub rules: HashMap<String, HashMap<String, Exception>>,
}

impl ExceptionSet {
    /// Load from a TOML file. Returns empty set if file doesn't exist.
    pub fn load(path: &Path) -> Result<Self, FitnessError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| FitnessError::Io(path.to_path_buf(), e))?;
        let set: Self =
            toml::from_str(&content).map_err(|e| FitnessError::ParseExceptions(e.to_string()))?;
        // Validate all exceptions have issue references
        for (rule_id, entities) in &set.rules {
            for (entity, exception) in entities {
                if exception.issue.is_empty() {
                    return Err(FitnessError::MissingIssueRef(
                        rule_id.clone(),
                        entity.clone(),
                    ));
                }
            }
        }
        Ok(set)
    }

    /// Get exception for a specific rule + entity.
    pub fn get(&self, rule_id: &str, entity: &str) -> Option<&Exception> {
        self.rules.get(rule_id)?.get(entity)
    }
}

/// A single exception entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Exception {
    /// Current metric value at time of exception (ratchet budget).
    pub value: Option<f64>,
    /// For dependency rules: specific import targets excepted.
    pub targets: Option<Vec<String>>,
    /// GitHub issue reference (REQUIRED).
    pub issue: String,
}

// ─── Fitness Report (fitness-report.json) ───────────────────────────────────

/// Fitness validation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessReport {
    pub version: String,
    pub timestamp: String,
    pub summary: ReportSummary,
    pub results: Vec<RuleResult>,
    pub stale_exceptions: Vec<StaleException>,
}

/// Report summary counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_rules: usize,
    pub passed: usize,
    pub failed: usize,
    pub warned: usize,
    pub total_violations: usize,
    pub excepted_violations: usize,
    pub stale_exceptions: usize,
}

/// Result for a single rule evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    pub rule_id: String,
    pub rule_name: String,
    pub status: RuleStatus,
    pub violations: Vec<Violation>,
    pub exceptions_used: usize,
}

/// Rule evaluation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleStatus {
    Pass,
    Fail,
    Warn,
    Skip,
}

/// A single violation (entity exceeding threshold).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub entity: String,
    pub field: String,
    pub actual: f64,
    pub threshold: f64,
    pub direction: ThresholdDirection,
    pub excepted: bool,
}

/// Which bound was violated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThresholdDirection {
    Max,
    Min,
}

/// A stale exception (entity no longer violates or doesn't exist).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaleException {
    pub rule_id: String,
    pub entity: String,
    pub reason: StaleReason,
}

/// Why an exception is stale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaleReason {
    EntityNotFound,
    NowPassing,
}

// ─── Errors ─────────────────────────────────────────────────────────────────

/// Errors during fitness validation.
#[derive(Debug, thiserror::Error)]
pub enum FitnessError {
    #[error("fitness.toml not found: {0}")]
    MissingConfig(PathBuf),

    #[error("topology directory not found: {0}")]
    MissingTopologyDir(PathBuf),

    #[error("invalid rule: {0}")]
    InvalidRule(String),

    #[error("I/O error reading {0}: {1}")]
    Io(PathBuf, #[source] std::io::Error),

    #[error("failed to parse fitness.toml: {0}")]
    ParseConfig(String),

    #[error("failed to parse exceptions: {0}")]
    ParseExceptions(String),

    #[error("failed to parse topology artifact {0}: {1}")]
    ParseArtifact(PathBuf, String),

    #[error("missing issue reference for exception [{0}.\"{1}\"]")]
    MissingIssueRef(String, String),
}

// ─── Validator ──────────────────────────────────────────────────────────────

/// Fitness function validator — evaluates rules against topology artifacts.
pub struct FitnessValidator {
    config: FitnessConfig,
    exceptions: ExceptionSet,
    repo_root: PathBuf,
}

impl FitnessValidator {
    /// Load config and exceptions from a repository root.
    pub fn load(repo_root: &Path, config_path: Option<&Path>) -> Result<Self, FitnessError> {
        let config_file = config_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| repo_root.join("fitness.toml"));

        if !config_file.exists() {
            return Err(FitnessError::MissingConfig(config_file));
        }

        let config_content = std::fs::read_to_string(&config_file)
            .map_err(|e| FitnessError::Io(config_file.clone(), e))?;
        let config: FitnessConfig =
            toml::from_str(&config_content).map_err(|e| FitnessError::ParseConfig(e.to_string()))?;

        // Validate all rules
        for rule in &config.rules.threshold {
            rule.validate().map_err(FitnessError::InvalidRule)?;
        }

        let topology_dir = repo_root.join(&config.config.topology_dir);
        if !topology_dir.exists() {
            return Err(FitnessError::MissingTopologyDir(topology_dir));
        }

        let exceptions_path = repo_root.join(&config.config.exceptions);
        let exceptions = ExceptionSet::load(&exceptions_path)?;

        Ok(Self {
            config,
            exceptions,
            repo_root: repo_root.to_path_buf(),
        })
    }

    /// Run validation and produce a report.
    pub fn validate(&self) -> Result<FitnessReport, FitnessError> {
        let mut results = Vec::new();
        let mut all_stale = Vec::new();

        // Track which exceptions were used (for stale detection)
        let mut used_exceptions: HashMap<String, Vec<String>> = HashMap::new();

        for rule in &self.config.rules.threshold {
            let (result, stale) = self.evaluate_threshold_rule(rule)?;
            // Track used exceptions
            for v in &result.violations {
                if v.excepted {
                    used_exceptions
                        .entry(rule.id.clone())
                        .or_default()
                        .push(v.entity.clone());
                }
            }
            results.push(result);
            all_stale.extend(stale);
        }

        // Detect stale exceptions (exceptions for entities not seen in any evaluation)
        for (rule_id, entities) in &self.exceptions.rules {
            let used = used_exceptions.get(rule_id);
            for entity in entities.keys() {
                let was_used = used.is_some_and(|u| u.contains(entity));
                let already_stale = all_stale
                    .iter()
                    .any(|s: &StaleException| s.rule_id == *rule_id && s.entity == *entity);
                if !was_used && !already_stale {
                    all_stale.push(StaleException {
                        rule_id: rule_id.clone(),
                        entity: entity.clone(),
                        reason: StaleReason::EntityNotFound,
                    });
                }
            }
        }

        let summary = ReportSummary {
            total_rules: results.len(),
            passed: results.iter().filter(|r| r.status == RuleStatus::Pass).count(),
            failed: results.iter().filter(|r| r.status == RuleStatus::Fail).count(),
            warned: results.iter().filter(|r| r.status == RuleStatus::Warn).count(),
            total_violations: results.iter().map(|r| r.violations.len()).sum(),
            excepted_violations: results
                .iter()
                .flat_map(|r| &r.violations)
                .filter(|v| v.excepted)
                .count(),
            stale_exceptions: all_stale.len(),
        };

        Ok(FitnessReport {
            version: "0.1.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            summary,
            results,
            stale_exceptions: all_stale,
        })
    }

    /// Evaluate a single threshold rule against its topology artifact.
    fn evaluate_threshold_rule(
        &self,
        rule: &ThresholdRule,
    ) -> Result<(RuleResult, Vec<StaleException>), FitnessError> {
        let artifact_path = self
            .repo_root
            .join(&self.config.config.topology_dir)
            .join(&rule.source);

        // If artifact doesn't exist, skip
        if !artifact_path.exists() {
            return Ok((
                RuleResult {
                    rule_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    status: RuleStatus::Skip,
                    violations: vec![],
                    exceptions_used: 0,
                },
                vec![],
            ));
        }

        let content = std::fs::read_to_string(&artifact_path)
            .map_err(|e| FitnessError::Io(artifact_path.clone(), e))?;
        let artifact: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| FitnessError::ParseArtifact(artifact_path.clone(), e.to_string()))?;

        let entities = extract_entities(&artifact, &rule.scope);
        let mut violations = Vec::new();
        let mut exceptions_used = 0;
        let mut stale = Vec::new();

        for (entity_path, value) in &entities {
            // Check exclude patterns
            if is_excluded(entity_path, &rule.exclude) {
                continue;
            }

            // Extract field value
            let metric_value = match extract_field_value(value, &rule.field) {
                Some(v) => v,
                None => continue,
            };

            // Check thresholds
            let mut violated = false;
            let mut direction = ThresholdDirection::Max;
            let mut threshold = 0.0;

            if let Some(max) = rule.max {
                if metric_value > max {
                    violated = true;
                    direction = ThresholdDirection::Max;
                    threshold = max;
                }
            }
            if let Some(min) = rule.min {
                if metric_value < min {
                    violated = true;
                    direction = ThresholdDirection::Min;
                    threshold = min;
                }
            }

            if violated {
                // Check for exception
                let excepted = if let Some(exc) = self.exceptions.get(&rule.id, entity_path) {
                    // If exception has a value budget, check it
                    if let Some(budget) = exc.value {
                        match direction {
                            ThresholdDirection::Max => metric_value <= budget,
                            ThresholdDirection::Min => metric_value >= budget,
                        }
                    } else {
                        true
                    }
                } else {
                    false
                };

                if excepted {
                    exceptions_used += 1;
                }

                violations.push(Violation {
                    entity: entity_path.clone(),
                    field: rule.field.clone(),
                    actual: metric_value,
                    threshold,
                    direction,
                    excepted,
                });
            } else {
                // Entity passes — check if there's a now-stale exception
                if self.exceptions.get(&rule.id, entity_path).is_some() {
                    stale.push(StaleException {
                        rule_id: rule.id.clone(),
                        entity: entity_path.clone(),
                        reason: StaleReason::NowPassing,
                    });
                }
            }
        }

        let has_unexcepted = violations.iter().any(|v| !v.excepted);
        let severity = rule.effective_severity(self.config.config.severity_default);
        let status = if !has_unexcepted {
            RuleStatus::Pass
        } else {
            match severity {
                Severity::Error => RuleStatus::Fail,
                Severity::Warning => RuleStatus::Warn,
            }
        };

        Ok((
            RuleResult {
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                status,
                violations,
                exceptions_used,
            },
            stale,
        ))
    }

    /// Returns true if any error-severity rules failed.
    pub fn has_failures(report: &FitnessReport) -> bool {
        report.results.iter().any(|r| r.status == RuleStatus::Fail)
    }
}

// ─── Topology Artifact Helpers ──────────────────────────────────────────────

/// Extract entities from a topology artifact JSON.
///
/// Supports three shapes:
/// - **Wrapped object**: `{ "functions": [{ "id": "...", ... }, ...] }` — auto-detected
///   from scope (`"function"` → `"functions"` key, `"module"` → `"modules"`, etc.).
///   Falls back to checking for a single array-valued key if scope doesn't match.
/// - **Flat object**: `{ "entity_path": { "field": value, ... }, ... }`
/// - **Array**: `[{ "path": "entity_path", "field": value, ... }, ...]`
fn extract_entities(artifact: &serde_json::Value, scope: &str) -> Vec<(String, serde_json::Value)> {
    match artifact {
        serde_json::Value::Object(map) => {
            // Step 1: Derive wrapper key from scope (e.g., "function" → "functions")
            let wrapper_key = match scope {
                "function" => Some("functions"),
                "module" => Some("modules"),
                "file" => Some("files"),
                "slice" => Some("slices"),
                _ => None,
            };

            // Step 2: Check if map contains the scope-derived wrapper key pointing to an array
            if let Some(key) = wrapper_key {
                if let Some(serde_json::Value::Array(arr)) = map.get(key) {
                    return extract_entities_from_array(arr);
                }
            }

            // Step 3: Fallback heuristic — if exactly one key has an array value, unwrap it
            let array_entries: Vec<_> = map
                .iter()
                .filter(|(_, v)| v.is_array())
                .collect();
            if array_entries.len() == 1 {
                if let serde_json::Value::Array(arr) = array_entries[0].1 {
                    return extract_entities_from_array(arr);
                }
            }

            // Step 4: Fall through to flat-object behavior (backward compat)
            map.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        }
        serde_json::Value::Array(arr) => extract_entities_from_array(arr),
        _ => vec![],
    }
}

/// Extract entities from a JSON array, using `id` first, then `path`/`name`/`entity` as fallbacks.
fn extract_entities_from_array(arr: &[serde_json::Value]) -> Vec<(String, serde_json::Value)> {
    arr.iter()
        .filter_map(|item| {
            let id = item
                .get("id")
                .or_else(|| item.get("path"))
                .or_else(|| item.get("name"))
                .or_else(|| item.get("entity"))
                .and_then(|v| v.as_str())?;
            Some((id.to_string(), item.clone()))
        })
        .collect()
}

/// Extract a numeric field value from an entity JSON object.
///
/// Supports dot-path navigation (e.g., `"metrics.cognitive"` traverses into
/// nested objects). Single-segment paths (no dots) work as before.
fn extract_field_value(entity: &serde_json::Value, field: &str) -> Option<f64> {
    let mut current = entity;
    for segment in field.split('.') {
        current = current.get(segment)?;
    }
    current.as_f64()
}

/// Check if an entity path matches any exclude glob pattern.
fn is_excluded(entity_path: &str, exclude_patterns: &[String]) -> bool {
    for pattern in exclude_patterns {
        if let Ok(glob) = glob::Pattern::new(pattern) {
            if glob.matches(entity_path) {
                return true;
            }
        }
    }
    false
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_default() {
        assert_eq!(Severity::default(), Severity::Error);
    }

    #[test]
    fn test_threshold_rule_validate_requires_bound() {
        let rule = ThresholdRule {
            id: "test".to_string(),
            name: "Test".to_string(),
            source: "metrics/test.json".to_string(),
            field: "value".to_string(),
            max: None,
            min: None,
            scope: "function".to_string(),
            severity: None,
            exclude: vec![],
        };
        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_threshold_rule_validate_accepts_max() {
        let rule = ThresholdRule {
            id: "test".to_string(),
            name: "Test".to_string(),
            source: "metrics/test.json".to_string(),
            field: "value".to_string(),
            max: Some(10.0),
            min: None,
            scope: "function".to_string(),
            severity: None,
            exclude: vec![],
        };
        assert!(rule.validate().is_ok());
    }

    #[test]
    fn test_extract_entities_object() {
        let json: serde_json::Value = serde_json::json!({
            "src/foo.py": { "complexity": 5 },
            "src/bar.py": { "complexity": 15 }
        });
        let entities = extract_entities(&json, "file");
        assert_eq!(entities.len(), 2);
    }

    #[test]
    fn test_extract_entities_array() {
        let json: serde_json::Value = serde_json::json!([
            { "path": "src/foo.py", "complexity": 5 },
            { "path": "src/bar.py", "complexity": 15 }
        ]);
        let entities = extract_entities(&json, "file");
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].0, "src/foo.py");
    }

    #[test]
    fn test_extract_field_value() {
        let entity = serde_json::json!({ "cyclomatic_complexity": 42.0, "name": "foo" });
        assert_eq!(extract_field_value(&entity, "cyclomatic_complexity"), Some(42.0));
        assert_eq!(extract_field_value(&entity, "missing"), None);
    }

    #[test]
    fn test_is_excluded() {
        assert!(is_excluded("tests/test_foo.py", &["tests/**".to_string()]));
        assert!(is_excluded("src/test_bar.py", &["**/test_*".to_string()]));
        assert!(!is_excluded("src/main.py", &["**/test_*".to_string()]));
    }

    #[test]
    fn test_exception_set_empty_on_missing_file() {
        let set = ExceptionSet::load(Path::new("/nonexistent/path.toml")).unwrap();
        assert!(set.rules.is_empty());
    }

    #[test]
    fn test_rule_status_serialization() {
        assert_eq!(
            serde_json::to_string(&RuleStatus::Pass).unwrap(),
            "\"pass\""
        );
        assert_eq!(
            serde_json::to_string(&RuleStatus::Fail).unwrap(),
            "\"fail\""
        );
    }

    #[test]
    fn test_extract_field_value_dotpath() {
        let entity = serde_json::json!({
            "metrics": {
                "cognitive": 42.0,
                "martin": { "ce": 15.0 }
            },
            "name": "foo"
        });
        // Single segment (backward compat)
        assert_eq!(extract_field_value(&entity, "name"), None); // "foo" is not f64
        // Dot-path navigation
        assert_eq!(extract_field_value(&entity, "metrics.cognitive"), Some(42.0));
        // Deep dot-path
        assert_eq!(extract_field_value(&entity, "metrics.martin.ce"), Some(15.0));
        // Missing path
        assert_eq!(extract_field_value(&entity, "metrics.missing"), None);
    }

    #[test]
    fn test_extract_entities_wrapped_format() {
        // Wrapped functions format (scope-derived key)
        let json = serde_json::json!({
            "functions": [
                { "id": "python:mod::func_a", "metrics": { "cognitive": 5 } },
                { "id": "python:mod::func_b", "metrics": { "cognitive": 15 } }
            ]
        });
        let entities = extract_entities(&json, "function");
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].0, "python:mod::func_a");
        assert_eq!(entities[1].0, "python:mod::func_b");

        // Wrapped modules format
        let json = serde_json::json!({
            "modules": [
                { "id": "packages.syn-domain", "metrics": { "lines_of_code": 100 } }
            ]
        });
        let entities = extract_entities(&json, "module");
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].0, "packages.syn-domain");

        // Fallback heuristic: single array key, scope doesn't match key name
        let json = serde_json::json!({
            "schema_version": "1.0",
            "items": [
                { "id": "item_a", "value": 10 }
            ]
        });
        let entities = extract_entities(&json, "function");
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].0, "item_a");

        // Flat object still works (backward compat)
        let json = serde_json::json!({
            "src/foo.py": { "complexity": 5 },
            "src/bar.py": { "complexity": 15 }
        });
        let entities = extract_entities(&json, "file");
        assert_eq!(entities.len(), 2);
    }

    #[test]
    fn test_extract_entities_array_prefers_id() {
        let json: serde_json::Value = serde_json::json!([
            { "id": "entity_1", "path": "old_path_1", "complexity": 5 },
            { "id": "entity_2", "path": "old_path_2", "complexity": 15 }
        ]);
        let entities = extract_entities(&json, "file");
        assert_eq!(entities.len(), 2);
        // id takes priority over path
        assert_eq!(entities[0].0, "entity_1");
        assert_eq!(entities[1].0, "entity_2");
    }

    #[test]
    fn test_stale_reason_serialization() {
        assert_eq!(
            serde_json::to_string(&StaleReason::NowPassing).unwrap(),
            "\"now_passing\""
        );
        assert_eq!(
            serde_json::to_string(&StaleReason::EntityNotFound).unwrap(),
            "\"entity_not_found\""
        );
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[config]
topology_dir = ".topology"

[[rules.threshold]]
id = "max-cc"
name = "Max CC"
source = "metrics/complexity.json"
field = "cyclomatic_complexity"
max = 15
scope = "function"
"#;
        let config: FitnessConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.config.topology_dir, ".topology");
        assert_eq!(config.config.exceptions, "fitness-exceptions.toml");
        assert_eq!(config.rules.threshold.len(), 1);
        assert_eq!(config.rules.threshold[0].id, "max-cc");
        assert_eq!(config.rules.threshold[0].max, Some(15.0));
    }

    #[test]
    fn test_exception_deserialization() {
        let toml_str = r##"
[max-cc."src/foo.py::bar"]
value = 42
issue = "#138"

[max-cc."src/baz.py::qux"]
issue = "#185"
"##;
        let set: ExceptionSet = toml::from_str(toml_str).unwrap();
        let exc = set.get("max-cc", "src/foo.py::bar").unwrap();
        assert_eq!(exc.value, Some(42.0));
        assert_eq!(exc.issue, "#138");

        let exc2 = set.get("max-cc", "src/baz.py::qux").unwrap();
        assert_eq!(exc2.value, None);
        assert_eq!(exc2.issue, "#185");
    }
}
