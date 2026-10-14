# EXP-V1-0006: Agentic Workspace

**Version:** 0.2.1  
**Status:** Experimental  
**Category:** Technical

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHOULD", "SHOULD NOT",
"MAY", and "OPTIONAL" are interpreted as described in RFC 2119.

## 1. Purpose and boundary

An agent workspace is an execution boundary that accepts a declarative launch
manifest, hydrates inputs, launches one harness, exposes observable execution,
collects declared outputs, and tears down deterministically.

This experiment standardizes the boundary between an orchestrator and a
workspace provider. It does not standardize workflow authoring, scheduling,
provider SDKs, container commands, or any consumer's domain model.

An orchestrator such as Syntropic137 maps its workflow and phase objects into
the launch manifest. A provider such as Local, Docker, E2B, SBX, or a VPS
consumes the manifest through an implementation-specific adapter.

## 2. Normative artifact

The normative input artifact is a JSON document conforming to
`schemas/workspace-launch.schema.json` and the semantic rules in this document.
Its schema discriminator is `apss.workspace-launch/v1`. The operation contract
artifacts are listed in Section 5.1.1.

Unknown fields MUST be rejected. Additive evolution requires a new schema
version when an older strict reader could not accept the document.

## 3. Launch manifest

### 3.1 Identity

`execution_id` MUST be non-empty and stable for the orchestrated execution.
Providers MUST attach it to logs, results, and cleanup diagnostics.

### 3.2 Workspace request

`workspace.working_directory` MUST be relative to the workspace root and MUST
NOT contain a parent traversal component. Providers MUST NOT interpret it as a
host path.

`security_profile` is either:

- `insecure-local`: direct host filesystem and process execution. Providers
  MUST require explicit enablement and MUST reject it in production mode.
- `isolated`: the provider claims an enforced isolation boundary. Conformance
  tests for an isolated implementation MUST cover filesystem, process,
  credential, network, resource, and teardown behavior.

`provider_hint` is advisory. An orchestrator MAY omit or ignore it. Core
contract behavior MUST NOT branch on a provider brand.

### 3.3 Content hydration

Repositories declare a URL, immutable or otherwise explicit revision, and
workspace-relative destination. Providers MUST resolve the requested revision
before launching the harness and MUST report the resolved commit when Git is
used.

File and context inputs declare a source, destination, read-only intent, and
optional SHA-256. A provider MUST verify a supplied hash before launch. A
provider MUST fail before agent execution when required hydration fails.

Credentials are not content. Raw secret values MUST NOT appear in a manifest.
Providers MAY accept opaque credential references through implementation-owned
configuration outside this artifact.

### 3.4 Agent request

The agent request names a harness, optional model, prompt, and argument vector.
The harness adapter owns command construction, authentication discovery,
readiness, completion, and provider-native transcript location.

The `model` field records intent. The result MUST report the effective model
when the harness reveals it. Providers MUST NOT invent an effective model when
the harness does not reveal one.

### 3.5 Skills

Every skill entry MUST include a name, source, revision, and content digest.
Names MUST be unique within a launch. A provider MUST fail before launch when a
skill cannot be materialized or its digest differs.

The workspace standard does not define skill contents. Agent Skills and the
Vercel Skills distribution model are compatible upstream formats.

### 3.6 Tool and capability policy

`tools.allow` and `tools.deny` express requested policy. The same tool MUST NOT
appear in both. Harnesses differ in enforcement strength, so an adapter MUST
reject a policy it cannot honor. Silently accepting an unenforced tool policy
is non-conformant.

Capabilities are explicit opt-ins. An implementation MUST report unsupported
required capabilities before launch. Capability initialization, health checks,
and finalization MUST fail loudly.

### 3.7 Transcript destination

The transcript request declares a session identifier, provider-native source
format, workspace-relative destination, and whether capture is required.

Captured output MUST conform to APS-V1-0004 Session Capture. This experiment
does not redefine transcript content or normalization. When `required` is true,
a missing conformant envelope makes the workspace result unsuccessful.

### 3.8 Outputs and limits

Artifact patterns are interpreted relative to the workspace root. Providers
MUST NOT collect paths outside that root. Logs SHOULD be collected when
`collect_logs` is true, including setup and teardown diagnostics.

