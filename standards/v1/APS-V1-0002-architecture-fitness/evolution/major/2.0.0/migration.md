---
name: "Migrating to APS-V1-0002 2.0.0"
description: "How to move ST01 dependency rules onto the dependency graph code-topology 0.4.0 writes"
---

# Migrating from APS-V1-0002 1.x to 2.0.0

Under 1.x every ST01 dependency rule skipped or passed without seeing an
edge (see `rationale.toml`). 2.0.0 evaluates real edges, so expect new
failures: each one is something 1.x never checked.

## 1. Regenerate the graph

Install code-topology 0.4.0 or later and rerun it before fitness:

```bash
apss run topology analyze .
apss run architecture-fitness validate .
```

`.topology/graphs/dependency-graph.json` must now exist, with
`"schema_version": "2.0.0"`. Delete any old `.topology/graphs/dependencies.json`;
nothing reads it. A missing graph now fails every dependency rule on an
active dimension with `PROMOTION_REQUIREMENT_UNMET:graphs/dependency-graph.json`.

## 2. Write matchers as path globs

Matchers select nodes by their `path`, the source file relative to the
analysis root (`src/domain/model.rs`), never by module id
(`domain::model`).

```toml
[[rules.structural]]
id = "domain-not-infra"
name = "Domain must not import infra"
pattern = "forbidden_import"
from = { path = "**/domain/**" }
to = { path = "**/infra/**" }
```

A rule whose `from` (or, for a non-circular forbidden rule or a graph-shape
rule with an explicit `to`, whose `to`) selects no node fails with
`RULE_MATCHED_NO_NODE`. Fix the glob. If the rule is written ahead of the
code it guards, say so:

```toml
allow_empty_reason = "billing crate lands in Q1; rule guards it from day one"
```

The empty match then shows in the report as an excepted violation with
that reason.

## 3. Re-key exceptions by path

Dependency and graph-shape exceptions are keyed by the reported entity, now
the node path:

```toml
[domain-not-infra."src/domain/model.rs"]
issue = "#123"
targets = ["src/infra/db.rs"]
```

`targets` now excuses only edges to the listed paths. An empty list
excuses nothing, and `targets` never excuses a cycle or a missing required
edge. An entry with neither `value` nor `targets` still excuses every
violation of the entity.

## 4. Fix configs the loader now rejects

- An invalid glob in `from`, `to` or `path_not`.
- A blank `allow_empty_reason`.
- `circular = true` on a rule whose `type` is not `forbidden`.

## 5. Know what an edge is

An edge exists for a resolved import path (`use crate::infra::db::Db;`). A
bare type use with no import line makes no edge (code-topology scanner,
issue #148), so a rule cannot yet catch that form.

## Hand-written graphs

A graph with no `schema_version` is still read in the pre-2.0.0 shape
(string nodes, `[from, to]` edges) as a temporary WORKAROUND, but every
edge endpoint must be a declared node. Move such graphs to schema 2.x; the
legacy reader will be removed.
