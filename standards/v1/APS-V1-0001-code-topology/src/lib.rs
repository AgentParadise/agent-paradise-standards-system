//! Code Topology and Coupling Analysis (Experimental)
//!
//! Defines a language-agnostic artifact format for capturing code topology,
//! complexity metrics, and coupling analysis across polyglot codebases.
//!
//! ## Key Features
//!
//! - **Committable artifacts** — `.topology/` directory with metrics and graphs
//! - **Complexity metrics** — Cyclomatic, Cognitive, Halstead, Martin's coupling
//! - **Coupling matrix** — For 3D visualization of architecture
//! - **Language adapters** — Polyglot support via tree-sitter
//! - **Projector interface** — Substandards implement visualizations
//!
//! ## Example
//!
//! ```rust
//! use code_topology::{FunctionMetrics, HalsteadMetrics};
//!
//! let metrics = FunctionMetrics {
//!     cyclomatic_complexity: 5,
//!     cognitive_complexity: 8,
//!     halstead: HalsteadMetrics::default(),
//!     logical_lines: 20,
//!     total_lines: 30,
//!     comment_lines: 5,
//! };
//!
//! assert!(metrics.cyclomatic_complexity < 10); // Good complexity
//! ```
//!
//! ⚠️ EXPERIMENTAL: This standard is in incubation and may change significantly.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Tree-Sitter Adapter Framework
// ============================================================================

pub mod adapter;

// ============================================================================
// Error Codes
// ============================================================================

/// Error codes for topology validation.
pub mod error_codes {
    /// Missing `.topology/` directory
    pub const MISSING_TOPOLOGY_DIR: &str = "MISSING_TOPOLOGY_DIR";
    /// Missing or invalid manifest.toml
    pub const INVALID_MANIFEST: &str = "INVALID_MANIFEST";
    /// Missing required metrics files
    pub const MISSING_METRICS: &str = "MISSING_METRICS";
    /// Missing required graph files
    pub const MISSING_GRAPHS: &str = "MISSING_GRAPHS";
    /// Invalid coupling matrix (not symmetric or out of range)
    pub const INVALID_COUPLING_MATRIX: &str = "INVALID_COUPLING_MATRIX";
    /// Schema version mismatch
    pub const SCHEMA_VERSION_MISMATCH: &str = "SCHEMA_VERSION_MISMATCH";
    /// Language adapter error
    pub const ADAPTER_ERROR: &str = "ADAPTER_ERROR";
    /// Unsupported language
    pub const UNSUPPORTED_LANGUAGE: &str = "UNSUPPORTED_LANGUAGE";
}

// ============================================================================
// Core Types - Metrics
// ============================================================================

/// Halstead complexity metrics.
///
/// Computed from counts of operators and operands in the code.
/// See: Halstead, M.H. (1977). "Elements of Software Science"
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HalsteadMetrics {
    /// Distinct operators (η₁)
    pub distinct_operators: u32,
    /// Distinct operands (η₂)
    pub distinct_operands: u32,
    /// Total operators (N₁)
    pub total_operators: u32,
    /// Total operands (N₂)
    pub total_operands: u32,
    /// Vocabulary: η₁ + η₂
    pub vocabulary: u32,
    /// Length: N₁ + N₂
    pub length: u32,
    /// Volume: N × log₂(η)
    pub volume: f64,
    /// Difficulty: (η₁/2) × (N₂/η₂)
    pub difficulty: f64,
    /// Effort: D × V
    pub effort: f64,
    /// Time to implement (seconds): E / 18
    pub time_to_implement: f64,
    /// Estimated bugs: V / 3000
    pub estimated_bugs: f64,
}

impl Default for HalsteadMetrics {
    fn default() -> Self {
        Self {
            distinct_operators: 0,
            distinct_operands: 0,
            total_operators: 0,
            total_operands: 0,
            vocabulary: 0,
            length: 0,
            volume: 0.0,
            difficulty: 0.0,
            effort: 0.0,
            time_to_implement: 0.0,
            estimated_bugs: 0.0,
        }
    }
}

