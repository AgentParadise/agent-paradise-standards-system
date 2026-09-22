"""Optional session-inventory/1 profile, checked against shared Rust fixtures."""

from __future__ import annotations

from hashlib import sha256
from typing import Annotated, Literal
from pydantic import (
    AfterValidator,
    BaseModel,
    ConfigDict,
    Field,
    TypeAdapter,
    model_validator,
)

PROFILE_VERSION = "session-inventory/1"


def _identifier(value: str) -> str:
    if not value.strip() or "\0" in value:
        raise ValueError("identifier must be nonblank and contain no NUL")
    value.encode("utf-8")
    return value


Identifier = Annotated[
    str, Field(strict=True, max_length=2048), AfterValidator(_identifier)
]
RoutingIdentifier = Annotated[
    str, Field(strict=True, max_length=128), AfterValidator(_identifier)
]
Sequence = Annotated[int, Field(strict=True, ge=0, le=2**63 - 1)]
EvidenceClass = Literal["registered", "corroborated", "candidate", "conflicting"]
Coverage = Literal[
    "unknown", "open", "reconciled", "missing", "unsupported", "conflicting"
]


class Model(BaseModel):
    model_config = ConfigDict(frozen=True, extra="forbid")


class QualifiedRun(Model):
    source_instance_id: RoutingIdentifier
    execution_id: RoutingIdentifier


class QualifiedTranscript(Model):
    source_instance_id: RoutingIdentifier
    harness: Identifier
    native_session_id: Identifier

    def storage_key(self) -> str:
        """Opaque locator; retain and compare the original tuple at storage."""
        digest = sha256(b"session-inventory/1:transcript\0")
        for value in (self.source_instance_id, self.harness, self.native_session_id):
            encoded = value.encode("utf-8")
            digest.update(len(encoded).to_bytes(8, "big"))
            digest.update(encoded)
        return "qts1:" + digest.hexdigest()


class CaptureReceipt(Model):
    """Durable acceptance, bound to submitted identity and original content."""

    storage_key: Annotated[str, Field(strict=True, pattern=r"^qts1:[a-f0-9]{64}$")]
    content_hash: Annotated[str, Field(strict=True, pattern=r"^sha256:[a-f0-9]{64}$")]
    stored_content_hash: Annotated[
        str, Field(strict=True, pattern=r"^sha256:[a-f0-9]{64}$")
    ]
    duplicate: Annotated[bool, Field(strict=True)]

    def validates_capture(
        self, identity: QualifiedTranscript, content_hash: str
    ) -> bool:
        return (
            self.storage_key == identity.storage_key()
            and self.content_hash == content_hash
        )


class NodeRef(Model):
    kind: Literal["platform", "invocation", "transcript"]
    source_instance_id: RoutingIdentifier
    local_id: Identifier
    harness: Identifier | None = None

    @model_validator(mode="after")
    def valid_harness(self) -> NodeRef:
        if (self.kind == "transcript") != (self.harness is not None):
            raise ValueError("only transcript references require a harness")
        return self


class EvidenceReference(Model):
    evidence_id: Identifier
    producer_id: Identifier
    source_revision: Identifier
    locator: Identifier
    extractor_version: Identifier


Evidence = Annotated[tuple[EvidenceReference, ...], Field(max_length=500)]
Identifiers = Annotated[tuple[Identifier, ...], Field(max_length=500)]


class Node(Model):
    ref: NodeRef
    evidence: Evidence


class Membership(Model):
    node: NodeRef
    run: QualifiedRun
    phase_id: Identifier | None = None
    attempt_id: Identifier | None = None
    segment: Identifier | None = None
    confidence: EvidenceClass
    evidence: Evidence


class Edge(Model):
    parent: NodeRef
    child: NodeRef
    relation: Literal["spawn", "resume", "fork"]
    confidence: EvidenceClass
    evidence: Evidence
    parent_segment: Identifier | None = None
    child_segment: Identifier | None = None


class Binding(Model):
    owner: NodeRef
    transcript: NodeRef
    segment: Identifier | None = None
    confidence: EvidenceClass
    evidence: Evidence

    @model_validator(mode="after")
    def valid_kinds(self) -> Binding:
        if self.owner.kind == "transcript" or self.transcript.kind != "transcript":
            raise ValueError("binding must join an owner to a transcript")
        return self


