//! Property-based tests for coherence conditions and adjunction triangle identities.
//!
//! Uses proptest to verify that monoidal coherence laws and the Z' ⊣ Z adjunction
//! hold for arbitrary well-formed inputs, not just hand-picked examples.

use proptest::prelude::*;

use irreducible::{
    AdjunctionIrreducibility, AdjunctionVerification, ComputationState, DiscreteInterval,
    ParallelIntervals, ZPrimeAdjunction, ZPrimeOps,
};
use irreducible::functor::monoidal::{
    verify_associator_coherence, verify_braiding_coherence, verify_left_unitor_coherence,
    verify_right_unitor_coherence,
};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a valid `DiscreteInterval` where start <= end.
fn arb_interval() -> impl Strategy<Value = DiscreteInterval> {
    (0_usize..100).prop_flat_map(|start| (Just(start), (start + 1)..=(start + 50)).prop_map(
        |(s, e)| DiscreteInterval::new(s, e),
    ))
}

/// Generate a `ParallelIntervals` with 1-5 branches.
fn arb_parallel_intervals() -> impl Strategy<Value = ParallelIntervals> {
    prop::collection::vec(arb_interval(), 1..=5).prop_map(|intervals| {
        let mut p = ParallelIntervals::new();
        for iv in intervals {
            p.add_branch(iv);
        }
        p
    })
}

/// Generate a `ComputationState` with valid step and complexity.
fn arb_computation_state() -> impl Strategy<Value = ComputationState> {
    (0_usize..100, 1_usize..50).prop_map(|(step, complexity)| {
        ComputationState::new(step, complexity)
    })
}

// ---------------------------------------------------------------------------
// Monoidal coherence: associator
// ---------------------------------------------------------------------------

proptest! {
    /// Associator coherence: (A ⊗ B) ⊗ C ≅ A ⊗ (B ⊗ C)
    /// Must hold for any three valid ParallelIntervals.
    #[test]
    fn associator_coherence_arbitrary(
        a in arb_parallel_intervals(),
        b in arb_parallel_intervals(),
        c in arb_parallel_intervals(),
    ) {
        prop_assert!(
            verify_associator_coherence(&a, &b, &c),
            "Associator coherence failed for a={:?}, b={:?}, c={:?}",
            a, b, c
        );
    }

    /// Braiding coherence: X ⊗ Y ≅ Y ⊗ X
    /// Must hold for any two valid ParallelIntervals.
    #[test]
    fn braiding_coherence_arbitrary(
        x in arb_parallel_intervals(),
        y in arb_parallel_intervals(),
    ) {
        prop_assert!(
            verify_braiding_coherence(&x, &y),
            "Braiding coherence failed for x={:?}, y={:?}",
            x, y
        );
    }

    /// Left unitor: I ⊗ X ≅ X for arbitrary X.
    #[test]
    fn left_unitor_arbitrary(x in arb_parallel_intervals()) {
        prop_assert!(
            verify_left_unitor_coherence(&x),
            "Left unitor failed for x={:?}",
            x
        );
    }

    /// Right unitor: X ⊗ I ≅ X for arbitrary X.
    #[test]
    fn right_unitor_arbitrary(x in arb_parallel_intervals()) {
        prop_assert!(
            verify_right_unitor_coherence(&x),
            "Right unitor failed for x={:?}",
            x
        );
    }

    // -----------------------------------------------------------------------
    // Adjunction triangle identities
    // -----------------------------------------------------------------------

    /// Triangle identity 1: ε_{Z'(C)} ∘ Z'(η_C) = id_{Z'(C)}
    /// Must hold for any ComputationState.
    #[test]
    fn adjunction_triangle_1_arbitrary(state in arb_computation_state()) {
        prop_assert!(
            ZPrimeAdjunction::verify_triangle_1(&state),
            "Triangle identity 1 failed for state={:?}",
            state
        );
    }

    /// Triangle identity 2: Z(ε_I) ∘ η_{Z(I)} = id_{Z(I)}
    /// Must hold for any DiscreteInterval.
    #[test]
    fn adjunction_triangle_2_arbitrary(interval in arb_interval()) {
        prop_assert!(
            ZPrimeAdjunction::verify_triangle_2(&interval),
            "Triangle identity 2 failed for interval={:?}",
            interval
        );
    }

    /// Combined adjunction verification for arbitrary state sequences.
    /// Both triangle identities must hold for every element.
    #[test]
    fn adjunction_triangle_arbitrary(
        states in prop::collection::vec(arb_computation_state(), 1..=10),
    ) {
        let verification = AdjunctionVerification::verify_sequence::<ZPrimeAdjunction>(&states);

        prop_assert!(
            verification.triangle_identities_hold,
            "Triangle identities failed for states={:?}: t1_failures={}, t2_failures={}",
            states,
            verification.triangle_1_failures(),
            verification.triangle_2_failures()
        );
        prop_assert!(
            verification.is_adjoint_pair,
            "Not an adjoint pair for states={:?}",
            states
        );
    }

    // -----------------------------------------------------------------------
    // Round-trip properties
    // -----------------------------------------------------------------------

    /// Z(Z'(c)) recovers the original computation state.
    #[test]
    fn zprime_z_roundtrip_arbitrary(state in arb_computation_state()) {
        let interval = ZPrimeAdjunction::zprime(&state);
        let recovered = ZPrimeAdjunction::z(&interval);

        prop_assert_eq!(recovered.step, state.step);
        prop_assert_eq!(recovered.complexity, state.complexity);
    }

    /// Z'(Z(i)) recovers the original interval.
    #[test]
    fn z_zprime_roundtrip_arbitrary(interval in arb_interval()) {
        let state = ZPrimeAdjunction::z(&interval);
        let recovered = ZPrimeAdjunction::zprime(&state);

        prop_assert_eq!(recovered.start, interval.start);
        prop_assert_eq!(recovered.end, interval.end);
    }

    /// Adjunction gap is zero for any well-formed state.
    #[test]
    fn adjunction_gap_zero_arbitrary(
        states in prop::collection::vec(arb_computation_state(), 1..=10),
    ) {
        let indicator = ZPrimeAdjunction::adjunction_irreducibility_indicator(&states);
        prop_assert!(
            indicator.abs() < f64::EPSILON,
            "Expected zero indicator, got {} for states={:?}",
            indicator, states
        );
    }
}
