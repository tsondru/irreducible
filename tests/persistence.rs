//! End-to-end persistence behaviour against the in-memory engine.
//!
//! Needs a real engine compiled in, so the whole file is gated on
//! `persist-mem`. Each test opens its own `memory` endpoint under its own
//! namespace and database, so nothing here shares state.

#![cfg(feature = "persist-mem")]

use catgraph::cospan::Cospan;
use catgraph_surreal::{DocStore, Store, StoreBuilder, StoreError};
use irreducible::machines::hypergraph::persistence::{CospanChainRecord, EvolutionPersistence};
use irreducible::machines::hypergraph::{Hypergraph, HypergraphEvolution, RewriteRule};

/// A well-formed cospan address the cospan tier does not hold: 64 zeros is a
/// valid digest, and nothing this suite stores hashes to it.
const ORPHAN_ADDR: &str = "b3_0000000000000000000000000000000000000000000000000000000000000000";

async fn connect(namespace: &str, database: &str) -> Store {
    StoreBuilder::new("memory")
        .namespace(namespace)
        .database(database)
        .connect()
        .await
        .expect("connecting to the in-memory engine")
}

async fn persistence(namespace: &str, database: &str) -> (Store, EvolutionPersistence) {
    let store = connect(namespace, database).await;
    let persistence = EvolutionPersistence::open(store.clone())
        .await
        .expect("opening both tiers bootstraps and verifies their schemas");
    (store, persistence)
}

/// `1 → 1`: the domain leg lands on the apex vertex labelled 7, the codomain
/// leg on the one labelled 9.
fn wire() -> Cospan<u32> {
    Cospan::new(vec![0], vec![1], vec![7, 9]).expect("both legs are in bounds")
}

/// The same morphism, apex vertices swapped and both legs remapped to follow
/// them — a different presentation, so a different content address.
fn wire_swapped() -> Cospan<u32> {
    Cospan::new(vec![1], vec![0], vec![9, 7]).expect("both legs are in bounds")
}

/// `2 → 2`, both wires crossing.
fn braid() -> Cospan<u32> {
    Cospan::new(vec![0, 1], vec![1, 0], vec![7, 7]).expect("both legs are in bounds")
}

/// `2 → 1`, both domain wires on one apex vertex.
fn merge() -> Cospan<u32> {
    Cospan::new(vec![0, 0], vec![0], vec![7]).expect("both legs are in bounds")
}

/// The round trip: an evolution's chain comes back structurally unchanged.
///
/// The chain is **one** cospan, not three. `A→BB` has an arity-3 left pattern
/// and produces only arity-2 edges, and `find_matches` requires equal arity, so
/// the rule cannot fire a second time: `run` halts after one step whatever
/// `max_steps` says, leaving a two-node path and one interval.
///
/// A one-entry chain cannot observe ordering, so the second half of this test
/// round-trips a hand-built chain of three pairwise-distinct morphisms and
/// pins that the order written is the order read back.
#[tokio::test]
async fn an_evolution_chain_round_trips() {
    let (_store, persistence) = persistence("irreducible_test", "roundtrip").await;

    let evolution = HypergraphEvolution::run(
        &Hypergraph::from_edges(vec![vec![0, 1, 2]]),
        &[RewriteRule::wolfram_a_to_bb()],
        3,
    );
    let chain = evolution.to_cospan_chain();
    assert_eq!(chain.len(), 1, "A→BB fires once, yielding one interval");

    let addrs = persistence
        .persist_evolution("wolfram", &evolution)
        .await
        .expect("persisting an evolution");
    assert_eq!(addrs.len(), 1);

    let loaded = persistence
        .load_cospan_chain("wolfram")
        .await
        .expect("loading a chain that was just written");
    assert_eq!(loaded.as_deref(), Some(chain.as_slice()));

    // Ordering, over three distinct morphisms.
    let ordered = [wire(), braid(), merge()];
    let ordered_addrs = persistence
        .persist_cospan_chain("ordered", &ordered)
        .await
        .expect("persisting three distinct morphisms");
    assert_eq!(ordered_addrs.len(), 3);
    assert_ne!(ordered_addrs[0], ordered_addrs[1]);
    assert_ne!(ordered_addrs[1], ordered_addrs[2]);
    assert_ne!(ordered_addrs[0], ordered_addrs[2]);

    let loaded_ordered = persistence
        .load_cospan_chain("ordered")
        .await
        .expect("loading a chain that was just written");
    assert_eq!(loaded_ordered.as_deref(), Some(ordered.as_slice()));
}

/// Two presentations of one morphism: the cospan tier refuses the second, the
/// fallback resolves it to the address the first landed at, and both entries
/// reload as the stored presentation — equal to their originals as morphisms,
/// which is the contract `canonical_form` decides.
#[tokio::test]
async fn two_presentations_of_one_morphism_share_an_address() {
    let (_store, persistence) = persistence("irreducible_test", "duplicate").await;

    let first = wire();
    let second = wire_swapped();
    assert_ne!(first, second, "the two presentations differ structurally");
    assert_eq!(
        first.canonical_form(),
        second.canonical_form(),
        "the two presentations are one morphism"
    );

    let addrs = persistence
        .persist_cospan_chain("swapped", &[first.clone(), second.clone()])
        .await
        .expect("the duplicate morphism resolves to the stored presentation");
    assert_eq!(addrs.len(), 2);
    assert_eq!(
        addrs[0], addrs[1],
        "the second entry resolves to the address the first landed at"
    );

    let loaded = persistence
        .load_cospan_chain("swapped")
        .await
        .expect("loading a chain that was just written")
        .expect("the chain was written under this name");
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].canonical_form(), first.canonical_form());
    assert_eq!(loaded[1].canonical_form(), second.canonical_form());
}

/// A name nothing was stored under is absence, not an error.
#[tokio::test]
async fn an_unknown_name_loads_as_none() {
    let (_store, persistence) = persistence("irreducible_test", "absent").await;

    let loaded = persistence
        .load_cospan_chain("absent")
        .await
        .expect("an unknown name is not a failure");
    assert_eq!(loaded, None);
}

/// A chain record naming an address the cospan tier does not hold is corrupt.
/// Dropping the entry would silently change which composite the chain
/// describes, so the load fails instead of returning a shorter chain.
#[tokio::test]
async fn a_record_naming_an_unstored_cospan_is_corrupt() {
    let (store, persistence) = persistence("irreducible_test", "orphan").await;

    // Written through a second handle on the same connection, so the record
    // reaches the document tier without going through `persist_cospan_chain`.
    let chains: DocStore<CospanChainRecord> = DocStore::open(store)
        .await
        .expect("opening a second document-tier handle");
    chains
        .put(
            "orphan",
            "cospan_chain",
            &CospanChainRecord {
                name: "orphan".to_owned(),
                addrs: vec![ORPHAN_ADDR.to_owned()],
            },
        )
        .await
        .expect("writing the record itself is a plain document write");

    let error = persistence
        .load_cospan_chain("orphan")
        .await
        .expect_err("the record names a cospan that is not stored");
    assert!(
        matches!(error, StoreError::Corrupt { .. }),
        "expected StoreError::Corrupt, got {error:?}"
    );
}
