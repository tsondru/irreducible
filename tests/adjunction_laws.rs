//! Integration tests for the Z' ⊣ Z adjunction.
//!
//! Verifies unit/counit, triangle identities, round-trip properties,
//! and the connection between the adjunction structure and irreducibility
//! for both TuringMachine and CellularAutomaton executions.

use irreducible::{
    AdjunctionIrreducibility, AdjunctionVerification, ComputationState, DiscreteInterval,
    ElementaryCA, TuringMachine, ZPrimeAdjunction, ZPrimeOps,
};

// ---------------------------------------------------------------------------
// Basic adjunction operations
// ---------------------------------------------------------------------------

#[test]
fn create_adjunction_verify_unit_and_counit() {
    let state = ComputationState::new(0, 5);

    // Z': computation state -> interval
    let interval = ZPrimeAdjunction::zprime(&state);
    assert_eq!(interval.start, 0);
    assert_eq!(interval.end, 5);

    // Z: interval -> computation state
    let recovered = ZPrimeAdjunction::z(&interval);
    assert_eq!(recovered.step, 0);
    assert_eq!(recovered.complexity, 5);

    // Unit: eta_c = Z(Z'(c))
    let unit_result = ZPrimeAdjunction::unit_at(&state);
    assert_eq!(unit_result.step, state.step);
    assert_eq!(unit_result.complexity, state.complexity);

    // Counit: epsilon_i = Z'(Z(i))
    let counit_result = ZPrimeAdjunction::counit_at(&interval);
    assert_eq!(counit_result.start, interval.start);
    assert_eq!(counit_result.end, interval.end);
}

// ---------------------------------------------------------------------------
// Triangle identities
// ---------------------------------------------------------------------------

#[test]
fn triangle_identities_hold_for_tm_derived_states() {
    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);

    // Derive computation states from the TM execution
    let states: Vec<ComputationState> = (0..=history.step_count())
        .map(|i| ComputationState::new(0, i))
        .collect();

    for state in &states {
        assert!(
            ZPrimeAdjunction::verify_triangle_1(state),
            "Triangle 1 failed for step complexity {}",
            state.complexity
        );
    }
}

#[test]
fn triangle_identities_hold_for_ca_derived_intervals() {
    let ca = ElementaryCA::rule_30(11);
    let history = ca.run(ca.single_cell_initial(), 10);

    // Derive intervals from the CA execution
    let intervals = history.to_intervals();
    for interval in &intervals {
        assert!(
            ZPrimeAdjunction::verify_triangle_2(interval),
            "Triangle 2 failed for interval [{}, {}]",
            interval.start,
            interval.end
        );
    }
}

#[test]
fn triangle_identity_2_on_composed_interval() {
    let ca = ElementaryCA::rule_30(21);
    let history = ca.run(ca.single_cell_initial(), 20);

    // The total interval [0, 20] should also satisfy triangle 2
    if let Some(total) = history.total_interval() {
        assert!(ZPrimeAdjunction::verify_triangle_2(&total));
    }
}

// ---------------------------------------------------------------------------
// Adjunction verification for sequences
// ---------------------------------------------------------------------------

#[test]
fn adjunction_preserves_irreducibility_status() {
    // Irreducible execution: busy beaver
    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);
    assert!(history.is_irreducible());

    // Derive states and verify the adjunction holds throughout
    let states: Vec<ComputationState> = (0..history.step_count())
        .map(|i| ComputationState::new(i, 1))
        .collect();

    let verification = AdjunctionVerification::verify_sequence(&states);
    assert!(verification.triangle_identities_hold);
    assert!(verification.is_adjoint_pair);
    assert_eq!(verification.triangle_1_failures(), 0);
    assert_eq!(verification.triangle_2_failures(), 0);
}

#[test]
fn non_irreducible_execution_has_zero_adjunction_gap() {
    // Even for non-irreducible computations, the adjunction itself
    // is well-defined and has zero gap (the gap measures Z/Z' structural
    // mismatch, not irreducibility per se)
    let cycling_tm = TuringMachine::builder()
        .states(vec![0, 1])
        .initial_state(0)
        .blank('_')
        .transition(0, '_', 1, '_', irreducible::machines::Direction::Stay)
        .transition(1, '_', 0, '_', irreducible::machines::Direction::Stay)
        .build();

    let history = cycling_tm.run("", 10);
    assert!(!history.is_irreducible());

    // Build states from the cycling execution
    let states: Vec<ComputationState> = (0..history.step_count())
        .map(|i| ComputationState::new(i, 1))
        .collect();

    let gap = ZPrimeAdjunction::adjunction_irreducibility_indicator(&states);
    assert!(
        gap.abs() < f64::EPSILON,
        "Expected zero gap for well-formed states, got {}",
        gap
    );
}

// ---------------------------------------------------------------------------
// Round-trip properties
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_z_then_zprime_recovers_interval() {
    let original = DiscreteInterval::new(3, 10);
    let state = ZPrimeAdjunction::z(&original);
    let recovered = ZPrimeAdjunction::zprime(&state);

    assert_eq!(recovered.start, original.start);
    assert_eq!(recovered.end, original.end);
}

#[test]
fn roundtrip_zprime_then_z_recovers_state() {
    let original = ComputationState::new(5, 10);
    let interval = ZPrimeAdjunction::zprime(&original);
    let recovered = ZPrimeAdjunction::z(&interval);

    assert_eq!(recovered.step, original.step);
    assert_eq!(recovered.complexity, original.complexity);
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn single_step_adjunction() {
    let state = ComputationState::new(0, 1);
    assert!(ZPrimeAdjunction::verify_triangle_1(&state));

    let interval = DiscreteInterval::new(0, 1);
    assert!(ZPrimeAdjunction::verify_triangle_2(&interval));

    let verification = AdjunctionVerification::verify_sequence(&[state]);
    assert!(verification.is_adjoint_pair);
}

#[test]
fn multi_step_contiguous_adjunction_sequence() {
    // Contiguous steps: (0,3) -> (3,4) -> (7,2)
    let states = vec![
        ComputationState::new(0, 3),
        ComputationState::new(3, 4),
        ComputationState::new(7, 2),
    ];

    let verification = AdjunctionVerification::verify_sequence(&states);
    assert!(verification.triangle_identities_hold);
    assert!(verification.is_adjoint_pair);

    // All triangle identities hold individually
    for (i, &result) in verification.triangle_1_results.iter().enumerate() {
        assert!(result, "Triangle 1 failed at index {}", i);
    }
    for (i, &result) in verification.triangle_2_results.iter().enumerate() {
        assert!(result, "Triangle 2 failed at index {}", i);
    }
}

#[test]
fn empty_states_indicator_is_zero() {
    let indicator = ZPrimeAdjunction::adjunction_irreducibility_indicator(&[]);
    assert!((indicator - 0.0).abs() < f64::EPSILON);
}
