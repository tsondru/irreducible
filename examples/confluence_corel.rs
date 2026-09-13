//! Two confluence tests on the same hypergraph evolution, side by side.
//!
//! A two-edge pipeline `{0,1,2}, {2,3,4}` is evolved under two rule orderings.
//! Each run's merge structure becomes a corelation over its vertex IDs:
//!
//! - over **all** vertices, the partition is far coarser than the vertex set —
//!   that is the merge structure the evolution actually found;
//! - over the **initial** vertices, the two runs share a boundary, so their
//!   partitions are comparable ([`Corel::refines`] /
//!   [`Corel::coarsest_common_refinement`]). Vertices minted during rewriting
//!   are not comparable across orderings — the same ID means different things
//!   in the two runs — so the cross-ordering comparison stays on that boundary.
//!
//! The Wilson-loop verdict (`is_causally_invariant`) is printed beside it for
//! contrast: it compares composite *boundaries* along branch pairs, whereas the
//! corelation compares which *vertices* the two orderings identify.

use irreducible::machines::hypergraph::{
    Hypergraph, HypergraphEvolution, MergesCorelExt, MultiwayCospanExt, MultiwayCospanGraph,
    RewriteRule,
};

/// The initial graph's vertex IDs — the boundary both runs share.
const INITIAL_VERTICES: [u32; 5] = [0, 1, 2, 3, 4];

