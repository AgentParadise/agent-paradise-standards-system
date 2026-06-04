//! Template scaffolding tests.
//!
//! These tests verify that templates produce valid, parseable output
//! that passes meta-standard validation.

mod fixtures;

use aps_core::templates::{ExperimentContext, StandardContext, TemplateEngine};
use aps_v1_0000_meta::{MetaStandard, Standard};
use fixtures::create_test_workspace;
use std::fs;
use std::path::Path;

/// Helper to create a minimal skeleton for standards.
fn create_standard_skeleton(dir: &Path) {
    fs::create_dir_all(dir.join("docs")).unwrap();
    fs::create_dir_all(dir.join("examples")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::create_dir_all(dir.join("agents/skills")).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();

    fs::write(
        dir.join("standard.toml"),
        r#"schema = "aps.standard/v1"

[standard]
id = "{{id}}"
name = "{{name}}"
slug = "{{slug}}"
version = "{{version}}"
category = "{{category}}"
status = "active"

[aps]
aps_major = "v1"
backwards_compatible_major_required = true

[ownership]
maintainers = ["{{#each maintainers}}{{this}}{{#unless @last}}", "{{/unless}}{{/each}}"]
"#,
    )
    .unwrap();

    fs::write(
        dir.join("Cargo.toml"),
        r#"[package]
name = "{{slug}}"
version = "{{version}}"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        dir.join("src/lib.rs"),
        "// {{name}}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_works() {}\n}\n",
    )
    .unwrap();
    fs::write(
        dir.join("docs/01_spec.md"),
        "# {{id}} — {{name}} (Canonical Specification)\n\n**Version**: {{version}}\n",
    )
    .unwrap();

    fs::write(dir.join("examples/README.md"), "# Examples\n").unwrap();
    fs::write(dir.join("examples/placeholder.toml"), "# Example\n").unwrap();
    fs::write(dir.join("agents/skills/README.md"), "# Skills\n").unwrap();
}

/// Helper to create a minimal skeleton for experiments.
fn create_experiment_skeleton(dir: &Path) {
    fs::create_dir_all(dir.join("docs")).unwrap();
    fs::create_dir_all(dir.join("examples")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::create_dir_all(dir.join("agents/skills")).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();

    fs::write(
        dir.join("experiment.toml"),
        r#"schema = "aps.experiment/v1"

[experiment]
id = "{{id}}"
name = "{{name}}"
slug = "{{slug}}"
version = "{{version}}"
category = "{{category}}"

[aps]
aps_major = "v1"

[ownership]
maintainers = ["{{#each maintainers}}{{this}}{{#unless @last}}", "{{/unless}}{{/each}}"]
"#,
    )
    .unwrap();

    fs::write(
        dir.join("Cargo.toml"),
        r#"[package]
name = "{{slug}}"
version = "{{version}}"
edition = "2021"
"#,
    )
    .unwrap();

    fs::write(
        dir.join("src/lib.rs"),
        "// {{name}} (Experimental)\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_works() {}\n}\n",
    )
    .unwrap();
    fs::write(
        dir.join("docs/01_spec.md"),
        "# {{id}} — {{name}} (Experimental Specification)\n\n**Version**: {{version}}\n",
    )
    .unwrap();

    fs::write(dir.join("examples/README.md"), "# Examples\n").unwrap();
    fs::write(dir.join("examples/placeholder.toml"), "# Example\n").unwrap();
    fs::write(dir.join("agents/skills/README.md"), "# Skills\n").unwrap();
}

#[test]
fn test_standard_template_produces_valid_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    // Create a skeleton
    let skeleton_dir = temp_dir.path().join("skeleton");
    create_standard_skeleton(&skeleton_dir);

    // Render the skeleton
    let engine = TemplateEngine::new();
    let context = StandardContext::new("APS-V1-0042", "Test Standard", "test-standard");

    let pkg_dir = temp_dir
        .path()
        .join("standards/v1/APS-V1-0042-test-standard");
    engine
        .render_skeleton(&skeleton_dir, &pkg_dir, &context)
        .expect("Template rendering should succeed");

    // Verify the output exists
    assert!(pkg_dir.exists(), "Package directory should exist");
    assert!(
        pkg_dir.join("standard.toml").exists(),
        "standard.toml should exist"
    );
    assert!(
        pkg_dir.join("Cargo.toml").exists(),
        "Cargo.toml should exist"
    );
    assert!(
        pkg_dir.join("src/lib.rs").exists(),
        "src/lib.rs should exist"
    );
    assert!(
        pkg_dir.join("docs/01_spec.md").exists(),
        "docs/01_spec.md should exist"
    );

    // Validate the output passes meta-standard validation
    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&pkg_dir);

    assert!(
        !diagnostics.has_errors(),
        "Scaffolded standard should pass validation. Errors: {:?}",
        diagnostics.errors().collect::<Vec<_>>()
    );
}

