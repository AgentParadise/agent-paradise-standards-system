//! Ungated Handlebars rendering shared across viz substandards.
//!
//! Both [`super::viz_3d`] (FD01) and [`super::viz_dashboard`] (VZ01) render
//! Handlebars templates. Keeping this helper ungated lets FD01 depend on it
//! without also enabling VZ01, and vice versa, so that
//! `--features FD01` / `--features VZ01` each build on their own.
//!
//! It also owns the shared page-shell partials so scaffolding boilerplate
//! ([`PAGE_HEAD_PARTIAL`], [`THREE_IMPORTMAP_PARTIAL`]) is defined in one
//! place and referenced from each viz template with `{{> partial_name}}`.
//!
//! Template conventions used repo-wide:
//! - `{{{title}}}` for the document title.
//! - `{{{styles}}}` for a raw `<style>` block or CSS content.
//! - `{{{script}}}` for a raw JS content block already escaped for embedding
//!   in a `<script>` tag.
//! - `{{{data_json}}}` for an embedded JSON payload already run through
//!   [`super::viz_dashboard::escape_json_for_html`] where available.

use handlebars::Handlebars;
use serde::Serialize;

/// Doctype + `<html>` + `<head>` opener shared by every viz template. Ends
/// after the viewport meta so each template can inject its own `<title>` and
/// `<style>` block without duplicating the preamble.
pub const PAGE_HEAD_PARTIAL: &str = include_str!("partials/page_head.hbs");

/// Three.js import map shared by every Three.js viz template (force_3d,
/// codecity, viz_3d/scene). Pins the CDN version in one place.
pub const THREE_IMPORTMAP_PARTIAL: &str = include_str!("partials/three_importmap.hbs");

/// Render a viz HTML template with the given context.
///
/// Uses a fresh Handlebars registry per call; the shared page-shell partials
/// are pre-registered so any template can reference `{{> page_head}}` and
/// `{{> three_importmap}}`. Strict mode surfaces missing-variable bugs at
/// test time instead of silently emitting empty strings.
pub fn render_template<T: Serialize>(template: &str, ctx: &T) -> String {
    let mut hb = Handlebars::new();
    hb.set_strict_mode(true);
    // Register partials as templates (Handlebars supports either form).
    hb.register_partial("page_head", PAGE_HEAD_PARTIAL)
        .expect("page_head partial is a valid handlebars template");
    hb.register_partial("three_importmap", THREE_IMPORTMAP_PARTIAL)
        .expect("three_importmap partial is a valid handlebars template");
    hb.render_template(template, ctx)
        .unwrap_or_else(|e| format!("<!-- handlebars render error: {e} -->"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct Ctx {
        title: String,
    }

    #[test]
    fn page_head_partial_renders_inline() {
        let template = "{{> page_head}}\n    <title>{{{title}}}</title>";
        let out = render_template(
            template,
            &Ctx {
                title: "hello".into(),
            },
        );
        assert!(out.starts_with("<!DOCTYPE html>"), "got: {out}");
        assert!(out.contains("<title>hello</title>"));
    }

    #[test]
    fn three_importmap_partial_contains_pinned_version() {
        let template = "{{> three_importmap}}";
        let out = render_template(template, &Ctx { title: "".into() });
        assert!(out.contains("three@0.160.0/build/three.module.js"));
        assert!(out.contains("three/addons/"));
    }
}
