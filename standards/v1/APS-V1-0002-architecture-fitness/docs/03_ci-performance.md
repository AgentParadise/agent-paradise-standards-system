# CI performance without weaker fitness checks

Architectural fitness latency includes tool setup, topology production, fitness
evaluation, and any consumer-owned invariant tests. Measure these separately.
APS-V1-0001 owns source analysis and artifact generation; APS-V1-0002 owns rule
evaluation. Keep an optimization in the component doing the expensive work.

## Operating guidance

- Measure successful required-check completion and total runner time separately.
  Include queue delays, setup, artifact transfer, and cold versus warm caches.
- A restored Cargo target directory does not prove a build was avoided. Check
  compilation logs. Reusable executable caches need exact source, compiler,
  platform, and build-configuration identities, without broad fallback keys.
- Always regenerate metrics for the source under review. Reusing a tool binary
  is different from reusing results. Preserve exceptions, stale-exception checks,
  rule selection, diagnostics, exit status, and downstream invariant tests.
- Compare all artifacts, including dependency graphs and coupling components.
  A faster metrics-only mode would need a separate explicit contract; silently
  omitting expensive outputs from `analyze` is not an equivalent optimization.
- Benchmark runner changes only after identifying the dominant work. Public
  repository runner-minutes do not necessarily correspond to a compute bill.