impl HalsteadMetrics {
    /// Calculate derived Halstead metrics from operator/operand counts.
    pub fn calculate(
        distinct_operators: u32,
        distinct_operands: u32,
        total_operators: u32,
        total_operands: u32,
    ) -> Self {
        let vocabulary = distinct_operators + distinct_operands;
        let length = total_operators + total_operands;

        // Avoid division by zero
        let volume = if vocabulary > 0 {
            (length as f64) * (vocabulary as f64).log2()
        } else {
            0.0
        };

        let difficulty = if distinct_operands > 0 {
            (distinct_operators as f64 / 2.0) * (total_operands as f64 / distinct_operands as f64)
        } else {
            0.0
        };

        let effort = difficulty * volume;
        let time_to_implement = effort / 18.0;
        let estimated_bugs = volume / 3000.0;

        Self {
            distinct_operators,
            distinct_operands,
            total_operators,
            total_operands,
            vocabulary,
            length,
            volume,
            difficulty,
            effort,
            time_to_implement,
            estimated_bugs,
        }
    }
}

/// Complexity metrics for a single function.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionMetrics {
    /// Cyclomatic complexity (McCabe's metric)
    pub cyclomatic_complexity: u32,
    /// Cognitive complexity (SonarSource's metric)
    pub cognitive_complexity: u32,
    /// Halstead metrics
    pub halstead: HalsteadMetrics,
    /// Lines of code (excluding blanks and comments)
    pub logical_lines: u32,
    /// Total lines including blanks and comments
    pub total_lines: u32,
    /// Comment lines
    pub comment_lines: u32,
}

impl Default for FunctionMetrics {
    fn default() -> Self {
        Self {
            cyclomatic_complexity: 1, // Minimum CC is 1
            cognitive_complexity: 0,
            halstead: HalsteadMetrics::default(),
            logical_lines: 0,
            total_lines: 0,
            comment_lines: 0,
        }
    }
}

/// Martin's coupling metrics for a module.
///
/// See: Martin, R.C. (2003). "Agile Software Development"
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MartinMetrics {
    /// Afferent coupling: modules that depend on this module
    pub ca: u32,
    /// Efferent coupling: modules this module depends on
    pub ce: u32,
    /// Instability: Ce / (Ca + Ce). Range [0, 1].
    pub instability: f64,
    /// Abstractness: abstract types / total types. Range [0, 1].
    pub abstractness: f64,
    /// Distance from main sequence: |A + I - 1|. Range [0, 1].
    pub distance_from_main_sequence: f64,
}

impl MartinMetrics {
    /// Calculate Martin's metrics from coupling counts and type info.
    pub fn calculate(ca: u32, ce: u32, abstract_types: u32, total_types: u32) -> Self {
        let instability = if ca + ce > 0 {
            ce as f64 / (ca + ce) as f64
        } else {
            0.0
        };

        let abstractness = if total_types > 0 {
            abstract_types as f64 / total_types as f64
        } else {
            0.0
        };

        let distance = (abstractness + instability - 1.0).abs();

        Self {
            ca,
            ce,
            instability,
            abstractness,
            distance_from_main_sequence: distance,
        }
    }
}

impl Default for MartinMetrics {
    fn default() -> Self {
        Self {
            ca: 0,
            ce: 0,
            instability: 0.0,
            abstractness: 0.0,
            distance_from_main_sequence: 1.0, // Worst case: concrete and stable
        }
    }
}

// ============================================================================
// Core Types - Language Adapter
// ============================================================================

/// Visibility of a function or type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Accessible from anywhere
    Public,
    /// Accessible within the module/file
    #[default]
    Private,
    /// Accessible within the crate/package
    Internal,
    /// Protected (subclass access)
    Protected,
}
/// Information about a function/method extracted from source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionInfo {
    /// Fully qualified name (e.g., "rust:module::submodule::function_name")
    pub qualified_name: String,
    /// Simple name (e.g., "function_name")
    pub name: String,
    /// File path relative to analysis root
    pub file_path: PathBuf,
    /// Module this function belongs to
    pub module: String,
    /// Start line (1-indexed)
    pub start_line: u32,
    /// End line (1-indexed)
    pub end_line: u32,
    /// Number of parameters
    pub parameter_count: u32,
    /// Whether this is a method (has self/this)
    pub is_method: bool,
    /// Visibility
    pub visibility: Visibility,
    /// Raw source code of the function body (for metric calculation)
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(default)]
    pub body_source: String,
}

