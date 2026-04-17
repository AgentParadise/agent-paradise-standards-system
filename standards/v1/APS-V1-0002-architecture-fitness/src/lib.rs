//! APS-V1-0002 — Architecture Fitness Functions
//!
//! Comprehensive architectural governance framework based on evolutionary architecture
//! principles (Ford et al., 2017). Provides declarative fitness functions — automated
//! assertions on architectural properties — organized into composable dimensional
//! substandards.
//!
//! This is the assertion layer on top of APS-V1-0001's measurement layer.
//!
//! ## Dimensions
//!
//! - **MT01** — Maintainability (complexity, Halstead, LOC, MI)
//! - **MD01** — Modularity (coupling, instability, abstractness, main sequence)
//! - **ST01** — Structural Integrity (ArchUnit-style checks, CK metrics)
//! - **SC01** — Security (vulnerability scanning)
//! - **LG01** — Legality (license compliance)
//! - **AC01** — Accessibility (WCAG / a11y)
//! - **PF01** — Performance (load testing, benchmarks)
//! - **AV01** — Availability (chaos engineering, uptime)
//!
//! Promoted from EXP-V1-0003 with expanded scope.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
    /// Unknown dimension code in configuration.
    pub const INVALID_DIMENSION: &str = "INVALID_DIMENSION";
    /// Default-enabled dimension disabled without reason.
    pub const DIMENSION_DISABLED_NO_REASON: &str = "DIMENSION_DISABLED_NO_REASON";
    /// System-level score below configured minimum.
    pub const SYSTEM_FITNESS_BELOW_THRESHOLD: &str = "SYSTEM_FITNESS_BELOW_THRESHOLD";
    /// Structural rule references unknown pattern.
    pub const INVALID_STRUCTURAL_PATTERN: &str = "INVALID_STRUCTURAL_PATTERN";
    /// System fitness weights do not sum to 1.0.
    pub const INVALID_WEIGHTS: &str = "INVALID_WEIGHTS";
    /// Circular dependency found (forbidden).
    pub const DEPENDENCY_CYCLE_DETECTED: &str = "DEPENDENCY_CYCLE_DETECTED";
}

// ─── Severity ───────────────────────────────────────────────────────────────

/// Rule severity level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    #[default]
    Error,
    Warning,
}

// ─── Dimension Model ────────────────────────────────────────────────────────

/// Architectural dimension codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DimensionCode {
    MT01,
    MD01,
    ST01,
    SC01,
    LG01,
    AC01,
    PF01,
    AV01,
}

impl DimensionCode {
    /// All known dimension codes.
    pub const ALL: &[DimensionCode] = &[
        DimensionCode::MT01,
        DimensionCode::MD01,
        DimensionCode::ST01,
        DimensionCode::SC01,
        DimensionCode::LG01,
        DimensionCode::AC01,
        DimensionCode::PF01,
        DimensionCode::AV01,
    ];

    /// Default-enabled dimensions.
    pub const DEFAULT_ENABLED: &[DimensionCode] = &[
        DimensionCode::MT01,
        DimensionCode::MD01,
        DimensionCode::ST01,
        DimensionCode::SC01,
        DimensionCode::LG01,
    ];

    /// Whether this dimension is enabled by default.
    pub fn is_default_enabled(self) -> bool {
        Self::DEFAULT_ENABLED.contains(&self)
    }

    /// Human-readable name for this dimension.
    pub fn name(self) -> &'static str {
        match self {
            DimensionCode::MT01 => "Maintainability",
            DimensionCode::MD01 => "Modularity & Coupling",
            DimensionCode::ST01 => "Structural Integrity",
            DimensionCode::SC01 => "Security",
            DimensionCode::LG01 => "Legality",
            DimensionCode::AC01 => "Accessibility",
            DimensionCode::PF01 => "Performance",
            DimensionCode::AV01 => "Availability",
        }
    }

    /// String code (e.g., "MT01").
    pub fn as_str(self) -> &'static str {
        match self {
            DimensionCode::MT01 => "MT01",
            DimensionCode::MD01 => "MD01",
            DimensionCode::ST01 => "ST01",
            DimensionCode::SC01 => "SC01",
            DimensionCode::LG01 => "LG01",
            DimensionCode::AC01 => "AC01",
            DimensionCode::PF01 => "PF01",
            DimensionCode::AV01 => "AV01",
        }
    }

    /// Parse from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "MT01" => Some(DimensionCode::MT01),
            "MD01" => Some(DimensionCode::MD01),
            "ST01" => Some(DimensionCode::ST01),
            "SC01" => Some(DimensionCode::SC01),
            "LG01" => Some(DimensionCode::LG01),
            "AC01" => Some(DimensionCode::AC01),
            "PF01" => Some(DimensionCode::PF01),
            "AV01" => Some(DimensionCode::AV01),
            _ => None,
        }
    }

    /// Promotion status per §3.4 and Appendix D.
    ///
    /// Source of truth for strict-artifact enforcement: when a rule's
    /// dimension is `Active`, a missing source artifact produces a failing
    /// rule result with `PROMOTION_REQUIREMENT_UNMET` (§12). When
    /// `Incubating`, a missing artifact is a `Skip` (advisory).
    ///
    /// This mapping MUST agree with the table in Appendix D of the spec.
    pub const fn promotion_status(self) -> PromotionStatus {
        match self {
            DimensionCode::MT01 => PromotionStatus::Active,
            DimensionCode::MD01 => PromotionStatus::Active,
            DimensionCode::ST01 => PromotionStatus::Incubating,
            DimensionCode::SC01 => PromotionStatus::Incubating,
            DimensionCode::LG01 => PromotionStatus::Incubating,
            DimensionCode::AC01 => PromotionStatus::Incubating,
            DimensionCode::PF01 => PromotionStatus::Incubating,
            DimensionCode::AV01 => PromotionStatus::Incubating,
        }
    }
}

