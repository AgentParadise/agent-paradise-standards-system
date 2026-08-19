# Expect: RECIPE_FROM_WIDENS_TOOLS

`agents/parent.yaml` permits only `Read`. `agents/child.yaml` inherits from
`parent` via `from: parent` but declares `tools: [Read, Write]`, granting
itself `Write`, which its parent does not permit. Inheritance narrows;
a child MUST NOT grant itself a tool its parent does not permit.