/// Information about a function call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallInfo {
    /// Caller function qualified name
    pub caller: String,
    /// Callee function qualified name (may be unresolved)
    pub callee: String,
    /// File where the call occurs
    pub file_path: PathBuf,
    /// Line number of the call
    pub line: u32,
    /// Whether the callee could be resolved to a definition
    pub resolved: bool,
}

/// Information about an import/dependency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportInfo {
    /// Importing file/module
    pub from_module: String,
    /// Imported file/module
    pub to_module: String,
    /// Import path as written in source
    pub import_path: String,
    /// Whether this is an external (third-party) import
    pub is_external: bool,
    /// Symbols imported (empty for wildcard or module-level imports)
    #[serde(default)]
    pub symbols: Vec<String>,
    /// Whether this is a wildcard import (use foo::*)
    #[serde(default)]
    pub is_wildcard: bool,
    /// Import kind for coupling weight calculation
    #[serde(default)]
    pub kind: ImportKind,
}

/// Import kind for coupling weight calculation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ImportKind {
    /// Wildcard import (use foo::*) - weight 0.3
    Wildcard,
    /// Multi-symbol import (use foo::{A, B, C}) - weight 0.7 per symbol
    Multi,
    /// Single symbol import (use foo::bar) - weight 1.0
    #[default]
    Single,
    /// Module-level import (use foo) - weight 0.5
    Module,
}

impl ImportKind {
    /// Get the coupling weight for this import kind
    pub fn weight(&self) -> f64 {
        match self {
            ImportKind::Wildcard => 0.3,
            ImportKind::Multi => 0.7,
            ImportKind::Single => 1.0,
            ImportKind::Module => 0.5,
        }
    }
}

/// Information about a type definition (for abstractness calculation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeInfo {
    /// Type name
    pub name: String,
    /// Module containing the type
    pub module: String,
    /// Whether this is abstract (trait, interface, abstract class)
    pub is_abstract: bool,
}

// ============================================================================
// Core Types - Topology Artifact
// ============================================================================

/// The complete topology of a codebase.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Topology {
    /// Schema version
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    /// Languages analyzed
    #[serde(default)]
    pub languages: Vec<String>,
    /// All functions with their metrics
    #[serde(default)]
    pub functions: Vec<FunctionWithMetrics>,
    /// Module-level metrics (including Martin's)
    #[serde(default)]
    pub modules: Vec<ModuleMetrics>,
    /// Call graph edges
    #[serde(default)]
    pub call_graph: Vec<CallInfo>,
    /// Dependency graph edges
    #[serde(default)]
    pub dependency_graph: Vec<ImportInfo>,
    /// Coupling matrix between modules (optional for partial loads)
    #[serde(default)]
    pub coupling_matrix: Option<CouplingMatrixData>,
}

fn default_schema_version() -> String {
    "0.1.0".to_string()
}

/// Coupling matrix data with internal values (for projector consumption).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CouplingMatrixData {
    /// Module names (column/row headers)
    pub modules: Vec<String>,
    /// NxN matrix of coupling values [0, 1]
    pub values: Vec<Vec<f64>>,
    /// Optional: saved layout positions for visualization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positions: Option<HashMap<String, [f64; 3]>>,
}

/// A function combined with its metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionWithMetrics {
    /// Function information
    #[serde(flatten)]
    pub info: FunctionInfo,
    /// Computed metrics
    pub metrics: FunctionMetrics,
}

