//! Integration tests for hypergraph rewriting.
//!
//! Tests DPO rewrite rules, deterministic and multiway evolution,
//! causal invariance via Wilson loops, gauge group construction,
//! plaquette/total action, and lattice construction.

use irreducible::machines::hypergraph::{
    Hypergraph, HypergraphEvolution, HypergraphLattice, HypergraphRewriteGroup, RewriteRule,
    plaquette_action, total_action,
};

// ---------------------------------------------------------------------------
// Hypergraph construction
// ---------------------------------------------------------------------------

#[test]
fn create_hypergraph_from_edges() {
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2], vec![2, 3, 4]]);

    assert_eq!(graph.vertex_count(), 5);
    assert_eq!(graph.edge_count(), 2);
}

#[test]
fn hypergraph_add_and_query_edges() {
    let mut graph = Hypergraph::new();
    graph.add_hyperedge(vec![0, 1, 2]);
    graph.add_hyperedge(vec![2, 3]);

    assert_eq!(graph.vertex_count(), 4);
    assert_eq!(graph.edge_count(), 2);
}

// ---------------------------------------------------------------------------
// Rewrite rules
// ---------------------------------------------------------------------------

#[test]
fn wolfram_a_to_bb_rule_structure() {
    let rule = RewriteRule::wolfram_a_to_bb();

    assert_eq!(rule.name(), Some("A\u{2192}BB")); // "A→BB"
    assert_eq!(rule.left_arity(), 1); // One ternary edge on left
    assert_eq!(rule.right_arity(), 2); // Two binary edges on right
    assert_eq!(rule.num_variables(), 3); // Variables 0, 1, 2

    // All variables are preserved (none created, none deleted)
    assert!(rule.deleted_variables().is_empty());
    assert!(rule.created_variables().is_empty());
}

#[test]
fn apply_wolfram_a_to_bb_rule() {
    let rule = RewriteRule::wolfram_a_to_bb();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);

    // Find matches for the rule in the graph
    let matches = rule.find_matches(&graph);
    assert!(
        !matches.is_empty(),
        "Rule should match the ternary edge in the graph"
    );
}

// ---------------------------------------------------------------------------
// Deterministic evolution
// ---------------------------------------------------------------------------

#[test]
fn multi_step_deterministic_evolution() {
    let rule = RewriteRule::wolfram_a_to_bb();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);

    let evolution = HypergraphEvolution::run(&graph, &[rule], 3);

    assert!(evolution.node_count() > 1);
    assert!(evolution.max_step() >= 1);

    // Root should be the original graph
    let root = evolution.root();
    assert_eq!(root.state.edge_count(), 1);
    assert_eq!(root.step, 0);
}

// ---------------------------------------------------------------------------
// Multiway evolution
// ---------------------------------------------------------------------------

#[test]
fn multiway_evolution_with_multiple_rules() {
    let rule1 = RewriteRule::wolfram_a_to_bb();
    let rule2 = RewriteRule::edge_split();

    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
    let evolution = HypergraphEvolution::run_multiway(&graph, &[rule1, rule2], 3, 50);

    // Multiple rules should generate more nodes than a single rule
    assert!(evolution.node_count() > 1);
}

#[test]
fn multiway_evolution_explores_all_matches() {
    let rule = RewriteRule::wolfram_a_to_bb();

    // Two independent ternary edges = two possible match sites
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2], vec![3, 4, 5]]);
    let evolution = HypergraphEvolution::run_multiway(&graph, &[rule], 2, 100);

    // Should explore both match sites
    assert!(evolution.node_count() > 2);
}

// ---------------------------------------------------------------------------
// Causal invariance (Wilson loops)
// ---------------------------------------------------------------------------

#[test]
fn causal_invariance_check() {
    let rule = RewriteRule::wolfram_a_to_bb();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);

    let evolution = HypergraphEvolution::run_multiway(&graph, &[rule], 3, 50);

    // Check causal invariance -- the API should work regardless of result
    let result = evolution.analyze_causal_invariance();
    let _ = result.is_invariant;
    let _ = result.average_deviation;
    let _ = result.max_deviation;
    let _ = result.loops_analyzed;
}

#[test]
fn is_causally_invariant_convenience_method() {
    let rule = RewriteRule::wolfram_a_to_bb();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);

    let evolution = HypergraphEvolution::run_multiway(&graph, &[rule], 3, 50);
    // This is a convenience wrapper; just verify it runs
    let _ = evolution.is_causally_invariant();
}

