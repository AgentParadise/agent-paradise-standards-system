# Agentic Workspace Fixtures

Shared fixtures for the 0.2.0 operation contract. Implementations in any
language consume these files; `tests/schema_conformance.rs` is the reference
consumer.

- `valid/<schema>/`: documents that MUST satisfy `schemas/<schema>.schema.json`.
- `invalid/`: each file names a `base` success fixture plus `set` and
  `remove` JSON Pointer edits. The edited document MUST be rejected.
- `behaviour/`: scripted call sequences. Each step names the call, its
  arguments, the expected response and its schema, and the total number of
  launches after the step.
- `launch-v1/`: launch manifests with known verdicts, and the frozen 0.1.0
  launch schema used to prove 0.2.0 verdicts are unchanged.
