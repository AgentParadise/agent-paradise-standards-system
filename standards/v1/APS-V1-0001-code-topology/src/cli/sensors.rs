//! `sensors` command: project `.topology/metrics/modules.json` into a stable,
//! agent-queryable signal set.
//!
//! While `viz` renders HTML for humans and `report` prints a markdown table,
//! `sensors` emits a deterministic JSON document ranking modules by health,
//! complexity, coupling, instability, or a composite risk score. Every
//! selection carries a `reasons` array that explains why the module ranked
//! where it did, so a downstream agent does not have to re-derive the signal.
//!
//! The output shape is intentionally schema-versioned so consumers can parse
//! it without watching for silent format drift.

use crate::cli::health::{
    calculate_health, detect_layer, get_slice_from_id, health_label, health_to_color,
};
use crate::{ModuleRecord, ModulesFile};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::Path;

/// Current sensors document schema version.
pub const SENSORS_SCHEMA_VERSION: &str = "0.1.0";

/// Which metric to rank modules by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    /// Worst health first (lowest health score).
    Health,
    /// Highest average cyclomatic complexity first.
    Complexity,
    /// Highest total coupling (ca + ce) first.
    Coupling,
    /// Highest instability (I = ce / (ca + ce)) first.
    Instability,
    /// Composite risk: low health, high complexity, and high coupling combined.
    Risk,
}

impl SortBy {
    /// Parse a `--by` value. Returns None for unknown values.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "health" => Some(SortBy::Health),
            "complexity" | "cc" | "cyclomatic" => Some(SortBy::Complexity),
            "coupling" => Some(SortBy::Coupling),
            "instability" => Some(SortBy::Instability),
            "risk" => Some(SortBy::Risk),
            _ => None,
        }
    }

    /// Canonical string representation used in the output document.
    pub fn as_str(self) -> &'static str {
        match self {
            SortBy::Health => "health",
            SortBy::Complexity => "complexity",
            SortBy::Coupling => "coupling",
            SortBy::Instability => "instability",
            SortBy::Risk => "risk",
        }
    }
}

/// Container for the sensors document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorsReport {
    /// Sensors schema version. Independent from the `modules.json` version.
    pub schema_version: String,
    /// The `modules.json` schema version this report was derived from.
    pub source_schema_version: String,
    /// Which ranking mode produced this list.
    pub ranking: String,
    /// Requested top N. `None` means "return all modules".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<usize>,
    /// Total modules considered before truncating to `top_n`.
    pub total_modules: usize,
    /// Ranked module entries, most-signal first.
    pub modules: Vec<SensorEntry>,
}

/// One ranked module entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorEntry {
    /// 1-based rank within this report.
    pub rank: usize,
    /// Module identifier (matches `modules.json`).
    pub id: String,
    /// Module display name.
    pub name: String,
    /// Module path.
    pub path: String,
    /// Architectural layer (best-effort inference from `path`).
    pub layer: String,
    /// Slice / top-level package.
    pub slice: String,
    /// Languages present in this module.
    pub languages: Vec<String>,
    /// Composite health score in `[0.0, 1.0]`.
    pub health: f64,
    /// Human-readable bucket for `health`.
    pub health_label: String,
    /// Hex color used in the visual dashboards, kept here so agents can
    /// correlate their signal choice with what a human reviewer sees.
    pub color: String,
    /// Number of functions in the module.
    pub function_count: u32,
    /// Average cyclomatic complexity across the module's functions.
    pub avg_cyclomatic: f64,
    /// Average cognitive complexity across the module's functions.
    pub avg_cognitive: f64,
    /// Total cyclomatic complexity in the module.
    pub total_cyclomatic: u32,
    /// Total cognitive complexity in the module.
    pub total_cognitive: u32,
    /// Lines of code in the module.
    pub lines_of_code: u32,
    /// Afferent coupling (incoming dependencies).
    pub ca: u32,
    /// Efferent coupling (outgoing dependencies).
    pub ce: u32,
    /// `ca + ce`, exposed for convenience.
    pub total_coupling: u32,
    /// Martin instability `I = ce / (ca + ce)`.
    pub instability: f64,
    /// Martin abstractness.
    pub abstractness: f64,
    /// Distance from Martin's main sequence.
    pub distance_from_main_sequence: f64,
    /// Composite risk score used when ranking by `risk`. Included on every
    /// entry so agents can compare across modes without re-deriving it.
    pub risk_score: f64,
    /// Machine-parseable reasons this module appears (which thresholds it
    /// tripped). Order is stable, values are short lower-case snippets.
    pub reasons: Vec<String>,
}

