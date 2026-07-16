//! Dashboard Index Visualization
//!
//! Landing page linking to all visualization types with summary statistics.
//! HTML markup lives in `templates/index.hbs`; this module just builds the
//! serialization context and hands it to the shared handlebars renderer.

use super::{health_label, health_to_color, render::render_template};
use serde::Serialize;

const TEMPLATE: &str = include_str!("templates/index.hbs");

#[derive(Serialize)]
struct IndexCtx<'a> {
    repo_name: &'a str,
    timestamp: String,
    module_count: usize,
    slice_count: usize,
    health_color: &'a str,
    health_pct: i32,
    health_label: &'a str,
}

/// Generate dashboard index HTML.
///
/// # Arguments
/// * `repo_name` - Repository name for the title
/// * `module_count` - Total number of modules
/// * `slice_count` - Number of feature slices
/// * `avg_health` - Average health score (0.0-1.0)
///
/// # Returns
/// Complete HTML document as a string
pub fn generate(
    repo_name: &str,
    module_count: usize,
    slice_count: usize,
    avg_health: f64,
) -> String {
    let ctx = IndexCtx {
        repo_name,
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        module_count,
        slice_count,
        health_color: health_to_color(avg_health),
        health_pct: (avg_health * 100.0).round() as i32,
        health_label: health_label(avg_health),
    };
    render_template(TEMPLATE, &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_contains_doctype() {
        let html = generate("test-repo", 10, 3, 0.75);
        assert!(html.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn test_generate_contains_stats() {
        let html = generate("my-project", 42, 5, 0.85);
        assert!(html.contains("42"));
        assert!(html.contains("5"));
        assert!(html.contains("my-project"));
    }

    #[test]
    fn test_generate_contains_all_viz_links() {
        let html = generate("x", 1, 1, 0.5);
        for href in [
            "topology-3d.html",
            "codecity.html",
            "clusters.html",
            "vsa.html",
        ] {
            assert!(html.contains(href), "missing link to {href}");
        }
    }

    #[test]
    fn test_generate_no_unrendered_handlebars() {
        // Guard against accidentally leaving `{{unknown}}` in the template.
        let html = generate("x", 1, 1, 0.5);
        assert!(
            !html.contains("{{"),
            "unrendered handlebars token in output"
        );
    }
}
