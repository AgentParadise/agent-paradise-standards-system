//! Ecosystem crate identity  -  the allowlist of crates that don't follow the
//! `apss-vN-NNNN-slug` standard-crate convention.
//!
//! Non-standard-shaped ecosystem crates fall into two buckets:
//!
//! - Named literals: `apss-core`, `apss`, `apss-project-config`,
//!   `apss-distribution`  -  shared engine and CLI bootstrap.
//! - Meta-substandard prefix: `aps-v1-0000-*`  -  substandards belonging to the
//!   meta standard (e.g. CF01, DI01, SS01).
//!
//! This list is shared by DI01 (distribution validation) and any future
//! tooling that needs to distinguish ecosystem crates from standards.

/// Named ecosystem crates that are not standards but still live in the workspace.
pub const ECOSYSTEM_CRATE_NAMES: &[&str] = &[
    "apss-core",
    "apss",
    "apss-project-config",
    "apss-distribution",
];

/// Prefix identifying meta-substandard crates (e.g. `aps-v1-0000-cf01-project-config`).
pub const META_SUBSTANDARD_PREFIX: &str = "aps-v1-0000";

/// Returns `true` if the crate is an ecosystem crate rather than a published standard.
pub fn is_ecosystem_crate(name: &str) -> bool {
    ECOSYSTEM_CRATE_NAMES.contains(&name) || name.starts_with(META_SUBSTANDARD_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_ecosystem_crates_are_ecosystem() {
        assert!(is_ecosystem_crate("apss-core"));
        assert!(is_ecosystem_crate("apss"));
        assert!(is_ecosystem_crate("apss-project-config"));
        assert!(is_ecosystem_crate("apss-distribution"));
    }

    #[test]
    fn meta_substandard_prefix_is_ecosystem() {
        assert!(is_ecosystem_crate("aps-v1-0000-meta"));
        assert!(is_ecosystem_crate("aps-v1-0000-ss01-substandard-structure"));
        assert!(is_ecosystem_crate("aps-v1-0000-cf01-project-config"));
    }

    #[test]
    fn standard_crates_are_not_ecosystem() {
        assert!(!is_ecosystem_crate("apss-v1-0001-code-topology"));
        assert!(!is_ecosystem_crate("apss-v1-0003-fitness"));
    }

    #[test]
    fn empty_and_unrelated_names_are_not_ecosystem() {
        assert!(!is_ecosystem_crate(""));
        assert!(!is_ecosystem_crate("serde"));
    }
}