/// Composite risk score: high when health is low and complexity / coupling
/// are high. Range roughly `[0.0, ~3.0]`, monotone in each contributor.
fn compute_risk(health: f64, avg_cc: f64, total_coupling: u32) -> f64 {
    let health_gap = (1.0 - health).max(0.0);
    let cc_component = (avg_cc / 10.0).min(1.0);
    let coupling_component = (total_coupling as f64 / 20.0).min(1.0);
    health_gap + cc_component + coupling_component
}

/// Enumerate the reasons a module scored the way it did. Deterministic order.
fn build_reasons(entry: &SensorEntry) -> Vec<String> {
    let mut reasons = Vec::new();
    if entry.function_count == 0 {
        reasons.push("no functions detected".to_string());
    }
    if entry.avg_cyclomatic > 10.0 {
        reasons.push(format!(
            "avg_cyclomatic {:.2} exceeds high threshold 10",
            entry.avg_cyclomatic
        ));
    } else if entry.avg_cyclomatic > 5.0 {
        reasons.push(format!(
            "avg_cyclomatic {:.2} exceeds nominal threshold 5",
            entry.avg_cyclomatic
        ));
    }
    if entry.avg_cognitive > 15.0 {
        reasons.push(format!(
            "avg_cognitive {:.2} exceeds high threshold 15",
            entry.avg_cognitive
        ));
    } else if entry.avg_cognitive > 10.0 {
        reasons.push(format!(
            "avg_cognitive {:.2} exceeds nominal threshold 10",
            entry.avg_cognitive
        ));
    }
    if entry.total_coupling >= 20 {
        reasons.push(format!(
            "coupling ca={} ce={} totals {} (high)",
            entry.ca, entry.ce, entry.total_coupling
        ));
    } else if entry.ce >= 5 {
        reasons.push(format!("efferent coupling ce={} is high", entry.ce));
    }
    if entry.total_coupling == 0 {
        reasons.push("isolated (ca=0 and ce=0)".to_string());
    }
    if entry.instability >= 0.9 {
        reasons.push(format!(
            "instability {:.2} (fully unstable)",
            entry.instability
        ));
    }
    if entry.distance_from_main_sequence >= 0.7 {
        reasons.push(format!(
            "distance from main sequence {:.2}",
            entry.distance_from_main_sequence
        ));
    }
    if entry.health < 0.5 {
        reasons.push(format!(
            "health {:.2} ({})",
            entry.health, entry.health_label
        ));
    }
    reasons
}

/// Project a `ModuleRecord` into a `SensorEntry`. Rank is filled in later.
fn module_to_entry(record: &ModuleRecord) -> SensorEntry {
    let m = &record.metrics;
    let health = calculate_health(
        m.function_count,
        m.total_cyclomatic,
        m.total_cognitive,
        m.lines_of_code,
        m.martin.ca,
        m.martin.ce,
    );
    let total_coupling = m.martin.ca + m.martin.ce;
    let risk_score = compute_risk(health, m.avg_cyclomatic, total_coupling);

    let mut entry = SensorEntry {
        rank: 0,
        id: record.id.clone(),
        name: record.name.clone(),
        path: record.path.clone(),
        layer: detect_layer(&record.path).to_string(),
        slice: get_slice_from_id(&record.id),
        languages: record.languages.clone(),
        health,
        health_label: health_label(health).to_string(),
        color: health_to_color(health).to_string(),
        function_count: m.function_count,
        avg_cyclomatic: m.avg_cyclomatic,
        avg_cognitive: m.avg_cognitive,
        total_cyclomatic: m.total_cyclomatic,
        total_cognitive: m.total_cognitive,
        lines_of_code: m.lines_of_code,
        ca: m.martin.ca,
        ce: m.martin.ce,
        total_coupling,
        instability: m.martin.instability,
        abstractness: m.martin.abstractness,
        distance_from_main_sequence: m.martin.distance_from_main_sequence,
        risk_score,
        reasons: Vec::new(),
    };
    entry.reasons = build_reasons(&entry);
    entry
}