// ---------------------------------------------------------------------------
// Wilson loops
// ---------------------------------------------------------------------------

#[test]
fn wilson_loop_computation() {
    let rule = RewriteRule::wolfram_a_to_bb();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);

    let evolution = HypergraphEvolution::run_multiway(&graph, &[rule], 4, 100);
    let loops = evolution.find_wilson_loops();

    // Wilson loops exist only when branches merge (same fingerprint)
    // We just verify the API works and returns valid structures
    for wl in &loops {
        assert!(!wl.path.is_empty());
        assert!(wl.holonomy >= 0.0);
        assert!(wl.holonomy <= 1.0);
    }
}

// ---------------------------------------------------------------------------
// Gauge group
// ---------------------------------------------------------------------------

#[test]
fn gauge_group_construction() {
    let group = HypergraphRewriteGroup::new(3);
    assert_eq!(group.num_rules(), 3);

    // Structure constants should be antisymmetric: f^{aab} = 0
    let f_000 = group.structure_constant_for(0, 0, 0);
    assert!((f_000 - 0.0).abs() < 1e-10);

    // Non-trivial mixing for distinct indices
    let f_012 = group.structure_constant_for(0, 1, 2);
    // Structure constant is non-zero when all indices differ
    assert!((f_012 - 1.0).abs() < 1e-10);
}

#[test]
fn gauge_group_representation_dim() {
    let group = HypergraphRewriteGroup::new(4);
    assert_eq!(group.representation_dim(), 16); // num_rules^2
}

// ---------------------------------------------------------------------------
// Plaquette and total action
// ---------------------------------------------------------------------------

#[test]
fn plaquette_action_values() {
    // Holonomy = 1.0 -> flat connection (action = 0)
    assert!((plaquette_action(1.0) - 0.0).abs() < 1e-10);

    // Holonomy = 0.5 -> non-trivial curvature
    let action = plaquette_action(0.5);
    assert!(action > 0.0);
    assert!((action - 0.5_f64.ln().abs()).abs() < 1e-10);

    // Holonomy = 0.0 -> infinite action (singular)
    assert!(plaquette_action(0.0).is_infinite());
}

#[test]
fn total_action_computation() {
    let holonomies = vec![1.0, 1.0, 1.0];
    assert!((total_action(&holonomies) - 0.0).abs() < 1e-10);

    let mixed = vec![1.0, 0.5, 1.0];
    let action = total_action(&mixed);
    assert!(action > 0.0);
}

// ---------------------------------------------------------------------------
// Lattice construction
// ---------------------------------------------------------------------------

#[test]
fn lattice_construction_1d() {
    let group = HypergraphRewriteGroup::new(2);
    let lattice: HypergraphLattice<1> = HypergraphLattice::new([5], group, vec![]);

    // Lattice should be constructable
    let _ = lattice;
}

#[test]
fn lattice_construction_2d() {
    let group = HypergraphRewriteGroup::new(3);
    let lattice: HypergraphLattice<2> = HypergraphLattice::new([4, 4], group, vec![]);

    let _ = lattice;
}

// ---------------------------------------------------------------------------
// Evolution statistics
// ---------------------------------------------------------------------------

#[test]
fn evolution_statistics() {
    let rule = RewriteRule::wolfram_a_to_bb();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);

    let evolution = HypergraphEvolution::run_multiway(&graph, &[rule], 3, 50);
    let stats = evolution.statistics();

    assert!(stats.total_nodes > 0);
    assert!(stats.max_step >= 1);
    assert!(stats.branch_count >= 1);
    assert!(!stats.rule_applications.is_empty());
}

// ---------------------------------------------------------------------------
// Regression: leaves() must not include the last node when it has children
// ---------------------------------------------------------------------------

