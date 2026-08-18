# Expect: RECIPE_MALFORMED_HARNESS_YAML

`agents/main.yaml` declares `harness: gemini`, which is not a recognized v1
harness, so the agent manifest fails to parse.