/// A wider boundary reaching into rewrite-minted IDs, where the partition is
/// visibly non-discrete. Only meaningful per ordering, never across them.
const WIDE_BOUNDARY: [u32; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

fn pipeline() -> Hypergraph {
    Hypergraph::from_edges(vec![vec![0, 1, 2], vec![2, 3, 4]])
}

fn evolve(rules: &[RewriteRule]) -> MultiwayCospanGraph {
    HypergraphEvolution::run_multiway(&pipeline(), rules, 5, 100).to_multiway_cospan_graph()
}

/// Every distinct vertex ID the evolution touched.
fn vertices(graph: &MultiwayCospanGraph) -> Vec<u32> {
    let mut all: Vec<u32> = graph
        .edges
        .iter()
        .flat_map(|e| e.cospan.middle().iter().copied())
        .collect();
    all.sort_unstable();
    all.dedup();
    all
}

fn main() {
    let a_to_bb = RewriteRule::wolfram_a_to_bb();
    let split = RewriteRule::edge_split();

    let graph_a = evolve(&[a_to_bb.clone(), split.clone()]);
    let graph_b = evolve(&[split, a_to_bb]);

    // -----------------------------------------------------------------
    println!("== Merge structure over all vertices ==");
    let mut whole = Vec::new();
    for (name, graph) in [
        ("A (A→BB, then split)", &graph_a),
        ("B (split, then A→BB)", &graph_b),
    ] {
        let all = vertices(graph);
        let corel = graph.merges_corel().expect("corelation builds");
        let classes = corel.as_cospan().middle().len();
        println!(
            "  ordering {name}: {} merge group(s), {} vertices → {classes} class(es)",
            graph.merge_points.len(),
            all.len()
        );
        println!("    representatives: {:?}", corel.as_cospan().middle());
        assert!(
            classes < all.len(),
            "the evolution must identify vertices: {classes} classes over {} vertices",
            all.len()
        );
        whole.push((all.len(), classes));
    }
    // Pinned: an empty merge map would leave one class per vertex.
    assert_eq!(whole[0], (90, 9), "ordering A: (vertices, classes)");
    assert_eq!(whole[1], (89, 9), "ordering B: (vertices, classes)");

    // -----------------------------------------------------------------
    println!("\n== Per ordering, on a boundary reaching into minted IDs ==");
    let wide_a = graph_a
        .merges_corel_over(&WIDE_BOUNDARY)
        .expect("corelation A");
    let wide_b = graph_b
        .merges_corel_over(&WIDE_BOUNDARY)
        .expect("corelation B");
    for (name, corel) in [("A", &wide_a), ("B", &wide_b)] {
        println!(
            "  ordering {name}: {} vertices → {} class(es), representatives {:?}",
            WIDE_BOUNDARY.len(),
            corel.as_cospan().middle().len(),
            corel.as_cospan().middle()
        );
    }
    assert_eq!(wide_a.as_cospan().middle().len(), 6, "ordering A, wide");
    assert_eq!(wide_b.as_cospan().middle().len(), 7, "ordering B, wide");
    assert!(
        wide_a.as_cospan().middle().len() < WIDE_BOUNDARY.len(),
        "the wide boundary must not be discrete"
    );

    // -----------------------------------------------------------------
    println!("\n== Cross-ordering comparison, on the shared boundary ==");
    let corel_a = graph_a
        .merges_corel_over(&INITIAL_VERTICES)
        .expect("corelation A");
    let corel_b = graph_b
        .merges_corel_over(&INITIAL_VERTICES)
        .expect("corelation B");
    for (name, corel) in [("A", &corel_a), ("B", &corel_b)] {
        println!(
            "  ordering {name}: boundary {:?} → {} class(es), representatives {:?}",
            INITIAL_VERTICES,
            corel.as_cospan().middle().len(),
            corel.as_cospan().middle()
        );
    }
    println!("  The five initial vertices survive into distinct classes in both");
    println!("  runs, so the restricted partitions are discrete — and equal.");

    let ccr = corel_a
        .coarsest_common_refinement(&corel_b)
        .expect("a shared boundary makes them comparable");
    let a_refines_b = corel_a.refines(&corel_b).expect("same boundary");
    let b_refines_a = corel_b.refines(&corel_a).expect("same boundary");
    println!("  A refines B: {a_refines_b}, B refines A: {b_refines_a}");
    println!(
        "  coarsest common refinement: {} class(es)",
        ccr.as_cospan().middle().len()
    );

    assert_eq!(corel_a.as_cospan().middle(), &INITIAL_VERTICES);
    assert_eq!(corel_b.as_cospan().middle(), &INITIAL_VERTICES);
    assert_eq!(ccr.as_cospan().middle().len(), 5);
    // Mutual refinement is equality of partitions: rule order permuted the
    // exploration but not which initial vertices get identified.
    assert!(a_refines_b && b_refines_a, "the two orderings must agree");
    assert!(ccr.refines(&corel_a).expect("same boundary"));
    assert!(ccr.refines(&corel_b).expect("same boundary"));

    // -----------------------------------------------------------------
    println!("\n== Wilson-loop verdict, for contrast ==");
    let evo_a = HypergraphEvolution::run_multiway(
        &pipeline(),
        &[RewriteRule::wolfram_a_to_bb(), RewriteRule::edge_split()],
        5,
        100,
    );
    let evo_b = HypergraphEvolution::run_multiway(
        &pipeline(),
        &[RewriteRule::edge_split(), RewriteRule::wolfram_a_to_bb()],
        5,
        100,
    );
    let (loops_a, loops_b) = (
        evo_a.find_wilson_loops().len(),
        evo_b.find_wilson_loops().len(),
    );
    let (wilson_a, wilson_b) = (evo_a.is_causally_invariant(), evo_b.is_causally_invariant());
    println!("  ordering A: {loops_a} Wilson loop(s), causally invariant = {wilson_a}");
    println!("  ordering B: {loops_b} Wilson loop(s), causally invariant = {wilson_b}");
    println!("  The corelations agree on the shared boundary while the Wilson");
    println!("  check reports its own verdict: the two measure different things.");

    // The Wilson verdict is independent of the comparison above — it is pinned
    // to this fixture, not implied by the corelations agreeing.
    assert_eq!(loops_a, 1278, "Wilson loop census for ordering A");
    assert_eq!(loops_b, 1056, "Wilson loop census for ordering B");
    assert!(!wilson_a, "ordering A's Wilson verdict");
    assert!(!wilson_b, "ordering B's Wilson verdict");

    println!("\nAll comparisons checked.");
}
