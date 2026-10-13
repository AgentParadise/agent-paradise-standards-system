//! Knowledge Store principles (EXPERIMENTAL placeholder).
//!
//! EXP-V1-0007 records the principles of an event-sourced, privacy-first
//! personal knowledge store as data, so they can be printed, tested and
//! iterated on while a reference implementation proves them. Nothing here
//! validates a consumer's repository; see `docs/01_spec.md`.

use serde::Serialize;

pub const ID: &str = "EXP-V1-0007";
pub const SLUG: &str = "knowledge-store";
pub const NAME: &str = "Knowledge Store";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// One principle of the knowledge store, as stated in `docs/01_spec.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Principle {
    pub id: &'static str,
    pub title: &'static str,
    pub statement: &'static str,
}

/// The principles, in spec order. The spec is normative; this list is its
/// machine-readable mirror.
pub fn principles() -> Vec<Principle> {
    vec![
        Principle {
            id: "P1",
            title: "Facts carry references, never content",
            statement: "A fact must not carry the owner's content, a title, a source path, or an unkeyed hash of any of them; it may carry a content-addressed reference. An append-only log cannot un-write a payload.",
        },
        Principle {
            id: "P2",
            title: "Provenance is private referenced content",
            statement: "Where an item came from is held in a referenced blob carrying a fresh random nonce, never in a fact.",
        },
        Principle {
            id: "P3",
            title: "Deletion is real",
            statement: "Deleting an item removes its bytes and leaves the log intact; a reference to deleted bytes reads as content erased, and a missing provenance record reads as origin unknown.",
        },
        Principle {
            id: "P4",
            title: "Read models are rebuildable",
            statement: "Every read model is derivable by folding the log plus the referenced blobs still present (an erased blob folds as erased), and none is the source of truth.",
        },
        Principle {
            id: "P5",
            title: "One stream per aggregate, and the item is the aggregate",
            statement: "Each aggregate instance has its own stream keyed by its entity id, which is the consistency boundary; in a knowledge store the item is the aggregate. Cross-item rules use a reservation or a read-model check, never a compartment- or corpus-wide stream.",
        },
        Principle {
            id: "P6",
            title: "The query surface is read models, never raw storage",
            statement: "Agents query read models through an API or CLI and are never given the log's storage or the object store; resolving a reference is a separate, authorizable hop.",
        },
        Principle {
            id: "P7",
            title: "The Compartment is the isolation unit",
            statement: "An item belongs to exactly one compartment, and a reader of one compartment learns nothing about another.",
        },
        Principle {
            id: "P8",
            title: "The log is built to be hash-chained",
            statement: "Facts serialize deterministically, append at an expected position, and never derive idempotency from the payload, so a hash chain with externally anchored, signed heads can be added without changing a fact.",
        },
        Principle {
            id: "P9",
            title: "The content address is opaque",
            statement: "A reference's address is a keyed MAC under a per-isolation-unit key (recommended) or a digest over ciphertext, never an unkeyed digest of plaintext; it names its algorithm and key scope, never the key. The object store checks integrity on read.",
        },
    ]
}

pub fn register(registry: &mut dyn apss_core::registry::StandardRegistry) {
    registry.register(
        apss_core::registry::RegisteredStandard {
            id: ID.into(),
            slug: SLUG.into(),
            name: NAME.into(),
            description: "Principles of an event-sourced, privacy-first knowledge store (experimental placeholder)".into(),
            version: VERSION.into(),
            commands: vec!["principles".into()],
        },
        Box::new(PrinciplesCommandHandler),
    );
}

struct PrinciplesCommandHandler;

impl apss_core::registry::CommandHandler for PrinciplesCommandHandler {
    fn execute(&self, command: &str, _args: &[String], _config: &toml::Value) -> i32 {
        if command != "principles" {
            eprintln!("usage: apss run knowledge-store principles");
            return 3;
        }
        println!("{ID} {NAME} {VERSION} (EXPERIMENTAL: nothing is enforced)");
        for p in principles() {
            println!("{}. {}\n   {}", p.id, p.title, p.statement);
        }
        0
    }

    fn commands(&self) -> Vec<apss_core::registry::CommandInfo> {
        vec![apss_core::registry::CommandInfo {
            name: "principles".into(),
            description: "Print the knowledge store principles".into(),
            usage: "principles".into(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn principles_serialize_as_json_in_spec_order() {
        let json: serde_json::Value = serde_json::to_value(principles()).unwrap();
        let ids: Vec<&str> = json
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, ["P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8", "P9"]);
    }
}
