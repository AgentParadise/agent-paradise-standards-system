---
name: "Migrating to APS-V1-0002.ST01 2.0.0"
description: "ST01 dependency patterns follow APS-V1-0002 2.0.0: path globs, fail-closed graph"
---

# Migrating from APS-V1-0002.ST01 1.x to 2.0.0

The ST01 patterns (`forbidden_import`, `required_import`,
`layer_enforcement`, `max_dependency_depth`, `max_fan_out`) are evaluated by
the APS-V1-0002 engine, so ST01 2.0.0 carries exactly the parent's breaking
changes. Follow the parent migration guide:
[`../../../../../evolution/major/2.0.0/migration.md`](../../../../../evolution/major/2.0.0/migration.md).

In short: regenerate `.topology/graphs/dependency-graph.json` with
code-topology 0.4.0, write `from`/`to` as path globs, give a rule written
ahead of its code an `allow_empty_reason`, and re-key exceptions by node
path.