/// Stable, total ordering suitable for `sort_by`: primary key by mode,
/// tiebreak on module id ascending so the output is deterministic across
/// runs.
fn compare_entries(mode: SortBy, a: &SensorEntry, b: &SensorEntry) -> Ordering {
    // Every mode surfaces the "worst / highest-signal" first.
    let primary = match mode {
        // Lower health = higher signal, so ascending on health.
        SortBy::Health => a.health.partial_cmp(&b.health).unwrap_or(Ordering::Equal),
        SortBy::Complexity => b
            .avg_cyclomatic
            .partial_cmp(&a.avg_cyclomatic)
            .unwrap_or(Ordering::Equal),
        SortBy::Coupling => b.total_coupling.cmp(&a.total_coupling),
        SortBy::Instability => b
            .instability
            .partial_cmp(&a.instability)
            .unwrap_or(Ordering::Equal),
        SortBy::Risk => b
            .risk_score
            .partial_cmp(&a.risk_score)
            .unwrap_or(Ordering::Equal),
    };
    primary.then_with(|| a.id.cmp(&b.id))
}

/// Build a `SensorsReport` from an already-loaded `ModulesFile`.
///
/// `top_n = None` returns all modules; otherwise the list is truncated
/// after ranking. `total_modules` always reflects the full input.
pub fn build_report(modules: &ModulesFile, sort_by: SortBy, top_n: Option<usize>) -> SensorsReport {
    let mut entries: Vec<SensorEntry> = modules.modules.iter().map(module_to_entry).collect();
    entries.sort_by(|a, b| compare_entries(sort_by, a, b));

    let total_modules = entries.len();
    if let Some(n) = top_n {
        entries.truncate(n);
    }
    for (i, entry) in entries.iter_mut().enumerate() {
        entry.rank = i + 1;
    }

    SensorsReport {
        schema_version: SENSORS_SCHEMA_VERSION.to_string(),
        source_schema_version: modules.schema_version.clone(),
        ranking: sort_by.as_str().to_string(),
        top_n,
        total_modules,
        modules: entries,
    }
}

/// Load `<topology_dir>/metrics/modules.json` and build a sensors report.
pub fn load_report(
    topology_dir: &Path,
    sort_by: SortBy,
    top_n: Option<usize>,
) -> Result<SensorsReport, String> {
    let modules_path = topology_dir.join("metrics/modules.json");
    if !modules_path.exists() {
        return Err(format!(
            "No topology artifacts found at {}",
            topology_dir.display()
        ));
    }
    let content = std::fs::read_to_string(&modules_path)
        .map_err(|e| format!("Could not read {}: {e}", modules_path.display()))?;
    let modules: ModulesFile = serde_json::from_str(&content)
        .map_err(|e| format!("Could not parse {}: {e}", modules_path.display()))?;
    Ok(build_report(&modules, sort_by, top_n))
}

/// Options parsed from the CLI for the `sensors` subcommand.
#[derive(Debug, Clone)]
struct SensorsOptions<'a> {
    path: &'a str,
    sort_by: SortBy,
    top_n: Option<usize>,
    format: &'a str,
    output: Option<&'a str>,
    persist: bool,
}

