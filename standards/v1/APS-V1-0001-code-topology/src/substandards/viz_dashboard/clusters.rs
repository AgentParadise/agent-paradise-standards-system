//! Package Clusters Visualization
//!
//! 2D force-directed graph showing package/slice relationships.
//! - **Nodes** = packages/slices
//! - **Edges** = coupling between packages
//! - **Size** = module count
//! - **Color** = average health
//!
//! Markup lives in `templates/clusters.hbs`.

use super::{escape_json_for_html, render::render_template};
use serde::Serialize;

const TEMPLATE: &str = include_str!("templates/clusters.hbs");

#[derive(Serialize)]
struct ClustersCtx<'a> {
    modules_json: &'a str,
    coupling_json: &'a str,
}

/// Generate Package Clusters HTML visualization.
///
/// # Arguments
/// * `modules_json` - JSON array of module data
/// * `coupling_json` - JSON coupling matrix data
///
/// # Returns
/// Complete HTML document as a string
pub fn generate(modules_json: &str, coupling_json: &str) -> String {
    let modules_escaped = escape_json_for_html(modules_json);
    let coupling_escaped = escape_json_for_html(coupling_json);
    let ctx = ClustersCtx {
        modules_json: &modules_escaped,
        coupling_json: &coupling_escaped,
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
        assert!(html.contains("Package Clusters"));
    }

    #[test]
    fn test_generate_embeds_both_payloads() {
        let html = generate(r#"[{"id":"x"}]"#, r#"{"m":[]}"#);
        assert!(html.contains(r#"const MODULES = [{"id":"x"}];"#));
        assert!(html.contains(r#"const COUPLING = {"m":[]};"#));
    }
}
