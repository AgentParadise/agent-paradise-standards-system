use knowledge_store_standard::{ID, SLUG, principles};
use serde_json::Value;

const EXAMPLE: &str = include_str!("../examples/minimal/knowledge-fact.json");

#[test]
fn identity_is_the_experiment() {
    assert_eq!(ID, "EXP-V1-0007");
    assert_eq!(SLUG, "knowledge-store");
}

#[test]
fn principles_have_unique_ids_and_statements() {
    let list = principles();
    assert_eq!(list.len(), 9);
    let mut ids: Vec<&str> = list.iter().map(|p| p.id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), list.len(), "principle ids must be unique");
    for p in list {
        assert!(p.id.starts_with('P'));
        assert!(!p.title.trim().is_empty());
        assert!(!p.statement.trim().is_empty());
    }
}

#[test]
fn example_fact_carries_references_only() {
    let fact: Value = serde_json::from_str(EXAMPLE).unwrap();
    let object = fact.as_object().unwrap();
    let mut top: Vec<&str> = object.keys().map(String::as_str).collect();
    top.sort_unstable();
    assert_eq!(
        top,
        ["compartment_id", "content", "fact", "item_id", "provenance"],
        "a fact carries only allowlisted keys"
    );
    for key in ["content", "provenance"] {
        let reference = object[key].as_object().unwrap();
        let mut fields: Vec<&str> = reference.keys().map(String::as_str).collect();
        fields.sort_unstable();
        assert_eq!(
            fields,
            ["address", "alg", "key_scope", "kind", "length"],
            "{key} reference carries only allowlisted fields"
        );
        let alg = reference["alg"].as_str().unwrap();
        assert!(
            ["hmac-sha256", "sha256-ciphertext"].contains(&alg),
            "{key} address must be keyed or over ciphertext, got {alg}"
        );
        assert!(
            !reference["key_scope"].as_str().unwrap().is_empty(),
            "{key} must name its key scope"
        );
        let address = reference["address"].as_str().unwrap();
        let prefix = format!("{alg}:");
        assert!(
            address.starts_with(&prefix),
            "{key} address must be prefixed by its alg"
        );
        assert_eq!(address.len(), prefix.len() + 64);
        assert!(reference["length"].as_u64().is_some());
        for leak in ["hash", "key", "secret"] {
            assert!(
                !reference.contains_key(leak),
                "{key} must not carry {leak} (no unkeyed digest, no key material)"
            );
        }
    }
    for forbidden in ["body", "text", "title", "path", "source_path"] {
        assert!(
            !object.contains_key(forbidden),
            "a fact must not carry {forbidden}"
        );
    }
}
