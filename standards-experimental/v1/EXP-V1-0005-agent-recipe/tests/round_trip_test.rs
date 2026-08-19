//! Round-trip guarantee for the generate leg of EXP-V1-0005: generator output
//! always validates. This scaffolds a recipe into a temp directory and asserts
//! `validate_recipe_dir` reports zero errors. It is the strongest correctness
//! guarantee for the shape (T1) + validate (T2) + generate (T3) triad: the
//! template can never emit a non-conformant recipe without this test failing.

use agent_recipe::{scaffold_recipe, validate_recipe_dir};

#[test]
fn generated_recipe_validates_cleanly() {
    let temp = tempfile::tempdir().expect("temp dir");
    let dest = temp.path().join("generated-recipe");

    let files = scaffold_recipe("generated-recipe", &dest).expect("scaffold should succeed");
    assert!(!files.is_empty(), "generator should write files");

    let diagnostics = validate_recipe_dir(&dest);
    let codes: Vec<&str> = diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert!(
        !diagnostics.has_errors(),
        "generated recipe must validate with zero errors, got: {codes:?}"
    );
}

#[test]
fn generated_recipe_loads_with_resolved_default_agent() {
    let temp = tempfile::tempdir().expect("temp dir");
    let dest = temp.path().join("loadable-recipe");

    scaffold_recipe("loadable-recipe", &dest).expect("scaffold should succeed");

    let recipe = agent_recipe::load_recipe_dir(&dest).expect("generated recipe should load");
    assert_eq!(recipe.manifest.name, "loadable-recipe");
    assert_eq!(recipe.manifest.default_agent, "main");
    let default_agent = recipe.default_agent().expect("default agent resolves");
    assert_eq!(
        default_agent.harness,
        Some(agent_recipe::HarnessKind::Claude)
    );
}