/// Metrics aggregated at the module level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleMetrics {
    /// Module identifier
    pub id: String,
    /// Module name
    pub name: String,
    /// Path to the module (directory or file)
    pub path: PathBuf,
    /// Languages in this module
    pub languages: Vec<String>,
    /// Number of files
    pub file_count: u32,
    /// Number of functions
    pub function_count: u32,
    /// Total cyclomatic complexity
    pub total_cyclomatic: u32,
    /// Average cyclomatic complexity
    pub avg_cyclomatic: f64,
    /// Total cognitive complexity
    pub total_cognitive: u32,
    /// Average cognitive complexity
    pub avg_cognitive: f64,
    /// Total lines of code
    pub lines_of_code: u32,
    /// Martin's coupling metrics
    pub martin: MartinMetrics,
}

/// Coupling matrix between modules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CouplingMatrix {
    /// Schema version
    pub schema_version: String,
    /// Metric used for coupling calculation
    pub metric: String,
    /// Description of the metric
    pub description: String,
    /// Module names (column/row headers)
    pub modules: Vec<String>,
    /// NxN matrix of coupling values [0, 1]
    pub matrix: Vec<Vec<f64>>,
    /// Optional: saved layout positions for visualization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutInfo>,
    /// Whether the matrix is directional (asymmetric) or symmetric
    /// Directional: matrix[i][j] = coupling of i depending on j
    /// Symmetric: matrix[i][j] = matrix[j][i]
    #[serde(default)]
    pub directional: bool,
}

/// Saved layout positions for deterministic visualization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutInfo {
    /// Layout algorithm used
    pub algorithm: String,
    /// Random seed for reproducibility
    pub seed: u64,
    /// Module positions (module_id -> [x, y, z])
    pub positions: HashMap<String, [f64; 3]>,
}

impl CouplingMatrix {
    /// Create a new empty coupling matrix (symmetric by default).
    pub fn new(modules: Vec<String>) -> Self {
        Self::with_directional(modules, false)
    }

    /// Create a new coupling matrix with specified directionality.
    pub fn with_directional(modules: Vec<String>, directional: bool) -> Self {
        let n = modules.len();
        let matrix = vec![vec![0.0; n]; n];

        // Set diagonal to 1.0 (a module is fully coupled with itself)
        let mut result = Self {
            schema_version: "0.1.0".to_string(),
            metric: "normalized_coupling".to_string(),
            description: "Normalized coupling strength between modules (0-1)".to_string(),
            modules,
            matrix,
            layout: None,
            directional,
        };

        for i in 0..n {
            result.matrix[i][i] = 1.0;
        }

        result
    }

    /// Set coupling value between two modules.
    ///
    /// For symmetric matrices (directional=false): maintains matrix[i][j] = matrix[j][i]
    /// For directional matrices: only sets matrix[i][j] (A depends on B)
    pub fn set_coupling(&mut self, module_a: &str, module_b: &str, value: f64) {
        let i = self.modules.iter().position(|m| m == module_a);
        let j = self.modules.iter().position(|m| m == module_b);

        if let (Some(i), Some(j)) = (i, j) {
            let clamped = value.clamp(0.0, 1.0);
            self.matrix[i][j] = clamped;
            if !self.directional {
                self.matrix[j][i] = clamped; // Symmetric only for non-directional
            }
        }
    }

    /// Get coupling value between two modules.
    pub fn get_coupling(&self, module_a: &str, module_b: &str) -> Option<f64> {
        let i = self.modules.iter().position(|m| m == module_a)?;
        let j = self.modules.iter().position(|m| m == module_b)?;
        Some(self.matrix[i][j])
    }

