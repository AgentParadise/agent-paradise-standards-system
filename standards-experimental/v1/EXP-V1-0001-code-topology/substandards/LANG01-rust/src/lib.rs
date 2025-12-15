//! Rust Language Adapter (EXP-V1-0001.LANG01)
//!
//! Analyzes Rust codebases and generates `.topology/` artifacts.
//!
//! ## Features
//!
//! - **Complexity metrics**: Cyclomatic, Cognitive, Halstead per function
//! - **Call graphs**: Function-to-function call relationships
//! - **Module metrics**: Martin's Ca/Ce/I/A/D per module
//! - **Coupling matrix**: Module-to-module coupling strength
//!
//! ## Usage
//!
//! ```ignore
//! use code_topology_rust_adapter::RustAdapter;
//!
//! let adapter = RustAdapter::new();
//! let result = adapter.analyze(Path::new("my-rust-project"))?;
//! result.write_artifacts(Path::new(".topology/"))?;
//! ```
//!
//! ⚠️ EXPERIMENTAL: This substandard is in incubation.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use syn::visit::Visit;
use syn::{self, Expr, Item, Stmt};
use thiserror::Error;
use walkdir::WalkDir;

use code_topology::HalsteadMetrics;

// ============================================================================
// Error Types
// ============================================================================

/// Errors from the Rust adapter.
#[derive(Debug, Error)]
pub enum RustAdapterError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error in {file}: {message}")]
    Parse { file: PathBuf, message: String },

    #[error("Not a Cargo project: missing Cargo.toml")]
    NotCargoProject,

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

// ============================================================================
// Cargo.toml Parsing
// ============================================================================

/// Minimal Cargo.toml structure for discovery.
#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Option<PackageInfo>,
    workspace: Option<WorkspaceInfo>,
}

#[derive(Debug, Deserialize)]
struct PackageInfo {
    name: String,
}

#[derive(Debug, Deserialize)]
struct WorkspaceInfo {
    members: Option<Vec<String>>,
}

// ============================================================================
// Analysis Result Types
// ============================================================================

/// Complete analysis result for a Rust project.
#[derive(Debug, Clone, Default)]
pub struct AnalysisResult {
    /// Project/crate name
    pub name: String,
    /// Root path analyzed
    pub root: PathBuf,
    /// All discovered functions with metrics
    pub functions: Vec<FunctionAnalysis>,
    /// All discovered modules
    pub modules: Vec<ModuleAnalysis>,
    /// Call graph edges
    pub calls: Vec<CallEdge>,
    /// Import relationships (module -> [imported modules])
    pub imports: HashMap<String, Vec<String>>,
}

/// Analysis of a single function.
#[derive(Debug, Clone)]
pub struct FunctionAnalysis {
    /// Unique ID (module::function)
    pub id: String,
    /// Function name
    pub name: String,
    /// Module path
    pub module: String,
    /// File path
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Cyclomatic complexity
    pub cyclomatic: u32,
    /// Cognitive complexity
    pub cognitive: u32,
    /// Halstead metrics
    pub halstead: HalsteadMetrics,
    /// Lines of code (function body)
    pub loc: u32,
    /// Is this a public function?
    pub is_public: bool,
    /// Is this an async function?
    pub is_async: bool,
}

/// Analysis of a module.
#[derive(Debug, Clone)]
pub struct ModuleAnalysis {
    /// Unique ID (crate::module::path)
    pub id: String,
    /// Module name
    pub name: String,
    /// File or directory path
    pub path: PathBuf,
    /// Functions in this module
    pub function_count: u32,
    /// Total lines of code
    pub loc: u32,
    /// Afferent coupling (who depends on me)
    pub ca: u32,
    /// Efferent coupling (what I depend on)
    pub ce: u32,
    /// Number of abstract types (traits)
    pub abstract_count: u32,
    /// Number of concrete types (structs, enums)
    pub concrete_count: u32,
}

/// A call edge in the call graph.
#[derive(Debug, Clone)]
pub struct CallEdge {
    /// Caller function ID
    pub from: String,
    /// Callee function ID
    pub to: String,
    /// Call site line number
    pub line: usize,
}

// ============================================================================
// Rust Adapter
// ============================================================================

/// The Rust Language Adapter.
pub struct RustAdapter {
    /// Configuration
    config: RustAdapterConfig,
}