#[test]
fn leaves_excludes_last_node_that_is_a_parent() {
    // Use multiway evolution with two rules so the tree branches.
    // This guarantees intermediate nodes (including the last one added in
    // earlier steps) become parents of later nodes.
    //
    // The buggy code had `|| *id == self.nodes.len() - 1`, which
    // unconditionally included the last node even when it had children.
    let rule1 = RewriteRule::wolfram_a_to_bb();
    let rule2 = RewriteRule::edge_split();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);

    let evolution = HypergraphEvolution::run_multiway(&graph, &[rule1, rule2], 3, 50);
    assert!(
        evolution.node_count() >= 3,
        "Need at least 3 nodes for this regression test, got {}",
        evolution.node_count()
    );

    let leaves = evolution.leaves();

    // Collect the set of all node IDs that are parents of at least one other node.
    let parent_ids: std::collections::HashSet<usize> = (0..evolution.node_count())
        .filter_map(|id| {
            let node = evolution.get_node(id).unwrap();
            node.parent
        })
        .collect();

    // Core invariant: no leaf should also be a parent.
    for &leaf in &leaves {
        assert!(
            !parent_ids.contains(&leaf),
            "Node {leaf} is a parent but was returned by leaves()"
        );
    }

    // Every non-parent should appear in leaves.
    for id in 0..evolution.node_count() {
        if !parent_ids.contains(&id) {
            assert!(
                leaves.contains(&id),
                "Node {id} has no children but is missing from leaves()"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Error / negative path tests
// ---------------------------------------------------------------------------

#[test]
fn rewrite_empty_graph_produces_no_change() {
    let rule = RewriteRule::wolfram_a_to_bb();
    let empty_graph = Hypergraph::new();

    // No edges in the graph, so the rule should find no matches
    let matches = rule.find_matches(&empty_graph);
    assert!(matches.is_empty());

    // Deterministic evolution on an empty graph should produce only the root node
    let evolution = HypergraphEvolution::run(&empty_graph, std::slice::from_ref(&rule), 5);
    assert_eq!(evolution.node_count(), 1); // only the root
    assert_eq!(evolution.max_step(), 0); // no steps taken

    // The root's state should still be an empty graph
    let root = evolution.root();
    assert_eq!(root.state.edge_count(), 0);
    assert_eq!(root.state.vertex_count(), 0);
}

#[test]
fn evolution_zero_steps_is_trivial() {
    let rule = RewriteRule::wolfram_a_to_bb();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);

    // run_multiway with 0 max_steps should produce only the root
    let evolution = HypergraphEvolution::run_multiway(&graph, &[rule], 0, 100);

    assert_eq!(evolution.node_count(), 1);
    assert_eq!(evolution.max_step(), 0);

    let root = evolution.root();
    assert_eq!(root.step, 0);
    assert_eq!(root.state.edge_count(), 1);
}

// ---------------------------------------------------------------------------
// Combined pipeline: multiway evolution + gauge analysis
// ---------------------------------------------------------------------------

#[test]
fn multiway_evolution_with_gauge_analysis_pipeline() {
    // 1. Create a hypergraph with a couple of edges
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2], vec![2, 3, 4]]);
    assert_eq!(graph.edge_count(), 2);

    // 2. Define two rewrite rules
    let rule1 = RewriteRule::wolfram_a_to_bb();
    let rule2 = RewriteRule::edge_split();

    // 3. Run multiway evolution
    let evolution = HypergraphEvolution::run_multiway(&graph, &[rule1, rule2], 5, 100);
    assert!(
        evolution.node_count() > 1,
        "Two rules on two edges should produce branching"
    );

    // 4. Compute Wilson loops
    let loops = evolution.find_wilson_loops();
    // Wilson loops exist when branches merge; collect holonomies
    let holonomies: Vec<f64> = loops.iter().map(|wl| wl.holonomy).collect();

    // 5 & 6. Compute plaquette action per holonomy and total action
    for &h in &holonomies {
        let pa = plaquette_action(h);
        assert!(
            pa.is_finite() || h == 0.0,
            "Plaquette action should be finite for h > 0"
        );
    }

    let action = total_action(&holonomies);
    assert!(
        action.is_finite(),
        "Total action across all holonomies should be finite (got {action})"
    );

    // 7. Check causal invariance
    let _invariant = evolution.is_causally_invariant();

    // 8. Verify statistics are populated
    let stats = evolution.statistics();
    assert!(stats.total_nodes > 0);
    assert!(stats.max_step >= 1);
    assert!(!stats.rule_applications.is_empty());
}

// ---------------------------------------------------------------------------
// Merge partition as a corelation (F&S 2018 Ex 6.64) — issue #14
// ---------------------------------------------------------------------------

#[test]
fn merges_corel_builds_over_all_vertices() {
    use irreducible::machines::hypergraph::{MergesCorelExt, MultiwayCospanExt};

    let rule = RewriteRule::wolfram_a_to_bb();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
    let evolution = HypergraphEvolution::run_multiway(&graph, &[rule], 3, 50);

    let cospan_graph = evolution.to_multiway_cospan_graph();
    let corel = cospan_graph.merges_corel().expect("corelation builds");
    assert!(
        !corel.equivalence_classes().is_empty(),
        "evolution has vertices, so the partition has classes"
    );
}

