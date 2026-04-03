//! Integration tests for the `persist` feature.
//!
//! Run with: `cargo test --test persistence --features persist`

#![cfg(feature = "persist")]

use irreducible::machines::hypergraph::{
    persistence::EvolutionPersistence, Hypergraph, HypergraphEvolution,
    RewriteRule as HypergraphRewriteRule,
};
use surrealdb::engine::local::{Db, Mem};
use surrealdb::types::RecordId;
use surrealdb::Surreal;

async fn setup_db() -> Surreal<Db> {
    let db = Surreal::new::<Mem>(()).await.unwrap();
    db.use_ns("test").use_db("test").await.unwrap();
    catgraph_surreal::init_schema_v2(&db).await.unwrap();
    db
}

/// Cospan chain roundtrip: persist -> load each step -> verify structural equality.
#[tokio::test]
async fn cospan_chain_roundtrip() {
    let db = setup_db().await;
    let persist = EvolutionPersistence::new(&db);

    let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
    let rules = vec![HypergraphRewriteRule::wolfram_a_to_bb()];
    let evolution = HypergraphEvolution::run(&initial, &rules, 3);

    let original_chain = evolution.to_cospan_chain();
    let hub_ids = persist
        .persist_cospan_chain(&evolution, "roundtrip_test")
        .await
        .unwrap();

    assert_eq!(hub_ids.len(), original_chain.len());

    for (hub_id, original) in hub_ids.iter().zip(&original_chain) {
        let loaded = persist.load_cospan(hub_id).await.unwrap();
        assert_eq!(
            loaded.left_to_middle().len(),
            original.left_to_middle().len()
        );
        assert_eq!(
            loaded.right_to_middle().len(),
            original.right_to_middle().len()
        );
        assert_eq!(loaded.middle().len(), original.middle().len());
    }
}

/// Span roundtrip: persist rewrite rule -> load -> verify left/right/kernel sizes.
#[tokio::test]
async fn span_roundtrip() {
    let db = setup_db().await;
    let persist = EvolutionPersistence::new(&db);

    let rule = HypergraphRewriteRule::wolfram_a_to_bb();
    let original_span = rule.to_span();
    let hub_id = persist.persist_span(&rule, "span_roundtrip").await.unwrap();

    let loaded = persist.load_span(&hub_id).await.unwrap();
    assert_eq!(loaded.left().len(), original_span.left().len());
    assert_eq!(loaded.right().len(), original_span.right().len());
    assert_eq!(
        loaded.middle_pairs().len(),
        original_span.middle_pairs().len()
    );
}

/// Multiple different rules persist independently.
#[tokio::test]
async fn multiple_span_roundtrips() {
    let db = setup_db().await;
    let persist = EvolutionPersistence::new(&db);

    let rules = [
        ("wolfram_a_to_bb", HypergraphRewriteRule::wolfram_a_to_bb()),
        ("edge_split", HypergraphRewriteRule::edge_split()),
    ];

    let mut hub_ids = Vec::new();
    for (name, rule) in &rules {
        hub_ids.push(persist.persist_span(rule, name).await.unwrap());
    }

    // Load back and verify each has distinct structure
    for (i, (_, rule)) in rules.iter().enumerate() {
        let original = rule.to_span();
        let loaded = persist.load_span(&hub_ids[i]).await.unwrap();
        assert_eq!(loaded.left().len(), original.left().len());
        assert_eq!(loaded.right().len(), original.right().len());
    }
}

/// Multiway graph persistence: all edges stored, counts match.
#[tokio::test]
async fn multiway_graph_persistence() {
    let db = setup_db().await;
    let persist = EvolutionPersistence::new(&db);

    let initial = Hypergraph::from_edges(vec![vec![0, 1, 2], vec![1, 2, 3]]);
    let rules = vec![HypergraphRewriteRule::wolfram_a_to_bb()];
    let evolution = HypergraphEvolution::run_multiway(&initial, &rules, 2, 20);

    let graph = evolution.to_multiway_cospan_graph();
    let hub_ids = persist
        .persist_multiway_graph(&evolution, "multiway_test")
        .await
        .unwrap();

    assert_eq!(hub_ids.len(), graph.edges.len());

    // Each persisted hub should be loadable as a cospan
    for hub_id in &hub_ids {
        let loaded = persist.load_cospan(hub_id).await.unwrap();
        assert!(!loaded.middle().is_empty());
    }
}

/// Two independent chains in same DB don't interfere.
#[tokio::test]
async fn multi_chain_isolation() {
    let db = setup_db().await;
    let persist = EvolutionPersistence::new(&db);

    let initial_a = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
    let initial_b = Hypergraph::from_edges(vec![vec![0, 1]]);

    let rules_a = vec![HypergraphRewriteRule::wolfram_a_to_bb()];
    let rules_b = vec![HypergraphRewriteRule::edge_split()];

    let evo_a = HypergraphEvolution::run(&initial_a, &rules_a, 2);
    let evo_b = HypergraphEvolution::run(&initial_b, &rules_b, 3);

    let ids_a = persist
        .persist_cospan_chain(&evo_a, "chain_a")
        .await
        .unwrap();
    let ids_b = persist
        .persist_cospan_chain(&evo_b, "chain_b")
        .await
        .unwrap();

    assert_eq!(ids_a.len(), evo_a.to_cospan_chain().len());
    assert_eq!(ids_b.len(), evo_b.to_cospan_chain().len());

    // Each chain's IDs are distinct
    for id_a in &ids_a {
        assert!(!ids_b.contains(id_a));
    }
}

/// Loading a nonexistent cospan returns an empty cospan (no source/target edges found).
#[tokio::test]
async fn load_nonexistent_cospan_returns_empty() {
    let db = setup_db().await;
    let persist = EvolutionPersistence::new(&db);

    let fake_id = RecordId::new("hub", "does_not_exist");
    let result = persist.load_cospan(&fake_id).await.unwrap();
    // No source_of / target_of edges exist, so reconstruction yields an empty cospan
    assert!(result.is_empty());
}

/// Loading a nonexistent span returns an error.
#[tokio::test]
async fn load_nonexistent_span_errors() {
    let db = setup_db().await;
    let persist = EvolutionPersistence::new(&db);

    let fake_id = RecordId::new("hub", "does_not_exist");
    let result = persist.load_span(&fake_id).await;
    assert!(result.is_err());
}

/// Empty evolution produces empty cospan chain.
#[tokio::test]
async fn empty_evolution_produces_no_hubs() {
    let db = setup_db().await;
    let persist = EvolutionPersistence::new(&db);

    // Hypergraph with no matching rules -> 0 steps
    let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
    let rules = vec![HypergraphRewriteRule::wolfram_a_to_bb()]; // needs arity 3
    let evolution = HypergraphEvolution::run(&initial, &rules, 5);

    let chain = evolution.to_cospan_chain();
    let hub_ids = persist
        .persist_cospan_chain(&evolution, "empty_chain")
        .await
        .unwrap();
    assert_eq!(hub_ids.len(), chain.len());
}