class Capture(Model):
    node: NodeRef
    availability: Literal["present", "pending", "missing", "expired", "unknown"]
    receipt_sequence: Sequence
    evidence: EvidenceReference
    destination: Literal["local", "remote"]
    transcript_revision: Identifier | None = None
    archived_byte_hash: (
        Annotated[str, Field(strict=True, pattern=r"^[a-f0-9]{64}$")] | None
    ) = None

    @model_validator(mode="after")
    def valid_archive(self) -> Capture:
        if (
            self.availability == "present"
            and self.destination == "local"
            and self.archived_byte_hash is None
        ):
            raise ValueError("present local capture requires an archive hash")
        return self


class Gap(Model):
    reason: Identifier
    node_keys: Identifiers
    evidence_ids: Identifiers


class Retraction(Model):
    target: EvidenceReference
    evidence: EvidenceReference

    @model_validator(mode="after")
    def valid_authority(self) -> Retraction:
        if (
            self.target.producer_id != self.evidence.producer_id
            or self.target.evidence_id == self.evidence.evidence_id
        ):
            raise ValueError(
                "correction requires distinct evidence from the same producer"
            )
        return self


class NodeFact(Model):
    kind: Literal["node"]
    payload: Node


class MembershipFact(Model):
    kind: Literal["membership"]
    payload: Membership


class EdgeFact(Model):
    kind: Literal["edge"]
    payload: Edge


class BindingFact(Model):
    kind: Literal["binding"]
    payload: Binding


class CaptureFact(Model):
    kind: Literal["capture"]
    payload: Capture


class GapFact(Model):
    kind: Literal["gap"]
    payload: Gap


class RetractionFact(Model):
    kind: Literal["retraction"]
    payload: Retraction


Fact = Annotated[
    NodeFact
    | MembershipFact
    | EdgeFact
    | BindingFact
    | CaptureFact
    | GapFact
    | RetractionFact,
    Field(discriminator="kind"),
]


class InventoryRecord(Model):
    run: QualifiedRun
    producer_id: RoutingIdentifier
    record_id: Identifier
    producer_sequence: Sequence
    fact: Fact

    @model_validator(mode="after")
    def valid_scope(self) -> InventoryRecord:
        payload = self.fact.payload
        refs: tuple[NodeRef, ...] = ()
        if isinstance(payload, Node):
            refs = (payload.ref,)
        elif isinstance(payload, (Membership, Capture)):
            refs = (payload.node,)
        elif isinstance(payload, Edge):
            refs = (payload.parent, payload.child)
        elif isinstance(payload, Binding):
            refs = (payload.owner, payload.transcript)
        if any(ref.source_instance_id != self.run.source_instance_id for ref in refs):
            raise ValueError("node belongs to another source namespace")
        if isinstance(payload, Membership) and payload.run != self.run:
            raise ValueError("membership run differs from record run")
        if len(self.model_dump_json().encode()) > 1024 * 1024:
            raise ValueError("inventory record exceeds 1 MiB")
        return self


class InventoryRevision(Model):
    run: QualifiedRun
    revision_id: Identifier
    parent_revision_id: Identifier | None = None
    revision_sequence: Annotated[int, Field(strict=True, ge=1, le=2**63 - 1)]
    producer_id: RoutingIdentifier
    sequence_high_watermark: Sequence
    resolver_version: Identifier
    coverage: Coverage
    expected_record_count: Sequence

    @model_validator(mode="after")
    def valid_parent(self) -> InventoryRevision:
        if self.revision_id == self.parent_revision_id:
            raise ValueError("revision cannot parent itself")
        return self


class InventoryManifestBatch(Model):
    revision: InventoryRevision
    start: Sequence
    record_ids: Annotated[Identifiers, Field(min_length=1)]

    @model_validator(mode="after")
    def valid_range(self) -> InventoryManifestBatch:
        if self.start + len(self.record_ids) > self.revision.expected_record_count:
            raise ValueError("manifest exceeds declared record count")
        return self


class RecordOperation(Model):
    operation: Literal["record"]
    body: InventoryRecord


class StageOperation(Model):
    operation: Literal["stage"]
    body: InventoryRevision


class ManifestOperation(Model):
    operation: Literal["manifest"]
    body: InventoryManifestBatch


class PublishOperation(Model):
    operation: Literal["publish"]
    body: InventoryRevision


InventoryOperation = Annotated[
    RecordOperation | StageOperation | ManifestOperation | PublishOperation,
    Field(discriminator="operation"),
]
operation_adapter: TypeAdapter[InventoryOperation] = TypeAdapter(InventoryOperation)