/// Configuration for the Rust adapter.
#[derive(Debug, Clone)]
pub struct RustAdapterConfig {
    /// Paths to exclude from analysis
    pub exclude_paths: Vec<String>,
    /// Whether to analyze test code
    pub include_tests: bool,
    /// Whether to follow workspace members
    pub follow_workspace: bool,
}

impl Default for RustAdapterConfig {
    fn default() -> Self {
        Self {
            exclude_paths: vec!["target".into(), ".git".into()],
            include_tests: false,
            follow_workspace: true,
        }
    }
}

impl RustAdapter {
    /// Create a new adapter with default configuration.
    pub fn new() -> Self {
        Self {
            config: RustAdapterConfig::default(),
        }
    }

    /// Create an adapter with custom configuration.
    pub fn with_config(config: RustAdapterConfig) -> Self {
        Self { config }
    }

    /// Analyze a Rust project at the given path.
    pub fn analyze(&self, project_path: &Path) -> Result<AnalysisResult, RustAdapterError> {
        let cargo_toml_path = project_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Err(RustAdapterError::NotCargoProject);
        }

        let cargo_content = fs::read_to_string(&cargo_toml_path)?;
        let cargo: CargoToml = toml::from_str(&cargo_content)?;

        let name = cargo
            .package
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "workspace".into());

        let mut result = AnalysisResult {
            name: name.clone(),
            root: project_path.to_path_buf(),
            ..Default::default()
        };

        // If workspace, analyze all members
        if let Some(workspace) = &cargo.workspace {
            if self.config.follow_workspace {
                if let Some(members) = &workspace.members {
                    for member in members {
                        // Handle glob patterns simply
                        if member.contains('*') {
                            continue; // Skip globs for now
                        }
                        let member_path = project_path.join(member);
                        if member_path.exists() {
                            self.analyze_crate(&member_path, &mut result)?;
                        }
                    }
                }
            }
        }

        // Analyze the root crate if it has src/
        let src_path = project_path.join("src");
        if src_path.exists() {
            self.analyze_crate(project_path, &mut result)?;
        }

        // Compute module-level metrics
        self.compute_module_metrics(&mut result);

        Ok(result)
    }

    /// Analyze a single crate.
    fn analyze_crate(
        &self,
        crate_path: &Path,
        result: &mut AnalysisResult,
    ) -> Result<(), RustAdapterError> {
        let src_path = crate_path.join("src");
        if !src_path.exists() {
            return Ok(());
        }

        // Get crate name from Cargo.toml
        let cargo_path = crate_path.join("Cargo.toml");
        let crate_name = if cargo_path.exists() {
            let content = fs::read_to_string(&cargo_path)?;
            let cargo: CargoToml = toml::from_str(&content)?;
            cargo
                .package
                .map(|p| p.name.replace('-', "_"))
                .unwrap_or_else(|| "unknown".into())
        } else {
            "unknown".into()
        };

        // Walk all .rs files
        for entry in WalkDir::new(&src_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            let file_path = entry.path();

            // Check exclusions
            let path_str = file_path.to_string_lossy();
            if self
                .config
                .exclude_paths
                .iter()
                .any(|ex| path_str.contains(ex))
            {
                continue;
            }

            // Skip test files if configured
            if !self.config.include_tests
                && (path_str.contains("/tests/") || path_str.ends_with("_test.rs"))
            {
                continue;
            }

            self.analyze_file(file_path, &crate_name, &src_path, result)?;
        }

        Ok(())
    }

    /// Analyze a single Rust file.
    fn analyze_file(
        &self,
        file_path: &Path,
        crate_name: &str,
        src_root: &Path,
        result: &mut AnalysisResult,
    ) -> Result<(), RustAdapterError> {
        let content = fs::read_to_string(file_path)?;

        let syntax = syn::parse_file(&content).map_err(|e| RustAdapterError::Parse {
            file: file_path.to_path_buf(),
            message: e.to_string(),
        })?;

        // Compute module path from file path
        let module_path = self.compute_module_path(file_path, crate_name, src_root);

        // Track imports for this file
        let mut file_imports = Vec::new();

        // Visit all items
        for item in &syntax.items {
            match item {
                Item::Fn(func) => {
                    let analysis = self.analyze_function(func, &module_path, file_path);
                    result.functions.push(analysis);
                }
                Item::Impl(impl_block) => {
                    // Analyze methods in impl blocks
                    let type_name = self.type_to_string(&impl_block.self_ty);
                    for impl_item in &impl_block.items {
                        if let syn::ImplItem::Fn(method) = impl_item {
                            let method_module = format!("{module_path}::{type_name}");
                            let analysis =
                                self.analyze_impl_method(method, &method_module, file_path);
                            result.functions.push(analysis);
                        }
                    }
                }
                Item::Use(use_item) => {
                    // Track imports
                    let imports = self.extract_use_paths(&use_item.tree);
                    file_imports.extend(imports);
                }
                Item::Trait(_) => {
                    // Count as abstract type
                    if let Some(module) = result.modules.iter_mut().find(|m| m.id == module_path) {
                        module.abstract_count += 1;
                    }
                }
                Item::Struct(_) | Item::Enum(_) => {
                    // Count as concrete type
                    if let Some(module) = result.modules.iter_mut().find(|m| m.id == module_path) {
                        module.concrete_count += 1;
                    }
                }
                _ => {}
            }
        }

        // Record imports
        result.imports.insert(module_path.clone(), file_imports);

        // Ensure module exists
        if !result.modules.iter().any(|m| m.id == module_path) {
            let loc = content.lines().count() as u32;
            result.modules.push(ModuleAnalysis {
                id: module_path.clone(),
                name: module_path.split("::").last().unwrap_or(&module_path).into(),
                path: file_path.to_path_buf(),
                function_count: 0,
                loc,
                ca: 0,
                ce: 0,
                abstract_count: 0,
                concrete_count: 0,
            });
        }

        Ok(())
    }

    /// Compute module path from file path.
    fn compute_module_path(&self, file_path: &Path, crate_name: &str, src_root: &Path) -> String {
        let relative = file_path
            .strip_prefix(src_root)
            .unwrap_or(file_path)
            .with_extension("");

        let relative_str = relative.to_string_lossy().replace(['/', '\\'], "::");

        if relative_str == "lib" || relative_str == "main" {
            crate_name.to_string()
        } else if relative_str.ends_with("::mod") {
            format!(
                "{}::{}",
                crate_name,
                relative_str.trim_end_matches("::mod")
            )
        } else {
            format!("{crate_name}::{relative_str}")
        }
    }

    /// Analyze a top-level function.
    fn analyze_function(
        &self,
        func: &syn::ItemFn,
        module_path: &str,
        file_path: &Path,
    ) -> FunctionAnalysis {
        let name = func.sig.ident.to_string();
        let id = format!("{module_path}::{name}");

        let mut visitor = ComplexityVisitor::new();
        visitor.visit_block(&func.block);

        let halstead = self.compute_halstead(&func.block);
        let loc = self.count_loc(&func.block);

        FunctionAnalysis {
            id,
            name,
            module: module_path.to_string(),
            file: file_path.to_path_buf(),
            line: 0, // Would need span info
            cyclomatic: visitor.cyclomatic,
            cognitive: visitor.cognitive,
            halstead,
            loc,
            is_public: matches!(func.vis, syn::Visibility::Public(_)),
            is_async: func.sig.asyncness.is_some(),
        }
    }

    /// Analyze a method in an impl block.
    fn analyze_impl_method(
        &self,
        method: &syn::ImplItemFn,
        module_path: &str,
        file_path: &Path,
    ) -> FunctionAnalysis {
        let name = method.sig.ident.to_string();
        let id = format!("{module_path}::{name}");

        let mut visitor = ComplexityVisitor::new();
        visitor.visit_block(&method.block);

        let halstead = self.compute_halstead(&method.block);
        let loc = self.count_loc(&method.block);

        FunctionAnalysis {
            id,
            name,
            module: module_path.to_string(),
            file: file_path.to_path_buf(),
            line: 0,
            cyclomatic: visitor.cyclomatic,
            cognitive: visitor.cognitive,
            halstead,
            loc,
            is_public: matches!(method.vis, syn::Visibility::Public(_)),
            is_async: method.sig.asyncness.is_some(),
        }
    }

    /// Compute Halstead metrics for a block.
    fn compute_halstead(&self, block: &syn::Block) -> HalsteadMetrics {
        let mut visitor = HalsteadVisitor::new();
        visitor.visit_block(block);
        visitor.compute()
    }

    /// Count lines of code in a block.
    fn count_loc(&self, _block: &syn::Block) -> u32 {
        // Simplified: would need span info for accurate count
        10 // Placeholder
    }

    /// Convert a type to string.
    fn type_to_string(&self, ty: &syn::Type) -> String {
        match ty {
            syn::Type::Path(type_path) => type_path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::"),
            _ => "Unknown".into(),
        }
    }

    /// Extract paths from a use tree.
    fn extract_use_paths(&self, tree: &syn::UseTree) -> Vec<String> {
        let mut paths = Vec::new();
        self.collect_use_paths(tree, String::new(), &mut paths);
        paths
    }

    fn collect_use_paths(&self, tree: &syn::UseTree, prefix: String, paths: &mut Vec<String>) {
        match tree {
            syn::UseTree::Path(path) => {
                let new_prefix = if prefix.is_empty() {
                    path.ident.to_string()
                } else {
                    format!("{}::{}", prefix, path.ident)
                };
                self.collect_use_paths(&path.tree, new_prefix, paths);
            }
            syn::UseTree::Name(name) => {
                let full = if prefix.is_empty() {
                    name.ident.to_string()
                } else {
                    format!("{}::{}", prefix, name.ident)
                };
                paths.push(full);
            }
            syn::UseTree::Rename(rename) => {
                let full = if prefix.is_empty() {
                    rename.ident.to_string()
                } else {
                    format!("{}::{}", prefix, rename.ident)
                };
                paths.push(full);
            }
            syn::UseTree::Glob(_) => {
                paths.push(format!("{prefix}::*"));
            }
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    self.collect_use_paths(item, prefix.clone(), paths);
                }
            }
        }
    }

    /// Compute module-level metrics after function analysis.
    fn compute_module_metrics(&self, result: &mut AnalysisResult) {
        // Group functions by module
        let mut module_functions: HashMap<String, Vec<&FunctionAnalysis>> = HashMap::new();
        for func in &result.functions {
            module_functions
                .entry(func.module.clone())
                .or_default()
                .push(func);
        }

        // Update module function counts
        for module in &mut result.modules {
            module.function_count = module_functions
                .get(&module.id)
                .map(|f| f.len() as u32)
                .unwrap_or(0);
        }

        // Compute Ca/Ce from imports
        let module_ids: HashSet<_> = result.modules.iter().map(|m| m.id.clone()).collect();

        for module in &mut result.modules {
            // Ce: What do I depend on?
            if let Some(imports) = result.imports.get(&module.id) {
                for import in imports {
                    // Check if import refers to an internal module
                    let import_module = import.split("::").take(2).collect::<Vec<_>>().join("::");
                    if module_ids.contains(&import_module) && import_module != module.id {
                        module.ce += 1;
                    }
                }
            }
        }

        // Ca: Who depends on me?
        for (importer_id, imports) in &result.imports {
            for import in imports {
                let import_module = import.split("::").take(2).collect::<Vec<_>>().join("::");
                if let Some(target) = result.modules.iter_mut().find(|m| m.id == import_module) {
                    if target.id != *importer_id {
                        target.ca += 1;
                    }
                }
            }
        }
    }
}

