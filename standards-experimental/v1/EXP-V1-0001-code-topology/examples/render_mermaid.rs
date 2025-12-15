//! Render the Agent Paradise Standards System as Mermaid diagrams.
//!
//! Run with:
//! ```bash
//! cd standards-experimental/v1/EXP-V1-0001-code-topology
//! cargo run --example render_mermaid
//! ```

use code_topology::{
    CouplingMatrixFile, MartinMetrics, ModuleMetrics, ModuleRecord, OutputFormat, Projector,
    Topology,
};
use code_topology_mermaid::{DiagramStyle, MermaidConfig, MermaidProjector};
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

    println!("🧜 Mermaid Diagram Generator");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\n📊 Loading topology from: {}", topology_dir.display());

    // Load coupling matrix
    let matrix_path = topology_dir.join("graphs/coupling-matrix.json");
    let matrix_content = fs::read_to_string(&matrix_path)?;
    let matrix_file: CouplingMatrixFile = serde_json::from_str(&matrix_content)?;

    // Load modules for metrics
    let modules_path = topology_dir.join("metrics/modules.json");
    let modules_content = fs::read_to_string(&modules_path)?;

    #[derive(serde::Deserialize)]
    struct ModulesFileLocal {
        modules: Vec<ModuleRecord>,
    }
    let modules_file: ModulesFileLocal = serde_json::from_str(&modules_content)?;

    // Build topology
    let mut topology = Topology::default();
    topology.languages = vec!["rust".to_string()];

    let positions: Option<HashMap<String, [f64; 3]>> = matrix_file.layout.map(|l| l.positions);
    topology.coupling_matrix = Some(code_topology::CouplingMatrixData {
        modules: matrix_file.modules.clone(),
        values: matrix_file.matrix.clone(),
        positions,
    });

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

    println!("   ✅ Loaded {} modules", topology.modules.len());

    // Render as flowchart
    println!("\n📊 Flowchart Diagram:");
    println!("━━━━━━━━━━━━━━━━━━━━━");
    let flowchart_projector = MermaidProjector::new();
    let flowchart = flowchart_projector.render(&topology, OutputFormat::Mermaid, None)?;
    println!("\n```mermaid");
    println!("{}", String::from_utf8_lossy(&flowchart));
    println!("```");

    // Render as C4 Context
    println!("\n🏛️  C4 Context Diagram:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━");
    let c4_config = MermaidConfig {
        style: DiagramStyle::C4Context,
        ..Default::default()
    };
    let c4_projector = MermaidProjector::with_config(c4_config);
    let c4 = c4_projector.render(&topology, OutputFormat::Mermaid, None)?;
    println!("\n```mermaid");
    println!("{}", String::from_utf8_lossy(&c4));
    println!("```");

    // Render as C4 Container
    println!("\n📦 C4 Container Diagram:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━");
    let container_config = MermaidConfig {
        style: DiagramStyle::C4Container,
        ..Default::default()
    };
    let container_projector = MermaidProjector::with_config(container_config);
    let container = container_projector.render(&topology, OutputFormat::Mermaid, None)?;
    println!("\n```mermaid");
    println!("{}", String::from_utf8_lossy(&container));
    println!("```");

    // Save to file
    let output_path = "ARCHITECTURE.md";
    let mut content = String::new();
    content.push_str("# Agent Paradise Standards System — Architecture\n\n");
    content.push_str("*Auto-generated from `.topology/` artifacts*\n\n");
    content.push_str("## Module Dependency Flowchart\n\n");
    content.push_str("```mermaid\n");
    content.push_str(&String::from_utf8_lossy(&flowchart));
    content.push_str("\n```\n\n");
    content.push_str("## C4 Context Diagram\n\n");
    content.push_str("```mermaid\n");
    content.push_str(&String::from_utf8_lossy(&c4));
    content.push_str("\n```\n\n");
    content.push_str("## C4 Container Diagram\n\n");
    content.push_str("```mermaid\n");
    content.push_str(&String::from_utf8_lossy(&container));
    content.push_str("\n```\n");

    fs::write(output_path, &content)?;
    println!("\n✅ Saved to: {}", output_path);

    Ok(())
}