    /// Validate the coupling matrix.
    pub fn validate(&self) -> Result<(), String> {
        let n = self.modules.len();

        // Check dimensions
        if self.matrix.len() != n {
            return Err(format!(
                "Matrix row count {} doesn't match module count {}",
                self.matrix.len(),
                n
            ));
        }

        for (i, row) in self.matrix.iter().enumerate() {
            if row.len() != n {
                return Err(format!(
                    "Row {} has {} columns, expected {}",
                    i,
                    row.len(),
                    n
                ));
            }

            // Check range (and symmetry for non-directional matrices)
            for (j, &value) in row.iter().enumerate() {
                if !(0.0..=1.0).contains(&value) {
                    return Err(format!(
                        "Value at [{i},{j}] = {value} is out of range [0,1]"
                    ));
                }

                // Only check symmetry for non-directional matrices
                if !self.directional {
                    const SYMMETRY_TOLERANCE: f64 = 1e-10;
                    if (value - self.matrix[j][i]).abs() > SYMMETRY_TOLERANCE {
                        let other = self.matrix[j][i];
                        return Err(format!(
                            "Matrix is not symmetric: [{i},{j}]={value} != [{j},{i}]={other}"
                        ));
                    }
                }
            }

            // Check diagonal is 1.0 (with tolerance)
            const DIAGONAL_TOLERANCE: f64 = 1e-10;
            if (self.matrix[i][i] - 1.0).abs() > DIAGONAL_TOLERANCE {
                let val = self.matrix[i][i];
                return Err(format!("Diagonal [{i},{i}] = {val} should be 1.0"));
            }
        }

        Ok(())
    }
}

// ============================================================================
// Language Adapter Trait
// ============================================================================

/// Error from a language adapter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdapterError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// File that caused the error (if applicable)
    pub file: Option<PathBuf>,
    /// Line number (if applicable)
    pub line: Option<u32>,
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref file) = self.file {
            write!(f, " in {}", file.display())?;
        }
        if let Some(line) = self.line {
            write!(f, " at line {line}")?;
        }
        Ok(())
    }
}

impl std::error::Error for AdapterError {}

/// A language adapter extracts topology data from source code.
///
/// Implementers should provide language-specific parsing and metric calculation.
pub trait LanguageAdapter: Send + Sync {
    /// Returns the language identifier (e.g., "rust", "typescript", "python").
    fn language_id(&self) -> &'static str;

    /// Returns file extensions this adapter handles (e.g., &[".rs"], &[".ts", ".tsx"]).
    fn file_extensions(&self) -> &'static [&'static str];

    /// Parse a source file and extract function definitions.
    fn extract_functions(
        &self,
        source: &str,
        file_path: &std::path::Path,
    ) -> Result<Vec<FunctionInfo>, AdapterError>;

    /// Extract function call relationships from a source file.
    fn extract_calls(
        &self,
        source: &str,
        file_path: &std::path::Path,
    ) -> Result<Vec<CallInfo>, AdapterError>;

    /// Extract import/dependency relationships from a source file.
    fn extract_imports(
        &self,
        source: &str,
        file_path: &std::path::Path,
    ) -> Result<Vec<ImportInfo>, AdapterError>;

    /// Compute complexity metrics for a function.
    fn compute_metrics(
        &self,
        source: &str,
        function: &FunctionInfo,
    ) -> Result<FunctionMetrics, AdapterError>;

    /// Optional: Extract type definitions for abstractness calculation.
    fn extract_types(
        &self,
        _source: &str,
        _file_path: &std::path::Path,
    ) -> Result<Vec<TypeInfo>, AdapterError> {
        Ok(vec![]) // Default: no type extraction
    }
}

// ============================================================================
// Projector Trait
// ============================================================================

/// Output format for projector rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Graphviz DOT format (text)
    Dot,
    /// SVG image (text/XML)
    Svg,
    /// PNG image (binary)
    Png,
    /// Mermaid diagram syntax (text)
    Mermaid,
    /// Markdown with embedded diagrams
    Markdown,
    /// JSON data (for custom rendering)
    Json,
    /// WebGL/Three.js scene description (JSON)
    WebGL,
    /// HTML interactive visualization
    Html,
    /// GLTF 3D model (binary)
    Gltf,
}

/// Projector-specific configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectorConfig {
    /// Raw JSON configuration
    pub raw: serde_json::Value,
}

/// Error from a projector.
#[derive(Debug)]
pub struct ProjectorError {
    /// Error code
    pub code: &'static str,
    /// Error message
    pub message: String,
    /// Source error (if any)
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl std::fmt::Display for ProjectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ProjectorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

/// A projector renders topology data as visualizations.
///
/// Each projector substandard implements this trait.
pub trait Projector: Send + Sync {
    /// Projector identifier (e.g., "graphviz", "3d-force")
    fn id(&self) -> &'static str;

