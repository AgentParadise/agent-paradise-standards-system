use knowledge_store_standard::{principles, ID, SLUG};
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
    assert_eq!(list.len(), 8);
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
    for key in ["content", "provenance"] {
        let reference = object[key].as_object().unwrap();
        let hash = reference["hash"].as_str().unwrap();
        assert!(hash.starts_with("sha256:"), "{key} must be content-addressed");
        assert_eq!(hash.len(), "sha256:".len() + 64);
        assert!(reference["length"].as_u64().is_some());
    }
    for forbidden in ["body", "text", "title", "path", "source_path"] {
        assert!(
            !object.contains_key(forbidden),
            "a fact must not carry {forbidden}"
        );
    }
}
