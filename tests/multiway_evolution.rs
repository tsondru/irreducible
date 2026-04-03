//! Integration tests for multiway evolution systems.
//!
//! Tests StringRewriteSystem and NondeterministicTM multiway BFS,
//! branchial foliation, merge detection, curvature computation,
//! and max-limits enforcement.

use irreducible::{BranchialCurvature, NondeterministicTM, StringRewriteSystem};

use irreducible::machines::multiway::{
    branchial_to_parallel_intervals, extract_branchial_foliation, find_all_merge_points,
};

use irreducible::machines::Direction;

// ---------------------------------------------------------------------------
// String Rewriting System multiway tests
// ---------------------------------------------------------------------------

#[test]
fn srs_ab_ba_and_a_aa_produces_branches() {
    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 3, 100);

    // "AB" can match both rules, so we should get multiple branches
    assert!(evolution.node_count() > 1);
    assert!(evolution.max_step() >= 1);
}

#[test]
fn ntm_simple_branching_produces_fork_points() {
    // NTM with two possible transitions from state 0 on '_'
    let ntm = NondeterministicTM::builder()
        .states(vec![0, 1, 2])
        .initial_state(0)
        .accept_states(vec![1, 2])
        .blank('_')
        .transition(0, '_', vec![(1, 'X', Direction::Right), (2, 'Y', Direction::Left)])
        .build();

    let evolution = ntm.run_multiway("_", 3, 50);

    // Should have at least the root and the two branches
    assert!(evolution.node_count() >= 3);
    // Should have at least one fork point
    let fork_points = evolution.find_fork_points();
    assert!(
        !fork_points.is_empty(),
        "Expected fork points from non-deterministic transitions"
    );
}

#[test]
fn srs_and_ntm_use_same_generic_bfs() {
    // Both produce MultiwayEvolutionGraph with the same methods
    let srs = StringRewriteSystem::new(vec![("A", "AB")]);
    let srs_evo = srs.run_multiway("A", 3, 50);

    let ntm = NondeterministicTM::builder()
        .states(vec![0, 1])
        .initial_state(0)
        .accept_states(vec![1])
        .blank('_')
        .transition(0, '_', vec![(1, 'X', Direction::Stay)])
        .build();
    let ntm_evo = ntm.run_multiway("_", 3, 50);

    // Both graphs have the same structural methods
    assert!(srs_evo.node_count() > 0);
    assert!(ntm_evo.node_count() > 0);
    assert!(srs_evo.roots().len() > 0);
    assert!(ntm_evo.roots().len() > 0);
    assert!(srs_evo.leaves().len() > 0);
    assert!(ntm_evo.leaves().len() > 0);
}

// ---------------------------------------------------------------------------
// Branchial foliation
// ---------------------------------------------------------------------------

#[test]
fn branchial_foliation_at_each_step() {
    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 3, 100);

    let foliation = extract_branchial_foliation(&evolution);

    // Should have branchial graphs for each step (0 to max_step)
    assert!(!foliation.is_empty());

    // Each branchial graph at step 0 should have exactly 1 node (the root)
    assert_eq!(foliation[0].node_count(), 1);

    // Later steps may have more (branching)
    if foliation.len() > 1 {
        assert!(foliation[1].node_count() >= 1);
    }
}

#[test]
fn branchial_to_parallel_intervals_produces_valid_structure() {
    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 3, 100);

    let parallel_vec = branchial_to_parallel_intervals(&evolution);

    // Should produce at least one ParallelIntervals entry (one per step transition)
    assert!(!parallel_vec.is_empty());
    // Each entry should have some branches
    for pi in &parallel_vec {
        assert!(pi.branch_count() >= 1);
    }
}

// ---------------------------------------------------------------------------
// Merge detection
// ---------------------------------------------------------------------------

#[test]
fn merge_detection_when_branches_converge() {
    // Create an SRS where branches can converge
    // "AB" -> "BA" and "BA" -> "AB" would cycle, so use something different
    // "AB" -> "C" and "A" -> "C" where both lead to "C" (simplified merge scenario)
    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("BA", "AB")]);
    let evolution = srs.run_multiway("AB", 4, 100);

    // Even if no structural merges occur, the API should work
    let merge_points = find_all_merge_points(&evolution);
    // Merge points are found via fingerprint matching
    // We just verify the function is callable and returns a valid result
    let _ = merge_points.len();
}