impl Default for RustAdapter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Complexity Visitor
// ============================================================================

/// Visitor that computes cyclomatic and cognitive complexity.
struct ComplexityVisitor {
    cyclomatic: u32,
    cognitive: u32,
    nesting_level: u32,
}

impl ComplexityVisitor {
    fn new() -> Self {
        Self {
            cyclomatic: 1, // Base complexity
            cognitive: 0,
            nesting_level: 0,
        }
    }

    fn add_cognitive(&mut self, base: u32) {
        self.cognitive += base + self.nesting_level;
    }
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        match expr {
            // If expressions
            Expr::If(_) => {
                self.cyclomatic += 1;
                self.add_cognitive(1);
                self.nesting_level += 1;
                syn::visit::visit_expr(self, expr);
                self.nesting_level -= 1;
                return;
            }
            // Match expressions
            Expr::Match(match_expr) => {
                // Each arm (except wildcard) adds complexity
                for arm in &match_expr.arms {
                    if !matches!(arm.pat, syn::Pat::Wild(_)) {
                        self.cyclomatic += 1;
                    }
                }
                self.add_cognitive(1);
                self.nesting_level += 1;
                syn::visit::visit_expr(self, expr);
                self.nesting_level -= 1;
                return;
            }
            // Loops
            Expr::While(_) | Expr::Loop(_) | Expr::ForLoop(_) => {
                self.cyclomatic += 1;
                self.add_cognitive(1);
                self.nesting_level += 1;
                syn::visit::visit_expr(self, expr);
                self.nesting_level -= 1;
                return;
            }
            // Try operator (?)
            Expr::Try(_) => {
                self.cyclomatic += 1;
                // Cognitive: +0 (linear flow in Rust idiom)
            }
            // Binary operators
            Expr::Binary(bin) => {
                match bin.op {
                    syn::BinOp::And(_) | syn::BinOp::Or(_) => {
                        self.cyclomatic += 1;
                        self.cognitive += 1;
                    }
                    _ => {}
                }
            }
            // Closures add cognitive complexity (context switch)
            Expr::Closure(_) => {
                self.cognitive += 1;
            }
            // Async blocks add cognitive complexity
            Expr::Async(_) => {
                self.cognitive += 1;
            }
            _ => {}
        }

