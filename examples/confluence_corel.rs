//! Two confluence tests on the same hypergraph evolution, side by side.
//!
//! The same initial graph is evolved under two rule orderings. Each run's
//! merge structure is restricted to the initial vertex IDs, which gives the
//! two partitions a shared boundary and makes them comparable as corelations
//! ([`Corel::refines`] / [`Corel::coarsest_common_refinement`]).
//!
//! The Wilson-loop verdict (`is_causally_invariant`) is printed beside it for
//! contrast: it compares composite *boundaries* along branch pairs, whereas
//! the corelation compares the *identification of vertices* the two orderings
//! produce — a finer question.

use irreducible::machines::hypergraph::{
    Hypergraph, HypergraphEvolution, MergesCorelExt, MultiwayCospanExt, RewriteRule,
};

/// The initial graph's vertex IDs — the boundary both partitions restrict to.
const INITIAL_VERTICES: [u32; 3] = [0, 1, 2];

fn main() {
    let a_to_bb = RewriteRule::wolfram_a_to_bb();
    let split = RewriteRule::edge_split();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);

    let evo_a = HypergraphEvolution::run_multiway(&graph, &[a_to_bb.clone(), split.clone()], 3, 50);
    let evo_b = HypergraphEvolution::run_multiway(&graph, &[split, a_to_bb], 3, 50);

    println!("== Evolutions ==");
    println!(
        "  ordering A (A→BB, then split): {} nodes",
        evo_a.node_count()
    );
    println!(
        "  ordering B (split, then A→BB): {} nodes",
        evo_b.node_count()
    );

    // -----------------------------------------------------------------
    println!("\n== Merge partitions over the initial vertices ==");
    let cospan_a = evo_a.to_multiway_cospan_graph();
    let cospan_b = evo_b.to_multiway_cospan_graph();
    println!(
        "  merge groups: A = {}, B = {}",
        cospan_a.merge_points.len(),
        cospan_b.merge_points.len()
    );

    let corel_a = cospan_a
        .merges_corel_over(&INITIAL_VERTICES)
        .expect("corelation A builds");
    let corel_b = cospan_b
        .merges_corel_over(&INITIAL_VERTICES)
        .expect("corelation B builds");
    for (name, corel) in [("A", &corel_a), ("B", &corel_b)] {
        println!(
            "  ordering {name}: boundary {:?} → {} class(es), representatives {:?}",
            INITIAL_VERTICES,
            corel.as_cospan().middle().len(),
            corel.as_cospan().middle()
        );
    }
    assert_eq!(corel_a.as_cospan().left_to_middle().len(), 3);
    assert_eq!(corel_b.as_cospan().left_to_middle().len(), 3);

    // -----------------------------------------------------------------
    println!("\n== Comparison (corelation lattice) ==");
    let ccr = corel_a
        .coarsest_common_refinement(&corel_b)
        .expect("a shared boundary makes them comparable");
    let a_refines_b = corel_a.refines(&corel_b).expect("same boundary");
    let b_refines_a = corel_b.refines(&corel_a).expect("same boundary");
    println!("  A refines B: {a_refines_b}");
    println!("  B refines A: {b_refines_a}");
    println!(
        "  coarsest common refinement: {} class(es)",
        ccr.as_cospan().middle().len()
    );
    println!(
        "  ccr refines A: {}, ccr refines B: {}",
        ccr.refines(&corel_a).expect("same boundary"),
        ccr.refines(&corel_b).expect("same boundary")
    );

    // Mutual refinement is equality of partitions: rule order permuted the
    // exploration but not which vertices get identified.
    assert!(a_refines_b && b_refines_a, "the two orderings must agree");
    assert!(ccr.refines(&corel_a).expect("same boundary"));
    assert!(ccr.refines(&corel_b).expect("same boundary"));

    // -----------------------------------------------------------------
    println!("\n== Wilson-loop verdict, for contrast ==");
    let wilson_a = evo_a.is_causally_invariant();
    let wilson_b = evo_b.is_causally_invariant();
    let loops_a = evo_a.find_wilson_loops().len();
    let loops_b = evo_b.find_wilson_loops().len();
    println!("  ordering A: {loops_a} Wilson loop(s), causally invariant = {wilson_a}");
    println!("  ordering B: {loops_b} Wilson loop(s), causally invariant = {wilson_b}");
    println!("  The corelations agree while the Wilson check reports its own");
    println!("  verdict: the two tests measure different things.");

    // The Wilson verdict is independent of the comparison above — it is
    // pinned to this fixture, not implied by the corelations agreeing.
    assert_eq!(loops_a, 16, "Wilson loop census for ordering A");
    assert_eq!(loops_b, 16, "Wilson loop census for ordering B");
    assert!(!wilson_a, "ordering A's Wilson verdict");
    assert!(!wilson_b, "ordering B's Wilson verdict");

    println!("\nAll comparisons checked.");
}