/// Promotion status of a dimension per APS-V1-0002 §3.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromotionStatus {
    /// All R1–R5 requirements satisfied; rules are strictly enforced.
    Active,
    /// Partial implementation; rule severities downgraded to warning;
    /// missing artifacts silently skip rather than fail.
    Incubating,
    /// Scheduled for removal; emit warnings about usage.
    Deprecated,
}

impl std::fmt::Display for DimensionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Dimension enable/disable configuration from `[dimensions]`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DimensionsConfig {
    #[serde(rename = "MT01")]
    pub mt01: bool,
    #[serde(rename = "MD01")]
    pub md01: bool,
    #[serde(rename = "ST01")]
    pub st01: bool,
    #[serde(rename = "SC01")]
    pub sc01: bool,
    #[serde(rename = "LG01")]
    pub lg01: bool,
    #[serde(rename = "AC01")]
    pub ac01: bool,
    #[serde(rename = "PF01")]
    pub pf01: bool,
    #[serde(rename = "AV01")]
    pub av01: bool,
    /// Reasons for disabling default-enabled dimensions.
    #[serde(default)]
    pub reasons: HashMap<String, String>,
}

impl Default for DimensionsConfig {
    fn default() -> Self {
        Self {
            mt01: true,
            md01: true,
            st01: true,
            sc01: true,
            lg01: true,
            ac01: false,
            pf01: false,
            av01: false,
            reasons: HashMap::new(),
        }
    }
}

impl DimensionsConfig {
    /// Check if a given dimension is enabled.
    pub fn is_enabled(&self, code: DimensionCode) -> bool {
        match code {
            DimensionCode::MT01 => self.mt01,
            DimensionCode::MD01 => self.md01,
            DimensionCode::ST01 => self.st01,
            DimensionCode::SC01 => self.sc01,
            DimensionCode::LG01 => self.lg01,
            DimensionCode::AC01 => self.ac01,
            DimensionCode::PF01 => self.pf01,
            DimensionCode::AV01 => self.av01,
        }
    }

