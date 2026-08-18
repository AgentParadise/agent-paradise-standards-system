# Expect: RECIPE_SUBAGENT_WIDENS_MCP

`agents/main.yaml` permits only the `list_issues` method on the `github`
MCP server and delegates to `helper`. `agents/helper.yaml` adds
`create_issue`, a method its delegator does not permit. The same subset
rule that governs the package and `from:` tiers governs delegation
(section 4.4a, section 7).
