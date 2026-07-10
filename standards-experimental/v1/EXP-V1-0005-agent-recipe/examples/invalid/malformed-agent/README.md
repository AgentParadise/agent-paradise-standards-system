# Expect: RECIPE_MALFORMED_AGENT_YAML

`agents/main.yaml` declares `agent: gemini`, which is not a recognized v1
harness, so the agent manifest fails to parse.
