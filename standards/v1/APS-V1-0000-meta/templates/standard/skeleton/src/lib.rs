//! {{name}}
//!
//! This standard defines...
//!
//! TODO: Add description

use aps_core::{Diagnostic, Diagnostics};
use std::path::Path;

/// Error codes for this standard's validation.
pub mod error_codes {
    // TODO: Add error codes
}

/// The {{name}} implementation.
pub struct {{id}}Standard;

impl {{id}}Standard {
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for {{id}}Standard {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: Implement the Standard trait
// impl aps_v1_0000_meta::Standard for {{id}}Standard {
//     fn validate_package(&self, path: &Path) -> Diagnostics {
//         Diagnostics::new()
//     }
//
//     fn validate_repo(&self, path: &Path) -> Diagnostics {
//         Diagnostics::new()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let _ = {{id}}Standard::new();
    }
}