fn parse_options<'a>(args: &'a [String]) -> Result<SensorsOptions<'a>, String> {
    let mut sort_by = SortBy::Health;
    let mut top_n: Option<usize> = Some(10);
    let mut format = "json";
    let mut output: Option<&str> = None;
    let mut path: Option<&str> = None;
    let mut persist = false;

    // Track indexes that are option values so they are not misread as `<path>`.
    let mut consumed_value_indexes: Vec<usize> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "--by" | "-b" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--by requires a value".to_string())?;
                sort_by = SortBy::parse(value).ok_or_else(|| {
                    format!(
                        "Unknown --by value '{value}'. Expected one of: \
                         health, complexity, coupling, instability, risk"
                    )
                })?;
                consumed_value_indexes.push(i + 1);
                i += 2;
            }
            "--top" | "-n" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--top requires a value".to_string())?;
                let parsed: usize = value
                    .parse()
                    .map_err(|_| format!("--top expects a non-negative integer, got '{value}'"))?;
                top_n = if parsed == 0 { None } else { Some(parsed) };
                consumed_value_indexes.push(i + 1);
                i += 2;
            }
            "--format" | "-f" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--format requires a value".to_string())?;
                if value != "json" && value != "text" {
                    return Err(format!("--format expects 'json' or 'text', got '{value}'"));
                }
                format = value.as_str();
                consumed_value_indexes.push(i + 1);
                i += 2;
            }
            "--output" | "-o" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| "--output requires a value".to_string())?;
                output = Some(value.as_str());
                consumed_value_indexes.push(i + 1);
                i += 2;
            }
            "--persist" => {
                persist = true;
                i += 1;
            }
            other if other.starts_with('-') => {
                return Err(format!("Unknown option '{other}'"));
            }
            _ => {
                if !consumed_value_indexes.contains(&i) && path.is_none() {
                    path = Some(arg);
                }
                i += 1;
            }
        }
    }

    Ok(SensorsOptions {
        path: path.unwrap_or(".topology"),
        sort_by,
        top_n,
        format,
        output,
        persist,
    })
}

fn render_text(report: &SensorsReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Code Topology Sensors ({}, top_n={})\n",
        report.ranking,
        report
            .top_n
            .map(|n| n.to_string())
            .unwrap_or_else(|| "all".to_string()),
    ));
    out.push_str(&format!(
        "Considered {} module(s); returning {}.\n\n",
        report.total_modules,
        report.modules.len()
    ));
    out.push_str("| Rank | Module | Health | Avg CC | Coupling | Instability | Reasons |\n");
    out.push_str("|------|--------|--------|--------|----------|-------------|---------|\n");
    for entry in &report.modules {
        let reasons = if entry.reasons.is_empty() {
            "-".to_string()
        } else {
            entry.reasons.join("; ")
        };
        out.push_str(&format!(
            "| {} | {} | {:.2} ({}) | {:.2} | {} (ca={}, ce={}) | {:.2} | {} |\n",
            entry.rank,
            entry.id,
            entry.health,
            entry.health_label,
            entry.avg_cyclomatic,
            entry.total_coupling,
            entry.ca,
            entry.ce,
            entry.instability,
            reasons,
        ));
    }
    out
}

fn write_output(text: &str, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Could not create {}: {e}", parent.display()))?;
        }
    }
    std::fs::write(path, text).map_err(|e| format!("Could not write {}: {e}", path.display()))
}

/// CLI entry point for `apss run code-topology sensors [<path>] [options]`.
pub(super) fn topology_sensors(args: &[String], _repo_root: &Path, verbose: bool) -> i32 {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_help();
        return 0;
    }

    let opts = match parse_options(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!("Run 'apss-dev run topology sensors --help' for usage.");
            return 3;
        }
    };

    let topology_dir = Path::new(opts.path);
    let report = match load_report(topology_dir, opts.sort_by, opts.top_n) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!("Run 'apss-dev run topology analyze' first.");
            return 1;
        }
    };

    let rendered = match opts.format {
        "json" => serde_json::to_string_pretty(&report)
            .map_err(|e| format!("Could not serialize sensors report: {e}")),
        "text" => Ok(render_text(&report)),
        other => Err(format!("Unsupported format '{other}'")),
    };
    let rendered = match rendered {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error: {e}");
            return 1;
        }
    };

    if opts.persist {
        let persist_path = topology_dir.join("sensors/sensors.json");
        let json = match serde_json::to_string_pretty(&report) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error: {e}");
                return 1;
            }
        };
        if let Err(e) = write_output(&json, &persist_path) {
            eprintln!("Error: {e}");
            return 1;
        }
        if verbose {
            eprintln!("Wrote {}", persist_path.display());
        }
    }

    if let Some(out) = opts.output {
        if let Err(e) = write_output(&rendered, Path::new(out)) {
            eprintln!("Error: {e}");
            return 1;
        }
        if verbose {
            eprintln!("Wrote {out}");
        }
    } else {
        print!("{rendered}");
        if !rendered.ends_with('\n') {
            println!();
        }
    }

    0
}

