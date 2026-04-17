//! Unit tests for the shared coupling-record computation.
//!
//! The writers in LANG01-rust and aps-cli both funnel through
//! `compute_coupling_records`, so the Martin formula (I, A, D) is pinned here.

use code_topology::coupling::{ModuleCouplingInput, compute_coupling_records};

#[test]
fn clean_case_matches_martin_formulas() {
    let inputs = vec![ModuleCouplingInput {
        id: "m",
        path: "src/m/".into(),
        afferent: 1,
        efferent: 3,
        abstract_types: 1,
        total_types: 4,
    }];
    let out = compute_coupling_records(&inputs);
    assert_eq!(out.len(), 1);
    let r = &out[0];
    assert_eq!(r.afferent_coupling, 1);
    assert_eq!(r.efferent_coupling, 3);
    // I = 3 / (1+3) = 0.75
    assert!((r.instability - 0.75).abs() < 1e-9, "I={}", r.instability);
    // A = 1/4 = 0.25
    assert!((r.abstractness - 0.25).abs() < 1e-9, "A={}", r.abstractness);
    // D = |0.25 + 0.75 - 1| = 0
    assert!(r.distance_from_main_sequence.abs() < 1e-9);
}

#[test]
fn zero_coupling_defaults_instability_to_half() {
    let inputs = vec![ModuleCouplingInput {
        id: "leaf",
        path: "src/leaf/".into(),
        afferent: 0,
        efferent: 0,
        abstract_types: 0,
        total_types: 0,
    }];
    let r = &compute_coupling_records(&inputs)[0];
    assert!((r.instability - 0.5).abs() < 1e-9);
    assert!(r.abstractness.abs() < 1e-9);
    // D = |0 + 0.5 - 1| = 0.5
    assert!((r.distance_from_main_sequence - 0.5).abs() < 1e-9);
}

#[test]
fn zero_types_keeps_abstractness_zero() {
    // A module with imports/exports but no type declarations stays concrete.
    let inputs = vec![ModuleCouplingInput {
        id: "concrete",
        path: "src/concrete/".into(),
        afferent: 5,
        efferent: 0,
        abstract_types: 0,
        total_types: 0,
    }];
    let r = &compute_coupling_records(&inputs)[0];
    // I = 0/(5+0) = 0, A = 0, D = |0+0-1| = 1 (Zone of Pain)
    assert!(r.instability.abs() < 1e-9);
    assert!(r.abstractness.abs() < 1e-9);
    assert!((r.distance_from_main_sequence - 1.0).abs() < 1e-9);
}

#[test]
fn preserves_input_order_and_ids() {
    let inputs = vec![
        ModuleCouplingInput {
            id: "alpha",
            path: "a/".into(),
            afferent: 1,
            efferent: 1,
            abstract_types: 0,
            total_types: 1,
        },
        ModuleCouplingInput {
            id: "beta",
            path: "b/".into(),
            afferent: 2,
            efferent: 0,
            abstract_types: 1,
            total_types: 2,
        },
    ];
    let out = compute_coupling_records(&inputs);
    assert_eq!(out[0].id, "alpha");
    assert_eq!(out[0].path, "a/");
    assert_eq!(out[1].id, "beta");
    assert_eq!(out[1].path, "b/");
}
