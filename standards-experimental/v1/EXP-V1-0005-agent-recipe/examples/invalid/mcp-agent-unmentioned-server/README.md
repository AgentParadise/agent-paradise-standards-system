# Expect: RECIPE_MCP_AGENT_WIDENS_POLICY

`recipe.yaml` (the package tier) declares an `mcp` policy naming only the
`warehouse` server. `agents/main.yaml` names a different server, `reporting`,
that the package policy does not mention at all. An agent MUST NOT invent
access to a server the package never granted, even though the agent's own
policy for that server (`include: [list_reports]`) is not, on its own,
unreasonable - the package simply never authorized `reporting` for any
agent. This is a widening, not a fresh grant: the naive check "compare only
servers present in both policies" would miss it, which is why this case has
its own fixture rather than being folded into the method-widening one.
