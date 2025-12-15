//! Analyze a Rust project and generate .topology/ artifacts.
//!
//! Run on APS itself:
//! ```bash
//! cd standards-experimental/v1/EXP-V1-0001-code-topology
//! cargo run --example analyze_rust -- --path ../../..
//! ```

use code_topology_rust_adapter::{RustAdapter, RustAdapterConfig};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse args
    let args: Vec<String> = std::env::args().collect();
    let project_path = if args.len() > 2 && args[1] == "--path" {
        Path::new(&args[2])
    } else {
        // Default: analyze the workspace root
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
    };

    println!("🔍 Rust Topology Analyzer");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n📂 Analyzing: {}", project_path.display());

    // Create adapter
    let config = RustAdapterConfig {
        include_tests: false,
        follow_workspace: true,
        exclude_paths: vec![
            "target".into(),
            ".git".into(),
            "lib/agentic-primitives".into(),
        ],
    };
    let adapter = RustAdapter::with_config(config);

    // Analyze
    let result = adapter.analyze(project_path)?;

    println!("\n📊 Analysis Complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━");
    println!("   Project: {}", result.name);
    println!("   Modules: {}", result.modules.len());
    println!("   Functions: {}", result.functions.len());

    // Print complexity stats
    if !result.functions.is_empty() {
        let total_cc: u32 = result.functions.iter().map(|f| f.cyclomatic).sum();
        let max_cc = result
            .functions
            .iter()
            .map(|f| f.cyclomatic)
            .max()
            .unwrap_or(0);
        let avg_cc = total_cc as f64 / result.functions.len() as f64;

        println!("\n📈 Complexity Metrics:");
        println!("   Total CC: {}", total_cc);
        println!("   Avg CC: {:.2}", avg_cc);
        println!("   Max CC: {}", max_cc);

        // Find hotspots
        let mut hotspots: Vec<_> = result
            .functions
            .iter()
            .filter(|f| f.cyclomatic > 5)
            .collect();
        hotspots.sort_by(|a, b| b.cyclomatic.cmp(&a.cyclomatic));

        if !hotspots.is_empty() {
            println!("\n🔥 Complexity Hotspots (CC > 5):");
            for func in hotspots.iter().take(10) {
                let status = if func.cyclomatic > 10 { "⚠️" } else { "  " };
                println!(
                    "   {} {} — CC:{} Cog:{}",
                    status, func.id, func.cyclomatic, func.cognitive
                );
            }
        }
    }

    // Print module stats
    println!("\n📦 Module Analysis:");
    let mut sorted_modules = result.modules.clone();
    sorted_modules.sort_by(|a, b| b.function_count.cmp(&a.function_count));

    for module in sorted_modules.iter().take(10) {
        let ca_ce = format!("Ca:{} Ce:{}", module.ca, module.ce);
        let instability = if module.ca + module.ce > 0 {
            module.ce as f64 / (module.ca + module.ce) as f64
        } else {
            0.5
        };
        let status = if instability < 0.3 {
            "🔵"
        } else if instability > 0.7 {
            "🔴"
        } else {
            "🟡"
        };
        println!(
            "   {} {} — {} funcs, {} (I={:.2})",
            status, module.id, module.function_count, ca_ce, instability
        );
    }

    // Write artifacts
    let output_dir = project_path.join(".topology-generated");
    println!("\n💾 Writing artifacts to: {}", output_dir.display());
    result.write_artifacts(&output_dir)?;
    println!("   ✅ manifest.toml");
    println!("   ✅ metrics/functions.json");
    println!("   ✅ metrics/modules.json");
    println!("   ✅ graphs/coupling-matrix.json");

    println!("\n🎉 Done! You can now visualize with:");
    println!("   cargo run --example render_aps");

    Ok(())
}
