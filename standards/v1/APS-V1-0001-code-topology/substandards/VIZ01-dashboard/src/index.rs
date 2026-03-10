//! Dashboard Index Visualization
//!
//! Landing page linking to all visualization types with summary statistics.

use crate::{health_label, health_to_color};

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
#[allow(clippy::uninlined_format_args)]
pub fn generate(repo_name: &str, module_count: usize, slice_count: usize, avg_health: f64) -> String {
    let health_color = health_to_color(avg_health);
    let health_lbl = health_label(avg_health);
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{repo_name} — Topology Dashboard</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, sans-serif; background: #0a0a0f; color: #fff; padding: 40px; min-height: 100vh; }}
        .container {{ max-width: 1000px; margin: 0 auto; }}
        h1 {{ font-size: 32px; margin-bottom: 8px; }}
        .subtitle {{ color: #666; margin-bottom: 40px; }}
        .stats {{ display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; margin-bottom: 40px; }}
        .stat {{ background: #15151a; padding: 24px; border-radius: 12px; border: 1px solid #222; }}
        .stat-value {{ font-size: 36px; font-weight: 600; margin-bottom: 8px; }}
        .stat-label {{ color: #666; font-size: 14px; }}
        .about {{ margin-bottom: 40px; }}
        .about-toggle {{ background: #15151a; border: 1px solid #222; border-radius: 12px; padding: 18px 24px; color: #fff; font-size: 15px; font-weight: 500; cursor: pointer; width: 100%; text-align: left; display: flex; justify-content: space-between; align-items: center; transition: border-color 0.2s; }}
        .about-toggle:hover {{ border-color: #00ff88; }}
        .about-toggle .chevron {{ transition: transform 0.3s; font-size: 12px; color: #666; }}
        .about-toggle.open .chevron {{ transform: rotate(180deg); }}
        .about-body {{ max-height: 0; overflow: hidden; transition: max-height 0.4s ease, padding 0.3s ease; background: #15151a; border: 1px solid #222; border-top: none; border-radius: 0 0 12px 12px; }}
        .about-body.open {{ max-height: 800px; padding: 24px; }}
        .about-body h3 {{ font-size: 15px; font-weight: 600; margin: 20px 0 8px 0; color: #ccc; }}
        .about-body h3:first-child {{ margin-top: 0; }}
        .about-body p {{ color: #888; font-size: 13px; line-height: 1.7; margin-bottom: 8px; }}
        .about-body .metric-grid {{ display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px; margin: 12px 0; }}
        .about-body .metric-item {{ background: #1a1a22; padding: 12px 16px; border-radius: 8px; }}
        .about-body .metric-item strong {{ color: #ddd; font-size: 13px; display: block; margin-bottom: 4px; }}
        .about-body .metric-item span {{ color: #666; font-size: 12px; line-height: 1.5; }}
        .viz-grid {{ display: grid; grid-template-columns: repeat(2, 1fr); gap: 20px; }}
        .viz-card {{ background: #15151a; border-radius: 12px; border: 1px solid #222; padding: 24px; text-decoration: none; color: #fff; transition: all 0.2s; }}
        .viz-card:hover {{ border-color: #00ff88; transform: translateY(-2px); }}
        .viz-icon {{ font-size: 40px; margin-bottom: 16px; }}
        .viz-title {{ font-size: 18px; font-weight: 600; margin-bottom: 8px; }}
        .viz-desc {{ color: #666; font-size: 13px; line-height: 1.5; }}
        footer {{ margin-top: 60px; text-align: center; color: #444; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>📊 {repo_name} — Topology Dashboard</h1>
        <p class="subtitle">Generated: {timestamp}</p>

        <div class="stats">
            <div class="stat">
                <div class="stat-value">{module_count}</div>
                <div class="stat-label">Modules</div>
            </div>
            <div class="stat">
                <div class="stat-value">{slice_count}</div>
                <div class="stat-label">Slices</div>
            </div>
            <div class="stat">
                <div class="stat-value" style="color:{health_color}">{health_pct}%</div>
                <div class="stat-label">Avg Health ({health_label})</div>
            </div>
        </div>

        <div class="about">
            <button class="about-toggle" onclick="this.classList.toggle('open'); document.getElementById('about-body').classList.toggle('open');">
                About This Dashboard <span class="chevron">▼</span>
            </button>
            <div id="about-body" class="about-body">
                <h3>Philosophy</h3>
                <p>
                    Code topology treats a codebase as a living structure — not just lines of text,
                    but a system of interconnected modules with measurable structural properties.
                    Like an X-ray for your architecture, topology analysis reveals the hidden forces
                    that make code easy or hard to change: coupling density, complexity hotspots,
                    and boundary violations.
                </p>
                <p>
                    The goal is <strong style="color:#ccc">deterministic, self-validating artifacts</strong>.
                    Same code + same seed = same output. No opinions, no heuristics that drift —
                    just structural facts derived from the AST via tree-sitter parsing.
                </p>

                <h3>What Gets Measured</h3>
                <div class="metric-grid">
                    <div class="metric-item">
                        <strong>Cyclomatic Complexity</strong>
                        <span>Branch points per function. High values signal code that's hard to test and reason about.</span>
                    </div>
                    <div class="metric-item">
                        <strong>Cognitive Complexity</strong>
                        <span>Measures how hard code is to understand, penalizing nesting and broken flow.</span>
                    </div>
                    <div class="metric-item">
                        <strong>Afferent Coupling (Ca)</strong>
                        <span>How many modules depend on this one. High Ca = widely used, risky to change.</span>
                    </div>
                    <div class="metric-item">
                        <strong>Efferent Coupling (Ce)</strong>
                        <span>How many modules this one depends on. High Ce = fragile, breaks when dependencies change.</span>
                    </div>
                </div>

                <h3>Health Score</h3>
                <p>
                    Each module gets a composite health score (0–100%) from five equally-weighted metrics:
                    cyclomatic complexity, cognitive complexity, lines-of-code per function, coupling fan-out,
                    and module size. The color scale runs from
                    <span style="color:#00ff88">green (healthy)</span> through
                    <span style="color:#ddaa33">yellow (warning)</span> to
                    <span style="color:#ff3333">red (critical)</span>.
                </p>

                <h3>How to Use the Visualizations</h3>
                <p>
                    <strong style="color:#ddd">CodeCity</strong> — look for tall, red buildings. Those are your complexity hotspots.
                    Districts group modules by package/slice. Hover any building for a full metric breakdown.
                </p>
                <p>
                    <strong style="color:#ddd">3D Coupling Graph</strong> — thick edges mean high coupling.
                    Clusters of tightly-connected nodes may indicate a bounded context — or a tangled dependency mess.
                    Hover nodes to see their connections.
                </p>
                <p>
                    <strong style="color:#ddd">Package Clusters</strong> — shows how packages relate to each other at a higher level.
                    Isolated clusters are healthy boundaries. Dense interconnections suggest leaky abstractions.
                </p>
                <p>
                    <strong style="color:#ddd">VSA Diagram</strong> — maps feature slices against architectural layers.
                    A well-structured vertical slice should touch exactly the layers it needs — no more, no less.
                </p>
            </div>
        </div>

        <div class="viz-grid">
            <a href="topology-3d.html" class="viz-card">
                <div class="viz-icon">🌐</div>
                <div class="viz-title">3D Coupling Graph</div>
                <div class="viz-desc">Force-directed graph showing module coupling relationships with Martin metrics.</div>
            </a>
            <a href="codecity.html" class="viz-card">
                <div class="viz-icon">🏙️</div>
                <div class="viz-title">CodeCity</div>
                <div class="viz-desc">3D city metaphor where buildings represent modules. Height = complexity, color = health.</div>
            </a>
            <a href="clusters.html" class="viz-card">
                <div class="viz-icon">🔧</div>
                <div class="viz-title">Package Clusters</div>
                <div class="viz-desc">2D force-directed graph of package relationships with coupling strength.</div>
            </a>
            <a href="vsa.html" class="viz-card">
                <div class="viz-icon">🍰</div>
                <div class="viz-title">VSA Diagram</div>
                <div class="viz-desc">Vertical Slice Architecture matrix showing feature slices vs. architectural layers.</div>
            </a>
        </div>

        <footer>
            Generated by Agent Paradise Standards System • APS-V1-0001 Code Topology
        </footer>
    </div>
</body>
</html>"##,
        repo_name = repo_name,
        timestamp = timestamp,
        module_count = module_count,
        slice_count = slice_count,
        health_color = health_color,
        health_pct = (avg_health * 100.0).round() as i32,
        health_label = health_lbl
    )
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
}