    /// Validate that default-enabled dimensions disabled without reason are flagged.
    pub fn validate(&self) -> Result<(), FitnessError> {
        for &code in DimensionCode::DEFAULT_ENABLED {
            if !self.is_enabled(code) && !self.reasons.contains_key(code.as_str()) {
                return Err(FitnessError::DimensionDisabledNoReason(
                    code.as_str().to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// System-level fitness function configuration from `[system_fitness]`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SystemFitnessConfig {
    pub enabled: bool,
    pub min_score: f64,
    #[serde(default)]
    pub weights: HashMap<String, f64>,
}

impl Default for SystemFitnessConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_score: 0.7,
            weights: HashMap::new(),
        }
    }
}

impl SystemFitnessConfig {
    /// Validate that weights sum to ~1.0 (within epsilon) if specified.
    pub fn validate(&self) -> Result<(), FitnessError> {
        if !self.weights.is_empty() {
            let sum: f64 = self.weights.values().sum();
            if (sum - 1.0).abs() > 0.01 {
                return Err(FitnessError::InvalidWeights(format!(
                    "weights sum to {sum:.4}, expected 1.0"
                )));
            }
        }
        Ok(())
    }
}

// ─── Fitness Config (fitness.toml) ──────────────────────────────────────────

/// Top-level fitness configuration deserialized from `fitness.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct FitnessConfig {
    pub config: ConfigSection,
    #[serde(default)]
    pub rules: RulesSection,
    #[serde(default)]
    pub dimensions: DimensionsConfig,
    #[serde(default)]
    pub system_fitness: SystemFitnessConfig,
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
    #[serde(default)]
    pub structural: Vec<StructuralRule>,
}

/// A threshold rule: asserts a metric value per entity is within bounds.
#[derive(Debug, Clone, Deserialize)]
pub struct ThresholdRule {
    pub id: String,
    pub name: String,
    /// Dimension this rule belongs to (e.g., "MT01").
    pub dimension: Option<String>,
    /// Path to topology artifact relative to `topology_dir`.
    pub source: String,
    /// JSON field to evaluate.
    pub field: String,
    /// Upper bound — violation if value > max.
    pub max: Option<f64>,
    /// Lower bound — violation if value < min.
    pub min: Option<f64>,
    /// Entity granularity: "module", "file", "function", "class", "slice", "system".
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
#[derive(Debug, Clone, Deserialize)]
pub struct DependencyRule {
    pub id: String,
    pub name: String,
    /// Dimension this rule belongs to (e.g., "MD01").
    pub dimension: Option<String>,
    /// "forbidden", "allowed", or "required".
    #[serde(rename = "type")]
    pub rule_type: String,
    pub from: PathMatcher,
    pub to: PathMatcher,
    #[serde(default)]
    pub circular: bool,
    pub severity: Option<Severity>,
}

/// Path matcher for dependency and structural rules.
#[derive(Debug, Clone, Deserialize)]
pub struct PathMatcher {
    pub path: String,
    pub path_not: Option<String>,
}

/// A structural rule: ArchUnit-style constraints on code organization.
#[derive(Debug, Clone, Deserialize)]
pub struct StructuralRule {
    pub id: String,
    pub name: String,
    /// Dimension this rule belongs to (default: "ST01").
    pub dimension: Option<String>,
    /// Pattern type from ST01 catalog (e.g., "forbidden_import").
    pub pattern: String,
    pub from: Option<PathMatcher>,
    pub to: Option<PathMatcher>,
    pub severity: Option<Severity>,
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
        let content =
            std::fs::read_to_string(path).map_err(|e| FitnessError::Io(path.to_path_buf(), e))?;
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
    /// Per-dimension scoring results.
    #[serde(default)]
    pub dimensions: HashMap<String, DimensionResult>,
    /// System-level fitness score (weighted aggregate).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_fitness: Option<SystemFitnessResult>,
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
    pub skipped: usize,
    pub total_violations: usize,
    pub excepted_violations: usize,
    pub stale_exceptions: usize,
}

/// Result for a single rule evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    pub rule_id: String,
    pub rule_name: String,
    /// Dimension this rule belongs to (e.g., "MT01").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimension: Option<String>,
    pub status: RuleStatus,
    pub violations: Vec<Violation>,
    pub exceptions_used: usize,
    /// Total entities evaluated (for dimension scoring denominator).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_entities: Option<usize>,
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

// ─── Dimension Results ─────────────────────────────────────────────────────

/// Status of a dimension in the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DimensionStatus {
    /// Dimension is enabled and has evaluated rules or no rules needed.
    Active,
    /// Dimension is enabled but all rules were skipped (missing artifacts).
    Skipped,
    /// Dimension is disabled in config.
    Disabled,
}

/// Per-dimension scoring result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionResult {
    pub name: String,
    pub status: DimensionStatus,
    pub score: Option<f64>,
    pub rules_evaluated: usize,
    pub rules_passed: usize,
    pub rules_failed: usize,
    pub rules_warned: usize,
    pub total_violations: usize,
    pub excepted_violations: usize,
}

// ─── System Fitness Results ────────────────────────────────────────────────

/// System-level fitness result (weighted aggregate of dimension scores).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemFitnessResult {
    pub score: f64,
    pub min_score: f64,
    pub passing: bool,
    pub weights_used: HashMap<String, f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weights_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trend: Option<TrendData>,
}

/// Trend data comparing current vs. previous report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub previous_score: f64,
    pub delta: f64,
    pub direction: TrendDirection,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dimension_deltas: HashMap<String, f64>,
}

/// Direction of score change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
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

    #[error("unknown dimension code: {0}")]
    InvalidDimension(String),

    #[error("default-enabled dimension {0} disabled without reason")]
    DimensionDisabledNoReason(String),

    #[error("system fitness weights invalid: {0}")]
    InvalidWeights(String),

    #[error("structural pattern not found: {0}")]
    InvalidStructuralPattern(String),

    #[error("system fitness score {0:.2} below threshold {1:.2}")]
    SystemFitnessBelowThreshold(f64, f64),

    #[error("circular dependency detected: {0}")]
    DependencyCycleDetected(String),
}

// ─── Validator ──────────────────────────────────────────────────────────────

/// Fitness function validator — evaluates rules against topology artifacts.
#[derive(Debug)]
pub struct FitnessValidator {
    config: FitnessConfig,
    exceptions: ExceptionSet,
    repo_root: PathBuf,
    previous_report: Option<FitnessReport>,
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
        let config: FitnessConfig = toml::from_str(&config_content)
            .map_err(|e| FitnessError::ParseConfig(e.to_string()))?;

        // Validate all threshold rules
        for rule in &config.rules.threshold {
            rule.validate().map_err(FitnessError::InvalidRule)?;
        }

        // Validate dimension configuration
        config.dimensions.validate()?;

