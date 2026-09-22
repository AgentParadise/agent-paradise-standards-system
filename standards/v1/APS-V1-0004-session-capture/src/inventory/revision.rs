//! Immutable manifests separate transport completeness from session coverage.
use super::{InventoryValidationError, QualifiedRun};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InventoryCoverage {
    Unknown,
    Open,
    Reconciled,
    Missing,
    Unsupported,
    Conflicting,
}
impl InventoryCoverage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Open => "open",
            Self::Reconciled => "reconciled",
            Self::Missing => "missing",
            Self::Unsupported => "unsupported",
            Self::Conflicting => "conflicting",
        }
    }
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryRevision {
    pub run: QualifiedRun,
    pub revision_id: String,
    pub parent_revision_id: Option<String>,
    pub revision_sequence: i64,
    /// Replication producer owns every record reference in this manifest.
    pub producer_id: String,
    pub sequence_high_watermark: i64,
    pub resolver_version: String,
    pub coverage: InventoryCoverage,
    pub expected_record_count: i64,
}
impl InventoryRevision {
    pub fn validate(&self) -> Result<(), InventoryValidationError> {
        for text in [&self.revision_id, &self.producer_id, &self.resolver_version]
            .into_iter()
            .chain(self.parent_revision_id.iter())
        {
            if text.trim().is_empty() || text.contains('\0') || text.chars().count() > 2048 {
                return Err(InventoryValidationError::Invalid(
                    "invalid revision identifier".into(),
                ));
            }
        }
        if self.producer_id.chars().count() > 128
            || self.revision_sequence < 1
            || self.sequence_high_watermark < 0
            || self.expected_record_count < 0
            || self.parent_revision_id.as_ref() == Some(&self.revision_id)
        {
            return Err(InventoryValidationError::Invalid(
                "invalid revision bounds or parent".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum InventoryPublication {
    Published,
    AlreadyPublished,
    PendingManifest,
    PendingRecords,
    PendingParent,
}
