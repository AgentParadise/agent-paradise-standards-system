//! APS-V1-0002.MD01 — Modularity and Coupling Dimension
//!
//! Reference substandard for module-level coupling governance. Evaluates
//! Martin's package metrics (Afferent/Efferent Coupling, Instability, Distance
//! from Main Sequence) emitted at `.topology/metrics/coupling.json` by
//! APS-V1-0001.LANG01-rust.
//!
//! This crate is a *reference implementation* per APS-V1-0002 §3.3 R5: it does
//! not re-implement the fitness engine. Instead it publishes the canonical
//! default rule set (`DEFAULT_RULES_TOML`) and integration tests that pin the
//! end-to-end contract against fixture artifacts.

/// Dimension code (matches APS-V1-0002 §1.4 and §3.1).
pub const DIMENSION_CODE: &str = "MD01";

/// Human-readable dimension name.
pub const DIMENSION_NAME: &str = "Modularity and Coupling";

/// Substandard semver.
pub const DIMENSION_VERSION: &str = "1.0.0";

/// Canonical default rule set for MD01, as a TOML snippet.
///
/// Thresholds are drawn from:
/// - **Ce ≤ 20**: Martin (2003), *Agile Software Development*, §20 "Package
///   Design" — high fan-out signals tight coupling and brittle modules.
/// - **0.1 ≤ I ≤ 0.9**: Martin (1994), *OO Design Quality Metrics*. The extremes
///   (I = 0 rigid, I = 1 volatile) are both architectural smells.
/// - **D ≤ 0.7**: Martin (1994). Distance > 0.7 puts a module in the Zone of
///   Pain (concrete & stable) or Zone of Uselessness (abstract & unstable).
///
/// To enable: append to your `fitness.toml` or merge programmatically. These
/// rules read `metrics/coupling.json` (APS-V1-0001 coupling schema).
pub const DEFAULT_RULES_TOML: &str = r#"
[[rules.threshold]]
id = "md01-max-efferent-coupling"
name = "Maximum Efferent Coupling (Ce)"
dimension = "MD01"
source = "metrics/coupling.json"
field = "efferent_coupling"
max = 20
scope = "module"
severity = "error"

[[rules.threshold]]
id = "md01-instability-balance"
name = "Instability Balance"
dimension = "MD01"
source = "metrics/coupling.json"
field = "instability"
min = 0.1
max = 0.9
scope = "module"
severity = "warning"

[[rules.threshold]]
id = "md01-max-main-sequence-distance"
name = "Maximum Distance from Main Sequence"
dimension = "MD01"
source = "metrics/coupling.json"
field = "distance_from_main_sequence"
max = 0.7
scope = "module"
severity = "error"
"#;

/// Source artifact path consumed by MD01 default rules, relative to
/// `config.topology_dir`.
pub const SOURCE_ARTIFACT: &str = "metrics/coupling.json";

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
            assert_eq!(scope, "module");
        }
    }

    #[test]
    fn dimension_constants_match_spec() {
        assert_eq!(DIMENSION_CODE, "MD01");
        assert!(DIMENSION_VERSION.split('.').count() == 3);
    }
}
