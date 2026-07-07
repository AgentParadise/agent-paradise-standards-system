# Syntropic137 / Plan B Mapping

This document is **informative**: it describes how a recipe directory is
consumed by an executor, using the Syntropic137 / Plan B (`itmux run`) executor
as the reference consumer. None of the consumer-side types below are defined by
this standard; this standard defines only the recipe directory.

## A recipe directory maps to an `AgentRunSpec`

A consumer combines a recipe with the runtime concerns the recipe deliberately
omits (task, credentials, limits, observability) to form a run specification.
The key point: the recipe is referenced by its **directory path**, not inlined.

```text
AgentRunSpec {
    recipe: PathBuf,      // <-- the recipe DIRECTORY path (this standard's artifact)
    task: ...,            // consumer concern (not in the recipe)
    credentials: ...,     // consumer concern (never in the recipe)
    limits: ...,          // consumer concern
    observability: ...,   // consumer concern
}
```

The `recipe` field is a path to the directory (the one containing
`recipe.yaml`). The consumer does not re-encode the recipe's fields into its own
schema; it holds the path and calls the loader.

## The consumer flow

A consumer such as `itmux run` (Plan B) processes an `AgentRunSpec` like this:

1. **Load** - call `agent_recipe::load_recipe_dir(&spec.recipe)`. This is the
   single source of truth for parsing the directory; the consumer depends on
   this crate (a git-pinned Cargo dependency, per the plan) rather than
   re-implementing the shape.
2. **Resolve the entry point** - the loaded `Recipe` exposes
   `recipe.default_agent()`, the `AgentManifest` named by
   `recipe.yaml: default_agent`. That agent's `agent` field selects the harness,
   `model` selects the model + effort, and `resolved_system(agent, system_md)`
   computes the final system prompt (merging `SYSTEM.md` with the agent's
   `system_instructions` per the append/replace rules).
3. **Resolve skills** - map the entry agent's `skills` to plugin-dir paths in
   listed order (a `skills/<ref>/` subdir of the recipe if present, else the ref
   as-is). Plan B maps these to `claude_plugin_dirs`, preserving order.
4. **Run** - launch the selected harness with the resolved model, effort, system
   prompt, and skills, against the run's `task`.

## Subagents are validated-only in v1

An entry agent may list `subagents:` (other agents in the same recipe). In v1
these are **validated only, not executed**: `validate_recipe_dir` confirms each
`subagents:` entry resolves to a sibling `agents/<name>.yaml`
(`RECIPE_SUBAGENT_UNRESOLVED` otherwise), but this standard does not define how a
consumer should orchestrate delegation between an agent and its subagents. A v1
consumer therefore runs the `default_agent`; multi-agent execution of subagents
is a planned follow-on.

## What the consumer must NOT do

- Do not re-implement the recipe shape or hand-maintain a parallel schema; call
  `load_recipe_dir`.
- Do not read credentials, task input, or infrastructure config from the recipe;
  those are `AgentRunSpec` fields the recipe intentionally omits.
- Do not treat `tools` as executable; they are references only (see
  [pi-compatibility](./02-pi-compatibility.md)).
