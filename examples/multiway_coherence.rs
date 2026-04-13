//! Multiway coherence verification -- real non-strict SMC checks.
//!
//! Demonstrates that coherence verification uses actual multiway
//! evolution graphs from catgraph-physics, where confluence (causal
//! commutativity) is a falsifiable property.

use catgraph_physics::multiway::MultiwayEvolutionGraph;
use irreducible::multiway_coherence::{verify_all_coherence, verify_associator, verify_braiding};

fn main() {
    println!("=== Multiway Coherence Verification ===\n");

    // 1. Confluent graph: all branches merge
    println!("1. Confluent 3-fork (all branches merge):");
    let mut g: MultiwayEvolutionGraph<&str, &str> = MultiwayEvolutionGraph::new();
    let root = g.add_root("s0");
    let branches = g.add_fork(
        root,
        vec![("s1a", "e1", 0), ("s1b", "e2", 1), ("s1c", "e3", 2)],
    );
    let merge = g.add_sequential_step(branches[0], "s2", "m1");
    g.add_merge_edge(branches[1], merge, "m2");
    g.add_merge_edge(branches[2], merge, "m3");

    let forks = g.find_fork_points();
    match verify_associator(&g, forks[0]) {
        Ok(w) => println!(
            "   Associator: OK ({} pairs verified, {} diamonds)",
            w.pairs_verified, w.diamonds_found
        ),
        Err(e) => println!("   Associator: FAILED ({e})"),
    }

    let pairs = g.parallel_independent_events(forks[0]);
    for (i, (a, b)) in pairs.iter().enumerate() {
        match verify_braiding(&g, *a, *b) {
            Ok(_) => println!("   Braiding pair {i}: commutes"),
            Err(e) => println!("   Braiding pair {i}: FAILS ({e})"),
        }
    }
    println!("   All coherence errors: {:?}\n", verify_all_coherence(&g));

    // 2. Non-confluent graph: one branch diverges
    println!("2. Non-confluent 3-fork (one branch diverges):");
    let mut g2: MultiwayEvolutionGraph<&str, &str> = MultiwayEvolutionGraph::new();
    let root2 = g2.add_root("s0");
    let branches2 = g2.add_fork(
        root2,
        vec![("s1a", "e1", 0), ("s1b", "e2", 1), ("s1c", "e3", 2)],
    );
    let merge2 = g2.add_sequential_step(branches2[0], "s2", "m1");
    g2.add_merge_edge(branches2[1], merge2, "m2");
    g2.add_sequential_step(branches2[2], "diverged", "no_merge");

    let forks2 = g2.find_fork_points();
    match verify_associator(&g2, forks2[0]) {
        Ok(w) => println!("   Associator: OK ({} pairs)", w.pairs_verified),
        Err(e) => println!("   Associator: FAILED ({e})"),
    }

    let errors = verify_all_coherence(&g2);
    println!("   All coherence errors: {} found\n", errors.len());
    for e in &errors {
        println!("     - {e}");
    }
}
