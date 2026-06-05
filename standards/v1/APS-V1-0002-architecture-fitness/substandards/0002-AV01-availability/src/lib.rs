//! APS-V1-0002.AV01 - Availability Dimension
//!
//! Availability / SLO / error-budget governance via adapters. Incubating per
//! ADR 0003: thresholds are project-specific and require an ADR setting
//! concrete SLOs. The engine lives in the parent crate; this substandard
//! supplies the dimension identity and CLI registration.

/// Dimension code (matches APS-V1-0002 §1.4 and §3.1).
pub const DIMENSION_CODE: &str = "AV01";

/// Human-readable dimension name.
pub const DIMENSION_NAME: &str = "Availability";

/// Substandard semver.
pub const DIMENSION_VERSION: &str = "1.0.0";

/// Register this substandard with the apss-core composition registry per
/// APS-V1-0000.DI01.
pub fn register(registry: &mut dyn apss_core::registry::StandardRegistry) {
    registry.register(
        apss_core::registry::RegisteredStandard {
            id: "APS-V1-0002.AV01".to_string(),
            slug: "availability".to_string(),
            name: DIMENSION_NAME.to_string(),
            description: "Availability dimension (SLO / error-budget; incubating)".to_string(),
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
            "No composed CLI commands for architecture-fitness-av01; use the parent \
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
        assert_eq!(DIMENSION_CODE, "AV01");
        assert!(DIMENSION_VERSION.split('.').count() == 3);
    }
}