        syn::visit::visit_expr(self, expr);
    }
}

// ============================================================================
// Halstead Visitor
// ============================================================================

/// Visitor that collects operators and operands for Halstead metrics.
struct HalsteadVisitor {
    operators: HashMap<String, u32>,
    operands: HashMap<String, u32>,
}

impl HalsteadVisitor {
    fn new() -> Self {
        Self {
            operators: HashMap::new(),
            operands: HashMap::new(),
        }
    }

    fn add_operator(&mut self, op: &str) {
        *self.operators.entry(op.to_string()).or_insert(0) += 1;
    }

    fn add_operand(&mut self, op: &str) {
        *self.operands.entry(op.to_string()).or_insert(0) += 1;
    }

    fn compute(self) -> HalsteadMetrics {
        let n1 = self.operators.len() as u32; // Distinct operators
        let n2 = self.operands.len() as u32; // Distinct operands
        let big_n1: u32 = self.operators.values().sum(); // Total operators
        let big_n2: u32 = self.operands.values().sum(); // Total operands

        HalsteadMetrics::calculate(n1, n2, big_n1, big_n2)
    }
}

impl<'ast> Visit<'ast> for HalsteadVisitor {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        match expr {
            Expr::Binary(bin) => {
                let op = format!("{:?}", bin.op);
                self.add_operator(&op);
            }
            Expr::Unary(un) => {
                let op = format!("{:?}", un.op);
                self.add_operator(&op);
            }
            Expr::Call(_) => {
                self.add_operator("()");
            }
            Expr::MethodCall(m) => {
                self.add_operator(".");
                self.add_operand(&m.method.to_string());
            }
            Expr::Field(f) => {
                self.add_operator(".");
                if let syn::Member::Named(ident) = &f.member {
                    self.add_operand(&ident.to_string());
                }
            }
            Expr::Index(_) => {
                self.add_operator("[]");
            }
            Expr::Lit(lit) => {
                let lit_str = format!("{:?}", lit.lit);
                self.add_operand(&lit_str);
            }
            Expr::Path(p) => {
                let path_str = p
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect::<Vec<_>>()
                    .join("::");
                self.add_operand(&path_str);
            }
            Expr::If(_) => self.add_operator("if"),
            Expr::Match(_) => self.add_operator("match"),
            Expr::While(_) => self.add_operator("while"),
            Expr::Loop(_) => self.add_operator("loop"),
            Expr::ForLoop(_) => self.add_operator("for"),
            Expr::Return(_) => self.add_operator("return"),
            Expr::Break(_) => self.add_operator("break"),
            Expr::Continue(_) => self.add_operator("continue"),
            Expr::Try(_) => self.add_operator("?"),
            Expr::Await(_) => self.add_operator(".await"),
            Expr::Assign(_) => self.add_operator("="),
            _ => {}
        }

