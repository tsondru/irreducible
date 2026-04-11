//! Integration tests for monoidal functor coherence.
//!
//! Verifies tensor product symmetry and associativity, unit laws,
//! monoidal functor results for multiway computations, differential
//! coherence, and bifunctor operations on ParallelIntervals.
//!
//! NOTE (v0.4.1): exercises deprecated coherence APIs (see `src/coherence.rs`
//! module docs). Kept green until v0.4.3 Phase 2.5 rewrite.
#![allow(deprecated)]

use irreducible::{
    tensor_bimap, verify_associativity, verify_symmetry, verify_unit_laws, DifferentialCoherence,
    DiscreteInterval, IntervalTransform, ParallelIntervals, StringRewriteSystem, TensorProduct,
};

use irreducible::functor::monoidal::{
    verify_associator_coherence, verify_braiding_coherence, verify_left_unitor_coherence,
    verify_right_unitor_coherence, CoherenceVerification,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_parallel(intervals: Vec<(usize, usize)>) -> ParallelIntervals {
    let mut p = ParallelIntervals::new();
    for (s, e) in intervals {
        p.add_branch(DiscreteInterval::new(s, e));
    }
    p
}

// ---------------------------------------------------------------------------
// Tensor product structure
// ---------------------------------------------------------------------------

#[test]
fn tensor_product_of_intervals_is_symmetric() {
    let a = make_parallel(vec![(0, 5)]);
    let b = make_parallel(vec![(10, 15)]);

    assert!(verify_symmetry(&a, &b));
    // Also check the reverse direction
    assert!(verify_symmetry(&b, &a));
}

#[test]
fn associativity_holds_for_triple_tensor() {
    let a = make_parallel(vec![(0, 5)]);
    let b = make_parallel(vec![(10, 15)]);
    let c = make_parallel(vec![(20, 25)]);

    assert!(verify_associativity(&a, &b, &c));
}

#[test]
fn unit_laws_hold_tensor_with_identity() {
    let a = make_parallel(vec![(0, 5), (10, 15)]);
    assert!(verify_unit_laws(&a));

    // Empty should also be a unit
    let empty = ParallelIntervals::new();
    assert!(empty.is_unit());
    assert!(verify_unit_laws(&empty));
}

// ---------------------------------------------------------------------------
// Monoidal functor verification on multiway systems
// ---------------------------------------------------------------------------

#[test]
fn monoidal_functor_result_for_irreducible_srs() {
    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 3, 50);

    let result = irreducible::IrreducibilityFunctor::verify_symmetric_monoidal_functor(&evolution);

    // The result should be well-formed
    assert!(!result.branch_results.is_empty());
    // Display should work
    let display = format!("{result}");
    assert!(display.contains("Monoidal Functor Verification"));
}

#[test]
fn monoidal_functor_result_for_deterministic_srs() {
    // A single rule with a single match position produces no branching
    let srs = StringRewriteSystem::new(vec![("AB", "CD")]);
    let evolution = srs.run_multiway("AB", 3, 50);

    let result = irreducible::IrreducibilityFunctor::verify_symmetric_monoidal_functor(&evolution);

    // Single branch should be trivially irreducible in the multiway sense
    assert_eq!(result.branch_results.len(), 1);
}

// ---------------------------------------------------------------------------
// Coherence verification
// ---------------------------------------------------------------------------

#[test]
fn coherence_verification_on_parallel_intervals() {
    let intervals = vec![
        make_parallel(vec![(0, 2)]),
        make_parallel(vec![(2, 4)]),
        make_parallel(vec![(4, 6)]),
    ];

    let coherence = CoherenceVerification::verify_all(&intervals);
    assert!(coherence.fully_coherent);
    assert!(coherence.associator_coherent);
    assert!(coherence.left_unitor_coherent);
    assert!(coherence.right_unitor_coherent);
    assert!(coherence.braiding_coherent);
}

#[test]
fn individual_coherence_conditions() {
    let a = make_parallel(vec![(0, 5)]);
    let b = make_parallel(vec![(10, 15)]);
    let c = make_parallel(vec![(20, 25)]);

    assert!(verify_associator_coherence(&a, &b, &c));
    assert!(verify_left_unitor_coherence(&a));
    assert!(verify_right_unitor_coherence(&a));
    assert!(verify_braiding_coherence(&a, &b));
}

// ---------------------------------------------------------------------------
// Differential coherence
// ---------------------------------------------------------------------------

#[test]
fn differential_coherence_conditions() {
    let intervals = vec![
        make_parallel(vec![(0, 2)]),
        make_parallel(vec![(2, 4)]),
        make_parallel(vec![(4, 6)]),
    ];

    let diff_coh = DifferentialCoherence::verify(&intervals);

    // Algebraic coherence should hold for well-formed intervals
    assert!(diff_coh.algebraic.fully_coherent);
    assert!(diff_coh.differential_coherent);
    assert!(diff_coh.coherence_form_closed);
    assert!((diff_coh.conservation_ratio - 1.0).abs() < 1e-10);
}

// ---------------------------------------------------------------------------
// Bifunctor operations
// ---------------------------------------------------------------------------

#[test]
fn tensor_bimap_with_identity_transforms() {
    let left = make_parallel(vec![(0, 5)]);
    let right = make_parallel(vec![(10, 15)]);

    // Identity transforms should not change anything
    let (new_left, new_right) = tensor_bimap(left.clone(), right.clone(), |p| p, |p| p);

    assert_eq!(new_left.branches[0].start, 0);
    assert_eq!(new_left.branches[0].end, 5);
    assert_eq!(new_right.branches[0].start, 10);
    assert_eq!(new_right.branches[0].end, 15);
}

#[test]
fn tensor_bimap_with_shift_and_scale() {
    let left = make_parallel(vec![(0, 5)]);
    let right = make_parallel(vec![(10, 15)]);

    let (new_left, new_right) =
        tensor_bimap(left, right, |p| p.shift_all(10), |p| p.scale_all(2));

    // Left shifted by 10: [0,5] -> [10,15]
    assert_eq!(new_left.branches[0].start, 10);
    assert_eq!(new_left.branches[0].end, 15);

    // Right scaled by 2: [10,15] -> [10, 10+(15-10)*2] = [10,20]
    assert_eq!(new_right.branches[0].start, 10);
    assert_eq!(new_right.branches[0].end, 20);
}

#[test]
fn monoidal_functor_tensor_violation_count() {
    // Construct a result manually to verify the count works
    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 3, 50);

    let result = irreducible::IrreducibilityFunctor::verify_symmetric_monoidal_functor(&evolution);

    // Violation count should be well-defined
    let violations = result.tensor_violation_count();
    // If multicomputationally irreducible, violations should be 0
    if result.is_multicomputationally_irreducible {
        assert_eq!(violations, 0);
    }
}