fn print_help() {
    println!("apss-dev run topology sensors [<path>] [OPTIONS]");
    println!();
    println!("Project .topology/metrics/modules.json into an agent-queryable JSON");
    println!("document ranking modules by a chosen signal. Deterministic: same");
    println!("input artifact, same output every run.");
    println!();
    println!("ARGUMENTS:");
    println!("    <path>              Path to a .topology/ directory (default: .topology)");
    println!();
    println!("OPTIONS:");
    println!("    --by <mode>         Ranking mode. One of:");
    println!("                          health       - worst health first (default)");
    println!("                          complexity   - highest avg cyclomatic first");
    println!("                          coupling     - highest ca+ce first");
    println!("                          instability  - highest instability first");
    println!("                          risk         - composite risk score first");
    println!("    --top N             Return the top N modules. Use 0 for all. Default: 10");
    println!("    --format <fmt>      Output format: json (default) or text");
    println!("    --output <file>     Write output to <file> instead of stdout");
    println!("    --persist           Also write JSON to <path>/sensors/sensors.json");
    println!("    --help              Show this help message");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MartinRecord, ModuleMetricsRecord, ModuleRecord};

    #[allow(clippy::too_many_arguments)]
    fn record(
        id: &str,
        path: &str,
        function_count: u32,
        avg_cc: f64,
        total_cc: u32,
        total_cognitive: u32,
        loc: u32,
        ca: u32,
        ce: u32,
        instability: f64,
    ) -> ModuleRecord {
        ModuleRecord {
            id: id.to_string(),
            name: id.to_string(),
            path: path.to_string(),
            languages: vec!["rust".to_string()],
            metrics: ModuleMetricsRecord {
                file_count: 1,
                function_count,
                total_cyclomatic: total_cc,
                avg_cyclomatic: avg_cc,
                total_cognitive,
                avg_cognitive: total_cognitive as f64 / function_count.max(1) as f64,
                lines_of_code: loc,
                martin: MartinRecord {
                    ca,
                    ce,
                    instability,
                    abstractness: 0.0,
                    distance_from_main_sequence: (1.0 - instability).abs(),
                },
            },
        }
    }

    fn sample_modules() -> ModulesFile {
        ModulesFile {
            schema_version: "0.1.0".to_string(),
            modules: vec![
                // healthy, low-signal
                record("utils", "src/utils/", 1, 1.0, 1, 0, 10, 3, 0, 0.0),
                // most-complex
                record("api", "src/api/", 2, 9.0, 18, 26, 103, 0, 3, 1.0),
                // ok
                record("auth", "src/auth/", 2, 5.5, 11, 16, 51, 2, 2, 0.5),
            ],
        }
    }

    #[test]
    fn parse_by_accepts_known_modes() {
        assert_eq!(SortBy::parse("health"), Some(SortBy::Health));
        assert_eq!(SortBy::parse("Complexity"), Some(SortBy::Complexity));
        assert_eq!(SortBy::parse("cc"), Some(SortBy::Complexity));
        assert_eq!(SortBy::parse("cyclomatic"), Some(SortBy::Complexity));
        assert_eq!(SortBy::parse("coupling"), Some(SortBy::Coupling));
        assert_eq!(SortBy::parse("instability"), Some(SortBy::Instability));
        assert_eq!(SortBy::parse("risk"), Some(SortBy::Risk));
        assert_eq!(SortBy::parse("nope"), None);
    }

    #[test]
    fn build_report_ranks_by_complexity_desc() {
        let modules = sample_modules();
        let report = build_report(&modules, SortBy::Complexity, None);
        assert_eq!(report.total_modules, 3);
        assert_eq!(report.modules.len(), 3);
        assert_eq!(report.modules[0].id, "api");
        assert!(report.modules[0].avg_cyclomatic >= report.modules[1].avg_cyclomatic);
        assert!(report.modules[1].avg_cyclomatic >= report.modules[2].avg_cyclomatic);
        assert_eq!(report.modules[0].rank, 1);
        assert_eq!(report.modules[2].rank, 3);
        assert_eq!(report.ranking, "complexity");
    }

    #[test]
    fn build_report_worst_health_first() {
        let modules = sample_modules();
        let report = build_report(&modules, SortBy::Health, None);
        for pair in report.modules.windows(2) {
            assert!(pair[0].health <= pair[1].health);
        }
        // api has the highest complexity and low ca, expect it near the top.
        assert_eq!(report.modules[0].id, "api");
    }

    #[test]
    fn build_report_by_coupling() {
        let modules = sample_modules();
        let report = build_report(&modules, SortBy::Coupling, None);
        // utils has ca=3, ce=0 => total 3; auth ca=2, ce=2 => 4; api ca=0, ce=3 => 3.
        // auth should lead; utils and api both total 3 so tiebreak by id ascending: "api" before "utils".
        assert_eq!(report.modules[0].id, "auth");
        assert_eq!(report.modules[1].id, "api");
        assert_eq!(report.modules[2].id, "utils");
    }

    #[test]
    fn build_report_truncates_to_top_n() {
        let modules = sample_modules();
        let report = build_report(&modules, SortBy::Complexity, Some(1));
        assert_eq!(report.total_modules, 3);
        assert_eq!(report.modules.len(), 1);
        assert_eq!(report.top_n, Some(1));
        assert_eq!(report.modules[0].id, "api");
    }

    #[test]
    fn build_report_is_deterministic_on_ties() {
        // Two modules with identical avg_cyclomatic must tiebreak on id ascending.
        let modules = ModulesFile {
            schema_version: "0.1.0".to_string(),
            modules: vec![
                record("zeta", "src/zeta/", 2, 4.0, 8, 5, 40, 1, 1, 0.5),
                record("alpha", "src/alpha/", 2, 4.0, 8, 5, 40, 1, 1, 0.5),
            ],
        };
        let a = build_report(&modules, SortBy::Complexity, None);
        let b = build_report(&modules, SortBy::Complexity, None);
        assert_eq!(a, b, "same input must produce same output");
        assert_eq!(a.modules[0].id, "alpha");
    }

    #[test]
    fn reasons_flag_high_complexity_and_coupling() {
        let modules = ModulesFile {
            schema_version: "0.1.0".to_string(),
            modules: vec![record(
                "hot", "src/hot/", 10, 12.0, 120, 200, 500, 12, 15, 0.95,
            )],
        };
        let report = build_report(&modules, SortBy::Risk, None);
        let entry = &report.modules[0];
        assert!(entry.reasons.iter().any(|r| r.contains("avg_cyclomatic")));
        assert!(entry.reasons.iter().any(|r| r.contains("coupling")));
        assert!(entry.reasons.iter().any(|r| r.contains("instability")));
    }

    #[test]
    fn parse_options_defaults() {
        let args: Vec<String> = vec![];
        let opts = parse_options(&args).unwrap();
        assert_eq!(opts.path, ".topology");
        assert_eq!(opts.sort_by, SortBy::Health);
        assert_eq!(opts.top_n, Some(10));
        assert_eq!(opts.format, "json");
        assert!(opts.output.is_none());
        assert!(!opts.persist);
    }

    #[test]
    fn parse_options_reads_all_flags() {
        let args: Vec<String> = [
            "custom",
            "--by",
            "risk",
            "--top",
            "3",
            "--format",
            "text",
            "--output",
            "out.md",
            "--persist",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let opts = parse_options(&args).unwrap();
        assert_eq!(opts.path, "custom");
        assert_eq!(opts.sort_by, SortBy::Risk);
        assert_eq!(opts.top_n, Some(3));
        assert_eq!(opts.format, "text");
        assert_eq!(opts.output, Some("out.md"));
        assert!(opts.persist);
    }

    #[test]
    fn parse_options_top_zero_means_all() {
        let args: Vec<String> = ["--top", "0"].iter().map(|s| s.to_string()).collect();
        let opts = parse_options(&args).unwrap();
        assert!(opts.top_n.is_none());
    }

    #[test]
    fn parse_options_rejects_unknown_by() {
        let args: Vec<String> = ["--by", "nope"].iter().map(|s| s.to_string()).collect();
        assert!(parse_options(&args).is_err());
    }
}
