//! Shared coupling-record computation for `coupling.json` emitters.
//!
//! Producers (LANG01-rust adapter, the CLI's multi-language writer) funnel
//! pre-aggregated Ca/Ce/type-count data through this function so the on-disk
//! shape is computed in exactly one place. The output is the flat projection
//! consumed by APS-V1-0002 MD01 rules.

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CouplingRecord {
    pub id: String,
    pub path: String,
    pub afferent_coupling: u32,
    pub efferent_coupling: u32,
    pub instability: f64,
    pub abstractness: f64,
    pub distance_from_main_sequence: f64,
}

/// Minimum-viable per-module input to the shared computation.
///
/// Both producers (LANG01-rust adapter and the CLI's multi-language writer)
/// already have ca/ce counts and abstract/total type counts in hand; this
/// struct is the narrowest shape over which shape-computation can be shared.
pub struct ModuleCouplingInput<'a> {
    pub id: &'a str,
    pub path: String,
    pub afferent: u32,
    pub efferent: u32,
    pub abstract_types: u32,
    pub total_types: u32,
}

/// Flat per-module Martin metrics (Ca, Ce, I, A, D) per Martin 1994, 2003.
///
/// - `I = Ce / (Ca + Ce)`, defaults to 0.5 when `Ca + Ce == 0` (no evidence)
/// - `A = abstract / total_types`, defaults to 0.0 when no types observed
/// - `D = |A + I − 1|`
pub fn compute_coupling_records(inputs: &[ModuleCouplingInput<'_>]) -> Vec<CouplingRecord> {
    inputs
        .iter()
        .map(|m| {
            let instability = if m.afferent + m.efferent > 0 {
                m.efferent as f64 / (m.afferent + m.efferent) as f64
            } else {
                0.5
            };
            let abstractness = if m.total_types > 0 {
                m.abstract_types as f64 / m.total_types as f64
            } else {
                0.0
            };
            let distance = (abstractness + instability - 1.0).abs();
            CouplingRecord {
                id: m.id.to_string(),
                path: m.path.clone(),
                afferent_coupling: m.afferent,
                efferent_coupling: m.efferent,
                instability,
                abstractness,
                distance_from_main_sequence: distance,
            }
        })
        .collect()
}