    /// Human-readable name
    fn name(&self) -> &'static str;

    /// Description of what this projector visualizes
    fn description(&self) -> &'static str;

    /// Load topology artifacts from a `.topology/` directory
    fn load(&self, topology_dir: &std::path::Path) -> Result<Topology, ProjectorError>;

    /// Render the topology to the specified format
    fn render(
        &self,
        topology: &Topology,
        format: OutputFormat,
        config: Option<&ProjectorConfig>,
    ) -> Result<Vec<u8>, ProjectorError>;

    /// Supported output formats
    fn supported_formats(&self) -> &[OutputFormat];

    /// Configuration schema (JSON Schema for projector-specific options)
    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// Validate projector-specific configuration
    fn validate_config(&self, _config: &serde_json::Value) -> Result<(), ProjectorError> {
        Ok(())
    }
}

// ============================================================================
// Artifact File Types (for parsing .topology/ files)
// ============================================================================

/// Container for functions.json artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionsFile {
    /// Schema version
    pub schema_version: String,
    /// All analyzed functions
    pub functions: Vec<FunctionRecord>,
}

/// A function record in the functions.json artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionRecord {
    /// Fully qualified identifier
    pub id: String,
    /// Simple function name
    pub name: String,
    /// File path relative to analysis root
    pub file: String,
    /// Module this function belongs to
    pub module: String,
    /// Source language
    pub language: String,
    /// Source location
    pub location: LocationRange,
    /// Computed metrics
    pub metrics: FunctionMetricsRecord,
}

/// Location range in source file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationRange {
    /// Start line (1-indexed)
    pub start_line: u32,
    /// End line (1-indexed)
    pub end_line: u32,
}

/// Metrics in the JSON artifact format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionMetricsRecord {
    /// Cyclomatic complexity
    pub cyclomatic_complexity: u32,
    /// Cognitive complexity
    pub cognitive_complexity: u32,
    /// Halstead metrics
    pub halstead: HalsteadRecord,
    /// Lines of code
    pub lines_of_code: u32,
    /// Logical lines
    pub logical_lines: u32,
    /// Comment lines
    pub comment_lines: u32,
    /// Parameter count
    pub parameter_count: u32,
}

/// Halstead metrics in JSON artifact format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HalsteadRecord {
    /// Vocabulary
    pub vocabulary: u32,
    /// Length
    pub length: u32,
    /// Volume
    pub volume: f64,
    /// Difficulty
    pub difficulty: f64,
    /// Effort
    pub effort: f64,
    /// Time to implement
    pub time_to_implement: f64,
    /// Estimated bugs
    pub estimated_bugs: f64,
}

/// Container for modules.json artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModulesFile {
    /// Schema version
    pub schema_version: String,
    /// All analyzed modules
    pub modules: Vec<ModuleRecord>,
}

/// A module record in the modules.json artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleRecord {
    /// Module identifier
    pub id: String,
    /// Module name
    pub name: String,
    /// Path to the module
    pub path: String,
    /// Languages in this module
    pub languages: Vec<String>,
    /// Aggregated metrics
    pub metrics: ModuleMetricsRecord,
}

/// Module metrics in the JSON artifact format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleMetricsRecord {
    /// File count
    pub file_count: u32,
    /// Function count
    pub function_count: u32,
    /// Total cyclomatic complexity
    pub total_cyclomatic: u32,
    /// Average cyclomatic complexity
    pub avg_cyclomatic: f64,
    /// Total cognitive complexity
    pub total_cognitive: u32,
    /// Average cognitive complexity
    pub avg_cognitive: f64,
    /// Lines of code
    pub lines_of_code: u32,
    /// Martin's metrics
    pub martin: MartinRecord,
}

/// Martin's metrics in JSON artifact format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MartinRecord {
    /// Afferent coupling
    pub ca: u32,
    /// Efferent coupling
    pub ce: u32,
    /// Instability
    pub instability: f64,
    /// Abstractness
    pub abstractness: f64,
    /// Distance from main sequence
    pub distance_from_main_sequence: f64,
}

