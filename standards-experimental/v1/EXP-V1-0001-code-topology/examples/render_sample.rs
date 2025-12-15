//! Quick example: Render sample topology as 3D visualization.
//!
//! Run with:
//! ```bash
//! cd standards-experimental/v1/EXP-V1-0001-code-topology
//! cargo run --example render_sample
//! open coupling-3d.html
//! ```

use code_topology::{CouplingMatrixFile, ModulesFile, OutputFormat, Projector, Topology};
use code_topology_3d::ForceDirectedProjector;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sample_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/sample-topology");

    println!("📊 Loading sample topology from: {}", sample_dir.display());

    // Load coupling matrix
    let matrix_path = sample_dir.join("graphs/coupling-matrix.json");
    let matrix_content = fs::read_to_string(&matrix_path)?;
    let matrix_file: CouplingMatrixFile = serde_json::from_str(&matrix_content)?;

    println!("   ✅ Loaded coupling matrix: {} modules", matrix_file.modules.len());

    // Load modules for metrics
    let modules_path = sample_dir.join("metrics/modules.json");
    let modules_content = fs::read_to_string(&modules_path)?;
    let modules_file: ModulesFile = serde_json::from_str(&modules_content)?;

    println!("   ✅ Loaded module metrics: {} modules", modules_file.modules.len());

    // Build topology for projector
    let mut topology = Topology::default();
    topology.languages = vec!["rust".to_string(), "typescript".to_string()];

    // Convert coupling matrix to internal format
    let positions: Option<HashMap<String, [f64; 3]>> = matrix_file.layout.map(|l| l.positions);

    topology.coupling_matrix = Some(code_topology::CouplingMatrixData {
        modules: matrix_file.modules.clone(),
        values: matrix_file.matrix.clone(),
        positions,
    });

    println!("\n🎨 Rendering 3D visualization...");

    // Create projector and render
    let projector = ForceDirectedProjector::new();
    let html = projector.render(&topology, OutputFormat::Html, None)?;

    // Write output
    let output_path = "coupling-3d.html";
    fs::write(output_path, &html)?;

    println!("   ✅ Generated: {}", output_path);
    println!("\n🌐 Open in browser:");
    println!("   open {}", output_path);
    println!("\n📈 Module coupling visualization:");
    println!("   - Drag to rotate");
    println!("   - Scroll to zoom");
    println!("   - Clustered nodes = tightly coupled");

    // Print some stats
    println!("\n📊 Coupling Summary:");
    for (i, module) in matrix_file.modules.iter().enumerate() {
        let max_coupling = matrix_file.matrix[i]
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(j, v)| (matrix_file.modules[j].clone(), *v));

        if let Some((other, strength)) = max_coupling {
            println!("   {} ↔ {} : {:.2}", module, other, strength);
        }
    }

    Ok(())
}

