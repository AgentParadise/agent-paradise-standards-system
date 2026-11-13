//! `validate` command: confirm required `.topology/` artifacts exist.

/// Validate existing .topology/ artifacts.
pub(super) fn topology_validate(path: &str, _verbose: bool) -> i32 {
    use std::path::Path;

    let topology_path = Path::new(path);

    // Check required files exist
    let required = [
        "manifest.toml",
        "metrics/functions.json",
        "metrics/modules.json",
        "graphs/coupling-matrix.json",
        super::analyze::DEPENDENCY_GRAPH_FILE,
    ];

    let mut errors = 0;
    for file in required {
        let file_path = topology_path.join(file);
        if file_path.exists() {
            println!("✓ {file}");
        } else {
            println!("✗ {file} (missing)");
            errors += 1;
        }
    }

    if errors > 0 {
        println!();
        println!(
            "{errors} error(s) found. Run 'apss-dev run topology analyze' to generate artifacts."
        );
        1
    } else {
        println!();
        println!("✓ All required artifacts present");
        0
    }
}
