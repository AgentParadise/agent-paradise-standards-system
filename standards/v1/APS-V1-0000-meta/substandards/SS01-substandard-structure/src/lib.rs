//! APS-V1-0000.SS01 — Substandard Structure
//!
//! This substandard defines the structural requirements for all APS substandards.
//! Substandards are domain-specific extensions that inherit from a parent standard
//! and provide specialized implementations or profiles.
//!
//! # Key Principles
//!
//! 1. **Same Structure**: Substandards follow the same package structure as standards
//! 2. **Parent Reference**: Every substandard MUST reference a valid parent standard
//! 3. **Relaxed Compatibility**: Substandards MAY break within parent major version
//! 4. **Domain Specialization**: Used for language bindings, platform profiles, etc.

use aps_core::{Diagnostic, Diagnostics};
use std::path::Path;

/// Substandard ID regex pattern: APS-V1-XXXX.YY01
pub const SUBSTANDARD_ID_PATTERN: &str = r"^APS-V1-\d{4}\.[A-Z]{2}\d{2}$";

/// Error codes for substandard validation.
pub mod error_codes {
    pub const INVALID_SUBSTANDARD_ID: &str = "INVALID_SUBSTANDARD_ID";
    pub const INVALID_PARENT_REF: &str = "INVALID_PARENT_REF";
    pub const PARENT_NOT_FOUND: &str = "PARENT_NOT_FOUND";
    pub const SUBSTANDARD_WRONG_LOCATION: &str = "SUBSTANDARD_WRONG_LOCATION";
}

/// Validate a substandard ID format.
pub fn is_valid_substandard_id(id: &str) -> bool {
    let re = regex::Regex::new(SUBSTANDARD_ID_PATTERN).unwrap();
    re.is_match(id)
}

/// Extract the parent standard ID from a substandard ID.
///
/// # Example
/// ```
/// use aps_v1_0000_ss01_substandard_structure::extract_parent_id;
/// assert_eq!(extract_parent_id("APS-V1-0000.SS01"), Some("APS-V1-0000".to_string()));
/// ```
pub fn extract_parent_id(substandard_id: &str) -> Option<String> {
    substandard_id
        .find('.')
        .map(|dot_pos| substandard_id[..dot_pos].to_string())
}

/// Validate substandard-specific metadata.
pub fn validate_substandard_metadata(
    path: &Path,
    metadata: &aps_core::metadata::SubstandardMetadata,
    diagnostics: &mut Diagnostics,
) {
    use error_codes::*;

    // Validate ID format
    if !is_valid_substandard_id(&metadata.substandard.id) {
        diagnostics.push(
            Diagnostic::error(
                INVALID_SUBSTANDARD_ID,
                format!(
                    "Invalid substandard ID '{}': must match pattern APS-V1-XXXX.YY01",
                    metadata.substandard.id
                ),
            )
            .with_path(path)
            .with_hint("Use format: APS-V1-0000.SS01, APS-V1-0001.GH01, etc."),
        );
    }

    // Validate parent_id matches the prefix of the substandard ID
    if let Some(expected_parent) = extract_parent_id(&metadata.substandard.id)
        && metadata.substandard.parent_id != expected_parent
    {
        diagnostics.push(
            Diagnostic::error(
                INVALID_PARENT_REF,
                format!(
                    "parent_id '{}' does not match substandard ID prefix '{}'",
                    metadata.substandard.parent_id, expected_parent
                ),
            )
            .with_path(path)
            .with_hint(format!("Set parent_id = \"{expected_parent}\"")),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_substandard_ids() {
        assert!(is_valid_substandard_id("APS-V1-0000.SS01"));
        assert!(is_valid_substandard_id("APS-V1-0001.GH01"));
        assert!(is_valid_substandard_id("APS-V1-9999.PY99"));
    }

    #[test]
    fn test_invalid_substandard_ids() {
        assert!(!is_valid_substandard_id("APS-V1-0000")); // No suffix
        assert!(!is_valid_substandard_id("APS-V1-0000.ss01")); // Lowercase
        assert!(!is_valid_substandard_id("APS-V1-0000.S01")); // Only one letter
        assert!(!is_valid_substandard_id("EXP-V1-0000.SS01")); // Wrong prefix
    }

    #[test]
    fn test_extract_parent_id() {
        assert_eq!(
            extract_parent_id("APS-V1-0000.SS01"),
            Some("APS-V1-0000".to_string())
        );
        assert_eq!(
            extract_parent_id("APS-V1-0001.GH01"),
            Some("APS-V1-0001".to_string())
        );
        assert_eq!(extract_parent_id("APS-V1-0000"), None);
    }
}
