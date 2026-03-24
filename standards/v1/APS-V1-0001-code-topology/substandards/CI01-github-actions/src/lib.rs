//! GitHub Actions CI Integration (APS-V1-0001.CI01)
//!
//! Provides GitHub Actions workflow generation and topology check integration
//! for continuous integration pipelines.
//!
//! This substandard consumes `.topology/` artifacts produced by APS-V1-0001
//! and generates CI workflow configuration for topology diff checks on PRs.

/// Severity levels for topology check results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Pass,
    Warning,
    Failure,
}

impl Severity {
    /// Map severity to CI exit code per §6.
    pub fn exit_code(&self) -> i32 {
        match self {
            Severity::Pass => 0,
            Severity::Warning => 0, // unless fail_on_warning
            Severity::Failure => 1,
        }
    }

    /// Map severity with fail_on_warning behavior.
    pub fn exit_code_with_config(&self, fail_on_warning: bool) -> i32 {
        match self {
            Severity::Pass => 0,
            Severity::Warning if fail_on_warning => 1,
            Severity::Warning => 0,
            Severity::Failure => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_exit_codes() {
        assert_eq!(Severity::Pass.exit_code(), 0);
        assert_eq!(Severity::Warning.exit_code(), 0);
        assert_eq!(Severity::Failure.exit_code(), 1);
    }

    #[test]
    fn test_severity_fail_on_warning() {
        assert_eq!(Severity::Warning.exit_code_with_config(false), 0);
        assert_eq!(Severity::Warning.exit_code_with_config(true), 1);
        assert_eq!(Severity::Failure.exit_code_with_config(false), 1);
        assert_eq!(Severity::Pass.exit_code_with_config(true), 0);
    }
}
