//! {{name}} (Experimental)
//!
//! This is an experimental standard for...
//!
//! ⚠️ EXPERIMENTAL: This standard is in incubation and may change significantly.

use aps_core::{Diagnostic, Diagnostics};
use std::path::Path;

/// Error codes for this experiment's validation.
pub mod error_codes {
    // TODO: Add error codes
}

/// The {{name}} implementation.
pub struct Experiment;

impl Experiment {
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for Experiment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let _ = Experiment::new();
    }
}

