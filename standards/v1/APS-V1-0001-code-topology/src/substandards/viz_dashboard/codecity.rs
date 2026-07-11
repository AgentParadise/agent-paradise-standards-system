//! CodeCity Visualization
//!
//! 3D city metaphor where buildings represent modules.
//! - **Height** = cyclomatic complexity (log-scaled)
//! - **Footprint** = lines of code (sqrt-scaled)
//! - **Color** = health score (green -> red gradient)
//! - **Districts** = slices/packages (treemap layout with labeled ground planes)
//!
//! Markup lives in `templates/codecity.hbs`. The `coupling_json` argument
//! is currently unused by the JS front-end (buildings are laid out from
//! the module data alone); it is kept in the public signature to match
//! the sibling viz generators.

use super::{escape_json_for_html, render::render_template};
use serde::Serialize;

const TEMPLATE: &str = include_str!("templates/codecity.hbs");

#[derive(Serialize)]
struct CodeCityCtx<'a> {
    modules_json: &'a str,
}

/// Generate CodeCity HTML visualization.
///
/// # Arguments
/// * `modules_json` - JSON array of module data with slice, layer, health, complexity
/// * `coupling_json` - JSON coupling matrix data (accepted for API parity; not yet consumed)
///
/// # Returns
/// Complete HTML document as a string
pub fn generate(modules_json: &str, coupling_json: &str) -> String {
    let modules_escaped = escape_json_for_html(modules_json);
    // Accepted for future use; also validates the payload is a legal string.
    let _coupling_escaped = escape_json_for_html(coupling_json);
    let ctx = CodeCityCtx {
        modules_json: &modules_escaped,
    };
    render_template(TEMPLATE, &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_contains_doctype() {
        let html = generate("[]", "{}");
        assert!(html.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn test_generate_contains_title() {
        let html = generate("[]", "{}");
        assert!(html.contains("<title>CodeCity"));
    }

    #[test]
    fn test_generate_embeds_modules_payload() {
        let html = generate(r#"[{"id":"x"}]"#, "{}");
        assert!(html.contains(r#"const MODULES = [{"id":"x"}];"#));
    }
}