        syn::visit::visit_expr(self, expr);
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        if let Stmt::Local(local) = stmt {
            self.add_operator("let");
            if local.init.is_some() {
                self.add_operator("=");
            }
        }
        syn::visit::visit_stmt(self, stmt);
    }
}

// ============================================================================
// Artifact Generation
// ============================================================================

impl AnalysisResult {
    /// Write analysis results to .topology/ directory.
    pub fn write_artifacts(&self, output_dir: &Path) -> Result<(), RustAdapterError> {
        fs::create_dir_all(output_dir)?;
        fs::create_dir_all(output_dir.join("metrics"))?;
        fs::create_dir_all(output_dir.join("graphs"))?;

        // Write manifest
        self.write_manifest(output_dir)?;

        // Write metrics
        self.write_function_metrics(output_dir)?;
        self.write_module_metrics(output_dir)?;

        // Write graphs
        self.write_coupling_matrix(output_dir)?;

        Ok(())
    }

    fn write_manifest(&self, output_dir: &Path) -> Result<(), RustAdapterError> {
        let manifest = format!(
            r#"schema_version = "1.0.0"
generator = "code-topology-rust-adapter"
generated_at = "2025-12-15T00:00:00Z"

[project]
name = "{}"
languages = ["rust"]

[analysis]
include_tests = false
exclude_paths = ["target", ".git"]
"#,
            self.name
        );

        fs::write(output_dir.join("manifest.toml"), manifest)?;
        Ok(())
    }

