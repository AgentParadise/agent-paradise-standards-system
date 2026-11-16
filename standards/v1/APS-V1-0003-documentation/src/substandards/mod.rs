//! Substandard implementations as feature-gated modules (ADR-0002).
//!
//! Each cargo feature name equals the substandard profile code (the id suffix
//! after the dot). The codes are cryptic on purpose, so here is the mapping to
//! each substandard's human name (from its `substandard.toml`):
//!
//! - `AD01` -> Architecture Decision Records
//! - `PV01` -> North Star (Mission, Vision, Position)
//! - `RQ01` -> Requirements (optional doc type)
//! - `RT01` -> Retrospectives

#[cfg(feature = "AD01")]
pub mod adr;

#[cfg(feature = "PV01")]
pub mod purpose_and_vision;

#[cfg(feature = "RQ01")]
pub mod requirements;

#[cfg(feature = "RT01")]
pub mod retrospectives;
