//! `analyze` command: walk a codebase and produce `.topology/` artifacts.

/// Analyze a codebase and generate .topology/ artifacts.
pub(super) fn topology_analyze(
    path: &str,
    output: &str,
    language_filter: Option<&str>,
    _repo_root: &std::path::Path,
    verbose: bool,
) -> i32 {
    use crate::LanguageAdapter;
    use crate::adapter::grammars::{PythonGrammar, RustGrammar, TsxGrammar, TypeScriptGrammar};
    use crate::adapter::{GrammarRegistry, TreeSitterAdapter};
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use walkdir::WalkDir;

    let project_path = Path::new(path);
    let output_path = Path::new(output);

    if verbose {
        println!("Analyzing: {}", project_path.display());
        println!("Output:    {}", output_path.display());
        if let Some(lang) = language_filter {
            println!("Language:  {lang}");
        }
    }

    // Create grammar registry with available grammars
    let mut registry = GrammarRegistry::new();
    registry.register(Box::new(RustGrammar::new()));
    registry.register(Box::new(PythonGrammar::new()));
    registry.register(Box::new(TypeScriptGrammar::new()));
    registry.register(Box::new(TsxGrammar::new()));

    let adapter = TreeSitterAdapter::new(registry);

    // Collect files to analyze
    let mut files_by_lang: HashMap<String, Vec<std::path::PathBuf>> = HashMap::new();

    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            // Allow the root entry even if it's "."
            if e.depth() == 0 {
                return true;
            }
            // Skip hidden dirs, test dirs, and common non-source dirs
            !name.starts_with('.')
                && name != "target"
                && name != "node_modules"
                && name != "__pycache__"
                && name != "tests"
                && !name.ends_with("_test.rs")
                && !name.starts_with("test_")
                && !name.ends_with("_test.py")
                && name != "venv"
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let file_path = entry.path();

        // Check if we have a grammar for this file
        if let Some(grammar) = adapter.registry().get_for_path(file_path) {
            let lang = grammar.language_id();

            // Apply language filter if specified
            if let Some(filter) = language_filter {
                if lang != filter {
                    continue;
                }
            }

            files_by_lang
                .entry(lang.to_string())
                .or_default()
                .push(file_path.to_path_buf());
        }
    }

    if files_by_lang.is_empty() {
        let msg = if let Some(lang) = language_filter {
            format!("No {lang} files found in {}", project_path.display())
        } else {
            format!(
                "No supported source files found in {}",
                project_path.display()
            )
        };
        eprintln!("Error: {msg}");
        eprintln!("Supported: .rs (Rust), .py/.pyi (Python)");
        return 1;
    }

    // Print summary
    let total_files: usize = files_by_lang.values().map(|v| v.len()).sum();
    println!("Found {total_files} source file(s):");
    for (lang, files) in &files_by_lang {
        println!("  {lang}: {} files", files.len());
    }

    // Analyze all files - extract functions, imports, types, AND calls
    let mut all_functions = Vec::new();
    let mut all_imports: Vec<crate::ImportInfo> = Vec::new();
    let mut all_types: Vec<crate::TypeInfo> = Vec::new();
    let mut all_calls: Vec<crate::CallInfo> = Vec::new();
    let mut errors = 0;

    for (lang, files) in &files_by_lang {
        if verbose {
            println!("Analyzing {lang} files...");
        }

        for file_path in files {
            let source = match fs::read_to_string(file_path) {
                Ok(s) => s,
                Err(e) => {
                    if verbose {
                        eprintln!("  Warning: Could not read {}: {e}", file_path.display());
                    }
                    errors += 1;
                    continue;
                }
            };

            // Extract imports for coupling analysis
            if let Ok(imports) = adapter.extract_imports(&source, file_path) {
                all_imports.extend(imports);
            }

            // Extract types for abstractness calculation
            if let Ok(types) = adapter.extract_types(&source, file_path) {
                all_types.extend(types);
            }

            // Extract calls for call coupling analysis
            if let Ok(calls) = adapter.extract_calls(&source, file_path) {
                all_calls.extend(calls);
            }

            match adapter.analyze_functions(&source, file_path) {
                Ok(functions) => all_functions.extend(functions),
                Err(e) => {
                    if verbose {
                        eprintln!("  Warning: Could not parse {}: {e}", file_path.display());
                    }
                    errors += 1;
                }
            }
        }
    }

    println!(
        "✓ Analyzed {} functions ({}  warnings)",
        all_functions.len(),
        errors
    );

    // Write artifacts
    if let Err(e) = write_topology_artifacts(
        output_path,
        &all_functions,
        &all_imports,
        &all_types,
        &all_calls,
        &files_by_lang,
    ) {
        eprintln!("Error writing artifacts: {e}");
        return 1;
    }

    println!("✓ Wrote artifacts to {}", output_path.display());
    0
}

