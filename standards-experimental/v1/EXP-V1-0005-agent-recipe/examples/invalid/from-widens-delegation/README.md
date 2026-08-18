# Expect: RECIPE_FROM_WIDENS_DELEGATION

`agents/parent.yaml` declares `allow_delegation: false`. `agents/child.yaml`
inherits from `parent` via `from: parent` but declares
`allow_delegation: true`, granting itself permission to delegate to a peer
harness that its parent does not permit. `allow_delegation` is a permission,
not a capability declaration, so it is narrowing-only via `from:` exactly
like `tools` and `mcp`: a child MAY tighten it from `true` to `false`, but
MUST NOT widen it from `false` to `true`.
