//! VSA (Vertical Slice Architecture) Visualization
//!
//! Matrix showing the intersection of feature slices and architectural layers.
//! - **Columns** = feature slices
//! - **Rows** = architectural layers
//! - **Cells** = module count with health indicator
//!
//! Markup lives in `templates/vsa.hbs` (matrix render) and
//! `templates/vsa_placeholder.hbs` (rendered when no `vsa.yaml` is
//! configured); Rust code builds the context and delegates to
//! [`super::render::render_template`].

use super::{escape_json_for_html, render::render_template};
use serde::Serialize;

const TEMPLATE: &str = include_str!("templates/vsa.hbs");
const PLACEHOLDER: &str = include_str!("templates/vsa_placeholder.hbs");

#[derive(Serialize)]
struct VsaCtx<'a> {
    modules_json: &'a str,
}

/// Generate VSA Diagram HTML visualization.
///
/// # Arguments
/// * `modules_json` - JSON array of module data with slice and layer fields
///
/// # Returns
/// Complete HTML document as a string
pub fn generate(modules_json: &str) -> String {
    let escaped = escape_json_for_html(modules_json);
    let ctx = VsaCtx {
        modules_json: &escaped,
    };
    render_template(TEMPLATE, &ctx)
}

/// Generate the placeholder page shown when a project has no `vsa.yaml`.
pub fn generate_placeholder() -> String {
    PLACEHOLDER.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_contains_doctype() {
        let html = generate("[]");
        assert!(html.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn test_generate_contains_title() {
        let html = generate("[]");
        assert!(html.contains("<title>VSA Diagram"));
    }

    #[test]
    fn test_generate_embeds_module_data() {
        let html = generate(r#"[{"id":"x"}]"#);
        assert!(html.contains(r#"const MODULES = [{"id":"x"}];"#));
    }

    #[test]
    fn test_placeholder_contains_hint() {
        let html = generate_placeholder();
        assert!(html.contains("No VSA Configuration Found"));
        assert!(html.contains("vsa.yaml"));
    }
}