`timeout_seconds` MUST be greater than zero. Supported CPU, memory, and disk
limits MUST be enforced for isolated implementations. An adapter MUST reject a
requested hard limit it cannot enforce.

Metadata is string-to-string correlation data. Providers MUST preserve it in
the result but MUST NOT treat it as executable configuration.

## 4. Provider lifecycle

A conformant provider exposes these state transitions:

1. `provision`: allocate a workspace root and implementation resources.
2. `hydrate`: materialize declared content and verify integrity.
3. `execute`: launch exactly one primary harness request.
4. `observe`: expose logs, health, status, and optional interaction.
5. `collect`: return declared artifacts and transcript envelope.
6. `finalize`: run capability finalizers and flush observability.
7. `destroy`: release implementation resources.

Cancellation and timeout may interrupt `execute`, but MUST still lead through
`finalize` and `destroy`. Teardown MUST be idempotent. A failed teardown MUST be
reported and MUST NOT replace the earlier primary failure.

## 5. Provider port

Implementations MUST sit behind dependency injection. Core consumers depend on
the provider port, never Docker, E2B, SBX, VPS, or local-process APIs directly.

The port MUST support provision, hydration, execution, cancellation, status,
logs, file collection, and destruction. Interactive send, await, and capture
operations are OPTIONAL capabilities and MUST be discoverable rather than
assumed.

### 5.1 Operation contract

Added in 0.2.0. The operation contract describes what crosses the provider
port: one operation started inside a workspace, its status, its output stream,
the provider's published capabilities, the finalization outcome, and the
terminal result. It is purely additive: `apss.workspace-launch/v1` and its
schema are unchanged, and a document valid under 0.1.0 is valid under 0.2.0.

Scheduling, budgets, placement, attempt numbering, and queueing remain out of
scope (Section 1). Harness-specific formats remain in provider and harness
implementations.

#### 5.1.1 Artifacts

| Schema file (`schemas/`) | Discriminator | Defines |
|---|---|---|
| `workspace-ref.schema.json` | `apss.workspace-ref/v1` | `WorkspaceRef` |
| `content-ref.schema.json` | `apss.content-ref/v1` | `ContentRef` |
| `environment-ref.schema.json` | `apss.environment-ref/v1` | `EnvironmentRef` |
| `credential-ref.schema.json` | `apss.credential-ref/v1` | `CredentialRef`, `CredentialBinding` |
| `workspace-operation.schema.json` | `apss.workspace-operation/v1` | `OperationSpec`, `OperationRef` |
| `workspace-status.schema.json` | `apss.workspace-status/v1` | `OperationStatus`, `OperationError` |
| `workspace-observation.schema.json` | `apss.workspace-observation/v1` | `ObservationPage` |
| `workspace-provider.schema.json` | `apss.workspace-provider/v1` | `ProviderDescriptor` |
| `workspace-finalization.schema.json` | `apss.workspace-finalization/v1` | `FinalizationOutcome` |
| `workspace-result.schema.json` | `apss.workspace-result/v1` | `WorkspaceResult` |

Every object is closed: unknown fields MUST be rejected. A standalone document
MUST carry its `schema` discriminator. A copy embedded in another document
MAY omit it and, when present, it MUST equal the discriminator above.
`OperationRef`, `CredentialBinding`, and `OperationError` inside a result are
embedded types only.

A status response is exactly one of `OperationStatus` or `OperationError`,
both under `apss.workspace-status/v1`.

#### 5.1.2 References

- `WorkspaceRef` is opaque. A consumer MUST NOT parse it or derive a path from
  it. Equality is the only operation.
- `ContentRef` identity is `digest` (`sha256:<64 lowercase hex>`). `locator`
  is opaque to core consumers and resolved only by the provider. A provider
  MUST verify digest and `size_bytes` before the content is visible to the
  workload, and MUST fail hydration with `hydration_failed` on mismatch.
  `authority` is a `CredentialRef`, never a value.
- `EnvironmentRef.kind` is an open, provider-interpreted namespace, for
  example an OCI image, a template version, or a snapshot id. Core consumers
  MUST treat the ref as opaque and compare by equality only. A provider MUST
  report the effective environment in the result. Given a requested ref it
  cannot run exactly, it MUST refuse rather than substitute.