#[test]
fn test_experiment_template_produces_valid_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    // Create a skeleton
    let skeleton_dir = temp_dir.path().join("skeleton");
    create_experiment_skeleton(&skeleton_dir);

    // Render the skeleton
    let engine = TemplateEngine::new();
    let context = ExperimentContext::new("EXP-V1-0001", "Test Experiment", "test-experiment");

    let pkg_dir = temp_dir
        .path()
        .join("standards-experimental/v1/EXP-V1-0001-test-experiment");
    engine
        .render_skeleton(&skeleton_dir, &pkg_dir, &context)
        .expect("Template rendering should succeed");

    // Verify the output exists
    assert!(pkg_dir.exists(), "Package directory should exist");
    assert!(
        pkg_dir.join("experiment.toml").exists(),
        "experiment.toml should exist"
    );
    assert!(
        pkg_dir.join("Cargo.toml").exists(),
        "Cargo.toml should exist"
    );
    assert!(
        pkg_dir.join("src/lib.rs").exists(),
        "src/lib.rs should exist"
    );
    assert!(
        pkg_dir.join("docs/01_spec.md").exists(),
        "docs/01_spec.md should exist"
    );

    // Validate the output passes meta-standard validation
    let meta = MetaStandard::new();
    let diagnostics = meta.validate_package(&pkg_dir);

    assert!(
        !diagnostics.has_errors(),
        "Scaffolded experiment should pass validation. Errors: {:?}",
        diagnostics.errors().collect::<Vec<_>>()
    );
}

#[test]
fn test_template_generates_parseable_toml() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    // Create a skeleton
    let skeleton_dir = temp_dir.path().join("skeleton");
    create_standard_skeleton(&skeleton_dir);

    let engine = TemplateEngine::new();
    let context = StandardContext::new("APS-V1-0100", "TOML Parse Test", "toml-test");

    let pkg_dir = temp_dir.path().join("standards/v1/APS-V1-0100-toml-test");
    engine
        .render_skeleton(&skeleton_dir, &pkg_dir, &context)
        .unwrap();

    // Parse the generated standard.toml
    let toml_content = fs::read_to_string(pkg_dir.join("standard.toml")).unwrap();
    let parsed: toml::Value =
        toml::from_str(&toml_content).expect("Generated TOML should be valid");

    // Verify key fields
    assert_eq!(parsed["standard"]["id"].as_str().unwrap(), "APS-V1-0100");
    assert_eq!(
        parsed["standard"]["name"].as_str().unwrap(),
        "TOML Parse Test"
    );
    assert_eq!(parsed["standard"]["version"].as_str().unwrap(), "1.0.0");
}

#[test]
fn test_template_generates_parseable_cargo_toml() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_workspace(temp_dir.path());

    // Create a skeleton
    let skeleton_dir = temp_dir.path().join("skeleton");
    create_standard_skeleton(&skeleton_dir);

    let engine = TemplateEngine::new();
    let context = StandardContext::new("APS-V1-0101", "Cargo Parse Test", "cargo-test");

    let pkg_dir = temp_dir.path().join("standards/v1/APS-V1-0101-cargo-test");
    engine
        .render_skeleton(&skeleton_dir, &pkg_dir, &context)
        .unwrap();

    // Parse the generated Cargo.toml
    let toml_content = fs::read_to_string(pkg_dir.join("Cargo.toml")).unwrap();
    let parsed: toml::Value =
        toml::from_str(&toml_content).expect("Generated Cargo.toml should be valid");

    // Verify package section exists
    assert!(
        parsed.get("package").is_some(),
        "Should have [package] section"
    );
}
