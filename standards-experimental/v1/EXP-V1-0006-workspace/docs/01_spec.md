# EXP-V1-0006: Agentic Workspace

**Version:** 0.1.0  
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
Its schema discriminator is `apss.workspace-launch/v1`.

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
