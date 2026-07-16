# apss-v1-0001-code-topology

Code Topology and Coupling Analysis: an executable engineering standard for the
Agent Paradise Standards System (APSS).

This crate measures the architectural shape of a codebase and turns it into
inspectable artifacts and visualizations. It computes complexity, coupling, and
module-structure metrics, writes them under a `.topology/` artifact directory,
and renders up to five visualizations from that data:

- An interactive dashboard (HTML).
- A Mermaid module-dependency diagram.
- A 3D force-directed coupling graph.
- A code-city style structure view.
- An APS topology summary view.

## How it is consumed

This is a standard crate, not a standalone tool. You do not run it directly.
APSS standards are composed into a project-local CLI by the `apss` bootstrap,
which resolves the standard from crates.io, pins it in `apss.lock`, and builds a
binary that contains only the standards your project declares.

```bash
cargo install apss              # install the bootstrap CLI
apss init --standard code-topology   # declare the standard in apss.yaml (set id to APS-V1-0001)
apss install                    # resolve, pin, and build the composed binary
apss run code-topology analyze .            # analyze the codebase into .topology/
apss run code-topology viz .topology --type all   # render the visualizations
apss run code-topology sensors .topology --by health --top 10   # agent-queryable JSON signals
```

The `sensors` subcommand projects `.topology/metrics/modules.json` into a
deterministic, schema-versioned JSON document ranking modules by `health`,
`complexity`, `coupling`, `instability`, or a composite `risk` score, with a
per-module `reasons` array. Same input artifact, same output every run. Pass
`--persist` to also write the document to `<topology>/sensors/sensors.json`
alongside the visualizations.

Commit `apss.yaml` and `apss.lock`. The generated `.apss/` runtime is build
output and stays out of git.

## Substandards as cargo features

The visualizers and language analyzers ship as cargo features of this crate, so
a project can compose a smaller binary by selecting only what it needs:
`lang-rust`, `viz-dashboard`, `viz-mermaid`, `viz-3d`, and `ci-github-actions`.
APSS maps your declared substandard selection to these features automatically.

## Documentation

Full specifications, the standards catalog, and the distribution lifecycle live
in the
[agent-paradise-standards-system repository](https://github.com/AgentParadise/agent-paradise-standards-system).

## License

MIT
