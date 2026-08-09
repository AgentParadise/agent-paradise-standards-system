# Expect: RECIPE_MCP_FROM_WIDENS_POLICY

Three-level `from:` chain: `grandparent` (no `from:`) permits `warehouse`
methods `[run_query, drop_table]`. `parent` (`from: grandparent`) narrows to
`[run_query]` only, which is legal. `child` (`from: parent`) declares
`[run_query, drop_table]` again, widening back beyond its immediate parent's
`[run_query]`.

The package's own `mcp` policy also permits `[run_query, drop_table]` on
`warehouse`, so `child`'s resolved policy is within the package ceiling and
would NOT be caught by a package-only check (`RECIPE_MCP_AGENT_WIDENS_POLICY`).
This fixture is deliberately built that way to isolate the per-`from:`-link
check: the only violation here is `child` widening relative to `parent`, not
relative to the package or to `grandparent` directly. Depth three also
proves the check runs at the link where the widening actually occurs
(parent -> child), not only between the package and the top of the chain.
