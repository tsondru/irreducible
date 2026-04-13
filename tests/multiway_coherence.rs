//! Integration tests for non-strict SMC coherence on multiway evolution graphs.

use catgraph_physics::multiway::MultiwayEvolutionGraph;
use irreducible::multiway_coherence::{
    verify_all_coherence, verify_associator, verify_braiding, verify_unitor, CoherenceError,
};

/// Build a multiway graph with a 3-way fork and full confluence.
fn confluent_3fork() -> MultiwayEvolutionGraph<&'static str, &'static str> {
    let mut g = MultiwayEvolutionGraph::new();
    let root = g.add_root("s0");
    let branches = g.add_fork(
        root,
        vec![("s1a", "e1", 0), ("s1b", "e2", 1), ("s1c", "e3", 2)],
    );
    let merge_target = g.add_sequential_step(branches[0], "s2", "m1");
    g.add_merge_edge(branches[1], merge_target, "m2");
    g.add_merge_edge(branches[2], merge_target, "m3");
    g
}

/// Build a non-confluent graph: 3 branches, only 2 merge.
fn non_confluent_3fork() -> MultiwayEvolutionGraph<&'static str, &'static str> {
    let mut g = MultiwayEvolutionGraph::new();
    let root = g.add_root("s0");
    let branches = g.add_fork(
        root,
        vec![("s1a", "e1", 0), ("s1b", "e2", 1), ("s1c", "e3", 2)],
    );
    let merge_target = g.add_sequential_step(branches[0], "s2", "m1");
    g.add_merge_edge(branches[1], merge_target, "m2");
    g.add_sequential_step(branches[2], "s_diverged", "no_merge");
    g
}

#[test]
fn associator_passes_on_confluent_graph() {
    let g = confluent_3fork();
    let forks = g.find_fork_points();
    assert!(!forks.is_empty());
    let result = verify_associator(&g, forks[0]);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

#[test]
fn associator_fails_on_non_confluent_graph() {
    let g = non_confluent_3fork();
    let forks = g.find_fork_points();
    assert!(!forks.is_empty());
    let result = verify_associator(&g, forks[0]);
    assert!(
        matches!(result, Err(CoherenceError::NonConfluent { .. })),
        "expected NonConfluent, got {result:?}",
    );
}

#[test]
fn braiding_passes_on_commuting_events() {
    let g = confluent_3fork();
    let forks = g.find_fork_points();
    let pairs = g.parallel_independent_events(forks[0]);
    assert!(!pairs.is_empty());
    let (a, b) = pairs[0];
    let result = verify_braiding(&g, a, b);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

#[test]
fn braiding_fails_on_non_commuting_events() {
    let g = non_confluent_3fork();
    let forks = g.find_fork_points();
    let pairs = g.parallel_independent_events(forks[0]);
    let divergent_pair = pairs.iter().find(|(a, b)| !g.events_commute(*a, *b));
    if let Some(&(a, b)) = divergent_pair {
        let result = verify_braiding(&g, a, b);
        assert!(
            matches!(result, Err(CoherenceError::NonConfluent { .. })),
            "expected NonConfluent, got {result:?}",
        );
    }
}

#[test]
fn unitor_passes_on_sequential_graph() {
    let mut g: MultiwayEvolutionGraph<&str, &str> = MultiwayEvolutionGraph::new();
    let root = g.add_root("s0");
    let n1 = g.add_sequential_step(root, "s1", "step");
    let _n2 = g.add_sequential_step(n1, "s2", "step");
    let result = verify_unitor(&g, root);
    assert!(result.is_ok());
}

#[test]
fn verify_all_confluent_graph_no_errors() {
    let g = confluent_3fork();
    let errors = verify_all_coherence(&g);
    assert!(errors.is_empty(), "expected no errors, got {errors:?}");
}

#[test]
fn verify_all_non_confluent_graph_has_errors() {
    let g = non_confluent_3fork();
    let errors = verify_all_coherence(&g);
    assert!(!errors.is_empty(), "expected errors for non-confluent graph");
}

// ---------------------------------------------------------------------------
// Monoidal functor verification (migrated from tests/monoidal_coherence.rs)
// ---------------------------------------------------------------------------

#[test]
fn monoidal_functor_result_for_irreducible_srs() {
    let srs = irreducible::StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 3, 50);
    let result = irreducible::IrreducibilityFunctor::verify_symmetric_monoidal_functor(&evolution);
    assert!(!result.branch_results.is_empty());
    let display = format!("{result}");
    assert!(display.contains("Monoidal Functor Verification"));
}

#[test]
fn monoidal_functor_result_for_deterministic_srs() {
    let srs = irreducible::StringRewriteSystem::new(vec![("AB", "CD")]);
    let evolution = srs.run_multiway("AB", 3, 50);
    let result = irreducible::IrreducibilityFunctor::verify_symmetric_monoidal_functor(&evolution);
    assert_eq!(result.branch_results.len(), 1);
}

#[test]
fn monoidal_functor_tensor_violation_count() {
    let srs = irreducible::StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 3, 50);
    let result = irreducible::IrreducibilityFunctor::verify_symmetric_monoidal_functor(&evolution);
    if result.is_multicomputationally_irreducible {
        assert_eq!(result.tensor_violation_count(), 0);
    }
}