    fn write_function_metrics(&self, output_dir: &Path) -> Result<(), RustAdapterError> {
        #[derive(Serialize)]
        struct FunctionsFile {
            schema_version: String,
            functions: Vec<FunctionRecord>,
        }

        #[derive(Serialize)]
        struct FunctionRecord {
            id: String,
            name: String,
            module: String,
            file: String,
            line: usize,
            metrics: FunctionMetricsRecord,
        }

        #[derive(Serialize)]
        struct FunctionMetricsRecord {
            cyclomatic: u32,
            cognitive: u32,
            halstead: HalsteadRecord,
            loc: u32,
        }

        #[derive(Serialize)]
        struct HalsteadRecord {
            vocabulary: u32,
            length: u32,
            volume: f64,
            difficulty: f64,
            effort: f64,
        }

        let functions: Vec<FunctionRecord> = self
            .functions
            .iter()
            .map(|f| FunctionRecord {
                id: f.id.clone(),
                name: f.name.clone(),
                module: f.module.clone(),
                file: f.file.to_string_lossy().to_string(),
                line: f.line,
                metrics: FunctionMetricsRecord {
                    cyclomatic: f.cyclomatic,
                    cognitive: f.cognitive,
                    halstead: HalsteadRecord {
                        vocabulary: f.halstead.vocabulary,
                        length: f.halstead.length,
                        volume: f.halstead.volume,
                        difficulty: f.halstead.difficulty,
                        effort: f.halstead.effort,
                    },
                    loc: f.loc,
                },
            })
            .collect();

        let file = FunctionsFile {
            schema_version: "1.0.0".into(),
            functions,
        };

        let content = serde_json::to_string_pretty(&file)?;
        fs::write(output_dir.join("metrics/functions.json"), content)?;
        Ok(())
    }

    fn write_module_metrics(&self, output_dir: &Path) -> Result<(), RustAdapterError> {
        #[derive(Serialize)]
        struct ModulesFile {
            schema_version: String,
            modules: Vec<ModuleRecord>,
        }

        #[derive(Serialize)]
        struct ModuleRecord {
            id: String,
            name: String,
            path: String,
            languages: Vec<String>,
            metrics: ModuleMetricsRecord,
        }

        #[derive(Serialize)]
        struct ModuleMetricsRecord {
            file_count: u32,
            function_count: u32,
            total_cyclomatic: u32,
            avg_cyclomatic: f64,
            total_cognitive: u32,
            avg_cognitive: f64,
            lines_of_code: u32,
            martin: MartinRecord,
        }

        #[derive(Serialize)]
        struct MartinRecord {
            ca: u32,
            ce: u32,
            instability: f64,
            abstractness: f64,
            distance_from_main_sequence: f64,
        }

        // Compute aggregates per module
        let mut module_aggregates: HashMap<String, (u32, u32, u32, u32)> = HashMap::new();
        for func in &self.functions {
            let entry = module_aggregates.entry(func.module.clone()).or_default();
            entry.0 += 1; // function count
            entry.1 += func.cyclomatic;
            entry.2 += func.cognitive;
            entry.3 += func.loc;
        }

        let modules: Vec<ModuleRecord> = self
            .modules
            .iter()
            .map(|m| {
                let (func_count, total_cc, total_cog, total_loc) =
                    module_aggregates.get(&m.id).cloned().unwrap_or((0, 0, 0, 0));

                let avg_cc = if func_count > 0 {
                    total_cc as f64 / func_count as f64
                } else {
                    0.0
                };
                let avg_cog = if func_count > 0 {
                    total_cog as f64 / func_count as f64
                } else {
                    0.0
                };

                // Martin's metrics
                let ca = m.ca;
                let ce = m.ce;
                let instability = if ca + ce > 0 {
                    ce as f64 / (ca + ce) as f64
                } else {
                    0.5
                };
                let total_types = m.abstract_count + m.concrete_count;
                let abstractness = if total_types > 0 {
                    m.abstract_count as f64 / total_types as f64
                } else {
                    0.0
                };
                let distance = (abstractness + instability - 1.0).abs();

                ModuleRecord {
                    id: m.id.clone(),
                    name: m.name.clone(),
                    path: m.path.to_string_lossy().to_string(),
                    languages: vec!["rust".into()],
                    metrics: ModuleMetricsRecord {
                        file_count: 1,
                        function_count: func_count,
                        total_cyclomatic: total_cc,
                        avg_cyclomatic: avg_cc,
                        total_cognitive: total_cog,
                        avg_cognitive: avg_cog,
                        lines_of_code: total_loc.max(m.loc),
                        martin: MartinRecord {
                            ca,
                            ce,
                            instability,
                            abstractness,
                            distance_from_main_sequence: distance,
                        },
                    },
                }
            })
            .collect();

        let file = ModulesFile {
            schema_version: "1.0.0".into(),
            modules,
        };

        let content = serde_json::to_string_pretty(&file)?;
        fs::write(output_dir.join("metrics/modules.json"), content)?;
        Ok(())
    }

