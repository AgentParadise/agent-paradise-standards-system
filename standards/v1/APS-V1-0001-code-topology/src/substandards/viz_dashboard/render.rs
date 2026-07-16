//! Handlebars rendering entry point for dashboard visualizations.
//!
//! The real implementation lives in [`crate::substandards::render_shared`]
//! so that both VZ01 (this substandard) and FD01 (viz_3d) can share the
//! same renderer without one feature gate depending on the other. This
//! module is a thin re-export kept so intra-substandard callers can write
//! `super::render::render_template` without knowing where the helper lives.
//!
//! Substitutions use triple-brace `{{{name}}}` syntax so that pre-rendered
//! HTML, CSS, and JS content is not HTML-escaped. Shared page-shell
//! scaffolding is available as Handlebars partials (`{{> page_head}}`,
//! `{{> three_importmap}}`); see [`crate::substandards::render_shared`].

pub use crate::substandards::render_shared::render_template;
