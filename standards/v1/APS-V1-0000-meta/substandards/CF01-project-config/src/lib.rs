//! Project Configuration (APS-V1-0000.CF01)
//!
//! Standard-facing wrapper around the APSS project configuration runtime.

pub use apss_core::project_config_validation::*;

pub fn register(registry: &mut dyn apss_core::registry::StandardRegistry) {
    apss_core::project_config_validation::register(registry);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapper_reexports_project_config_validation() {
        let diags = validate_project_config(std::path::Path::new("/definitely/missing/APSS.yaml"));
        assert!(diags.has_errors());
    }
}
