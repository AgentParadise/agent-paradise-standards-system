# Agentic Workspace Fixtures

Shared fixtures for the 0.2.x operation contract. Implementations in any
language consume these files; `tests/schema_conformance.rs` is the reference
consumer.

- `valid/<schema>/`: documents that MUST satisfy `schemas/<schema>.schema.json`.
- `invalid/`: each file names a `base` success fixture plus `set` and
  `remove` JSON Pointer edits. The edited document MUST be rejected.
- `behaviour/`: scripted call sequences. Each step names the call, its
  arguments, the expected response and its schema, and the total number of
  launches after the step. A scenario that observes a gap seeds `output`
  (`generation`, `produced_bytes`); with the descriptor's `retention_bytes`
  this fixes the oldest retained offset, so every `lost_bytes` is derived,
  not asserted.
- `behaviour/hazard-*.json`: lifecycle hazards (spec 5.1.11). Each names its
  `hazard` id. Steps may carry `given` (conditions the provider harness
  establishes before the call), `settles_within_seconds` (the call returns
  within this bound), `expect_one_of` (each correct response, in place of
  `expect`), and `descendant_alive` on a response (whether any workload
  process, detached descendants included, survives the call). A response
  with `descendant_alive: true` is never `cancelled`. The unconfirmed cancel
  outcome is `OperationError` kind `timeout` carrying the `ref`. Conditions:
  - `detached_descendant`: the workload double-forked a descendant that
    called `setsid` and was reparented when its parent exited.
  - `caller_cancelled_during_confirmation`: the caller abandons the call
    while the provider is confirming termination.
  - `launch_header_not_seen`: the provider has not yet read the launch
    header, so it does not know the workload leader.
  - `destroyed_during_credential_resolution`: `destroy` runs while this
    start is blocked resolving a credential binding.
  - `workspace_destroyed`: the workspace has been destroyed.
  - `backend_stuck_removing`: the backend reports removal in progress on
    every query, forever.
  - `observation_gap`: output overran retention before this read.
  - `fifo_in_workspace`: the workload left a FIFO among the files the
    provider probes or reads.
- `valid/workspace-provider/small-retention.json` is a descriptor whose
  16-byte retention lets one page both report a gap and complete.
- Cursors in these fixtures use the fixture provider's encoding
  `g<generation>:<byte offset>`. Only the fixture oracle parses them; portable
  consumers treat cursors as opaque. On every page, `next_cursor` equals
  `cursor` plus `gap.lost_bytes` plus the decoded length of `data_b64`.
- `launch-v1/`: launch manifests with known verdicts, and the frozen 0.1.0
  launch schema used to prove 0.2.0 verdicts are unchanged.
