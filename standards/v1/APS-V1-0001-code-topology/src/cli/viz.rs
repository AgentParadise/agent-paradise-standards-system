//! `viz` command: render HTML visualizations from `.topology/` artifacts.
//!
//! Per ADR-0002, the visualization bodies are feature-gated. The `3d`
//! visualization requires the `FD01` feature; the dashboard visualizations
//! (`codecity`, `clusters`, `vsa`, and the `index` page) require the
//! `VZ01` feature. With a needed feature disabled the command returns
//! exit code 5 with a message naming the missing feature.

use super::health::{
    calculate_health, detect_layer, get_slice_from_id, health_label, health_to_color,
};

/// Generate visualization from topology artifacts.
pub(super) fn topology_viz(path: &str, viz_type: &str, output: Option<&str>, verbose: bool) -> i32 {
    use crate::{
        CouplingMatrixData, CouplingMatrixFile, MartinMetrics, ModuleMetrics, ModuleRecord,
        Topology,
    };
    use std::collections::HashMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    let topology_path = Path::new(path);
    let modules_path = topology_path.join("metrics/modules.json");
    let coupling_path = topology_path.join("graphs/coupling-matrix.json");

    // Check for required artifacts
    if !modules_path.exists() {
        eprintln!("Error: No modules.json found at {}", modules_path.display());
        eprintln!("Run 'apss-dev run topology analyze' first.");
        return 1;
    }

    if !coupling_path.exists() {
        eprintln!(
            "Error: No coupling-matrix.json found at {}",
            coupling_path.display()
        );
        eprintln!("Run 'apss-dev run topology analyze' first.");
        return 1;
    }

    if verbose {
        println!("Loading topology from: {}", topology_path.display());
    }

    // Load VSA config if present (look in repo root, i.e. parent of .topology/)
    let repo_root = topology_path.parent().unwrap_or(Path::new("."));
    let vsa_config = match super::vsa_config::VsaConfig::load(repo_root) {
        Ok(Some(config)) => {
            if verbose {
                println!(
                    "  Found vsa.yaml (v{})  -  root: {}",
                    config.version,
                    config.normalized_root()
                );
                if let Some(names) = config.contexts.as_ref() {
                    println!(
                        "  Contexts: {}",
                        names.keys().cloned().collect::<Vec<_>>().join(", ")
                    );
                }
            }
            Some(config)
        }
        Ok(None) => {
            if verbose {
                println!("  No vsa.yaml found (VSA viz will show placeholder)");
            }
            None
        }
        Err(e) => {
            eprintln!("Warning: {e}");
            eprintln!("VSA config load/validation failed; VSA viz will show placeholder.");
            None
        }
    };

    // Load coupling matrix
    let coupling_content = match fs::read_to_string(&coupling_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading coupling matrix: {e}");
            return 1;
        }
    };

    let matrix_file: CouplingMatrixFile = match serde_json::from_str(&coupling_content) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error parsing coupling matrix: {e}");
            return 1;
        }
    };

    if verbose {
        println!(
            "  Loaded {} modules from coupling matrix",
            matrix_file.modules.len()
        );
    }

    // Load module metrics
    let modules_content = match fs::read_to_string(&modules_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading modules: {e}");
            return 1;
        }
    };

    #[derive(serde::Deserialize)]
    struct ModulesFile {
        modules: Vec<ModuleRecord>,
    }

    let modules_file: ModulesFile = match serde_json::from_str(&modules_content) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error parsing modules: {e}");
            return 1;
        }
    };

    if verbose {
        println!("  Loaded {} module metrics", modules_file.modules.len());
    }

    // Build topology for 3D viz (used by 3d type)
    let mut topology = Topology {
        languages: vec!["rust".to_string()],
        ..Default::default()
    };

    // Convert coupling matrix to internal format
    let positions = matrix_file.layout.as_ref().map(|l| l.positions.clone());
    topology.coupling_matrix = Some(CouplingMatrixData {
        modules: matrix_file.modules.clone(),
        values: matrix_file.matrix.clone(),
        positions,
    });

    // Build enriched module data for visualizations. The typed row is
    // defined once in viz_dashboard::model (when VZ01 is enabled) so
    // downstream dashboard vizzes can consume the same struct. When VZ01
    // is off (only the 3d viz is available) we fall back to a
    // structurally identical local definition so this module still
    // compiles without the dashboard feature.
    #[cfg(feature = "VZ01")]
    use crate::substandards::viz_dashboard::VizModule;

    #[cfg(not(feature = "VZ01"))]
    #[derive(serde::Serialize)]
    struct VizModule {
        id: String,
        name: String,
        path: String,
        slice: String,
        layer: String,
        function_count: u32,
        total_cyclomatic: u32,
        total_cognitive: u32,
        lines_of_code: u32,
        ca: u32,
        ce: u32,
        health: f64,
        color: String,
        health_label: String,
    }

    let mut viz_modules: Vec<VizModule> = Vec::new();

    for record in &modules_file.modules {
        let health = calculate_health(
            record.metrics.function_count,
            record.metrics.total_cyclomatic,
            record.metrics.total_cognitive,
            record.metrics.lines_of_code,
            record.metrics.martin.ca,
            record.metrics.martin.ce,
        );

        viz_modules.push(VizModule {
            id: record.id.clone(),
            name: record.name.clone(),
            path: record.path.clone(),
            slice: get_slice_from_id(&record.id),
            layer: detect_layer(&record.path).to_string(),
            function_count: record.metrics.function_count,
            total_cyclomatic: record.metrics.total_cyclomatic,
            total_cognitive: record.metrics.total_cognitive,
            lines_of_code: record.metrics.lines_of_code,
            ca: record.metrics.martin.ca,
            ce: record.metrics.martin.ce,
            health,
            color: health_to_color(health).to_string(),
            health_label: health_label(health).to_string(),
        });

        // Also add to topology for 3D viz
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

    // Determine which visualizations to generate
    let viz_types: Vec<&str> = match viz_type {
        "all" => vec!["3d", "codecity", "clusters", "vsa"],
        t => vec![t],
    };

    // Create viz output directory if generating multiple
    let viz_dir = topology_path.join("viz");
    if viz_type == "all" {
        if let Err(e) = fs::create_dir_all(&viz_dir) {
            eprintln!("Error creating viz directory: {e}");
            return 1;
        }
    }

    let mut generated_files: Vec<String> = Vec::new();

    for vtype in &viz_types {
        let (html_content, output_path): (String, PathBuf) = match *vtype {
            "3d" => {
                #[cfg(feature = "FD01")]
                {
                    use crate::OutputFormat;
                    use crate::Projector;
                    use crate::substandards::viz_3d::ForceDirectedProjector;
                    let projector = ForceDirectedProjector::new();
                    if verbose {
                        println!("Rendering 3D force-directed visualization...");
                    }
                    match projector.render(&topology, OutputFormat::Html, None) {
                        Ok(html_bytes) => {
                            let html = String::from_utf8_lossy(&html_bytes).to_string();
                            let out = if viz_type == "all" {
                                viz_dir.join("topology-3d.html")
                            } else {
                                PathBuf::from(output.unwrap_or("topology-3d.html"))
                            };
                            (html, out)
                        }
                        Err(e) => {
                            eprintln!("Error rendering 3D visualization: {}", e.message);
                            return 1;
                        }
                    }
                }
                #[cfg(not(feature = "FD01"))]
                {
                    eprintln!(
                        "Error: the '3d' visualization requires the 'FD01' feature, which is not enabled in this build."
                    );
                    return 5;
                }
            }
            "codecity" => {
                #[cfg(feature = "VZ01")]
                {
                    if verbose {
                        println!("Rendering CodeCity visualization...");
                    }
                    let modules_json =
                        serde_json::to_string_pretty(&viz_modules).unwrap_or_default();
                    let coupling_json =
                        serde_json::to_string_pretty(&matrix_file).unwrap_or_default();
                    let html = crate::substandards::viz_dashboard::codecity::generate(
                        &modules_json,
                        &coupling_json,
                    );
                    let out = if viz_type == "all" {
                        viz_dir.join("codecity.html")
                    } else {
                        PathBuf::from(output.unwrap_or("codecity.html"))
                    };
                    (html, out)
                }
                #[cfg(not(feature = "VZ01"))]
                {
                    eprintln!(
                        "Error: the 'codecity' visualization requires the 'VZ01' feature, which is not enabled in this build."
                    );
                    return 5;
                }
            }
            "clusters" => {
                #[cfg(feature = "VZ01")]
                {
                    if verbose {
                        println!("Rendering Package Clusters visualization...");
                    }
                    let modules_json =
                        serde_json::to_string_pretty(&viz_modules).unwrap_or_default();
                    let coupling_json =
                        serde_json::to_string_pretty(&matrix_file).unwrap_or_default();
                    let html = crate::substandards::viz_dashboard::clusters::generate(
                        &modules_json,
                        &coupling_json,
                    );
                    let out = if viz_type == "all" {
                        viz_dir.join("clusters.html")
                    } else {
                        PathBuf::from(output.unwrap_or("clusters.html"))
                    };
                    (html, out)
                }
                #[cfg(not(feature = "VZ01"))]
                {
                    eprintln!(
                        "Error: the 'clusters' visualization requires the 'VZ01' feature, which is not enabled in this build."
                    );
                    return 5;
                }
            }
            "vsa" => {
                #[cfg(feature = "VZ01")]
                {
                    if verbose {
                        println!("Rendering VSA diagram...");
                    }
                    let out = if viz_type == "all" {
                        viz_dir.join("vsa.html")
                    } else {
                        PathBuf::from(output.unwrap_or("vsa.html"))
                    };

                    let html = if let Some(ref vsa_cfg) = vsa_config {
                        // Filter to only modules under the VSA root and fix slice names
                        let vsa_modules: Vec<serde_json::Value> = viz_modules
                            .iter()
                            .filter_map(|m| {
                                let path = &m.path;
                                let id = &m.id;
                                // Check if module is under the VSA root
                                if !vsa_cfg.contains_path(path) && !vsa_cfg.contains_path(id) {
                                    return None;
                                }
                                // Extract context name as the slice
                                let context = vsa_cfg
                                    .extract_context(path)
                                    .or_else(|| vsa_cfg.extract_context(id))?;
                                // If v1 config has explicit contexts, only include listed ones
                                if !vsa_cfg.is_context_allowed(&context) {
                                    return None;
                                }
                                // Re-serialize with the correct slice and layer names
                                let mut val = serde_json::to_value(m).ok()?;
                                val["slice"] = serde_json::Value::String(context);
                                // Override layer from directory structure instead of keyword matching
                                if let Some(layer) = vsa_cfg
                                    .extract_layer(path)
                                    .or_else(|| vsa_cfg.extract_layer(id))
                                {
                                    val["layer"] = serde_json::Value::String(layer);
                                }
                                Some(val)
                            })
                            .collect();

                        if verbose {
                            println!(
                                "  VSA: {} of {} modules matched config",
                                vsa_modules.len(),
                                viz_modules.len()
                            );
                        }
                        let modules_json =
                            serde_json::to_string_pretty(&vsa_modules).unwrap_or_default();
                        crate::substandards::viz_dashboard::vsa::generate(&modules_json)
                    } else {
                        // No vsa.yaml  -  render placeholder
                        crate::substandards::viz_dashboard::vsa::generate_placeholder()
                    };

                    (html, out)
                }
                #[cfg(not(feature = "VZ01"))]
                {
                    eprintln!(
                        "Error: the 'vsa' visualization requires the 'VZ01' feature, which is not enabled in this build."
                    );
                    return 5;
                }
            }
            unknown => {
                eprintln!("Error: Unknown visualization type '{unknown}'");
                eprintln!("Available types: 3d, codecity, clusters, vsa, all");
                return 1;
            }
        };

        if let Err(e) = fs::write(&output_path, &html_content) {
            eprintln!("Error writing {}: {e}", output_path.display());
            return 1;
        }
        generated_files.push(output_path.display().to_string());
    }

    // Generate index if --all
    if viz_type == "all" {
        #[cfg(feature = "VZ01")]
        {
            if verbose {
                println!("Generating index...");
            }

            // Calculate summary stats
            let total_modules = viz_modules.len();
            let mut slices: HashMap<String, u32> = HashMap::new();
            let mut total_health = 0.0;
            for m in &viz_modules {
                *slices.entry(m.slice.clone()).or_insert(0) += 1;
                total_health += m.health;
            }
            let avg_health = if total_modules > 0 {
                total_health / total_modules as f64
            } else {
                0.0
            };

            // Derive repo name from topology path or current directory
            let repo_name = topology_path
                .canonicalize()
                .ok()
                .and_then(|p| {
                    // Go up from .topology to the repo root
                    let repo_root = if p.ends_with(".topology") || p.ends_with(".topology/") {
                        p.parent()
                    } else {
                        Some(p.as_path())
                    };
                    repo_root
                        .and_then(|r| r.file_name())
                        .map(|n| n.to_string_lossy().to_string())
                })
                .unwrap_or_else(|| "Project".to_string());

            let index_html = crate::substandards::viz_dashboard::index::generate(
                &repo_name,
                total_modules,
                slices.len(),
                avg_health,
            );
            let index_path = viz_dir.join("index.html");
            if let Err(e) = fs::write(&index_path, &index_html) {
                eprintln!("Error writing index: {e}");
                return 1;
            }
            generated_files.push(index_path.display().to_string());
        }
        #[cfg(not(feature = "VZ01"))]
        {
            eprintln!(
                "Error: the dashboard index requires the 'VZ01' feature, which is not enabled in this build."
            );
            return 5;
        }
    }

    // Print results
    println!("✓ Generated visualizations:");
    for file in &generated_files {
        println!("  {file}");
    }
    // Auto-open in browser
    let open_path = if viz_type == "all" {
        viz_dir.join("index.html")
    } else {
        PathBuf::from(generated_files.first().unwrap_or(&String::new()))
    };

    println!();
    println!("Opening in browser: {}", open_path.display());

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&open_path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&open_path)
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", &open_path.display().to_string()])
            .spawn();
    }

    0
}