For the metric/artifact contract, see [Integration](INTEGRATION.md). General CI
optimization techniques are described in [Linear's CI case study](https://linear.app/now/ci-bottleneck-reworked).

## September 2026: reuse the syntax tree during function measurement

Syntropic137's [representative CI run](https://github.com/syntropic137/syntropic137/actions/runs/35503417134)
spent about 70 seconds compiling the APS CLI despite a target-cache hit, and
83 seconds analyzing topology and writing artifacts. The consumer cache fix is
separate from the producer optimization below.

APSS baseline: `eafd2bb4a4c5a8fbdb8d5f89e0ced34fca43148b`, release build,
Rust 1.97.1 (`8bab26f4f`), macOS ARM64. Unrelated local session-capture changes were present in the CLI
build but held fixed between measurements. Input: archived Syntropic137 commit
`5304dab36a5ca9e72ca319a7b40ab00b585ba3a4` and its pinned submodule sources.
Both builds found 1,892 files and 16,594 function matches, deduplicated to
10,395 function records across 1,427 modules. Input stayed outside both output
directories and unchanged between runs.

An 8-second `sample` CPU profile captured 6,199 main-call samples; 2,347 were
in per-function metric evaluation, including 1,631 in repeated parsing.
This is an early analysis-phase sample, not a whole-run percentage. Opportunity
score: impact 4, confidence 5, effort 1, giving 20 on impact*confidence/effort.

Previously, extraction parsed a source file once and each function's metrics
parsed that same file again. The optimized internal adapter method uses one
tree and one complexity calculator for all function metrics in that file.
Import, type, and call extraction retain their existing independent paths.
The public adapter API and metric formulas are unchanged. Tree reuse lasts
only for that call, so later source revisions cannot hit a stale tree cache.

### Evidence

The focused benchmark analyzes the snapshot's `apps/syn-api/src/syn_api/_wiring.py`
and writes all artifacts. Hyperfine, 3 warmups and 10 measured runs per binary:

| Implementation | Mean | Standard deviation |
|---|---:|---:|
| Before | 344.5 ms | 12.3 ms |
| Shared syntax tree | 97.8 ms | 9.7 ms |

That workload is 3.52 times faster. It is not a whole-CI speedup. Earlier timings
collected while compilation and other analyses were active were discarded as
contended; compare both binaries in the same uncontended measurement session.

The complete 1,892-file snapshot, including all artifact serialization, used one
warmup and three measured runs per binary:

| Implementation | Mean | Standard deviation | Median |
|---|---:|---:|---:|
| Before | 50.886 s | 1.179 s | 51.133 s |
| Shared syntax tree | 36.800 s | 0.251 s | 36.675 s |

End-to-end elapsed time fell 27.7%, or 14.1 seconds. This is a local producer
benchmark, not a hosted CI result. Three full runs are insufficient to estimate
a reliable p95. Peak resident memory reported by Hyperfine was 1.055 GB before
and 1.081 GB after; this change does not claim a memory reduction.

Raw runs: [focused benchmark](benchmarks/2026-09-22-small.json) and
[full benchmark](benchmarks/2026-09-22-full.json). The focused measurement has ten
samples per binary; the full one has three. Both include individual times,
memory readings, and exit codes.

A follow-up CPU sample no longer enters `compute_function_metrics` for each
function. Query compilation and metric traversal remain visible hotspots;
optimizing those would be a separate measured change.

### Equivalence

- Function extraction order and duplicate selection stay unchanged within each
  file. The existing cross-module HashMap iteration order is unspecified.
- Tie-breaking and formulas are unchanged. No numeric tolerances were used in
  artifact comparison, and no randomization was introduced.
- Existing coupling matrices index an unordered module list. To compare runs,
  reorder both matrix axes with the module IDs, including all component matrices.
  Sorting module IDs alone would corrupt the comparison.
- Canonicalize unordered entity arrays and JSON object keys. Exclude only the
  manifest generation timestamp. All five complete JSON artifacts match exactly
  after this normalization, including a second baseline run used to establish
  existing nondeterminism.
- Unit tests compare the optimized method against independent per-function
  parsing for Rust, Python, TypeScript, and TSX, including nested functions,
  methods, Unicode, and multiple functions per file. Another test changes the
  same path's source and checks empty-source and unsupported-language behavior.
- Fitness reports also match after removing timestamps and normalizing unordered
  violation lists. This absolute-path snapshot changes IDs relative to normal
  `analyze .` usage, so both reports fail with the same 244 violations and one
  stale exception. This proves matching failure behavior, not that the snapshot
  passes Syntropic137's gate. The existing fitness tests cover passing cases.
- The topology and fitness crate suites passed 294 tests, with six existing
  ignored documentation examples. No thresholds or exceptions were changed.

Canonical SHA-256 values for this frozen input:

| Artifact | SHA-256 |
|---|---|
| graphs/coupling-matrix.json | `ea418730239ac8c9a240a8cac8ab71ce8045dc6a0202d1993986979dfd787216` |
| graphs/dependencies.json | `4eb69ac5eb3f7242be8583d9568d0ccb0f24ae50b93077cda28b9e701df6c6f8` |
| metrics/functions.json | `5591cfffd7d1f290ab882f37424b705a983492f070b735a2e417343e1480f330` |
| metrics/modules.json | `e7a92f4ae6ff5f8c2f00b39a7eed5ff1811cea7991ce897b61d4a299a4585089` |
| metrics/slices.json | `3e2e4572435c03d47cc980d0d3a4e3497179bc508c4fff5866060382f6655def` |

Paths are embedded in the artifacts, so the checksums are specific to the frozen
`/tmp/syn-ci-optimization/input` location. Reproduction at another path should
compare before and after there, rather than expect these exact hashes.

### Reproduce and roll out

Build both revisions in release mode with the same compiler and lockfile.
Snapshot the consumer and submodules before measuring; keep output directories
outside the input. Run this with the paths to each binary substituted:

```sh
hyperfine --warmup 3 --runs 10 \
  '/path/to/before run code-topology analyze /snapshot/file.py --output /tmp/before-output' \
  '/path/to/after run code-topology analyze /snapshot/file.py --output /tmp/after-output'
```

Repeat over the complete snapshot, compare normalized artifacts, and run the
fitness engine against both sets. Preserve matching pass/fail outcomes and
diagnostics, not just metric counts. Run the topology and fitness crate tests,
workspace checks, and standard validation before shipping.

The topology package patch version is 0.3.1. Consumers must update their pinned
APSS revision or installed package after publication. Local uncommitted changes
do not update Syntropic137's CI. Revert the adapter call-site change to restore
independent parsing if a regression appears; do not relax fitness thresholds.

## Validation status

- Full `just check` passed in a clean worktree based on the APSS baseline above:
  workspace format, strict Clippy, all-target type checks, tests, release build,
  standard validation, and distribution validation.
- That clean branch passed 856 workspace tests, with 13 existing ignored cases.
  It excludes the unrelated session-capture work present during benchmarking.
- Hosted CI timings and consumer rollout are separate verification steps. The
  local benchmark is not evidence that a consumer has adopted the new version.