/// Write topology artifacts to disk.
fn write_topology_artifacts(
    output_path: &std::path::Path,
    functions: &[(crate::FunctionInfo, crate::FunctionMetrics)],
    imports: &[crate::ImportInfo],
    types: &[crate::TypeInfo],
    calls: &[crate::CallInfo],
    files_by_lang: &std::collections::HashMap<String, Vec<std::path::PathBuf>>,
) -> std::io::Result<()> {
    use std::collections::{HashMap, HashSet};
    use std::fs;

    // Create directories
    fs::create_dir_all(output_path)?;
    fs::create_dir_all(output_path.join("metrics"))?;
    fs::create_dir_all(output_path.join("graphs"))?;

    // Deduplicate functions  -  tree-sitter queries can match the same function
    // multiple times (e.g. a class method matches both the function pattern
    // and the method-in-class pattern).  Keep the first occurrence per
    // (file_path, start_line) pair.
    let mut seen_functions: HashSet<(std::path::PathBuf, u32)> = HashSet::new();
    let functions: Vec<_> = functions
        .iter()
        .filter(|(func, _)| seen_functions.insert((func.file_path.clone(), func.start_line)))
        .collect();

    // Group functions by module
    let mut modules: HashMap<String, Vec<&&(crate::FunctionInfo, crate::FunctionMetrics)>> =
        HashMap::new();
    for func_with_metrics in &functions {
        modules
            .entry(func_with_metrics.0.module.clone())
            .or_default()
            .push(func_with_metrics);
    }

    // Group types by module for abstractness calculation
    // Map module -> (abstract_count, total_count)
    let mut module_types: HashMap<String, (u32, u32)> = HashMap::new();
    for type_info in types {
        let entry = module_types
            .entry(type_info.module.clone())
            .or_insert((0, 0));
        entry.1 += 1; // total count
        if type_info.is_abstract {
            entry.0 += 1; // abstract count
        }
    }

    // Build dependency graph from imports
    // Map module -> set of modules it depends on (efferent coupling)
    let mut efferent: HashMap<String, HashSet<String>> = HashMap::new();
    // Map module -> set of modules that depend on it (afferent coupling)
    let mut afferent: HashMap<String, HashSet<String>> = HashMap::new();
    // Map (from, to) -> list of imports with full details (for weighted coupling calculation)
    let mut import_edges: HashMap<(String, String), Vec<crate::ImportInfo>> = HashMap::new();

    // Initialize all modules
    for module in modules.keys() {
        efferent.entry(module.clone()).or_default();
        afferent.entry(module.clone()).or_default();
    }

    // Process imports to build coupling with weighted scoring
    for import in imports {
        let from_module = &import.from_module;

        // Skip external imports
        if import.is_external {
            continue;
        }

        // Try to resolve the import path to a known module
        let import_path = &import.import_path;

        // Find which module this import refers to
        for to_module in modules.keys() {
            // Check if the import path matches or is contained in the module
            let matches = import_path.contains(to_module.split("::").last().unwrap_or(to_module))
                || to_module.contains(import_path)
                || import_path.split("::").any(|part| to_module.contains(part));

            if matches && from_module != to_module {
                // from_module depends on to_module
                efferent
                    .entry(from_module.clone())
                    .or_default()
                    .insert(to_module.clone());
                // to_module is depended upon by from_module
                afferent
                    .entry(to_module.clone())
                    .or_default()
                    .insert(from_module.clone());
                // Track the full import for weighted coupling calculation
                import_edges
                    .entry((from_module.clone(), to_module.clone()))
                    .or_default()
                    .push(import.clone());
            }
        }
    }

    // Write manifest.toml
    let mut languages: Vec<&str> = files_by_lang.keys().map(|s| s.as_str()).collect();
    languages.sort();
    let total_files: usize = files_by_lang.values().map(|v| v.len()).sum();
    let total_deps: usize = efferent.values().map(|s| s.len()).sum();
    let manifest = format!(
        r#"[topology]
version = "0.1.0"
generated_at = "{}"
generator = "aps-cli"
generator_version = "0.1.0"

[analysis]
root = "."
languages = {:?}
total_files = {}
total_functions = {}
total_modules = {}
total_dependencies = {}
"#,
        chrono_lite_now(),
        languages,
        total_files,
        functions.len(),
        modules.len(),
        total_deps
    );
    fs::write(output_path.join("manifest.toml"), manifest)?;

    // Write functions.json
    let functions_json = serde_json::json!({
        "schema_version": "1.0.0",
        "functions": functions.iter().map(|&(func, metrics)| {
            serde_json::json!({
                "id": func.qualified_name,
                "name": func.name,
                "module": func.module,
                "file": func.file_path.to_string_lossy(),
                "line": func.start_line,
                "metrics": {
                    "cyclomatic": metrics.cyclomatic_complexity,
                    "cognitive": metrics.cognitive_complexity,
                    "halstead": {
                        "vocabulary": metrics.halstead.vocabulary,
                        "length": metrics.halstead.length,
                        "volume": metrics.halstead.volume,
                        "difficulty": metrics.halstead.difficulty,
                        "effort": metrics.halstead.effort
                    },
                    "loc": metrics.total_lines
                }
            })
        }).collect::<Vec<_>>()
    });
    fs::write(
        output_path.join("metrics/functions.json"),
        serde_json::to_string_pretty(&functions_json).unwrap(),
    )?;

    // Write modules.json with real Martin metrics
    let modules_json = serde_json::json!({
        "schema_version": "1.0.0",
        "modules": modules.iter().map(|(module_id, funcs)| {
            let total_cc: u32 = funcs.iter().map(|&&(_, m)| m.cyclomatic_complexity).sum();
            let total_cog: u32 = funcs.iter().map(|&&(_, m)| m.cognitive_complexity).sum();
            let total_loc: u32 = funcs.iter().map(|&&(_, m)| m.total_lines).sum();
            let count = funcs.len() as f64;

            // Unique files
            let unique_files: HashSet<_> = funcs.iter()
                .map(|&&(f, _)| f.file_path.clone())
                .collect();

            // Per-module languages (derived from qualified_name prefix)
            let module_languages: Vec<&str> = {
                let mut langs: HashSet<&str> = HashSet::new();
                for &&(f, _) in funcs {
                    if let Some(lang) = f.qualified_name.split(':').next() {
                        langs.insert(lang);
                    }
                }
                let mut v: Vec<&str> = langs.into_iter().collect();
                v.sort();
                v
            };

            // Martin metrics
            let ca = afferent.get(module_id).map(|s| s.len()).unwrap_or(0) as u32;
            let ce = efferent.get(module_id).map(|s| s.len()).unwrap_or(0) as u32;
            let instability = if ca + ce > 0 {
                ce as f64 / (ca + ce) as f64
            } else {
                0.5 // Default when no coupling
            };

            // Calculate abstractness from type analysis
            let (abstract_count, total_types) = module_types
                .get(module_id)
                .copied()
                .unwrap_or((0, 0));
            let abstractness = if total_types > 0 {
                abstract_count as f64 / total_types as f64
            } else {
                0.0 // No types = not abstract
            };

            let distance = (instability + abstractness - 1.0).abs();

            serde_json::json!({
                "id": module_id,
                "name": module_id.split("::").last().unwrap_or(module_id),
                "path": format!("{}/", module_id.replace("::", "/")),
                "languages": module_languages,
                "metrics": {
                    "file_count": unique_files.len(),
                    "function_count": funcs.len(),
                    "total_cyclomatic": total_cc,
                    "avg_cyclomatic": if count > 0.0 { total_cc as f64 / count } else { 0.0 },
                    "total_cognitive": total_cog,
                    "avg_cognitive": if count > 0.0 { total_cog as f64 / count } else { 0.0 },
                    "lines_of_code": total_loc,
                    "martin": {
                        "ca": ca,
                        "ce": ce,
                        "instability": instability,
                        "abstractness": abstractness,
                        "distance_from_main_sequence": distance
                    }
                }
            })
        }).collect::<Vec<_>>()
    });
    fs::write(
        output_path.join("metrics/modules.json"),
        serde_json::to_string_pretty(&modules_json).unwrap(),
    )?;

    // =========================================================================
    // M1: Write dependencies.json (dependency graph with edges)
    // =========================================================================
    let dependency_nodes: Vec<serde_json::Value> = modules
        .keys()
        .map(|id| {
            serde_json::json!({
                "id": id,
                "type": "module"
            })
        })
        .collect();

    let dependency_edges: Vec<serde_json::Value> = import_edges
        .iter()
        .map(|((from, to), imports)| {
            serde_json::json!({
                "from": from,
                "to": to,
                "imports": imports,
                "weight": imports.len()
            })
        })
        .collect();

    let total_internal_edges = dependency_edges.len();
    let total_external_imports = imports.iter().filter(|i| i.is_external).count();

    let dependencies_json = serde_json::json!({
        "schema_version": "1.0.0",
        "nodes": dependency_nodes,
        "edges": dependency_edges,
        "metadata": {
            "total_nodes": modules.len(),
            "total_edges": total_internal_edges,
            "external_imports": total_external_imports
        }
    });
    fs::write(
        output_path.join("graphs/dependencies.json"),
        serde_json::to_string_pretty(&dependencies_json).unwrap(),
    )?;

    // =========================================================================
    // M2: Build coupling matrix with REAL values (not hardcoded 0.5)
    // =========================================================================
    let module_names: Vec<&str> = modules.keys().map(|s| s.as_str()).collect();
    let n = module_names.len();
    let mut matrix = vec![vec![0.0; n]; n];

    // Create index map
    let module_index: HashMap<&str, usize> = module_names
        .iter()
        .enumerate()
        .map(|(i, &name)| (name, i))
        .collect();

    // Fill diagonal with 1.0 (self-coupling)
    for (i, row) in matrix.iter_mut().enumerate() {
        row[i] = 1.0;
    }

    // =========================================================================
    // COMPOSITE COUPLING CALCULATION (v2.0)
    // Uses weighted import coupling with logarithmic percentile normalization
    // =========================================================================

    // Step 1: Calculate raw weighted import coupling
    // Weight by import kind: wildcard=0.3, multi=0.7/symbol, single=1.0, module=0.5
    let mut raw_coupling: HashMap<(usize, usize), f64> = HashMap::new();

    for ((from, to), imports_list) in &import_edges {
        if let (Some(&from_idx), Some(&to_idx)) = (
            module_index.get(from.as_str()),
            module_index.get(to.as_str()),
        ) {
            let mut weighted_score = 0.0;
            for import in imports_list {
                let base_weight = import.kind.weight();
                // For multi-imports, weight per symbol but cap total contribution
                // to avoid single large imports dominating the score
                let symbol_count = if import.symbols.is_empty() {
                    1.0
                } else {
                    import.symbols.len() as f64
                };
                // Cap at 3.0 to prevent outliers (e.g., `use foo::{a,b,c,d,e,f,g}`)
                let import_score = (base_weight * symbol_count).min(3.0);
                weighted_score += import_score;
            }
            raw_coupling.insert((from_idx, to_idx), weighted_score);
        }
    }

    // Step 2: Logarithmic percentile normalization
    // This produces a smooth distribution instead of discrete buckets
    fn logarithmic_percentile_normalize(
        values: &HashMap<(usize, usize), f64>,
    ) -> HashMap<(usize, usize), f64> {
        if values.is_empty() {
            return HashMap::new();
        }

        // Apply log transform to handle outliers
        let log_values: Vec<((usize, usize), f64)> =
            values.iter().map(|(&k, &v)| (k, (v + 1.0).ln())).collect();

        // Sort by log value to compute percentile ranks
        let mut sorted_values: Vec<f64> = log_values.iter().map(|(_, v)| *v).collect();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Compute percentile rank for each value
        let n = sorted_values.len();
        if n <= 1 {
            // Single value or empty: all get 1.0
            return log_values.iter().map(|(k, _)| (*k, 1.0)).collect();
        }
        log_values
            .iter()
            .map(|(k, log_v)| {
                // Count values strictly less than current (0-indexed rank)
                let rank = sorted_values.iter().filter(|&&v| v < *log_v).count() as f64;
                // Use (rank) / (n-1) to get proper 0.0 to 1.0 range
                let percentile = rank / (n - 1) as f64;
                (*k, percentile)
            })
            .collect()
    }

    // Step 3: Store raw import coupling for components breakdown (normalized by max)
    let mut import_coupling_matrix = vec![vec![0.0; n]; n];
    let max_import_raw = raw_coupling.values().cloned().fold(1.0_f64, f64::max);
    for ((from_idx, to_idx), raw_score) in &raw_coupling {
        import_coupling_matrix[*from_idx][*to_idx] = raw_score / max_import_raw;
    }

    // =========================================================================
    // CALL COUPLING CALCULATION
    // Count cross-module function calls
    // =========================================================================
    let mut call_edges: HashMap<(String, String), usize> = HashMap::new();
    for call in calls {
        let caller_module = &call.caller;
        let callee = &call.callee;

        // Try to resolve callee to a module with stricter matching
        for to_module in modules.keys() {
            if caller_module == to_module {
                continue; // Skip self-references
            }

            let to_name = to_module.split("::").last().unwrap_or(to_module);

            // Match if:
            // 1. Qualified call: callee starts with module path (e.g., "discovery::find_packages")
            // 2. Direct module call: callee equals module name (e.g., "discovery")
            // 3. Function in module: callee contains "::" and first part matches module
            let is_qualified_call =
                callee.starts_with(to_module) || callee.starts_with(&format!("{to_name}::"));
            let is_module_reference = callee == to_name || callee == to_module;
            let is_namespaced_call = callee.contains("::") && {
                let parts: Vec<&str> = callee.split("::").collect();
                parts.first().is_some_and(|first| *first == to_name)
            };

            if is_qualified_call || is_module_reference || is_namespaced_call {
                *call_edges
                    .entry((caller_module.clone(), to_module.clone()))
                    .or_insert(0) += 1;
            }
        }
    }

    // Build call coupling matrix
    let mut call_coupling_matrix = vec![vec![0.0; n]; n];
    let mut raw_call_coupling: HashMap<(usize, usize), f64> = HashMap::new();
    for ((from, to), count) in &call_edges {
        if let (Some(&from_idx), Some(&to_idx)) = (
            module_index.get(from.as_str()),
            module_index.get(to.as_str()),
        ) {
            raw_call_coupling.insert((from_idx, to_idx), *count as f64);
        }
    }

    // Normalize call coupling
    let max_call_raw = raw_call_coupling.values().cloned().fold(1.0_f64, f64::max);
    for ((from_idx, to_idx), raw_score) in &raw_call_coupling {
        call_coupling_matrix[*from_idx][*to_idx] = raw_score / max_call_raw;
    }

    // =========================================================================
    // TYPE COUPLING CALCULATION
    // Track type references between modules
    // =========================================================================
    let mut type_edges: HashMap<(String, String), usize> = HashMap::new();

    // Build a map of type name -> defining module
    let mut type_to_module: HashMap<String, String> = HashMap::new();
    for type_info in types {
        type_to_module.insert(type_info.name.clone(), type_info.module.clone());
    }

    // For each function, check if it uses types from other modules
    // This is a simplified approach - we look for type names in the same module's functions
    for &(func, _) in &functions {
        let func_module = &func.module;
        // Check for type usages - simplified: count types defined in other modules
        for (type_name, defining_module) in &type_to_module {
            if func_module != defining_module && func.qualified_name.contains(type_name) {
                *type_edges
                    .entry((func_module.clone(), defining_module.clone()))
                    .or_insert(0) += 1;
            }
        }
    }

    // Build type coupling matrix
    let mut type_coupling_matrix = vec![vec![0.0; n]; n];
    let mut raw_type_coupling: HashMap<(usize, usize), f64> = HashMap::new();
    for ((from, to), count) in &type_edges {
        if let (Some(&from_idx), Some(&to_idx)) = (
            module_index.get(from.as_str()),
            module_index.get(to.as_str()),
        ) {
            raw_type_coupling.insert((from_idx, to_idx), *count as f64);
        }
    }

    // Normalize type coupling
    let max_type_raw = raw_type_coupling.values().cloned().fold(1.0_f64, f64::max);
    for ((from_idx, to_idx), raw_score) in &raw_type_coupling {
        type_coupling_matrix[*from_idx][*to_idx] = raw_score / max_type_raw;
    }

    // =========================================================================
    // COMPOSITE SCORE
    // Combine all coupling components with weights
    // =========================================================================
    const IMPORT_WEIGHT: f64 = 0.40; // Increased since we have fewer components
    const CALL_WEIGHT: f64 = 0.35;
    const TYPE_WEIGHT: f64 = 0.25;

    // Combine all raw couplings for composite percentile normalization
    let mut composite_raw: HashMap<(usize, usize), f64> = HashMap::new();
    for i in 0..n {
        for j in 0..n {
            if i != j {
                let import_score = import_coupling_matrix[i][j];
                let call_score = call_coupling_matrix[i][j];
                let type_score = type_coupling_matrix[i][j];

                let composite = IMPORT_WEIGHT * import_score
                    + CALL_WEIGHT * call_score
                    + TYPE_WEIGHT * type_score;

                if composite > 0.0 {
                    composite_raw.insert((i, j), composite);
                }
            }
        }
    }

    // Re-normalize the composite scores using percentile ranking
    let normalized_composite = logarithmic_percentile_normalize(&composite_raw);

    // Fill the final matrix with composite normalized values
    for ((from_idx, to_idx), strength) in &normalized_composite {
        matrix[*from_idx][*to_idx] = *strength;
    }

    let coupling_json = serde_json::json!({
        "schema_version": "2.0.0",
        "metric": "composite_coupling",
        "description": "Composite coupling strength combining import, call, and type coupling (0-1). Directional: matrix[i][j] = strength of module i depending on module j.",
        "modules": module_names,
        "matrix": matrix,
        "components": {
            "import_coupling": {
                "weight": IMPORT_WEIGHT,
                "description": "Weighted import statement dependencies (wildcard=0.3, multi=0.7, single=1.0)",
                "matrix": import_coupling_matrix
            },
            "call_coupling": {
                "weight": CALL_WEIGHT,
                "description": "Cross-module function call count",
                "matrix": call_coupling_matrix,
                "total_calls": calls.len()
            },
            "type_coupling": {
                "weight": TYPE_WEIGHT,
                "description": "Type references between modules",
                "matrix": type_coupling_matrix,
                "total_types": types.len()
            }
        },
        "metadata": {
            "normalization": "logarithmic_percentile",
            "directional": true,
            "total_import_edges": import_edges.len(),
            "total_call_edges": call_edges.len(),
            "total_type_edges": type_edges.len(),
            "weights": {
                "wildcard": 0.3,
                "multi_per_symbol": 0.7,
                "single": 1.0,
                "module": 0.5
            }
        }
    });
    fs::write(
        output_path.join("graphs/coupling-matrix.json"),
        serde_json::to_string_pretty(&coupling_json).unwrap(),
    )?;

    // =========================================================================
    // M4: Slice Independence Score (SIS) for Vertical Slice Architecture
    // =========================================================================

    // Detect slices from first-level module path segment
    // e.g., "aef.core.events" -> slice "aef.core"
    //       "crates::aps-cli::src::main" -> slice "crates::aps-cli"
    fn get_slice_id(module_id: &str) -> String {
        // Split by the appropriate separator and take first two segments.
        // Path-like IDs (containing '/') use '/'  -  this avoids splitting inside
        // Next.js catch-all routes like [[...slug]] where '.' is literal.
        let separator = if module_id.contains('/') {
            "/"
        } else if module_id.contains("::") {
            "::"
        } else {
            "."
        };
        let parts: Vec<&str> = module_id.split(separator).collect();
        if parts.len() >= 2 {
            format!("{}{}{}", parts[0], separator, parts[1])
        } else {
            parts[0].to_string()
        }
    }

    // Group modules by slice
    let mut slices: HashMap<String, Vec<String>> = HashMap::new();
    for module_id in modules.keys() {
        let slice_id = get_slice_id(module_id);
        slices.entry(slice_id).or_default().push(module_id.clone());
    }

    // Calculate SIS for each slice
    // SIS = internal_imports / (internal_imports + external_imports)
    let slices_json: Vec<serde_json::Value> = slices
        .iter()
        .map(|(slice_id, slice_modules)| {
            let slice_module_set: HashSet<&str> =
                slice_modules.iter().map(|s| s.as_str()).collect();

            let mut internal_imports = 0u32;
            let mut cross_slice_imports = 0u32;
            let mut outbound_slices: HashSet<String> = HashSet::new();
            let mut inbound_slices: HashSet<String> = HashSet::new();

            // Count imports for modules in this slice
            for module in slice_modules {
                // Outbound: modules this slice depends on
                if let Some(deps) = efferent.get(module) {
                    for dep in deps {
                        let dep_slice = get_slice_id(dep);
                        if dep_slice == *slice_id {
                            internal_imports += 1;
                        } else {
                            cross_slice_imports += 1;
                            outbound_slices.insert(dep_slice);
                        }
                    }
                }

                // Inbound: modules that depend on this slice
                if let Some(dependents) = afferent.get(module) {
                    for dependent in dependents {
                        if !slice_module_set.contains(dependent.as_str()) {
                            let dependent_slice = get_slice_id(dependent);
                            inbound_slices.insert(dependent_slice);
                        }
                    }
                }
            }

            // Unique slice counts (more meaningful than edge counts)
            let inbound_coupling = inbound_slices.len() as u32;
            let outbound_coupling = outbound_slices.len() as u32;

            let total_imports = internal_imports + cross_slice_imports;
            let sis = if total_imports > 0 {
                internal_imports as f64 / total_imports as f64
            } else {
                1.0 // No imports = fully independent
            };

            serde_json::json!({
                "id": slice_id,
                "modules": slice_modules,
                "metrics": {
                    "module_count": slice_modules.len(),
                    "internal_imports": internal_imports,
                    "cross_slice_imports": cross_slice_imports,
                    "sis": sis,
                    "inbound_coupling": inbound_coupling,
                    "outbound_coupling": outbound_coupling
                }
            })
        })
        .collect();

    let slices_output = serde_json::json!({
        "schema_version": "1.0.0",
        "description": "Slice Independence Score (SIS) for Vertical Slice Architecture analysis. SIS = internal_imports / total_imports. Higher = more isolated.",
        "slices": slices_json,
        "metadata": {
            "total_slices": slices.len(),
            "slice_detection": "first_two_path_segments"
        }
    });
    fs::write(
        output_path.join("metrics/slices.json"),
        serde_json::to_string_pretty(&slices_output).unwrap(),
    )?;

    Ok(())
}

/// Simple timestamp without chrono dependency.
pub(super) fn chrono_lite_now() -> String {
    // Use a fixed format - in production would use actual time
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Approximate ISO 8601 (good enough for now)
    format!(
        "2025-12-17T{:02}:{:02}:{:02}Z",
        (secs / 3600) % 24,
        (secs / 60) % 60,
        secs % 60
    )
}