        // Validate system fitness weights
        config.system_fitness.validate()?;

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
            previous_report: None,
        })
    }

    /// Run validation and produce a report.
    pub fn validate(&self) -> Result<FitnessReport, FitnessError> {
        let mut results = Vec::new();
        let mut all_stale = Vec::new();

        // Track which exceptions were matched (any exception entry found, budget or not)
        // and which rules were fully evaluated (not skipped).
        let mut matched_exceptions: HashMap<String, HashSet<String>> = HashMap::new();
        let mut evaluated_rule_ids: Vec<String> = Vec::new();

        for rule in &self.config.rules.threshold {
            let (result, stale, matched) = self.evaluate_threshold_rule(rule)?;
            // Only track stale detection for rules that were actually evaluated
            // (not skipped due to missing artifacts)
            if result.status != RuleStatus::Skip {
                evaluated_rule_ids.push(rule.id.clone());
                if !matched.is_empty() {
                    matched_exceptions.insert(rule.id.clone(), matched);
                }
            }
            results.push(result);
            all_stale.extend(stale);
        }

        // Evaluate dependency rules
        for rule in &self.config.rules.dependency {
            let (result, stale, matched) = self.evaluate_dependency_rule(rule)?;
            if result.status != RuleStatus::Skip {
                evaluated_rule_ids.push(rule.id.clone());
                if !matched.is_empty() {
                    matched_exceptions.insert(rule.id.clone(), matched);
                }
            }
            results.push(result);
            all_stale.extend(stale);
        }

        // Evaluate structural rules (stub — always skipped)
        for rule in &self.config.rules.structural {
            results.push(self.evaluate_structural_rule(rule));
        }

        // Detect stale exceptions — only for rules that were fully evaluated.
        // Skipped rules (missing artifact) should not trigger EntityNotFound.
        // Use matched_exceptions (not just excepted ones) so insufficient-budget
        // exceptions are not falsely reported as EntityNotFound.
        for (rule_id, entities) in &self.exceptions.rules {
            if !evaluated_rule_ids.contains(rule_id) {
                continue; // Rule was skipped or doesn't exist — don't flag exceptions as stale
            }
            let matched = matched_exceptions.get(rule_id);
            for entity in entities.keys() {
                let was_matched = matched.is_some_and(|m| m.contains(entity.as_str()));
                let already_stale = all_stale
                    .iter()
                    .any(|s: &StaleException| s.rule_id == *rule_id && s.entity == *entity);
                if !was_matched && !already_stale {
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
            passed: results
                .iter()
                .filter(|r| r.status == RuleStatus::Pass)
                .count(),
            failed: results
                .iter()
                .filter(|r| r.status == RuleStatus::Fail)
                .count(),
            warned: results
                .iter()
                .filter(|r| r.status == RuleStatus::Warn)
                .count(),
            skipped: results
                .iter()
                .filter(|r| r.status == RuleStatus::Skip)
                .count(),
            total_violations: results.iter().map(|r| r.violations.len()).sum(),
            excepted_violations: results
                .iter()
                .flat_map(|r| &r.violations)
                .filter(|v| v.excepted)
                .count(),
            stale_exceptions: all_stale.len(),
        };

        // Compute per-dimension results
        let dimensions = self.compute_dimension_results(&results);

        // Compute system-level fitness
        let system_fitness = self.compute_system_fitness(&dimensions);

        Ok(FitnessReport {
            version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            summary,
            dimensions,
            system_fitness,
            results,
            stale_exceptions: all_stale,
        })
    }

    /// Evaluate a single threshold rule against its topology artifact.
    ///
    /// Returns `(result, stale_exceptions, matched_exception_entities, entity_count)` where
    /// `matched_exception_entities` contains every entity path that had *any* exception
    /// entry (whether the budget was sufficient or not). This is used by `validate()` for
    /// accurate stale-exception detection — an insufficient-budget exception is not stale.
    fn evaluate_threshold_rule(
        &self,
        rule: &ThresholdRule,
    ) -> Result<(RuleResult, Vec<StaleException>, HashSet<String>), FitnessError> {
        let artifact_path = self
            .repo_root
            .join(&self.config.config.topology_dir)
            .join(&rule.source);

        // Strict-artifact enforcement (§3.3 R3, §12 PROMOTION_REQUIREMENT_UNMET):
        // When the rule belongs to an `active` dimension, a missing source
        // artifact is a hard failure — the dimension promised this data exists.
        // When `incubating`, the rule silently skips.
        if !artifact_path.exists() {
            let promotion = rule
                .dimension
                .as_deref()
                .and_then(DimensionCode::parse)
                .map(DimensionCode::promotion_status)
                .unwrap_or(PromotionStatus::Incubating);

            let status = if promotion == PromotionStatus::Active {
                RuleStatus::Fail
            } else {
                RuleStatus::Skip
            };

            let violations = if status == RuleStatus::Fail {
                vec![Violation {
                    entity: rule.source.clone(),
                    field: rule.field.clone(),
                    actual: 0.0,
                    threshold: 0.0,
                    direction: ThresholdDirection::Max,
                    excepted: false,
                }]
            } else {
                vec![]
            };

            return Ok((
                RuleResult {
                    rule_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    dimension: rule.dimension.clone(),
                    status,
                    violations,
                    exceptions_used: 0,
                    total_entities: None,
                },
                vec![],
                HashSet::new(),
            ));
        }

        let content = std::fs::read_to_string(&artifact_path)
            .map_err(|e| FitnessError::Io(artifact_path.clone(), e))?;
        let artifact: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| FitnessError::ParseArtifact(artifact_path.clone(), e.to_string()))?;

        let entities = extract_entities(&artifact, &rule.scope);
        let entity_count = entities.len();
        let mut violations = Vec::new();
        let mut exceptions_used = 0;
        let mut stale = Vec::new();
        // Track every entity that has an exception entry, regardless of budget sufficiency.
        // Used by the caller to avoid false EntityNotFound stale reports.
        let mut matched_exception_entities: HashSet<String> = HashSet::new();

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
                    // Record match regardless of budget — prevents false EntityNotFound stale
                    matched_exception_entities.insert(entity_path.clone());
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
                dimension: rule.dimension.clone(),
                status,
                violations,
                exceptions_used,
                total_entities: Some(entity_count),
            },
            stale,
            matched_exception_entities,
        ))
    }

    /// Evaluate a dependency rule against the topology dependency graph.
    ///
    /// Returns `(result, stale_exceptions, matched_exception_entities)`.
    fn evaluate_dependency_rule(
        &self,
        rule: &DependencyRule,
    ) -> Result<(RuleResult, Vec<StaleException>, HashSet<String>), FitnessError> {
        let graph_path = self
            .repo_root
            .join(&self.config.config.topology_dir)
            .join("graphs/dependency-graph.json");

        // If graph doesn't exist, skip
        if !graph_path.exists() {
            return Ok((
                RuleResult {
                    rule_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    dimension: rule.dimension.clone(),
                    status: RuleStatus::Skip,
                    violations: vec![],
                    exceptions_used: 0,
                    total_entities: None,
                },
                vec![],
                HashSet::new(),
            ));
        }

        let content = std::fs::read_to_string(&graph_path)
            .map_err(|e| FitnessError::Io(graph_path.clone(), e))?;
        let graph: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| FitnessError::ParseArtifact(graph_path.clone(), e.to_string()))?;

        let (nodes, edges) = load_dependency_graph(&graph);

        // Filter nodes by from/to path matchers
        let from_nodes: Vec<&str> = nodes
            .iter()
            .filter(|n| glob_matches(n, &rule.from.path))
            .filter(|n| {
                rule.from
                    .path_not
                    .as_ref()
                    .is_none_or(|pn| !glob_matches(n, pn))
            })
            .map(|s| s.as_str())
            .collect();

        let to_nodes: Vec<&str> = nodes
            .iter()
            .filter(|n| glob_matches(n, &rule.to.path))
            .filter(|n| {
                rule.to
                    .path_not
                    .as_ref()
                    .is_none_or(|pn| !glob_matches(n, pn))
            })
            .map(|s| s.as_str())
            .collect();

        let to_set: HashSet<&str> = to_nodes.iter().copied().collect();
        let from_set: HashSet<&str> = from_nodes.iter().copied().collect();
        let entity_count = from_nodes.len();

        let mut violations = Vec::new();
        let mut exceptions_used = 0;
        let mut stale = Vec::new();
        let mut matched_exception_entities: HashSet<String> = HashSet::new();

        match rule.rule_type.as_str() {
            "forbidden" => {
                if rule.circular {
                    // Use Tarjan SCC to detect cycles
                    let relevant_nodes: HashSet<&str> = from_set.union(&to_set).copied().collect();
                    let relevant_edges: Vec<(&str, &str)> = edges
                        .iter()
                        .filter(|(a, b)| {
                            relevant_nodes.contains(a.as_str())
                                && relevant_nodes.contains(b.as_str())
                        })
                        .map(|(a, b)| (a.as_str(), b.as_str()))
                        .collect();

                    let sccs = tarjan_scc(&relevant_nodes, &relevant_edges);
                    for scc in &sccs {
                        if scc.len() > 1 {
                            // Each node in the cycle is a violation
                            for node in scc {
                                let excepted = self.check_dependency_exception(
                                    &rule.id,
                                    node,
                                    &mut matched_exception_entities,
                                );
                                if excepted {
                                    exceptions_used += 1;
                                }
                                violations.push(Violation {
                                    entity: node.to_string(),
                                    field: "circular_dependency".to_string(),
                                    actual: 1.0,
                                    threshold: 0.0,
                                    direction: ThresholdDirection::Max,
                                    excepted,
                                });
                            }
                        }
                    }
                } else {
                    // Check for any edge from→to
                    for (from, to) in &edges {
                        if from_set.contains(from.as_str()) && to_set.contains(to.as_str()) {
                            let excepted = self.check_dependency_exception(
                                &rule.id,
                                from,
                                &mut matched_exception_entities,
                            );
                            if excepted {
                                exceptions_used += 1;
                            }
                            violations.push(Violation {
                                entity: from.clone(),
                                field: "depends_on".to_string(),
                                actual: 1.0,
                                threshold: 0.0,
                                direction: ThresholdDirection::Max,
                                excepted,
                            });
                        }
                    }
                }
            }
            "required" => {
                // Check that for each from_node, at least one edge to a to_node exists
                for from in &from_nodes {
                    let has_edge = edges
                        .iter()
                        .any(|(a, b)| a == from && to_set.contains(b.as_str()));
                    if !has_edge {
                        let excepted = self.check_dependency_exception(
                            &rule.id,
                            from,
                            &mut matched_exception_entities,
                        );
                        if excepted {
                            exceptions_used += 1;
                        }
                        violations.push(Violation {
                            entity: from.to_string(),
                            field: "depends_on".to_string(),
                            actual: 0.0,
                            threshold: 1.0,
                            direction: ThresholdDirection::Min,
                            excepted,
                        });
                    }
                }
            }
            _ => {} // "allowed" and unknown types — no-op for now
        }

        // Detect stale exceptions for dependency rules
        if let Some(rule_exceptions) = self.exceptions.rules.get(&rule.id) {
            for entity in rule_exceptions.keys() {
                let was_matched = matched_exception_entities.contains(entity.as_str());
                if !was_matched {
                    // Check if entity is even in the graph
                    let entity_exists = nodes.iter().any(|n| n == entity);
                    if entity_exists {
                        stale.push(StaleException {
                            rule_id: rule.id.clone(),
                            entity: entity.clone(),
                            reason: StaleReason::NowPassing,
                        });
                    } else {
                        stale.push(StaleException {
                            rule_id: rule.id.clone(),
                            entity: entity.clone(),
                            reason: StaleReason::EntityNotFound,
                        });
                    }
                }
            }
        }

        let has_unexcepted = violations.iter().any(|v| !v.excepted);
        let severity = rule.severity.unwrap_or(self.config.config.severity_default);
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
                dimension: rule.dimension.clone(),
                status,
                violations,
                exceptions_used,
                total_entities: Some(entity_count),
            },
            stale,
            matched_exception_entities,
        ))
    }

    /// Check if a dependency exception applies for a given entity.
    fn check_dependency_exception(
        &self,
        rule_id: &str,
        entity: &str,
        matched: &mut HashSet<String>,
    ) -> bool {
        if let Some(exc) = self.exceptions.get(rule_id, entity) {
            matched.insert(entity.to_string());
            // For dependency exceptions, targets field can limit which imports are excepted.
            // For simplicity, presence of exception = excepted (targets checked elsewhere).
            exc.value.is_none() || exc.targets.is_some()
        } else {
            false
        }
    }

    /// Evaluate a structural rule (stub — pattern catalog not yet implemented).
    fn evaluate_structural_rule(&self, rule: &StructuralRule) -> RuleResult {
        RuleResult {
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            dimension: rule.dimension.clone().or_else(|| Some("ST01".to_string())),
            status: RuleStatus::Skip,
            violations: vec![],
            exceptions_used: 0,
            total_entities: None,
        }
    }

    /// Attach a previous report for trend computation.
    pub fn with_previous_report(mut self, report: FitnessReport) -> Self {
        self.previous_report = Some(report);
        self
    }

    /// Compute system-level fitness score from dimension results.
    fn compute_system_fitness(
        &self,
        dimensions: &HashMap<String, DimensionResult>,
    ) -> Option<SystemFitnessResult> {
        if !self.config.system_fitness.enabled {
            return None;
        }

        // Collect active dimensions with scores
        let active: Vec<(&str, f64)> = dimensions
            .iter()
            .filter(|(_, d)| d.status == DimensionStatus::Active)
            .filter_map(|(code, d)| d.score.map(|s| (code.as_str(), s)))
            .collect();

        if active.is_empty() {
            return None;
        }

        let configured_weights = &self.config.system_fitness.weights;
        let (weights_used, weights_note) = if configured_weights.is_empty() {
            // Equal weights for active dimensions
            let w = 1.0 / active.len() as f64;
            let weights: HashMap<String, f64> = active
                .iter()
                .map(|(code, _)| (code.to_string(), w))
                .collect();
            (weights, None)
        } else {
            // Redistribute configured weights among active dimensions only
            let active_codes: HashSet<&str> = active.iter().map(|(c, _)| *c).collect();
            let active_weights: Vec<(&str, f64)> = configured_weights
                .iter()
                .filter(|(k, _)| active_codes.contains(k.as_str()))
                .map(|(k, &v)| (k.as_str(), v))
                .collect();

            let sum: f64 = active_weights.iter().map(|(_, w)| w).sum();

            let skipped_codes: Vec<&str> = configured_weights
                .keys()
                .filter(|k| !active_codes.contains(k.as_str()))
                .map(|k| k.as_str())
                .collect();

            if sum == 0.0 {
                return None;
            }

            let weights: HashMap<String, f64> = active_weights
                .iter()
                .map(|(code, w)| (code.to_string(), w / sum))
                .collect();

            let note = if !skipped_codes.is_empty() {
                Some(format!(
                    "{} skipped; weights redistributed proportionally among active dimensions",
                    skipped_codes.join(" and ")
                ))
            } else {
                None
            };

            (weights, note)
        };

        // Compute weighted score
        let score: f64 = active
            .iter()
            .map(|(code, s)| {
                let w = weights_used.get(*code).copied().unwrap_or(0.0);
                w * s
            })
            .sum();

        let min_score = self.config.system_fitness.min_score;
        let passing = score >= min_score;

        // Compute trend from previous report
        let trend = self.compute_trend(score, dimensions);

        Some(SystemFitnessResult {
            score,
            min_score,
            passing,
            weights_used,
            weights_note,
            trend,
        })
    }

    /// Compute trend data by comparing current results against previous report.
    fn compute_trend(
        &self,
        current_score: f64,
        current_dimensions: &HashMap<String, DimensionResult>,
    ) -> Option<TrendData> {
        let prev = self.previous_report.as_ref()?;
        let prev_fitness = prev.system_fitness.as_ref()?;
        let previous_score = prev_fitness.score;
        let delta = current_score - previous_score;

        let direction = if delta > 0.005 {
            TrendDirection::Improving
        } else if delta < -0.005 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        // Per-dimension deltas
        let mut dimension_deltas = HashMap::new();
        for (code, current) in current_dimensions {
            if let (Some(cur_score), Some(prev_dim)) = (current.score, prev.dimensions.get(code)) {
                if let Some(prev_score) = prev_dim.score {
                    let d = cur_score - prev_score;
                    // Round to 2 decimal places
                    dimension_deltas.insert(code.clone(), (d * 100.0).round() / 100.0);
                }
            }
        }

        Some(TrendData {
            previous_score,
            delta,
            direction,
            dimension_deltas,
        })
    }

    /// Compute per-dimension scoring results from evaluated rule results.
    fn compute_dimension_results(
        &self,
        results: &[RuleResult],
    ) -> HashMap<String, DimensionResult> {
        let mut dimensions: HashMap<String, DimensionResult> = HashMap::new();

        // Initialize all enabled dimensions
        for &code in DimensionCode::ALL {
            if !self.config.dimensions.is_enabled(code) {
                dimensions.insert(
                    code.as_str().to_string(),
                    DimensionResult {
                        name: code.name().to_string(),
                        status: DimensionStatus::Disabled,
                        score: None,
                        rules_evaluated: 0,
                        rules_passed: 0,
                        rules_failed: 0,
                        rules_warned: 0,
                        total_violations: 0,
                        excepted_violations: 0,
                    },
                );
                continue;
            }

            // Collect rules belonging to this dimension
            let dim_results: Vec<&RuleResult> = results
                .iter()
                .filter(|r| r.dimension.as_deref() == Some(code.as_str()))
                .collect();

            if dim_results.is_empty() {
                // Enabled but no rules — active with perfect score
                dimensions.insert(
                    code.as_str().to_string(),
                    DimensionResult {
                        name: code.name().to_string(),
                        status: DimensionStatus::Active,
                        score: Some(1.0),
                        rules_evaluated: 0,
                        rules_passed: 0,
                        rules_failed: 0,
                        rules_warned: 0,
                        total_violations: 0,
                        excepted_violations: 0,
                    },
                );
                continue;
            }

            let all_skipped = dim_results.iter().all(|r| r.status == RuleStatus::Skip);
            let rules_evaluated = dim_results
                .iter()
                .filter(|r| r.status != RuleStatus::Skip)
                .count();
            let rules_passed = dim_results
                .iter()
                .filter(|r| r.status == RuleStatus::Pass)
                .count();
            let rules_failed = dim_results
                .iter()
                .filter(|r| r.status == RuleStatus::Fail)
                .count();
            let rules_warned = dim_results
                .iter()
                .filter(|r| r.status == RuleStatus::Warn)
                .count();
            let total_violations: usize = dim_results.iter().map(|r| r.violations.len()).sum();
            let excepted_violations: usize = dim_results
                .iter()
                .flat_map(|r| &r.violations)
                .filter(|v| v.excepted)
                .count();

            // Score = 1.0 - (unexcepted_violations / total_entities)
            let score = if all_skipped {
                None
            } else {
                let total_entities: usize = dim_results
                    .iter()
                    .filter(|r| r.status != RuleStatus::Skip)
                    .filter_map(|r| r.total_entities)
                    .sum();
                let unexcepted = total_violations - excepted_violations;
                if total_entities == 0 {
                    Some(1.0)
                } else {
                    Some(1.0 - (unexcepted as f64 / total_entities as f64))
                }
            };

            let status = if all_skipped {
                DimensionStatus::Skipped
            } else {
                DimensionStatus::Active
            };

            dimensions.insert(
                code.as_str().to_string(),
                DimensionResult {
                    name: code.name().to_string(),
                    status,
                    score,
                    rules_evaluated,
                    rules_passed,
                    rules_failed,
                    rules_warned,
                    total_violations,
                    excepted_violations,
                },
            );
        }

        dimensions
    }

    /// Returns true if any error-severity rules failed or system fitness is below threshold.
    pub fn has_failures(report: &FitnessReport) -> bool {
        let rule_failures = report.results.iter().any(|r| r.status == RuleStatus::Fail);
        let system_failure = report.system_fitness.as_ref().is_some_and(|sf| !sf.passing);
        rule_failures || system_failure
    }

    /// Returns true if any warning-severity rules triggered (but no errors).
    pub fn has_warnings(report: &FitnessReport) -> bool {
        report.results.iter().any(|r| r.status == RuleStatus::Warn)
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
                "class" => Some("classes"),
                "system" => Some("systems"),
                _ => None,
            };

            // Step 2: Check if map contains the scope-derived wrapper key pointing to an array
            if let Some(key) = wrapper_key {
                if let Some(serde_json::Value::Array(arr)) = map.get(key) {
                    return extract_entities_from_array(arr);
                }
            }

            // Step 3: Fallback heuristic — if exactly one key has an array value, unwrap it
            let array_entries: Vec<_> = map.iter().filter(|(_, v)| v.is_array()).collect();
            if array_entries.len() == 1 {
                if let serde_json::Value::Array(arr) = array_entries[0].1 {
                    return extract_entities_from_array(arr);
                }
            }

            // Step 4: Fall through to flat-object behavior
            map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
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

