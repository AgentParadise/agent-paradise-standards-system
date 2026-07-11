//! Shared handlebars rendering helpers for dashboard visualizations.
//!
//! Each viz submodule embeds its template with `include_str!` and calls
//! [`render_template`]. Substitutions use triple-brace `{{{name}}}` syntax
//! so that pre-rendered HTML, CSS, and JS content is not HTML-escaped.
//!
//! Template conventions used repo-wide:
//! - `{{{title}}}` for the document title.
//! - `{{{styles}}}` for a raw `<style>` block or CSS content.
//! - `{{{script}}}` for a raw JS content block (already escaped for
//!   embedding in a `<script>` tag).
//! - `{{{data_json}}}` for an embedded JSON payload (already run through
//!   [`super::escape_json_for_html`]).

use handlebars::Handlebars;
use serde::Serialize;

/// Render a viz HTML template with the given context.
///
/// Uses a fresh handlebars registry per call: templates are small and the
/// registry has no reusable per-call state we need to cache. This keeps the
/// call sites side-effect-free and thread-safe.
pub fn render_template<T: Serialize>(template: &str, ctx: &T) -> String {
    let mut hb = Handlebars::new();
    // Strict mode surfaces missing-variable bugs at test time instead of
    // silently emitting empty strings.
    hb.set_strict_mode(true);
    hb.render_template(template, ctx)
        .unwrap_or_else(|e| format!("<!-- handlebars render error: {e} -->"))
}
