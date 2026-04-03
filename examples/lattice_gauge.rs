//! Lattice gauge theory for hypergraph rewriting.
//!
//! Demonstrates the gauge-theoretic interpretation of causal invariance:
//! Wilson loops, plaquette action, and holonomy on a 2D lattice.
//!
//! Run: `cargo run --example lattice_gauge`

use irreducible::machines::hypergraph::{
    plaquette_action, total_action, Hypergraph, HypergraphEvolution, HypergraphLattice,
    HypergraphRewriteGroup, RewriteRule as HypergraphRewriteRule,
};

fn main() {
    println!("=== Lattice Gauge Theory for Hypergraph Rewriting ===\n");
    wilson_loops_from_evolution();

    println!("\n=== 2D Lattice Gauge Field ===\n");
    lattice_gauge_field();

    println!("\n=== Plaquette Action ===\n");
    action_calculation();
}

/// Wilson loops from multiway hypergraph evolution.
fn wilson_loops_from_evolution() {
    let initial = Hypergraph::from_edges(vec![vec![0, 1, 2], vec![1, 2, 3]]);
    let rules = vec![HypergraphRewriteRule::wolfram_a_to_bb()];
    let evolution = HypergraphEvolution::run_multiway(&initial, &rules, 4, 50);

    let loops = evolution.find_wilson_loops();
    println!("  Rules:        A -> BB (Wolfram Physics)");
    println!("  Steps:        4 (multiway)");
    println!("  Wilson loops: {}", loops.len());

    for (i, wl) in loops.iter().enumerate() {
        println!("    Loop {i}: holonomy = {:.4}", wl.holonomy);
    }

    let invariant = evolution.is_causally_invariant();
    println!("  Causally invariant: {invariant}");
}

/// 2D lattice with gauge group and site states.
fn lattice_gauge_field() {
    let group = HypergraphRewriteGroup::new(4);
    let mut lattice: HypergraphLattice<2> = HypergraphLattice::new([3, 3], group);

    // Populate lattice sites with small hypergraphs
    for x in 0..3_usize {
        for y in 0..3_usize {
            let mut g = Hypergraph::new();
            g.add_hyperedge(vec![x, y, x + y]);
            lattice.set_state(&[x, y], g);
        }
    }

    println!("  Lattice:     3x3");
    println!("  Gauge rules: {}", lattice.group().num_rules());
    println!("  Sites:       {}", lattice.site_count());

    // Apply rewrites at a few sites
    lattice.apply_rewrite(&[0, 0], 0);
    lattice.apply_rewrite(&[1, 1], 1);
    lattice.apply_rewrite(&[2, 2], 2);
    println!("  Steps:       {}", lattice.step_count());

    // Check a Wilson loop around a plaquette
    let path: Vec<[usize; 2]> = vec![[0, 0], [1, 0], [1, 1], [0, 1]];
    let path_refs: Vec<&[usize; 2]> = path.iter().collect();
    let h = lattice.wilson_loop(&path_refs);
    println!("  Plaquette holonomy (0,0)->(1,0)->(1,1)->(0,1): {h:.4}");
    println!(
        "  Causally invariant: {}",
        lattice.is_causally_invariant(&path_refs)
    );
}

/// Plaquette action from holonomy values.
fn action_calculation() {
    let holonomies = [1.0, 0.95, 0.8, 0.5];

    for h in holonomies {
        let action = plaquette_action(h);
        println!("  holonomy = {h:.2}  ->  action = {action:.4}");
    }

    let total = total_action(&holonomies);
    println!("\n  Total action: {total:.4}");
    println!("  (Lower action = more causal invariance)");
}
