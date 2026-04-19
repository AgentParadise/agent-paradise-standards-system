//! examples/fitness.toml ↔ fitness-config.schema.json drift guard.

use aps_schema_test::{collect_errors, compile, toml_to_json};

const SCHEMA: &str = include_str!(
    "../../../standards/v1/APS-V1-0002-architecture-fitness/schemas/fitness-config.schema.json"
);
const EXAMPLE: &str = include_str!(
    "../../../standards/v1/APS-V1-0002-architecture-fitness/examples/fitness.toml"
);

#[test]
fn example_fitness_toml_matches_config_schema() {
    let value = toml_to_json(EXAMPLE);
    let schema = compile(SCHEMA);
    let errors = collect_errors(&schema, &value);
    assert!(errors.is_empty(), "schema errors: {errors:#?}");
}
