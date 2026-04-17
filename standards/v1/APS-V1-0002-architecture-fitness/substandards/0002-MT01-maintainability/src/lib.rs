//! APS-V1-0002.MT01 — Maintainability Dimension
//!
//! Reference substandard for function-level maintainability governance.
//! Evaluates McCabe cyclomatic complexity, SonarSource cognitive complexity,
//! and Halstead volume emitted at `.topology/metrics/functions.json` by
//! APS-V1-0001.LANG01-rust.
//!
//! This crate is a *reference implementation* per APS-V1-0002 §3.3 R5: it does
//! not re-implement the fitness engine. It publishes the canonical default
//! rule set (`DEFAULT_RULES_TOML`) and integration tests that pin the end-to-
//! end contract against fixture artifacts.

/// Dimension code (matches APS-V1-0002 §1.4 and §3.1).
pub const DIMENSION_CODE: &str = "MT01";

/// Human-readable dimension name.
pub const DIMENSION_NAME: &str = "Maintainability";

/// Substandard semver.
pub const DIMENSION_VERSION: &str = "1.0.0";

/// Canonical default rule set for MT01, as a TOML snippet.
///
/// Thresholds are drawn from:
/// - **Cyclomatic ≤ 10**: McCabe (1976), *A Complexity Measure*. McCabe's
///   original paper recommended 10 as the upper bound before testability
///   degrades; widely adopted by SonarQube, NDepend, ESLint.
/// - **Cognitive ≤ 15**: SonarSource (2017), *Cognitive Complexity: A New Way
///   of Measuring Understandability*. The default SonarQube threshold.
/// - **Halstead Volume ≤ 1000**: Halstead (1977), *Elements of Software
///   Science*. Volumes above 1000 correlate with elevated defect rates in
///   Halstead's empirical studies.
///
/// All rules read `metrics/functions.json` (APS-V1-0001 functions schema).
/// Field paths use dot-notation per §4.3.1.
pub const DEFAULT_RULES_TOML: &str = r#"
[[rules.threshold]]
id = "mt01-max-cyclomatic"
name = "Maximum Cyclomatic Complexity"
dimension = "MT01"
source = "metrics/functions.json"
field = "metrics.cyclomatic"
max = 10
scope = "function"
severity = "error"
exclude = ["**/tests/**", "**/test_*"]

[[rules.threshold]]
id = "mt01-max-cognitive"
name = "Maximum Cognitive Complexity"
dimension = "MT01"
source = "metrics/functions.json"
field = "metrics.cognitive"
max = 15
scope = "function"
severity = "error"
exclude = ["**/tests/**", "**/test_*"]

[[rules.threshold]]
id = "mt01-max-halstead-volume"
name = "Maximum Halstead Volume"
dimension = "MT01"
source = "metrics/functions.json"
field = "metrics.halstead.volume"
max = 1000
scope = "function"
severity = "warning"
exclude = ["**/tests/**", "**/test_*"]
"#;

/// Source artifact path consumed by MT01 default rules, relative to
/// `config.topology_dir`.
pub const SOURCE_ARTIFACT: &str = "metrics/functions.json";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rules_parse_as_toml() {
        let parsed: toml::Value = toml::from_str(DEFAULT_RULES_TOML).expect("valid TOML");
        let rules = parsed
            .get("rules")
            .and_then(|v| v.get("threshold"))
            .and_then(|v| v.as_array())
            .expect("rules.threshold array");
        assert_eq!(rules.len(), 3);
        for rule in rules {
            let dim = rule.get("dimension").and_then(|v| v.as_str()).unwrap();
            assert_eq!(dim, DIMENSION_CODE);
            let source = rule.get("source").and_then(|v| v.as_str()).unwrap();
            assert_eq!(source, SOURCE_ARTIFACT);
            let scope = rule.get("scope").and_then(|v| v.as_str()).unwrap();
            assert_eq!(scope, "function");
        }
    }

    #[test]
    fn dimension_constants_match_spec() {
        assert_eq!(DIMENSION_CODE, "MT01");
        assert!(DIMENSION_VERSION.split('.').count() == 3);
    }
}
