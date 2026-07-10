Shared base instructions for this recipe.

Every agent whose `system_instructions.mode` is `append` receives this text
first, followed by its own `content`. Agents with `mode: replace` ignore this
file. Put the guidance common to all agents here (project conventions, tone,
guardrails) and keep per-agent specifics in each agent's manifest.
