//! Render the Agent Paradise Standards System topology.
//!
//! Run with:
//! ```bash
//! cd standards-experimental/v1/EXP-V1-0001-code-topology
//! cargo run --example render_aps
//! open aps-topology-3d.html
//! ```

use code_topology::{
    CouplingMatrixFile, MartinMetrics, ModuleMetrics, ModuleRecord, OutputFormat, Projector,
    Topology,
};
use code_topology_3d::ForceDirectedProjector;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load from the root .topology/ directory
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let topology_dir = workspace_root.join(".topology");

    println!("🏗️  Agent Paradise Standards System - Topology Visualization");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n📊 Loading topology from: {}", topology_dir.display());

    // Load coupling matrix
    let matrix_path = topology_dir.join("graphs/coupling-matrix.json");
    let matrix_content = fs::read_to_string(&matrix_path)?;
    let matrix_file: CouplingMatrixFile = serde_json::from_str(&matrix_content)?;

    println!(
        "   ✅ Loaded coupling matrix: {} modules",
        matrix_file.modules.len()
    );

    // Load modules for metrics
    let modules_path = topology_dir.join("metrics/modules.json");
    let modules_content = fs::read_to_string(&modules_path)?;

    #[derive(serde::Deserialize)]
    struct ModulesFileLocal {
        modules: Vec<ModuleRecord>,
    }
    let modules_file: ModulesFileLocal = serde_json::from_str(&modules_content)?;

    println!(
        "   ✅ Loaded module metrics: {} modules",
        modules_file.modules.len()
    );

    // Build topology for projector
    let mut topology = Topology::default();
    topology.languages = vec!["rust".to_string()];

    // Convert coupling matrix to internal format
    let positions: Option<HashMap<String, [f64; 3]>> = matrix_file.layout.map(|l| l.positions);

    topology.coupling_matrix = Some(code_topology::CouplingMatrixData {
        modules: matrix_file.modules.clone(),
        values: matrix_file.matrix.clone(),
        positions,
    });

    // Convert module metrics to internal format
    for record in &modules_file.modules {
        topology.modules.push(ModuleMetrics {
            id: record.id.clone(),
            name: record.name.clone(),
            path: PathBuf::from(&record.path),
            languages: record.languages.clone(),
            file_count: record.metrics.file_count,
            function_count: record.metrics.function_count,
            total_cyclomatic: record.metrics.total_cyclomatic,
            avg_cyclomatic: record.metrics.avg_cyclomatic,
            total_cognitive: record.metrics.total_cognitive,
            avg_cognitive: record.metrics.avg_cognitive,
            lines_of_code: record.metrics.lines_of_code,
            martin: MartinMetrics {
                ca: record.metrics.martin.ca,
                ce: record.metrics.martin.ce,
                instability: record.metrics.martin.instability,
                abstractness: record.metrics.martin.abstractness,
                distance_from_main_sequence: record.metrics.martin.distance_from_main_sequence,
            },
        });
    }

    println!("\n🎨 Rendering 3D visualization...");

    // Create projector and render
    let projector = ForceDirectedProjector::new();
    let html = projector.render(&topology, OutputFormat::Html, None)?;

    // Write output
    let output_path = "aps-topology-3d.html";
    fs::write(output_path, &html)?;

    println!("   ✅ Generated: {}", output_path);
    println!("\n🌐 Open in browser:");
    println!("   open {}", output_path);

    // Print architecture insights
    println!("\n📈 APS Architecture Insights:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Find high coupling pairs
    println!("\n🔗 Strongest Coupling Relationships:");
    let mut couplings: Vec<(String, String, f64)> = Vec::new();
    for (i, row) in matrix_file.matrix.iter().enumerate() {
        for (j, &strength) in row.iter().enumerate() {
            if j > i && strength >= 0.5 {
                couplings.push((
                    matrix_file.modules[i].clone(),
                    matrix_file.modules[j].clone(),
                    strength,
                ));
            }
        }
    }
    couplings.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    for (from, to, strength) in couplings.iter().take(5) {
        let bar = "█".repeat((strength * 10.0) as usize);
        println!("   {} ↔ {} : {:.0}% {}", from, to, strength * 100.0, bar);
    }

    // Module complexity
    println!("\n📊 Module Complexity:");
    let mut modules_sorted: Vec<_> = modules_file.modules.iter().collect();
    modules_sorted.sort_by(|a, b| b.metrics.total_cyclomatic.cmp(&a.metrics.total_cyclomatic));
    for m in modules_sorted.iter().take(5) {
        println!(
            "   {} — CC:{} Cog:{} LOC:{}",
            m.name, m.metrics.total_cyclomatic, m.metrics.total_cognitive, m.metrics.lines_of_code
        );
    }

    // Instability analysis
    println!("\n⚖️  Stability Analysis:");
    for m in &modules_file.modules {
        let stability = if m.metrics.martin.instability < 0.3 {
            "🔵 Stable"
        } else if m.metrics.martin.instability > 0.7 {
            "🔴 Unstable"
        } else {
            "🟡 Balanced"
        };
        if m.metrics.martin.distance_from_main_sequence > 0.5 {
            println!(
                "   {} {} (I={:.2}) ⚠️ Zone of Pain",
                stability, m.name, m.metrics.martin.instability
            );
        }
    }

    Ok(())
}