#[test]
fn merge_partition_consistent_with_merge_points() {
    use irreducible::machines::hypergraph::{MergesCorelExt, MultiwayCospanExt};

    // Two independent match sites: different application orders reach the
    // same final state — fingerprint merges appear.
    let rule = RewriteRule::wolfram_a_to_bb();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2], vec![3, 4, 5]]);
    let evolution = HypergraphEvolution::run_multiway(&graph, &[rule], 3, 100);

    let cospan_graph = evolution.to_multiway_cospan_graph();
    let corel = cospan_graph.merges_corel().expect("corelation builds");
    assert!(
        !cospan_graph.merge_points.is_empty(),
        "two independent sites must produce fingerprint merges"
    );

    // Consistency contract: for every merge group, the sorted vertex-ID
    // alignment of any two member states must land in one equivalence
    // class of the corelation. (wolfram_a_to_bb creates no fresh vertices,
    // so aligned IDs may be equal — merges() must hold either way.)
    let boundary: Vec<u32> = {
        let mut all: Vec<u32> = cospan_graph
            .edges
            .iter()
            .flat_map(|e| e.cospan.middle().iter().copied())
            .collect();
        all.sort_unstable();
        all.dedup();
        all
    };
    let flat = |v: u32| -> usize {
        boundary
            .iter()
            .position(|&b| b == v)
            .expect("merged vertex appears in the evolution")
    };

    let vertices_of = |node: usize| -> Option<Vec<u32>> {
        cospan_graph.edges.iter().find_map(|e| {
            if e.child_id == node {
                Some(
                    e.cospan
                        .right_to_middle()
                        .iter()
                        .map(|&m| e.cospan.middle()[m])
                        .collect(),
                )
            } else if e.parent_id == node {
                Some(
                    e.cospan
                        .left_to_middle()
                        .iter()
                        .map(|&m| e.cospan.middle()[m])
                        .collect(),
                )
            } else {
                None
            }
        })
    };

    for group in &cospan_graph.merge_points {
        let (first, rest) = group.split_first().expect("groups are non-empty");
        let mut base = vertices_of(*first).expect("node has vertices");
        base.sort_unstable();
        for other in rest {
            let mut vs = vertices_of(*other).expect("node has vertices");
            vs.sort_unstable();
            assert_eq!(vs.len(), base.len(), "merged states have equal size");
            for (&a, &b) in base.iter().zip(vs.iter()) {
                assert!(
                    corel.merges(flat(a), flat(b)),
                    "aligned vertices {a} and {b} must share a class"
                );
            }
        }
    }
}

#[test]
fn rule_ordering_confluence_via_coarsest_common_refinement() {
    use irreducible::machines::hypergraph::{MergesCorelExt, MultiwayCospanExt};

    // Same initial graph, two rule orderings. Restricting both merge
    // partitions to the initial vertex IDs makes them comparable.
    let r1 = RewriteRule::wolfram_a_to_bb();
    let r2 = RewriteRule::edge_split();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
    let initial_vertices: Vec<u32> = vec![0, 1, 2];

    let evo_a = HypergraphEvolution::run_multiway(&graph, &[r1.clone(), r2.clone()], 3, 50);
    let evo_b = HypergraphEvolution::run_multiway(&graph, &[r2, r1], 3, 50);

    let corel_a = evo_a
        .to_multiway_cospan_graph()
        .merges_corel_over(&initial_vertices)
        .expect("corelation A");
    let corel_b = evo_b
        .to_multiway_cospan_graph()
        .merges_corel_over(&initial_vertices)
        .expect("corelation B");

    // The lattice law: the coarsest common refinement refines both inputs.
    let ccr = corel_a
        .coarsest_common_refinement(&corel_b)
        .expect("shared boundary -> comparable");
    assert!(ccr.refines(&corel_a).expect("same boundary"));
    assert!(ccr.refines(&corel_b).expect("same boundary"));

    // Rule order permutes exploration, not identification: the partitions
    // over the shared initial vertices must agree (the confluence signal).
    assert!(corel_a.refines(&corel_b).expect("same boundary"));
    assert!(corel_b.refines(&corel_a).expect("same boundary"));
}
