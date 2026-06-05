//! APS-V1-0002.ST01 - Structural Integrity Dimension
//!
//! Structural-pattern catalog (forbidden_import, required_import,
//! layer_enforcement) evaluated over the topology dependency graph. The
//! evaluator lives in the parent crate's `evaluate_structural_rule`; this
//! substandard supplies the dimension identity and CLI registration.
//!
//! CK class-level metrics (DIT, CBO, RFC, WMC, LCOM) remain a scoped
//! follow-on awaiting a class-level analyzer per ADR 0003.

/// Dimension code (matches APS-V1-0002 §1.4 and §3.1).
pub const DIMENSION_CODE: &str = "ST01";

/// Human-readable dimension name.
pub const DIMENSION_NAME: &str = "Structural Integrity";

/// Substandard semver.
pub const DIMENSION_VERSION: &str = "1.0.0";

/// Register this substandard with the apss-core composition registry per
/// APS-V1-0000.DI01. The engine lives in the parent crate, so the handler is
/// a no-op.
pub fn register(registry: &mut dyn apss_core::registry::StandardRegistry) {
    registry.register(
        apss_core::registry::RegisteredStandard {
            id: "APS-V1-0002.ST01".to_string(),
            slug: "structural".to_string(),
            name: DIMENSION_NAME.to_string(),
            description: "Structural integrity dimension (forbidden / required / layer patterns)"
                .to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            commands: Vec::new(),
        },
        Box::new(NoopCommandHandler),
    );
}

struct NoopCommandHandler;

impl apss_core::registry::CommandHandler for NoopCommandHandler {
    fn execute(&self, _command: &str, _args: &[String], _config: &toml::Value) -> i32 {
        eprintln!(
            "No composed CLI commands for architecture-fitness-st01; use the parent \
             architecture-fitness via `apss run fitness validate`."
        );
        5
    }

    fn commands(&self) -> Vec<apss_core::registry::CommandInfo> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_constants_match_spec() {
        assert_eq!(DIMENSION_CODE, "ST01");
        assert!(DIMENSION_VERSION.split('.').count() == 3);
    }
}