- A credential value MUST NOT appear in any APSS artifact. `CredentialRef`
  names a credential that the provider resolves through implementation-owned
  configuration (Section 3.3). `CredentialBinding` says how the resolved value
  is delivered (`env` or `file`) and where (`target`). A binding MUST NOT be
  persisted into results or observations.

#### 5.1.3 Operation rules

- `start` returns an `OperationRef` once the operation is accepted, without
  waiting for it to end. The workload's environment is exactly the provider
  defaults plus `environment` plus the resolved `credential_bindings`.
- `environment` and every `metadata` map MUST NOT carry credentials.
  Credentials go in `credential_bindings`, which is what lets an
  `OperationSpec` be persisted and logged. The schemas reject value fields on
  every typed object, and these maps share `CredentialFreeStringMap`, which
  rejects any key that names a credential: exactly `value`, or containing
  `secret`, `token`, `password`, `passwd`, `api_key`, `apikey`,
  `private_key`, `privatekey` or `credential`, in any letter case. No schema
  can recognize a secret placed under an innocuous key, so for that case this
  clause stays normative and is enforced by implementations.
- When `launch_marker` is present, the provider MUST emit `line` on the
  observation stream from a process that precedes the workload and is replaced
  by it, under the name `argv0`. Neither value may be readable by the workload
  through its environment, files, or argv after it starts. The marker is
  opaque to the provider.
- Idempotency scope is one workspace. The same `(workspace, operation_key)`
  with an equal spec MUST return the same ref and launch nothing. Concurrent
  same-key starts MUST yield exactly one launch. The same key with a different
  spec MUST fail with `conflict`. The idempotency record MUST live at least as
  long as the workspace, so a retried start whose first response was lost
  returns the original ref.
- `operation_key` is opaque to the provider. The orchestrator composes it.

#### 5.1.4 Status and error rules

- `state` is one of `pending`, `running`, `succeeded`, `failed`,
  `cancelled`, `timed_out`, `lost`. The last five are terminal.
- `state` MUST come from a channel the workload cannot write. Terminal states
  are sticky.
- A non-terminal status carries no `exit_code`, `signal`, or `ended_at`.
- `lost` is reported only when the provider has positively established that
  the resource or process is gone with no recorded outcome.
- When the provider cannot answer, `status` returns `OperationError` with kind
  `provider_unavailable`, not an `OperationStatus`. An unknown ref returns kind
  `unknown_operation`.
- `cancel` is idempotent. It returns `cancelled` only after the process and its
  descendants are confirmed gone. If the provider cannot confirm that, it MUST
  escalate to terminating the workspace's processes and report it. The report
  is `OperationError` kind `timeout` carrying the `ref`: termination was
  requested and escalated but not confirmed within the provider's bound.
  `status` MUST NOT report `cancelled` until termination is confirmed.
- `OperationError.kind` is closed for v1: `rejected_policy`,
  `unsupported_capability`, `hydration_failed`, `conflict`,
  `unknown_operation`, `timeout`, `cancelled`, `resource_lost`,
  `provider_unavailable`, `internal`. Adding a kind requires a new
  discriminator version.

#### 5.1.5 Observation rules

- `cursor` and `next_cursor` are opaque strings to consumers. A provider
  encodes a generation and a byte offset in them; offsets are monotonic within
  a generation.
- A cursor from another generation or another ref MUST return
  `OperationError` with kind `unknown_operation`. A cursor older than retention
  MUST return the oldest retained data plus `gap`.
- A serialized page MUST NOT exceed the descriptor's `max_page_bytes`, base64
  and envelope included.
- Data lost to retention MUST be reported as `gap`, never silently skipped.
  `lost_bytes` is at least 1. It MUST be exact when the descriptor declares
  `integrity: exact`, and MAY be `null` (loss detected, extent unknown) only
  when it declares `continuity_detected`.
- `state` is the operation state the provider observed when it served the
  page. `complete` is true only when that state is terminal and every retained
  byte up to the terminal offset has been returned. A read after a terminal
  state returns the remaining bytes before `complete` becomes true.
- Since 0.2.1 a page MAY carry `loss_detected: true` when its generation lost
  bytes to retention before `next_cursor`, on that page or an earlier one. The
  completing page of a drain that lost bytes MUST carry it, whether or not
  that page has a `gap` of its own. `gap` stays a per-page delta: a later page
  never repeats bytes already reported lost. The field is optional, so every
  0.2.0 page remains valid.

