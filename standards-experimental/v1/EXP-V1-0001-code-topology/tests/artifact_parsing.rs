//! Integration tests for parsing sample topology artifacts.

use code_topology::{CouplingMatrixFile, FunctionsFile, ModulesFile};
use std::fs;
use std::path::Path;

const SAMPLE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/sample-topology");

#[test]
fn test_parse_functions_json() {
    let path = Path::new(SAMPLE_DIR).join("metrics/functions.json");
    let content = fs::read_to_string(&path).expect("Failed to read functions.json");

    let functions_file: FunctionsFile =
        serde_json::from_str(&content).expect("Failed to parse functions.json");

    assert_eq!(functions_file.schema_version, "0.1.0");
    assert_eq!(functions_file.functions.len(), 8);

    // Check first function
    let first = &functions_file.functions[0];
    assert_eq!(first.id, "rust:auth::validator::validate_token");
    assert_eq!(first.module, "auth");
    assert_eq!(first.language, "rust");
    assert_eq!(first.metrics.cyclomatic_complexity, 8);
    assert_eq!(first.metrics.cognitive_complexity, 12);
}

#[test]
fn test_parse_modules_json() {
    let path = Path::new(SAMPLE_DIR).join("metrics/modules.json");
    let content = fs::read_to_string(&path).expect("Failed to read modules.json");

    let modules_file: ModulesFile =
        serde_json::from_str(&content).expect("Failed to parse modules.json");

    assert_eq!(modules_file.schema_version, "0.1.0");
    assert_eq!(modules_file.modules.len(), 5);

    // Check auth module
    let auth = modules_file
        .modules
        .iter()
        .find(|m| m.id == "auth")
        .expect("auth module not found");
    assert_eq!(auth.metrics.total_cyclomatic, 11);
    assert!((auth.metrics.martin.instability - 0.5).abs() < f64::EPSILON);

    // Check api module (should have I=1.0, D=0.0)
    let api = modules_file
        .modules
        .iter()
        .find(|m| m.id == "api")
        .expect("api module not found");
    assert!((api.metrics.martin.instability - 1.0).abs() < f64::EPSILON);
    assert!((api.metrics.martin.distance_from_main_sequence - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_parse_coupling_matrix_json() {
    let path = Path::new(SAMPLE_DIR).join("graphs/coupling-matrix.json");
    let content = fs::read_to_string(&path).expect("Failed to read coupling-matrix.json");

    let matrix_file: CouplingMatrixFile =
        serde_json::from_str(&content).expect("Failed to parse coupling-matrix.json");

    assert_eq!(matrix_file.schema_version, "0.1.0");
    assert_eq!(matrix_file.modules.len(), 5);
    assert_eq!(matrix_file.matrix.len(), 5);

    // Check matrix is square
    for row in &matrix_file.matrix {
        assert_eq!(row.len(), 5);
    }

    // Check diagonal is 1.0
    for i in 0..5 {
        assert!(
            (matrix_file.matrix[i][i] - 1.0).abs() < f64::EPSILON,
            "Diagonal [{i}][{i}] should be 1.0"
        );
    }

    // Check symmetry
    for i in 0..5 {
        for j in 0..5 {
            assert!(
                (matrix_file.matrix[i][j] - matrix_file.matrix[j][i]).abs() < f64::EPSILON,
                "Matrix should be symmetric at [{i}][{j}]"
            );
        }
    }

    // Check auth-crypto coupling (should be 0.75)
    let auth_idx = matrix_file
        .modules
        .iter()
        .position(|m| m == "auth")
        .unwrap();
    let crypto_idx = matrix_file
        .modules
        .iter()
        .position(|m| m == "crypto")
        .unwrap();
    assert!(
        (matrix_file.matrix[auth_idx][crypto_idx] - 0.75).abs() < f64::EPSILON,
        "auth-crypto coupling should be 0.75"
    );
}

#[test]
fn test_coupling_matrix_has_layout() {
    let path = Path::new(SAMPLE_DIR).join("graphs/coupling-matrix.json");
    let content = fs::read_to_string(&path).expect("Failed to read coupling-matrix.json");

    let matrix_file: CouplingMatrixFile =
        serde_json::from_str(&content).expect("Failed to parse coupling-matrix.json");

    let layout = matrix_file.layout.expect("Layout should be present");
    assert_eq!(layout.algorithm, "force-directed");
    assert_eq!(layout.seed, 42);
    assert_eq!(layout.positions.len(), 5);

    // Check auth position exists
    let auth_pos = layout
        .positions
        .get("auth")
        .expect("auth position should exist");
    assert!((auth_pos[0] - 1.2).abs() < f64::EPSILON);
}

#[test]
fn test_all_modules_in_coupling_matrix() {
    let modules_path = Path::new(SAMPLE_DIR).join("metrics/modules.json");
    let modules_content = fs::read_to_string(&modules_path).unwrap();
    let modules_file: ModulesFile = serde_json::from_str(&modules_content).unwrap();

    let matrix_path = Path::new(SAMPLE_DIR).join("graphs/coupling-matrix.json");
    let matrix_content = fs::read_to_string(&matrix_path).unwrap();
    let matrix_file: CouplingMatrixFile = serde_json::from_str(&matrix_content).unwrap();

    // Every module in modules.json should be in coupling matrix
    for module in &modules_file.modules {
        assert!(
            matrix_file.modules.contains(&module.id),
            "Module {} should be in coupling matrix",
            module.id
        );
    }
}

#[test]
fn test_values_in_valid_range() {
    let path = Path::new(SAMPLE_DIR).join("graphs/coupling-matrix.json");
    let content = fs::read_to_string(&path).unwrap();
    let matrix_file: CouplingMatrixFile = serde_json::from_str(&content).unwrap();

    for (i, row) in matrix_file.matrix.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            assert!(
                (0.0..=1.0).contains(&value),
                "Value at [{i}][{j}] = {value} should be in range [0, 1]"
            );
        }
    }
}

