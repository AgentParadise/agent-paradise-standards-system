//! Structural integration tests for the dashboard viz generators.
//!
//! Rather than snapshotting the full HTML output (which changes size on
//! every template tweak), these tests assert the invariants that matter
//! for behavior parity: rendered documents are well-shaped HTML, expected
//! element IDs are present, and embedded JSON payloads round-trip to the
//! same Rust value the caller passed in.
//!
//! Runs only when the dashboard feature (`VZ01`) is enabled; the whole
//! `viz_dashboard` module is feature-gated the same way.

#![cfg(feature = "VZ01")]

use code_topology::substandards::viz_dashboard::{clusters, codecity, force_3d, index, vsa};
use serde_json::Value;

fn expect_html_shell(html: &str) {
    assert!(
        html.starts_with("<!DOCTYPE html>"),
        "missing DOCTYPE prologue"
    );
    assert!(html.contains("<html"), "missing <html> root");
    assert!(html.contains("</html>"), "missing </html> terminator");
    assert!(html.contains("<head>"), "missing <head>");
    assert!(html.contains("<body>"), "missing <body>");
}

fn extract_between<'a>(hay: &'a str, start: &str, end: &str) -> &'a str {
    let s = hay
        .find(start)
        .unwrap_or_else(|| panic!("missing marker: {start}"))
        + start.len();
    let e_rel = hay[s..]
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker: {end} after start"));
    &hay[s..s + e_rel]
}

#[test]
fn index_page_has_shell_and_all_viz_links() {
    let html = index::generate("demo", 12, 4, 0.75);
    expect_html_shell(&html);
    for href in [
        "topology-3d.html",
        "codecity.html",
        "clusters.html",
        "vsa.html",
    ] {
        assert!(html.contains(href), "index missing link to {href}");
    }
    // No unrendered handlebars tokens leaked through.
    assert!(!html.contains("{{"), "unrendered handlebars token in index");
}

#[test]
fn vsa_embeds_modules_json_that_round_trips() {
    let payload = r#"[{"id":"a","name":"a","health":0.5}]"#;
    let html = vsa::generate(payload);
    expect_html_shell(&html);
    let embedded = extract_between(&html, "const MODULES = ", ";");
    let value: Value = serde_json::from_str(embedded).expect("MODULES payload must be JSON");
    let expected: Value = serde_json::from_str(payload).unwrap();
    assert_eq!(value, expected);
}

#[test]
fn vsa_placeholder_has_shell_and_hint() {
    let html = vsa::generate_placeholder();
    expect_html_shell(&html);
    assert!(html.contains("No VSA Configuration Found"));
}

#[test]
fn clusters_embeds_both_payloads() {
    let modules = r#"[{"id":"a","slice":"x","layer":"y"}]"#;
    let coupling = r#"{"modules":["a"],"matrix":[[0.0]]}"#;
    let html = clusters::generate(modules, coupling);
    expect_html_shell(&html);

    let m_embed = extract_between(&html, "const MODULES = ", ";");
    let c_embed = extract_between(&html, "const COUPLING = ", ";");
    let mv: Value = serde_json::from_str(m_embed).unwrap();
    let cv: Value = serde_json::from_str(c_embed).unwrap();
    assert_eq!(mv, serde_json::from_str::<Value>(modules).unwrap());
    assert_eq!(cv, serde_json::from_str::<Value>(coupling).unwrap());
}

#[test]
fn codecity_embeds_modules_payload() {
    let modules = r#"[{"id":"x","slice":"s","layer":"l","lines_of_code":10}]"#;
    let html = codecity::generate(modules, "{}");
    expect_html_shell(&html);
    let embedded = extract_between(&html, "const MODULES = ", ";");
    let v: Value = serde_json::from_str(embedded).unwrap();
    assert_eq!(v, serde_json::from_str::<Value>(modules).unwrap());
}

#[test]
fn force_3d_embeds_scene_and_counts() {
    let scene = r#"{"format":"topology-webgl/v1","nodes":[],"edges":[]}"#;
    let html = force_3d::generate(scene, 42, 17);
    expect_html_shell(&html);
    assert!(html.contains("42"));
    assert!(html.contains("17"));
    let embedded = extract_between(&html, "const data = ", ";");
    let v: Value = serde_json::from_str(embedded).unwrap();
    assert_eq!(v, serde_json::from_str::<Value>(scene).unwrap());
}

#[test]
fn dashboard_outputs_never_close_script_prematurely() {
    // The escape_json_for_html helper is the guard; verify it survives an
    // adversarial payload for every generator that embeds JSON.
    let evil = r#"[{"id":"</script><script>alert(1)</script>"}]"#;
    for html in [
        vsa::generate(evil),
        clusters::generate(evil, "{}"),
        codecity::generate(evil, "{}"),
    ] {
        assert!(
            !html.contains("</script><script>alert(1)</script>"),
            "raw injection leaked into rendered HTML"
        );
    }
}