#### 5.1.6 Provider descriptor

A provider MUST publish its `ProviderDescriptor` before launch. It declares
which limits it can enforce (`enforceable_limits`, every limit stated), its
capabilities, its observation retention, page bound and integrity, whether it
supports interaction, and which environment kinds it accepts. A request outside
the descriptor MUST be refused with `unsupported_capability` or
`rejected_policy` before launch.

#### 5.1.7 Finalization rules

- A cancel or timeout MUST pass through finalization before destroy.
- `completed` carries no error. `failed` carries an `error` and the `stage`
  that failed (`while_running` or `before_delete`).
- `retained` means a `before_delete` failure left the data in place. It
  carries the `error` and the `WorkspaceRef` to retry against. Deletion MUST
  NOT occur in that state, and a later finalization by the same ref MUST be
  possible.
- Process completion and capture success are separate facts.

#### 5.1.8 Result rules

- `process` is a terminal `OperationStatus`.
- `success` is derived, not chosen:
  `success = process.state == succeeded and (not capture.required or
  capture.state == captured)`.
- A result whose process succeeded has no `primary_failure`.
- `teardown_failure` MUST NOT overwrite `primary_failure` (Section 4).
- `capture.state` MUST be mapped from the implementation's authoritative
  capture verdict, not re-judged.

#### 5.1.9 Behaviour fixtures

`fixtures/behaviour/` holds scripted call sequences with the response expected
at each step: idempotent start, same-key conflict, repeated cancel, cursor
after retention, late read after termination, required capture, unknown
ref, and one scenario per lifecycle hazard (5.1.11). Schema acceptance cannot
express these sequences. Implementations in any language MUST reproduce them.

Since 0.2.1 a step may also carry, additively to the fixture format:

- `given`: conditions the provider harness establishes before the call, from a
  closed vocabulary listed in `fixtures/README.md`.
- `settles_within_seconds`: the call MUST return within this bound. Every step
  with a `given` carries one.
- `expect_one_of`: in place of `expect`, the correct responses when the
  contract allows more than one. The provider MUST produce one of them.
- `descendant_alive` on an expected response: whether any workload process,
  including a detached descendant, is still alive once the call returns. The
  provider harness MUST check it. A response with `descendant_alive: true`
  never has state `cancelled`.
- `operation_records` on a step: how many operation records (idempotency and
  registry entries) the provider holds for the workspace once the call, and
  anything it blocked on, has returned. The provider harness MUST observe the
  actual count.
- `repeat_until_complete` on an `observe` step: the harness repeats the call,
  following `next_cursor`, until a page is `complete` or the call returns an
  `OperationError`. `settles_within_seconds` bounds the whole loop.
- The harness calls `file_exists` and `read_file`, with `workspace` and a
  workspace-relative `path`. A probe result has the fixture-level shape
  `fixtures/file-probe.schema.json`; a refusal is an `OperationError`.

Every `settles_within_seconds` MUST be enforced by a watchdog outside the
provider's own scheduler (another thread or process), so a call that blocks
the provider's event loop synchronously still fails its bound.

A scenario may name the `hazard` it encodes. The only contract schema change
in 0.2.1 is the optional `loss_detected` on `ObservationPage` (5.1.5).

#### 5.1.10 Reserved: `apss.workspace-launch/v2`

`apss.workspace-launch/v2` is reserved as "launch/v1 plus a requested
`environment: EnvironmentRef` and `ContentRef` entries in `content.inputs`".
It is defined, not adopted: 0.2.0 ships no v2 schema, uses the reference types
only in results and alongside operations, and every launch manifest remains
`apss.workspace-launch/v1`. A v1 manifest carrying either field is rejected.
Adoption is deferred until a remote provider requires it.

#### 5.1.11 Lifecycle hazards

Each hazard below recurred across independent provider implementations. Each
is encoded as a behaviour fixture named after it, and a provider MUST pass
every one.

- **LH-1** Double-fork orphan (`hazard-double-fork-orphan.json`): when the
  workload started a descendant that detached into its own session and was
  reparented before cancel, `cancel` MUST NOT report `cancelled` while that
  descendant is alive. It MUST kill it and confirm, or return the unconfirmed
  outcome (5.1.4).
