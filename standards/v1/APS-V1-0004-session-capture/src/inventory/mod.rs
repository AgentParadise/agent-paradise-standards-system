//! Optional workflow inventory profile. Does not change the session envelope.
mod facts;
mod identity;
mod revision;
mod validation;

pub use facts::*;
pub use identity::*;
pub use revision::*;

pub const PROFILE_VERSION: &str = "session-inventory/1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryValidationError {
    Invalid(String),
}
impl std::fmt::Display for InventoryValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => write!(f, "invalid inventory: {message}"),
        }
    }
}
impl std::error::Error for InventoryValidationError {}

mod transport;
pub use transport::*;