/// Check if a path matches a glob pattern.
fn glob_matches(path: &str, pattern: &str) -> bool {
    glob::Pattern::new(pattern)
        .map(|p| p.matches(path))
        .unwrap_or(false)
}

// ─── Dependency Graph Helpers ──────────────────────────────────────────────

/// Parse a dependency graph JSON into (nodes, edges).
///
/// Expected format:
/// ```json
/// {
///   "nodes": ["src/api", "src/domain", "src/infra"],
///   "edges": [["src/api", "src/domain"], ["src/domain", "src/infra"]]
/// }
/// ```
fn load_dependency_graph(graph: &serde_json::Value) -> (Vec<String>, Vec<(String, String)>) {
    let nodes: Vec<String> = graph
        .get("nodes")
        .and_then(|n| n.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let edges: Vec<(String, String)> = graph
        .get("edges")
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|edge| {
                    let pair = edge.as_array()?;
                    let from = pair.first()?.as_str()?;
                    let to = pair.get(1)?.as_str()?;
                    Some((from.to_string(), to.to_string()))
                })
                .collect()
        })
        .unwrap_or_default();

    (nodes, edges)
}

/// Tarjan's strongly connected components algorithm.
///
/// Returns all SCCs (including single-node ones). Caller should filter for
/// `scc.len() > 1` to find actual cycles.
fn tarjan_scc<'a>(nodes: &HashSet<&'a str>, edges: &[(&'a str, &'a str)]) -> Vec<Vec<String>> {
    // Build adjacency list
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in nodes {
        adj.entry(node).or_default();
    }
    for (from, to) in edges {
        adj.entry(from).or_default().push(to);
    }

    let mut index_counter: usize = 0;
    let mut stack: Vec<&str> = Vec::new();
    let mut on_stack: HashSet<&str> = HashSet::new();
    let mut indices: HashMap<&str, usize> = HashMap::new();
    let mut lowlinks: HashMap<&str, usize> = HashMap::new();
    let mut result: Vec<Vec<String>> = Vec::new();

    #[allow(clippy::too_many_arguments)]
    fn strongconnect<'b>(
        v: &'b str,
        adj: &HashMap<&str, Vec<&'b str>>,
        index_counter: &mut usize,
        stack: &mut Vec<&'b str>,
        on_stack: &mut HashSet<&'b str>,
        indices: &mut HashMap<&'b str, usize>,
        lowlinks: &mut HashMap<&'b str, usize>,
        result: &mut Vec<Vec<String>>,
    ) {
        indices.insert(v, *index_counter);
        lowlinks.insert(v, *index_counter);
        *index_counter += 1;
        stack.push(v);
        on_stack.insert(v);

        if let Some(neighbors) = adj.get(v) {
            for &w in neighbors {
                if !indices.contains_key(w) {
                    strongconnect(
                        w,
                        adj,
                        index_counter,
                        stack,
                        on_stack,
                        indices,
                        lowlinks,
                        result,
                    );
                    let w_low = lowlinks[w];
                    let v_low = lowlinks[v];
                    lowlinks.insert(v, v_low.min(w_low));
                } else if on_stack.contains(w) {
                    let w_idx = indices[w];
                    let v_low = lowlinks[v];
                    lowlinks.insert(v, v_low.min(w_idx));
                }
            }
        }

        if lowlinks[v] == indices[v] {
            let mut scc = Vec::new();
            loop {
                let w = stack.pop().unwrap();
                on_stack.remove(w);
                scc.push(w.to_string());
                if w == v {
                    break;
                }
            }
            result.push(scc);
        }
    }

    for &node in nodes {
        if !indices.contains_key(node) {
            strongconnect(
                node,
                &adj,
                &mut index_counter,
                &mut stack,
                &mut on_stack,
                &mut indices,
                &mut lowlinks,
                &mut result,
            );
        }
    }

    result
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
            dimension: None,
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
            dimension: None,
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
        assert_eq!(
            extract_field_value(&entity, "cyclomatic_complexity"),
            Some(42.0)
        );
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
        assert_eq!(
            extract_field_value(&entity, "metrics.cognitive"),
            Some(42.0)
        );
        // Deep dot-path
        assert_eq!(
            extract_field_value(&entity, "metrics.martin.ce"),
            Some(15.0)
        );
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

        // Flat object still works
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