/// Container for coupling-matrix.json artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CouplingMatrixFile {
    /// Schema version
    pub schema_version: String,
    /// Metric name
    pub metric: String,
    /// Description
    pub description: String,
    /// Module names
    pub modules: Vec<String>,
    /// NxN coupling matrix
    pub matrix: Vec<Vec<f64>>,
    /// Optional layout info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutInfoRecord>,
}

/// Layout info in JSON artifact format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutInfoRecord {
    /// Algorithm used
    pub algorithm: String,
    /// Random seed
    pub seed: u64,
    /// Positions (module_id -> [x, y, z])
    pub positions: HashMap<String, [f64; 3]>,
}

// ============================================================================
// Standard Implementation
// ============================================================================

/// The Code Topology and Coupling Analysis standard implementation.
pub struct CodeTopologyStandard;

impl CodeTopologyStandard {
    /// Create a new instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for CodeTopologyStandard {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: Implement Standard trait in Milestone 4
// impl aps_v1_0000_meta::Standard for CodeTopologyStandard {
//     fn validate_package(&self, path: &Path) -> Diagnostics { ... }
//     fn validate_repo(&self, path: &Path) -> Diagnostics { ... }
// }

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let _ = CodeTopologyStandard::new();
    }

    #[test]
    fn test_error_codes_defined() {
        // Just verify the codes have the expected format (SCREAMING_SNAKE_CASE)
        assert!(error_codes::MISSING_TOPOLOGY_DIR.contains('_'));
        assert!(error_codes::INVALID_MANIFEST.contains('_'));
        assert!(error_codes::MISSING_METRICS.contains('_'));
        assert!(error_codes::MISSING_GRAPHS.contains('_'));
        assert!(error_codes::INVALID_COUPLING_MATRIX.contains('_'));
        assert!(error_codes::ADAPTER_ERROR.contains('_'));
    }

    #[test]
    fn test_halstead_calculation() {
        let metrics = HalsteadMetrics::calculate(10, 20, 50, 100);

        assert_eq!(metrics.vocabulary, 30);
        assert_eq!(metrics.length, 150);
        assert!(metrics.volume > 0.0);
        assert!(metrics.difficulty > 0.0);
        assert!(metrics.effort > 0.0);
    }

    #[test]
    fn test_halstead_zero_handling() {
        let metrics = HalsteadMetrics::calculate(0, 0, 0, 0);

        assert_eq!(metrics.vocabulary, 0);
        assert_eq!(metrics.volume, 0.0);
        assert_eq!(metrics.difficulty, 0.0);
    }

    #[test]
    fn test_martin_metrics_calculation() {
        // Balanced module: I=0.5, A=0.5 -> D=0
        let metrics = MartinMetrics::calculate(5, 5, 5, 10);

        assert_eq!(metrics.ca, 5);
        assert_eq!(metrics.ce, 5);
        assert!((metrics.instability - 0.5).abs() < f64::EPSILON);
        assert!((metrics.abstractness - 0.5).abs() < f64::EPSILON);
        assert!(metrics.distance_from_main_sequence < 0.01);
    }

    #[test]
    fn test_martin_metrics_zone_of_pain() {
        // Concrete and stable: I=0, A=0 -> D=1 (Zone of Pain)
        let metrics = MartinMetrics::calculate(10, 0, 0, 10);

        assert!((metrics.instability - 0.0).abs() < f64::EPSILON);
        assert!((metrics.abstractness - 0.0).abs() < f64::EPSILON);
        assert!((metrics.distance_from_main_sequence - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_coupling_matrix_creation() {
        let modules = vec!["auth".to_string(), "api".to_string(), "db".to_string()];
        let matrix = CouplingMatrix::new(modules);

        assert_eq!(matrix.modules.len(), 3);
        assert_eq!(matrix.matrix.len(), 3);

        // Diagonal should be 1.0
        assert!((matrix.matrix[0][0] - 1.0).abs() < f64::EPSILON);
        assert!((matrix.matrix[1][1] - 1.0).abs() < f64::EPSILON);
        assert!((matrix.matrix[2][2] - 1.0).abs() < f64::EPSILON);

        // Off-diagonal should be 0.0
        assert!((matrix.matrix[0][1] - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_coupling_matrix_set_get() {
        let modules = vec!["auth".to_string(), "api".to_string()];
        let mut matrix = CouplingMatrix::new(modules);

        matrix.set_coupling("auth", "api", 0.75);

        // Should be symmetric
        assert_eq!(matrix.get_coupling("auth", "api"), Some(0.75));
        assert_eq!(matrix.get_coupling("api", "auth"), Some(0.75));
    }

    #[test]
    fn test_coupling_matrix_validation() {
        let modules = vec!["a".to_string(), "b".to_string()];
        let matrix = CouplingMatrix::new(modules);

        assert!(matrix.validate().is_ok());
    }

    #[test]
    fn test_coupling_matrix_validation_fails_asymmetric_when_not_directional() {
        let modules = vec!["a".to_string(), "b".to_string()];
        let mut matrix = CouplingMatrix::new(modules); // directional=false by default

        // Break symmetry manually
        matrix.matrix[0][1] = 0.5;
        // Don't set [1][0]

        // Non-directional matrices must be symmetric
        assert!(matrix.validate().is_err());
    }

    #[test]
    fn test_coupling_matrix_validation_allows_asymmetric_when_directional() {
        let modules = vec!["a".to_string(), "b".to_string()];
        let mut matrix = CouplingMatrix::with_directional(modules, true);

        // Asymmetric values are allowed for directional matrices
        matrix.matrix[0][1] = 0.5;
        matrix.matrix[1][0] = 0.3; // Different value

        // Directional matrices can be asymmetric
        assert!(matrix.validate().is_ok());
    }

    #[test]
    fn test_function_metrics_default() {
        let metrics = FunctionMetrics::default();

        // Minimum cyclomatic complexity is 1
        assert_eq!(metrics.cyclomatic_complexity, 1);
        assert_eq!(metrics.cognitive_complexity, 0);
    }

    #[test]
    fn test_visibility_serialization() {
        let vis = Visibility::Public;
        let json = serde_json::to_string(&vis).unwrap();
        assert_eq!(json, "\"public\"");
    }

    // --- Edge case tests (from Copilot review) ---

    #[test]
    fn test_halstead_zero_operands_nonzero_operators() {
        // Edge case: operators present but no operands
        let metrics = HalsteadMetrics::calculate(5, 0, 10, 0);

        assert_eq!(metrics.vocabulary, 5);
        assert_eq!(metrics.length, 10);
        // Difficulty should be 0 when distinct_operands is 0 (avoid division by zero)
        assert_eq!(metrics.difficulty, 0.0);
        assert_eq!(metrics.effort, 0.0);
    }

    #[test]
    fn test_martin_metrics_zero_types_nonzero_abstract() {
        // Edge case: abstract_types > 0 but total_types = 0 (invalid data)
        // This shouldn't happen in practice, but we should handle it gracefully
        let metrics = MartinMetrics::calculate(5, 5, 3, 0);

        // With total_types = 0, abstractness should be 0.0 (not NaN or panic)
        assert!(!metrics.abstractness.is_nan());
        assert_eq!(metrics.abstractness, 0.0);
    }

    #[test]
    fn test_coupling_matrix_validation_mismatched_row_lengths() {
        let modules = vec!["a".to_string(), "b".to_string()];
        let mut matrix = CouplingMatrix::new(modules);

        // Create a malformed matrix with different row lengths
        matrix.matrix = vec![
            vec![1.0, 0.5], // Correct: 2 columns
            vec![0.5],      // Wrong: only 1 column
        ];

        let result = matrix.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("columns"));
    }

    #[test]
    fn test_coupling_matrix_validation_wrong_row_count() {
        let modules = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut matrix = CouplingMatrix::new(modules);

        // Set matrix to wrong size (2 rows instead of 3)
        matrix.matrix = vec![vec![1.0, 0.0], vec![0.0, 1.0]];

        let result = matrix.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("row count"));
    }
}