    fn write_coupling_matrix(&self, output_dir: &Path) -> Result<(), RustAdapterError> {
        #[derive(Serialize)]
        struct CouplingMatrixFile {
            schema_version: String,
            metric: String,
            modules: Vec<String>,
            matrix: Vec<Vec<f64>>,
        }

        let module_ids: Vec<String> = self.modules.iter().map(|m| m.id.clone()).collect();
        let n = module_ids.len();

        // Build coupling matrix based on import relationships
        let mut matrix = vec![vec![0.0; n]; n];

        for (i, module_a) in module_ids.iter().enumerate() {
            if let Some(imports) = self.imports.get(module_a) {
                for import in imports {
                    // Find which module this import refers to
                    for (j, module_b) in module_ids.iter().enumerate() {
                        if i != j && import.starts_with(module_b) {
                            // Coupling strength based on number of imports
                            matrix[i][j] += 0.1_f64;
                            matrix[i][j] = matrix[i][j].min(1.0_f64);
                        }
                    }
                }
            }
        }

        let file = CouplingMatrixFile {
            schema_version: "1.0.0".into(),
            metric: "import_coupling".into(),
            modules: module_ids,
            matrix,
        };

        let content = serde_json::to_string_pretty(&file)?;
        fs::write(output_dir.join("graphs/coupling-matrix.json"), content)?;
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_creation() {
        let adapter = RustAdapter::new();
        assert!(!adapter.config.include_tests);
    }

    #[test]
    fn test_complexity_visitor_if() {
        let code = "fn test() { if true { } }";
        let file = syn::parse_file(code).unwrap();
        if let Item::Fn(func) = &file.items[0] {
            let mut visitor = ComplexityVisitor::new();
            visitor.visit_block(&func.block);
            assert_eq!(visitor.cyclomatic, 2); // 1 base + 1 if
        }
    }

    #[test]
    fn test_complexity_visitor_match() {
        let code = "fn test() { match x { A => {}, B => {}, _ => {} } }";
        let file = syn::parse_file(code).unwrap();
        if let Item::Fn(func) = &file.items[0] {
            let mut visitor = ComplexityVisitor::new();
            visitor.visit_block(&func.block);
            assert_eq!(visitor.cyclomatic, 3); // 1 base + 2 non-wildcard arms
        }
    }

    #[test]
    fn test_halstead_basic() {
        let code = "fn test() { let x = 1 + 2; }";
        let file = syn::parse_file(code).unwrap();
        if let Item::Fn(func) = &file.items[0] {
            let mut visitor = HalsteadVisitor::new();
            visitor.visit_block(&func.block);
            let metrics = visitor.compute();
            assert!(metrics.vocabulary > 0);
            assert!(metrics.length > 0);
        }
    }
}