- **LH-2** Interrupted cancel (`hazard-interrupted-cancel.json`): when the
  caller abandons `cancel` while termination is being confirmed, status MUST
  NOT become `cancelled` without confirmation, and a retried `cancel` MUST
  settle within its bound.
- **LH-3** Cancel before the launch header
  (`hazard-cancel-before-launch-header.json`): a `cancel` that arrives before
  the provider knows the workload leader MUST settle within its bound without
  waiting for the header, and MUST fail closed: terminate the workspace's
  processes and confirm, or return the unconfirmed outcome.
- **LH-4** Destroy during start (`hazard-destroy-during-start.json`): when
  `destroy` runs while a `start` is blocked in credential resolution, that
  start MUST fail without launching, no workload MUST launch after destroy,
  and no operation record MUST be retained: the provider MUST hold zero
  operation records for the workspace once the blocked start has returned.
- **LH-5** Permanently stuck intermediate state
  (`hazard-stuck-removal-timeout-settles.json`): when the backend keeps
  reporting an intermediate state such as removal in progress, a timeout drain
  (`observe` repeated until complete, with no `status` call before it) MUST
  settle within its total bound, completing with `timed_out` or failing with
  `provider_unavailable`. Status MUST NOT be required to unblock it.
- **LH-6** Drain with an observation gap
  (`hazard-drain-with-gap-reports-loss.json`): a drain that loses data to
  retention MUST report the `gap` on the page where the bytes were lost, and
  its completing page MUST carry `loss_detected: true` even when the loss was
  on an earlier page. Completion MUST NOT be reached with output silently
  truncated.
- **LH-7** Non-regular file in the workspace (`hazard-fifo-in-workspace.json`):
  `file_exists` and `read_file` on a FIFO the workload left behind MUST NOT
  block: each MUST settle within its bound, and a read MUST return a typed
  error rather than open the FIFO. `finalize` MUST settle within its bound.

## 6. Conformance profiles

### 6.1 Functional provider

A functional provider MUST pass lifecycle, hydration, manifest validation,
execution result, artifact boundary, cancellation, timeout, and teardown tests.

### 6.2 Insecure Local provider

Local MAY use a caller-owned directory and host processes. It MUST identify
itself as insecure, require explicit opt-in, refuse production configuration,
and never be selected as an implicit fallback.

### 6.3 Isolated provider

An isolated provider MUST additionally prove filesystem containment, process
containment, least-privilege credential delivery, declared network behavior,
resource enforcement, and cleanup after abnormal termination.

## 7. Required conformance evidence

The conformance suite MUST map each normative requirement to one or more test
identifiers. It MUST include success and failure fixtures, manifest round trips,
fixture provenance, a coverage matrix, and a discrepancies register.

Implementations MUST NOT claim conformance below 95 percent MUST-clause test
coverage. Promotion targets 100 percent MUST coverage.

## 8. Security

- Manifests MUST NOT contain raw credentials.
- Workspace-relative paths MUST be traversal-free.
- Input hashes MUST be checked when supplied.
- An implementation MUST reject unsupported security or tool policy.
- Local MUST fail closed outside explicit insecure-test mode.
- Isolated implementations MUST not silently degrade to Local.
- Cleanup MUST run after success, failure, cancellation, and timeout.

## 9. Relationship to Syntropic137

Syntropic137 currently has workflow, phase, agent, resolved-skill, tool-policy,
hydration, artifact, workspace-port, and session-capture types. Those remain its
domain model. A Syntropic adapter maps one executable phase into this manifest
and maps the workspace result back into Syntropic events.

APSS MUST NOT depend on Syntropic packages. Syntropic MAY depend on the APSS
crate or generated schema.

## 10. Promotion criteria

Promotion requires all of the following:

- Local, Docker, and one remote provider pass the shared conformance suite.
- Local production rejection and Docker isolation tests pass.
- Manifest schema and Rust types round-trip without loss.
- APS-V1-0004 transcript integration passes.
- All MUST clauses are represented in the coverage matrix.
- Security review has no unresolved high-severity findings.
- No open question changes the provider port or manifest shape.

Local and Docker alone prove dependency injection, but do not prove remote
provisioning semantics. They are insufficient for promotion.
