//! Distribution & Installation (APS-V1-0000.DI01)
//!
//! Standard-facing wrapper around the APSS distribution runtime.

pub use apss_core::distribution::*;

pub fn register(registry: &mut dyn apss_core::registry::StandardRegistry) {
    apss_core::distribution::register(registry);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapper_reexports_distribution_validation() {
        let diags = validate_publishable_standard(std::path::Path::new("/definitely/missing"));
        assert!(diags.has_errors());
    }
}
