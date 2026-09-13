//! Persist a hypergraph evolution's cospan chain and load it back.
//!
//! `cargo run --example persist_evolution --features persist-mem`
//!
//! The endpoint is `memory`, so the store lives and dies with the process.
//! Swapping it for `rocksdb://<path>` (feature `persist-rocksdb`) is the only
//! change a durable run needs.
//!
//! The rule is `edge_split` (`{{x, y}} → {{x, z}, {z, y}}`) rather than
//! `wolfram_a_to_bb`: it rewrites an arity-2 edge into arity-2 edges, so it can
//! fire again on its own output and the run yields a chain of several cospans
//! instead of one.

use catgraph_surreal::{Result, StoreBuilder};
use irreducible::machines::hypergraph::persistence::EvolutionPersistence;
use irreducible::machines::hypergraph::{Hypergraph, HypergraphEvolution, RewriteRule};

const NAME: &str = "edge-split";

#[tokio::main]
async fn main() -> Result<()> {
    let store = StoreBuilder::new("memory")
        .namespace("irreducible")
        .database("persist_evolution")
        .connect()
        .await?;
    let persistence = EvolutionPersistence::open(store).await?;

    let evolution = HypergraphEvolution::run(
        &Hypergraph::from_edges(vec![vec![0, 1]]),
        &[RewriteRule::edge_split()],
        3,
    );
    let chain = evolution.to_cospan_chain();

    let addrs = persistence.persist_evolution(NAME, &evolution).await?;
    println!("persisted `{NAME}` — chain length {}", chain.len());
    for (position, addr) in addrs.iter().enumerate() {
        println!("  [{position}] {addr}");
    }

    let loaded = persistence.load_cospan_chain(NAME).await?;
    assert_eq!(loaded.as_deref(), Some(chain.as_slice()));
    println!(
        "loaded `{NAME}` — chain length {}, equal to the original",
        chain.len()
    );

    Ok(())
}