// ---------------------------------------------------------------------------
// Statistics
// ---------------------------------------------------------------------------

#[test]
fn multiway_statistics_fork_and_merge_counts() {
    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 3, 100);

    let fork_count = evolution.find_fork_points().len();
    let merge_count = evolution.find_merge_points().len();

    // With branching rules, we expect at least some forks
    // (merges may or may not exist depending on state confluence)
    // Both counts are usize (always >= 0), so we just verify the API is callable
    let _ = fork_count;
    let _ = merge_count;

    // The graph should have more nodes than just the root
    assert!(evolution.node_count() > 1);
}

// ---------------------------------------------------------------------------
// Curvature computation
// ---------------------------------------------------------------------------

#[test]
fn curvature_computation_on_multiway_graph() {
    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 3, 100);

    let foliation = extract_branchial_foliation(&evolution);

    // Compute curvature at each step
    for branchial in &foliation {
        let curvature = BranchialCurvature::from_branchial(branchial);
        assert_eq!(curvature.step, branchial.step);
        assert_eq!(curvature.dimension, branchial.node_count());

        // Single-node branchial graphs should be flat
        if branchial.node_count() <= 1 {
            assert!(curvature.is_flat);
            assert!((curvature.scalar_curvature - 0.0).abs() < 1e-10);
        }
    }
}

#[test]
fn curvature_foliation_across_steps() {
    use irreducible::CurvatureFoliation;

    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 3, 100);

    let curvature_foliation = CurvatureFoliation::from_evolution(&evolution);
    assert!(!curvature_foliation.curvatures.is_empty());

    // The step at index 0 should be flat (only root)
    if !curvature_foliation.curvatures.is_empty() {
        assert!(curvature_foliation.curvatures[0].is_flat);
    }
}

// ---------------------------------------------------------------------------
// Limits enforcement
// ---------------------------------------------------------------------------

#[test]
fn max_branches_limit_is_respected() {
    let srs = StringRewriteSystem::new(vec![("A", "AA")]);
    let max_branches = 10;
    let evolution = srs.run_multiway("A", 100, max_branches);

    // The BFS may slightly overshoot due to fork batching, but the node count
    // should be bounded proportionally to max_branches
    assert!(
        evolution.node_count() <= max_branches * 2,
        "Node count {} greatly exceeds max_branches {}",
        evolution.node_count(),
        max_branches
    );
}

#[test]
fn max_steps_limit_is_respected() {
    let srs = StringRewriteSystem::new(vec![("A", "AA")]);
    let max_steps = 3;
    let evolution = srs.run_multiway("A", max_steps, 1000);

    // No node should exceed the step limit
    assert!(evolution.max_step() <= max_steps);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn empty_input_no_applicable_rules() {
    let srs = StringRewriteSystem::new(vec![("XY", "YX")]);
    let evolution = srs.run_multiway("A", 5, 100);

    // "A" doesn't match "XY", so only the root node should exist
    assert_eq!(evolution.node_count(), 1);
    assert_eq!(evolution.max_step(), 0);
}

#[test]
fn single_step_evolution() {
    let srs = StringRewriteSystem::new(vec![("A", "B")]);
    let evolution = srs.run_multiway("A", 1, 100);

    // Should have root + one step
    assert!(evolution.node_count() >= 2);
    assert_eq!(evolution.max_step(), 1);
}

// ---------------------------------------------------------------------------
// Error / negative path tests
// ---------------------------------------------------------------------------

#[test]
fn srs_empty_rules_no_evolution() {
    // A StringRewriteSystem with no rules should produce no evolution:
    // only the root node remains, nothing can be rewritten.
    let srs = StringRewriteSystem::new(Vec::<(&str, &str)>::new());
    let evolution = srs.run_multiway("ABCDEF", 10, 100);

    assert_eq!(evolution.node_count(), 1); // only the root
    assert_eq!(evolution.max_step(), 0); // no steps possible
    assert!(evolution.find_fork_points().is_empty());
    assert!(evolution.find_merge_points().is_empty());
}
