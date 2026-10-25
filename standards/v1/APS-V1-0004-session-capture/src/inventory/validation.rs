//! Enforce source ownership and bounded evidence before adapters write records.
use super::{
    BodyAvailability, CaptureDestination, InventoryFact, InventoryNodeRef, InventoryRecord,
    InventoryValidationError, NodeKind,
};

fn invalid(message: &str) -> InventoryValidationError {
    InventoryValidationError::Invalid(message.into())
}

fn identifier(value: &str, limit: usize) -> Result<(), InventoryValidationError> {
    if value.trim().is_empty() || value.contains('\0') || value.chars().count() > limit {
        return Err(invalid(
            "identifier is blank, contains NUL, or exceeds its bound",
        ));
    }
    Ok(())
}

fn bounded_fields(value: &serde_json::Value) -> Result<(), InventoryValidationError> {
    match value {
        serde_json::Value::String(text) => identifier(text, 2048)?,
        serde_json::Value::Array(items) => {
            if items.len() > 500 {
                return Err(invalid("evidence array exceeds 500 records"));
            }
            for item in items {
                bounded_fields(item)?;
            }
        }
        serde_json::Value::Object(fields) => {
            for item in fields.values() {
                bounded_fields(item)?;
            }
        }
        _ => (),
    }
    Ok(())
}

impl InventoryNodeRef {
    fn validate(&self, source: &str) -> Result<(), InventoryValidationError> {
        identifier(&self.source_instance_id, 128)?;
        identifier(&self.local_id, 2048)?;
        if self.source_instance_id != source {
            return Err(invalid("node belongs to another source namespace"));
        }
        match (&self.kind, &self.harness) {
            (NodeKind::Transcript, Some(harness)) => identifier(harness, 2048),
            (NodeKind::Platform | NodeKind::Invocation, None) => Ok(()),
            _ => Err(invalid(
                "only native transcript references require a harness",
            )),
        }
    }
}

impl InventoryRecord {
    pub fn validate(&self) -> Result<(), InventoryValidationError> {
        identifier(&self.producer_id, 128)?;
        identifier(&self.record_id, 2048)?;
        if self.producer_sequence < 0 {
            return Err(invalid("producer sequence cannot be negative"));
        }
        let value = serde_json::to_value(self).map_err(|e| invalid(&e.to_string()))?;
        bounded_fields(&value)?;
        if serde_json::to_vec(&value)
            .map_err(|e| invalid(&e.to_string()))?
            .len()
            > 1024 * 1024
        {
            return Err(invalid("inventory record exceeds 1 MiB"));
        }
        let source = self.run.source_instance_id();
        match &self.fact {
            InventoryFact::Node { r#ref, .. } => r#ref.validate(source)?,
            InventoryFact::Membership { node, run, .. } => {
                node.validate(source)?;
                if run != &self.run {
                    return Err(invalid("membership run differs from record run"));
                }
            }
            InventoryFact::Edge { parent, child, .. } => {
                parent.validate(source)?;
                child.validate(source)?;
            }
            InventoryFact::Binding {
                owner, transcript, ..
            } => {
                owner.validate(source)?;
                transcript.validate(source)?;
                if owner.kind == NodeKind::Transcript || transcript.kind != NodeKind::Transcript {
                    return Err(invalid(
                        "binding must join a platform/invocation owner to a transcript",
                    ));
                }
            }
            InventoryFact::Capture {
                node,
                receipt_sequence,
                availability,
                destination,
                archived_byte_hash,
                ..
            } => {
                node.validate(source)?;
                validate_capture(
                    *receipt_sequence,
                    availability,
                    destination,
                    archived_byte_hash.as_deref(),
                )?;
            }
            InventoryFact::Retraction { target, evidence } => {
                if target.producer_id != evidence.producer_id
                    || target.evidence_id == evidence.evidence_id
                {
                    return Err(invalid(
                        "corrections require a distinct observation from the same producer",
                    ));
                }
            }
            InventoryFact::Gap { .. } => (),
        }
        Ok(())
    }
}

fn validate_capture(
    sequence: i64,
    availability: &BodyAvailability,
    destination: &CaptureDestination,
    hash: Option<&str>,
) -> Result<(), InventoryValidationError> {
    if sequence < 0 {
        return Err(invalid("receipt sequence cannot be negative"));
    }
    if let Some(hash) = hash {
        if hash.len() != 64
            || !hash
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(invalid("archive hash must be lowercase SHA256"));
        }
    }
    if *availability == BodyAvailability::Present
        && *destination == CaptureDestination::Local
        && hash.is_none()
    {
        return Err(invalid("present local capture requires an archive hash"));
    }
    Ok(())
}
