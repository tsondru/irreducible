//! End-to-end persistence behaviour against the in-memory engine.
//!
//! Needs a real engine compiled in, so the whole file is gated on
//! `persist-mem`. Each test opens its own `memory` endpoint under its own
//! namespace and database, so nothing here shares state.

#![cfg(feature = "persist-mem")]

use catgraph::cospan::Cospan;
use catgraph_surreal::{CospanStore, DocStore, Store, StoreBuilder, StoreError};
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

/// `edge_split` (`{{x,y}} → {{x,z},{z,y}}`) re-fires on its own output, so
/// three steps give a three-cospan chain that must come back whole.
#[tokio::test]
async fn a_multi_step_evolution_round_trips() {
    let (_store, persistence) = persistence("irreducible_test", "multistep").await;

    let evolution = HypergraphEvolution::run(
        &Hypergraph::from_edges(vec![vec![0, 1]]),
        &[RewriteRule::edge_split()],
        3,
    );
    let chain = evolution.to_cospan_chain();
    assert_eq!(chain.len(), 3, "edge_split fires at every step");

    let addrs = persistence
        .persist_evolution("split", &evolution)
        .await
        .expect("persisting an evolution");
    assert_eq!(addrs.len(), 3);

    let loaded = persistence
        .load_cospan_chain("split")
        .await
        .expect("loading a chain that was just written");
    assert_eq!(loaded.as_deref(), Some(chain.as_slice()));
}

/// A duplicate presentation in the middle of a chain keeps its position: the
/// addresses read `[a, a, b]`, the loaded chain has three entries whose
/// canonical forms match the written ones, and the middle entry reloads as the
/// stored presentation.
#[tokio::test]
async fn a_duplicate_mid_chain_keeps_its_position() {
    let (_store, persistence) = persistence("irreducible_test", "midchain").await;

    let written = [wire(), wire_swapped(), braid()];
    let addrs = persistence
        .persist_cospan_chain("midchain", &written)
        .await
        .expect("the duplicate morphism resolves to the stored presentation");
    assert_eq!(addrs.len(), 3);
    assert_eq!(
        addrs[0], addrs[1],
        "the duplicate resolves to the first's address"
    );
    assert_ne!(addrs[1], addrs[2]);

    let loaded = persistence
        .load_cospan_chain("midchain")
        .await
        .expect("loading a chain that was just written")
        .expect("the chain was written under this name");
    let loaded_forms: Vec<_> = loaded.iter().map(Cospan::canonical_form).collect();
    let written_forms: Vec<_> = written.iter().map(Cospan::canonical_form).collect();
    assert_eq!(
        loaded_forms, written_forms,
        "morphism sequence is preserved"
    );
    assert_eq!(
        loaded[1],
        wire(),
        "the middle entry reloads as the stored presentation"
    );
    assert_ne!(loaded[1], wire_swapped());
}

/// Re-persisting under a name replaces the record; the cospans the old record
/// pointed at stay stored.
#[tokio::test]
async fn re_persisting_a_name_replaces_the_record() {
    let (store, persistence) = persistence("irreducible_test", "replace").await;

    let first = persistence
        .persist_cospan_chain("same-name", &[wire(), braid(), merge()])
        .await
        .expect("persisting three morphisms");
    assert_eq!(first.len(), 3);

    let second = persistence
        .persist_cospan_chain("same-name", &[braid()])
        .await
        .expect("re-persisting under the same name");
    assert_eq!(second, vec![first[1].clone()]);

    let loaded = persistence
        .load_cospan_chain("same-name")
        .await
        .expect("loading the replaced chain");
    assert_eq!(loaded.as_deref(), Some(&[braid()][..]));

    let cospans: CospanStore<u32> = CospanStore::open(store)
        .await
        .expect("opening a cospan-tier handle");
    for addr in &first {
        assert!(
            cospans.contains(addr).await.expect("existence query"),
            "{addr:?} stays stored after the record is replaced"
        );
    }
}

/// An empty name is refused before any tier is written.
#[tokio::test]
async fn an_empty_name_is_refused_before_any_write() {
    let (store, persistence) = persistence("irreducible_test", "emptyname").await;

    let error = persistence
        .persist_cospan_chain("", &[wire()])
        .await
        .expect_err("an empty name is refused");
    assert!(
        matches!(error, StoreError::Corrupt { .. }),
        "expected StoreError::Corrupt, got {error:?}"
    );

    let cospans: CospanStore<u32> = CospanStore::open(store)
        .await
        .expect("opening a cospan-tier handle");
    assert_eq!(
        cospans
            .find_by_canon(&wire())
            .await
            .expect("canonical lookup"),
        None,
        "nothing reached the cospan tier"
    );
}

