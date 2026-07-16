//! Substandard implementations as feature-gated modules (ADR-0002).
//!
//! Each cargo feature name equals the substandard profile code (the id suffix
//! after the dot). The codes are cryptic on purpose, so here is the mapping to
//! each substandard's human name (from its `substandard.toml`):
//!
//! - `CI01` -> GitHub Actions CI Integration
//! - `MM01` -> Mermaid Diagram Projector
//! - `FD01` -> 3D Force-Directed Coupling Visualization
//! - `RS01` -> Rust Language Adapter
//! - `VZ01` -> Topology Visualization Dashboard

// Shared, ungated Handlebars renderer and page-shell partials. Used by both
// FD01 (viz_3d) and VZ01 (viz_dashboard), so it must not sit under either
// feature gate; otherwise building one substandard alone would fail to
// resolve the render helper.
#[cfg(any(feature = "FD01", feature = "VZ01"))]
pub mod render_shared;

#[cfg(feature = "CI01")]
pub mod ci_github_actions;

#[cfg(feature = "MM01")]
pub mod viz_mermaid;

#[cfg(feature = "FD01")]
pub mod viz_3d;

#[cfg(feature = "RS01")]
pub mod lang_rust;

#[cfg(feature = "VZ01")]
pub mod viz_dashboard;
