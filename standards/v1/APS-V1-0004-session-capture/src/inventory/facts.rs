//! Append-only normalized evidence received independently of transcript bodies.
use super::QualifiedRun;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    Registered,
    Corroborated,
    Candidate,
    Conflicting,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Platform,
    Invocation,
    Transcript,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryNodeRef {
    pub kind: NodeKind,
    pub source_instance_id: String,
    pub local_id: String,
    pub harness: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceReference {
    pub evidence_id: String,
    pub producer_id: String,
    pub source_revision: String,
    pub locator: String,
    pub extractor_version: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineageRelation {
    Spawn,
    Resume,
    Fork,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyAvailability {
    Present,
    Pending,
    Missing,
    Expired,
    Unknown,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureDestination {
    Local,
    Remote,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "payload",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum InventoryFact {
    Node {
        r#ref: InventoryNodeRef,
        evidence: Vec<EvidenceReference>,
    },
    Membership {
        node: InventoryNodeRef,
        run: QualifiedRun,
        phase_id: Option<String>,
        attempt_id: Option<String>,
        segment: Option<String>,
        confidence: EvidenceClass,
        evidence: Vec<EvidenceReference>,
    },
    Edge {
        parent: InventoryNodeRef,
        child: InventoryNodeRef,
        relation: LineageRelation,
        confidence: EvidenceClass,
        evidence: Vec<EvidenceReference>,
        parent_segment: Option<String>,
        child_segment: Option<String>,
    },
    Binding {
        owner: InventoryNodeRef,
        transcript: InventoryNodeRef,
        segment: Option<String>,
        confidence: EvidenceClass,
        evidence: Vec<EvidenceReference>,
    },
    Capture {
        node: InventoryNodeRef,
        availability: BodyAvailability,
        receipt_sequence: i64,
        evidence: EvidenceReference,
        destination: CaptureDestination,
        transcript_revision: Option<String>,
        archived_byte_hash: Option<String>,
    },
    Gap {
        reason: String,
        node_keys: Vec<String>,
        evidence_ids: Vec<String>,
    },
    Retraction {
        target: EvidenceReference,
        evidence: EvidenceReference,
    },
}

impl InventoryFact {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Node { .. } => "node",
            Self::Membership { .. } => "membership",
            Self::Edge { .. } => "edge",
            Self::Binding { .. } => "binding",
            Self::Gap { .. } => "gap",
            Self::Capture { .. } => "capture",
            Self::Retraction { .. } => "retraction",
        }
    }
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryRecord {
    pub run: QualifiedRun,
    pub producer_id: String,
    pub record_id: String,
    pub producer_sequence: i64,
    pub fact: InventoryFact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryIngestOutcome {
    Inserted,
    Duplicate,
}