/// `list_chains` reports every name filed under the chain kind and nothing
/// filed under another kind.
#[tokio::test]
async fn list_chains_reports_every_stored_name() {
    let (store, persistence) = persistence("irreducible_test", "listing").await;

    persistence
        .persist_cospan_chain("alpha", &[wire()])
        .await
        .expect("persisting alpha");
    persistence
        .persist_cospan_chain("beta", &[braid()])
        .await
        .expect("persisting beta");

    let other: DocStore<CospanChainRecord> = DocStore::open(store)
        .await
        .expect("opening a second document-tier handle");
    other
        .put(
            "gamma",
            "not_a_cospan_chain",
            &CospanChainRecord {
                kind: "not_a_cospan_chain".to_owned(),
                name: "gamma".to_owned(),
                addrs: vec![],
            },
        )
        .await
        .expect("writing a record of another kind");

    let mut names = persistence.list_chains().await.expect("listing chains");
    names.sort();
    assert_eq!(names, vec!["alpha".to_owned(), "beta".to_owned()]);

    // The load path agrees with the listing: the other kind is refused.
    let error = persistence
        .load_cospan_chain("gamma")
        .await
        .expect_err("a record of another kind is not a chain");
    assert!(
        matches!(error, StoreError::Corrupt { .. }),
        "expected StoreError::Corrupt, got {error:?}"
    );
}

/// The stored kind string is `"cospan_chain"`: a record written with that
/// literal, through a handle that never saw the crate's constant, loads and
/// lists.
#[tokio::test]
async fn the_chain_kind_is_the_literal_cospan_chain() {
    let (store, persistence) = persistence("irreducible_test", "kindliteral").await;

    let addr = persistence
        .persist_cospan_chain("seed", &[wire()])
        .await
        .expect("persisting the seed chain")
        .remove(0);

    let chains: DocStore<CospanChainRecord> = DocStore::open(store)
        .await
        .expect("opening a second document-tier handle");
    chains
        .put(
            "literal",
            "cospan_chain",
            &CospanChainRecord {
                kind: "cospan_chain".to_owned(),
                name: "literal".to_owned(),
                addrs: vec![addr.as_str().to_owned()],
            },
        )
        .await
        .expect("writing the record with the literal kind");

    let loaded = persistence
        .load_cospan_chain("literal")
        .await
        .expect("a record under the literal kind loads");
    assert_eq!(loaded.as_deref(), Some(&[wire()][..]));

    let mut names = persistence.list_chains().await.expect("listing chains");
    names.sort();
    assert_eq!(names, vec!["literal".to_owned(), "seed".to_owned()]);
}

/// A record whose `name` field disagrees with the id it is filed under is
/// corrupt.
#[tokio::test]
async fn a_record_whose_name_disagrees_with_its_id_is_corrupt() {
    let (store, persistence) = persistence("irreducible_test", "misnamed").await;

    let chains: DocStore<CospanChainRecord> = DocStore::open(store)
        .await
        .expect("opening a second document-tier handle");
    chains
        .put(
            "misnamed",
            "cospan_chain",
            &CospanChainRecord {
                kind: "cospan_chain".to_owned(),
                name: "other".to_owned(),
                addrs: vec![],
            },
        )
        .await
        .expect("writing the record itself is a plain document write");

    let error = persistence
        .load_cospan_chain("misnamed")
        .await
        .expect_err("the record's name disagrees with its id");
    assert!(
        matches!(error, StoreError::Corrupt { .. }),
        "expected StoreError::Corrupt, got {error:?}"
    );
}

/// A chain record holding a string that is not a cospan address is corrupt.
#[tokio::test]
async fn a_record_with_a_malformed_address_is_corrupt() {
    let (store, persistence) = persistence("irreducible_test", "malformed").await;

    let chains: DocStore<CospanChainRecord> = DocStore::open(store)
        .await
        .expect("opening a second document-tier handle");
    chains
        .put(
            "malformed",
            "cospan_chain",
            &CospanChainRecord {
                kind: "cospan_chain".to_owned(),
                name: "malformed".to_owned(),
                addrs: vec!["not-an-address".to_owned()],
            },
        )
        .await
        .expect("writing the record itself is a plain document write");

    let error = persistence
        .load_cospan_chain("malformed")
        .await
        .expect_err("the record holds a string that is not an address");
    assert!(
        matches!(error, StoreError::Corrupt { .. }),
        "expected StoreError::Corrupt, got {error:?}"
    );
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
                kind: "cospan_chain".to_owned(),
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
