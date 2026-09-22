//! Portable inventory validation remains independent from any store or harness parser.
use serde_json::json;
use session_capture::inventory::InventoryRecord;

#[test]
fn capture_receipts_match_shared_vectors() {
    use session_capture::inventory::{CaptureReceipt, QualifiedTranscript};
    #[derive(serde::Deserialize)]
    struct Vector {
        name: String,
        identity: QualifiedTranscript,
        content_hash: String,
        receipt: serde_json::Value,
        accepted: bool,
    }
    let vectors: Vec<Vector> =
        serde_json::from_str(include_str!("../fixtures/capture-receipts.json")).unwrap();
    for vector in vectors {
        let accepted =
            serde_json::from_value::<CaptureReceipt>(vector.receipt.clone()).is_ok_and(|receipt| {
                assert_eq!(serde_json::to_value(&receipt).unwrap(), vector.receipt);
                receipt.validates_capture(&vector.identity, &vector.content_hash)
            });
        assert_eq!(accepted, vector.accepted, "{}", vector.name);
    }
}

#[test]
fn qualified_storage_keys_match_shared_vectors() {
    #[derive(serde::Deserialize)]
    struct Vector {
        identity: session_capture::inventory::QualifiedTranscript,
        storage_key: String,
    }
    let vectors: Vec<Vector> =
        serde_json::from_str(include_str!("../fixtures/qualified-storage-keys.json")).unwrap();
    let mut keys = std::collections::HashSet::new();
    for vector in vectors {
        assert_eq!(vector.identity.storage_key(), vector.storage_key);
        assert!(keys.insert(vector.storage_key));
    }
}

fn node() -> serde_json::Value {
    json!({"run":{"source_instance_id":"installation","execution_id":"run"},
        "producer_id":"replicator","record_id":"record","producer_sequence":1,
        "fact":{"kind":"node","payload":{"ref":{"kind":"transcript","source_instance_id":"installation",
        "harness":"third-party-harness","local_id":"native/session %2F"},"evidence":[]}}})
}

#[test]
fn unknown_harness_and_native_spelling_round_trip() {
    let input = node();
    let record: InventoryRecord = serde_json::from_value(input.clone()).unwrap();
    record.validate().unwrap();
    assert_eq!(serde_json::to_value(record).unwrap(), input);
}

#[test]
fn namespace_mismatch_and_oversized_arrays_fail_validation() {
    let mut input = node();
    input["fact"]["payload"]["ref"]["source_instance_id"] = json!("another-installation");
    assert!(
        serde_json::from_value::<InventoryRecord>(input)
            .unwrap()
            .validate()
            .is_err()
    );
    let mut input = node();
    input["fact"] = json!({"kind":"gap","payload":{"reason":"missing", "node_keys":vec!["key";501],"evidence_ids":[]}});
    assert!(
        serde_json::from_value::<InventoryRecord>(input)
            .unwrap()
            .validate()
            .is_err()
    );
}

#[test]
fn unknown_fields_and_unknown_fact_kinds_are_not_silently_dropped() {
    let mut input = node();
    input["unexpected"] = json!(true);
    assert!(serde_json::from_value::<InventoryRecord>(input).is_err());
    let mut input = node();
    input["fact"]["kind"] = json!("unrecognized");
    assert!(serde_json::from_value::<InventoryRecord>(input).is_err());
}

#[test]
fn shared_python_rust_operation_contract() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/inventory-conformance.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let parsed = serde_json::from_value::<session_capture::inventory::InventoryOperation>(
            case["value"].clone(),
        );
        let valid = parsed
            .as_ref()
            .is_ok_and(|operation| operation.validate().is_ok());
        assert_eq!(valid, case["valid"].as_bool().unwrap(), "{}", case["name"]);
        if valid {
            assert_eq!(
                serde_json::to_value(parsed.unwrap()).unwrap(),
                case["value"],
                "{}",
                case["name"]
            );
        }
    }
}
