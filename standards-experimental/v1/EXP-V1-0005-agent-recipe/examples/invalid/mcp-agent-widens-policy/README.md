# Expect: RECIPE_MCP_AGENT_WIDENS_POLICY

`recipe.yaml` (the package tier) permits only `run_query` on the `warehouse`
MCP server. `agents/main.yaml` also names `warehouse` but grants itself
`drop_table` in addition, which the package does not permit. Permission
narrows monotonically from package to agent; an agent MUST NOT grant itself
a method the package does not.
