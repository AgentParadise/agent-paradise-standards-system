//! Shared health-score and layer-detection helpers used by visualizations.

/// Calculate health score for a module (0.0 to 1.0)
pub(crate) fn calculate_health(
    function_count: u32,
    total_cyclomatic: u32,
    total_cognitive: u32,
    lines_of_code: u32,
    ca: u32,
    ce: u32,
) -> f64 {
    let mut scores = Vec::new();

    let func_count = function_count.max(1) as f64;

    // 1. Complexity per function (ideal: 3-8, bad: >15)
    let avg_cc = total_cyclomatic as f64 / func_count;
    let cc_score = if avg_cc > 5.0 {
        (1.0 - (avg_cc - 5.0) / 15.0).max(0.0)
    } else {
        1.0
    };
    scores.push(cc_score);

    // 2. Cognitive load per function (ideal: <10, bad: >30)
    let avg_cog = total_cognitive as f64 / func_count;
    let cog_score = (1.0 - avg_cog / 30.0).max(0.0);
    scores.push(cog_score);

    // 3. LOC per function (ideal: 10-50, bad: >100)
    let loc_per_func = lines_of_code as f64 / func_count;
    let loc_score = if loc_per_func > 50.0 {
        (1.0 - (loc_per_func - 50.0) / 100.0).max(0.0)
    } else {
        1.0
    };
    scores.push(loc_score);

    // 4. Coupling balance (isolated or over-coupled is bad)
    let total_coupling = ca + ce;
    let coupling_score = if total_coupling == 0 {
        0.6 // Isolated
    } else if total_coupling > 20 {
        (1.0 - (total_coupling as f64 - 10.0) / 30.0).max(0.2)
    } else {
        1.0
    };
    scores.push(coupling_score);

    // 5. Module size (ideal: 5-30 functions)
    let size_score = if function_count < 2 {
        0.5
    } else if function_count > 50 {
        (1.0 - (function_count as f64 - 30.0) / 70.0).max(0.3)
    } else {
        1.0
    };
    scores.push(size_score);

    scores.iter().sum::<f64>() / scores.len() as f64
}

/// Convert health score (0.0-1.0) to hex color
pub(crate) fn health_to_color(health: f64) -> &'static str {
    match health {
        h if h >= 0.80 => "#00ff88", // Excellent
        h if h >= 0.65 => "#44dd77", // Good
        h if h >= 0.50 => "#88cc55", // OK
        h if h >= 0.35 => "#ddaa33", // Warning
        h if h >= 0.20 => "#ff7744", // Poor
        _ => "#ff3333",              // Critical
    }
}

/// Get health label from score
pub(crate) fn health_label(health: f64) -> &'static str {
    match health {
        h if h >= 0.80 => "Excellent",
        h if h >= 0.65 => "Good",
        h if h >= 0.50 => "OK",
        h if h >= 0.35 => "Warning",
        h if h >= 0.20 => "Poor",
        _ => "Critical",
    }
}

/// Detect architectural layer from module path
pub(crate) fn detect_layer(module_path: &str) -> &'static str {
    let path_lower = module_path.to_lowercase();

    // Check patterns in order of specificity - includes Rust patterns
    let patterns: [(&str, &[&str]); 6] = [
        // Entry points / handlers
        (
            "handlers",
            &[
                "handler",
                "controller",
                "api",
                "routes",
                "endpoint",
                "view",
                "main",
                "cli",
                "bin",
                "cmd",
            ],
        ),
        // Business logic
        (
            "services",
            &[
                "service",
                "usecase",
                "application",
                "interactor",
                "core",
                "engine",
                "processor",
                "worker",
            ],
        ),
        // Domain models and types
        (
            "models",
            &[
                "model", "entity", "domain", "schema", "types", "struct", "metadata", "config",
            ],
        ),
        // Data access
        (
            "data",
            &[
                "repository",
                "repo",
                "data",
                "store",
                "db",
                "persistence",
                "storage",
                "discovery",
            ],
        ),
        // Utilities and helpers
        (
            "utils",
            &[
                "util", "helper", "common", "shared", "lib", "support", "tools", "ext",
            ],
        ),
        // Adapters and integrations (Rust-specific)
        (
            "adapters",
            &[
                "adapter",
                "grammars",
                "queries",
                "parser",
                "lexer",
                "projector",
                "renderer",
                "visitor",
            ],
        ),
    ];

    for (layer, keywords) in patterns {
        for keyword in keywords.iter() {
            if path_lower.contains(keyword) {
                return layer;
            }
        }
    }

    // Fallback: Check Rust directory patterns
    if path_lower.contains("examples") {
        return "examples";
    }
    if path_lower.contains("tests") || path_lower.contains("test_") {
        return "tests";
    }
    if path_lower.contains("src") && !path_lower.contains("adapter") {
        return "core";
    }

    "other"
}

/// Get slice (top-level package) from module ID
/// For Rust: crates::foo -> "crates::foo", standards-experimental::v1::NAME -> "NAME"
pub(crate) fn get_slice_from_id(module_id: &str) -> String {
    // Handle Rust-style paths with ::
    if module_id.contains("::") {
        let parts: Vec<&str> = module_id.split("::").collect();

        // For standards-experimental, use the standard name as slice
        if parts.len() >= 3 && parts[0] == "standards-experimental" {
            return parts[2].to_string(); // e.g., "EXP-V1-0001-code-topology"
        }

        // For crates, use crate name
        if parts.len() >= 2 && parts[0] == "crates" {
            return parts[1].to_string(); // e.g., "apss-core"
        }

        // Default: first two segments
        if parts.len() >= 2 {
            return format!("{}::{}", parts[0], parts[1]);
        }
        return parts.first().unwrap_or(&module_id).to_string();
    }

    // Handle path-like IDs (containing '/')  -  split on '/' to avoid breaking
    // Next.js catch-all routes like [[...slug]] where '.' is literal.
    let separator = if module_id.contains('/') { "/" } else { "." };
    let parts: Vec<&str> = module_id.split(separator).collect();
    if parts.len() >= 2 {
        format!("{}{}{}", parts[0], separator, parts[1])
    } else {
        parts.first().unwrap_or(&module_id).to_string()
    }
}
