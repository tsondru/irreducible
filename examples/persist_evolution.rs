//! `SurrealDB` persistence for hypergraph evolution traces.
//!
//! Demonstrates the full persist -> load -> verify lifecycle using
//! catgraph-surreal's hub-node reification pattern.
//!
//! Run: `cargo run --example persist_evolution --features persist`

#[cfg(not(feature = "persist"))]
fn main() {
    eprintln!("This example requires the `persist` feature:");
    eprintln!("  cargo run --example persist_evolution --features persist");
}

#[cfg(feature = "persist")]
#[tokio::main]
async fn main() {
    persist_demo::run().await;
}

#[cfg(feature = "persist")]
mod persist_demo {
    use irreducible::machines::hypergraph::{
        persistence::EvolutionPersistence, Hypergraph, HypergraphEvolution,
        RewriteRule as HypergraphRewriteRule,
    };
    use surrealdb::engine::local::Mem;

    pub async fn run() {
        let db = surrealdb::Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("demo").use_db("demo").await.unwrap();
        catgraph_surreal::init_schema_v2(&db).await.unwrap();

        let persist = EvolutionPersistence::new(&db);

        println!("=== Persist Cospan Chain ===\n");
        cospan_chain_demo(&persist).await;

        println!("\n=== Persist Rewrite Rule as Span ===\n");
        span_demo(&persist).await;
    }

    async fn cospan_chain_demo(persist: &EvolutionPersistence<'_>) {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![HypergraphRewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let chain = evolution.to_cospan_chain();
        println!("  Evolution: A->BB rule, 3 steps");
        println!("  Cospan chain length: {}", chain.len());

        let hub_ids = persist
            .persist_cospan_chain(&evolution, "demo_chain")
            .await
            .unwrap();
        println!("  Persisted {} hub nodes", hub_ids.len());

        // Load each back and verify structural equivalence
        for (i, hub_id) in hub_ids.iter().enumerate() {
            let loaded = persist.load_cospan(hub_id).await.unwrap();
            let original = &chain[i];
            println!(
                "  Step {i}: left_map={}, middle={}, right_map={} ok",
                loaded.left_to_middle().len(),
                loaded.middle().len(),
                loaded.right_to_middle().len(),
            );
            assert_eq!(loaded.middle().len(), original.middle().len());
        }
    }

    async fn span_demo(persist: &EvolutionPersistence<'_>) {
        let rule = HypergraphRewriteRule::wolfram_a_to_bb();
        let original = rule.to_span();
        println!(
            "  Rule: A->BB (left={}, kernel={}, right={})",
            original.left().len(),
            original.middle_pairs().len(),
            original.right().len(),
        );

        let hub_id = persist.persist_span(&rule, "a_to_bb").await.unwrap();
        println!("  Persisted as hub: {hub_id:?}");

        let loaded = persist.load_span(&hub_id).await.unwrap();
        println!(
            "  Loaded:  left={}, kernel={}, right={} ok",
            loaded.left().len(),
            loaded.middle_pairs().len(),
            loaded.right().len(),
        );
        assert_eq!(loaded.left().len(), original.left().len());
    }
}
