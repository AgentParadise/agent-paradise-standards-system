# Expect: RECIPE_SUBAGENT_WIDENS_TOOLS

`agents/main.yaml` permits only `Read` and delegates to `helper` via
`subagents`. `agents/helper.yaml` declares `tools: [Read, Bash]`, granting
`Bash`, which its delegator does not permit. Delegation is not an escape
hatch from an agent's declared ceiling: a named subagent's resolved `tools`
MUST be within the delegating agent's own (section 4.4a).
