//! Integration tests for core computation types.
//!
//! Tests ComputationDomain, ComputationContext, CausalEffect,
//! DiscreteInterval composition and contiguity, ComputationState-to-interval
//! mapping, Complexity from step count, and ParallelIntervals construction.

use irreducible::{
    CausalEffect, Complexity, ComputationContext, ComputationDomain, ComputationState,
    DiscreteInterval, ParallelIntervals, StepCount,
};

// ---------------------------------------------------------------------------
// ComputationDomain
// ---------------------------------------------------------------------------

#[test]
fn computation_domain_variants_and_names() {
    let tm = ComputationDomain::TuringMachine {
        state: 2,
        head_pos: -5,
    };
    assert_eq!(tm.name(), "TuringMachine");
    assert!(!tm.is_multiway());

    let ca = ComputationDomain::CellularAutomaton {
        rule: 30,
        population: 15,
    };
    assert_eq!(ca.name(), "CellularAutomaton");
    assert!(!ca.is_multiway());

    let multiway = ComputationDomain::Multiway {
        branch_id: 42,
        depth: 5,
        state_hash: 0xDEAD_BEEF,
    };
    assert_eq!(multiway.name(), "Multiway");
    assert!(multiway.is_multiway());

    let ntm = ComputationDomain::NondeterministicTM {
        state: 1,
        head_pos: 3,
        branch_id: 7,
        choices: 3,
    };
    assert_eq!(ntm.name(), "NondeterministicTM");
    assert!(ntm.is_multiway());

    let srs = ComputationDomain::StringRewrite {
        string_length: 10,
        applicable_rules: 2,
        branch_id: 5,
    };
    assert_eq!(srs.name(), "StringRewrite");
    assert!(srs.is_multiway());
}

#[test]
fn computation_domain_default_is_generic_unknown() {
    let domain = ComputationDomain::default();
    assert_eq!(domain.name(), "unknown");
    assert!(!domain.is_multiway());
}

// ---------------------------------------------------------------------------
// ComputationContext
// ---------------------------------------------------------------------------

#[test]
fn computation_context_creation_and_fields() {
    let ctx = ComputationContext::new(
        ComputationDomain::TuringMachine {
            state: 1,
            head_pos: 3,
        },
        5,
    );

    assert_eq!(ctx.step, 5);
    assert_eq!(ctx.domain.name(), "TuringMachine");
    assert!(ctx.complexity_estimated.is_none());
    assert!(ctx.metadata.is_empty());
}

#[test]
fn computation_context_with_complexity_and_metadata() {
    let ctx = ComputationContext::with_complexity(
        ComputationDomain::CellularAutomaton {
            rule: 110,
            population: 20,
        },
        10,
        42.5,
    )
    .with_metadata("note", "test");

    assert_eq!(ctx.step, 10);
    assert_eq!(ctx.complexity_estimated, Some(42.5));
    assert_eq!(ctx.get_metadata("note"), Some("test"));
    assert_eq!(ctx.get_metadata("missing"), None);
}

#[test]
fn computation_context_serialization_roundtrip() {
    let ctx = ComputationContext::with_complexity(
        ComputationDomain::TuringMachine {
            state: 2,
            head_pos: 5,
        },
        10,
        25.0,
    )
    .with_metadata("key", "value");

    let json = serde_json::to_string(&ctx).unwrap();
    let recovered: ComputationContext = serde_json::from_str(&json).unwrap();
    assert_eq!(ctx, recovered);
}

// ---------------------------------------------------------------------------
// DiscreteInterval composition and contiguity
// ---------------------------------------------------------------------------

#[test]
fn discrete_interval_composition_contiguous() {
    let a = DiscreteInterval::new(0, 3);
    let b = DiscreteInterval::new(3, 7);

    assert!(a.is_composable_with(&b));
    let composed = a.then(b).unwrap();
    assert_eq!(composed.start, 0);
    assert_eq!(composed.end, 7);
}

#[test]
fn discrete_interval_composition_non_contiguous_returns_none() {
    let a = DiscreteInterval::new(0, 3);
    let b = DiscreteInterval::new(5, 7);

    assert!(!a.is_composable_with(&b));
    assert!(a.then(b).is_none());
}

#[test]
fn discrete_interval_edge_cases() {
    // Singleton (identity morphism)
    let id = DiscreteInterval::singleton(5);
    assert!(id.is_identity());
    assert_eq!(id.cardinality(), 1);
    assert_eq!(id.steps(), 0);

    // Zero-length interval [n, n] contains exactly n
    assert!(id.contains(5));
    assert!(!id.contains(4));
    assert!(!id.contains(6));
}

#[test]
fn discrete_interval_intersection() {
    let a = DiscreteInterval::new(0, 5);
    let b = DiscreteInterval::new(3, 8);

    let intersection = a.intersect(&b).unwrap();
    assert_eq!(intersection.start, 3);
    assert_eq!(intersection.end, 5);
}

#[test]
fn discrete_interval_no_intersection() {
    let a = DiscreteInterval::new(0, 3);
    let b = DiscreteInterval::new(5, 8);

    assert!(a.intersect(&b).is_none());
}

#[test]
fn discrete_interval_contains_interval() {
    let outer = DiscreteInterval::new(0, 10);
    let inner = DiscreteInterval::new(2, 5);

    assert!(outer.contains_interval(&inner));
    assert!(!inner.contains_interval(&outer));
}

// ---------------------------------------------------------------------------
// ComputationState to interval mapping
// ---------------------------------------------------------------------------

#[test]
fn computation_state_to_interval_mapping() {
    let state = ComputationState::new(3, 5);
    let interval = state.to_interval();

    assert_eq!(interval.start, 3);
    assert_eq!(interval.end, 8); // 3 + 5
}

#[test]
fn computation_state_initial() {
    let initial = ComputationState::initial();
    assert_eq!(initial.step, 0);
    assert_eq!(initial.complexity, 0);

    // to_interval uses max(1, complexity) so initial maps to [0, 1]
    let interval = initial.to_interval();
    assert_eq!(interval.start, 0);
    assert_eq!(interval.end, 1);
}

#[test]
fn computation_state_next_advances() {
    let state = ComputationState::new(3, 5);
    let next = state.next();

    assert_eq!(next.step, 4);
    assert_eq!(next.complexity, 6);
}

// ---------------------------------------------------------------------------
// Complexity from step count
// ---------------------------------------------------------------------------

#[test]
fn step_count_complexity() {
    let a = StepCount(3);
    let b = StepCount(5);

    assert_eq!(a.as_steps(), 3);
    assert_eq!(b.as_steps(), 5);

    let seq = a.sequential(&b);
    assert_eq!(seq.as_steps(), 8);
}

// ---------------------------------------------------------------------------
// ParallelIntervals
// ---------------------------------------------------------------------------

#[test]
fn parallel_intervals_creation_and_properties() {
    let mut pi = ParallelIntervals::new();
    assert!(pi.is_singleway()); // 0 branches = singleway
    assert_eq!(pi.branch_count(), 0);

    pi.add_branch(DiscreteInterval::new(0, 5));
    assert!(pi.is_singleway()); // 1 branch = still singleway
    assert_eq!(pi.branch_count(), 1);
    assert_eq!(pi.total_complexity(), 6); // cardinality of [0,5] = 6

    pi.add_branch(DiscreteInterval::new(0, 3));
    assert!(!pi.is_singleway()); // 2 branches = multiway
    assert_eq!(pi.branch_count(), 2);
    assert_eq!(pi.max_complexity(), 6); // max(6, 4) = 6
}

#[test]
fn parallel_intervals_tensor_product() {
    let a = ParallelIntervals::from_branch(DiscreteInterval::new(0, 5));
    let b = ParallelIntervals::from_branch(DiscreteInterval::new(10, 15));
    let combined = a.tensor(b);

    assert_eq!(combined.branch_count(), 2);
    assert_eq!(combined.branches[0], DiscreteInterval::new(0, 5));
    assert_eq!(combined.branches[1], DiscreteInterval::new(10, 15));
}

// ---------------------------------------------------------------------------
// CausalEffect
// ---------------------------------------------------------------------------

#[test]
fn causal_effect_success_and_map() {
    let effect = CausalEffect::success(42);
    assert!(effect.is_success());
    assert_eq!(effect.value, Some(42));

    let doubled = effect.map(|x| x * 2);
    assert!(doubled.is_success());
    assert_eq!(doubled.value, Some(84));
}

#[test]
fn causal_effect_error_and_log() {
    let effect: CausalEffect<i32> = CausalEffect::error("failed")
        .with_log("step 1")
        .with_log("step 2");

    assert!(!effect.is_success());
    assert!(effect.has_error);
    assert_eq!(effect.error_message, Some("failed".to_string()));
    assert_eq!(effect.log_entries.len(), 2);
}

#[test]
fn causal_effect_json_roundtrip() {
    let effect = CausalEffect::success(42).with_log("computed");
    let json = effect.to_json().unwrap();
    let recovered = CausalEffect::<i32>::from_json(&json).unwrap();

    assert_eq!(recovered.value, Some(42));
    assert!(recovered.is_success());
    assert_eq!(recovered.log_entries, vec!["computed"]);
}

// ---------------------------------------------------------------------------
// Builder error paths (TM and NTM try_build)
// ---------------------------------------------------------------------------

#[test]
fn tm_try_build_missing_blank_returns_error() {
    use irreducible::machines::{Direction, TuringMachineBuilder};
    use irreducible::BuilderError;

    let result = TuringMachineBuilder::new()
        .states(vec![0, 1])
        .initial_state(0)
        .accept_states(vec![1])
        .transition(0, '1', 1, '1', Direction::Right)
        .try_build();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), BuilderError::MissingBlank);
}

#[test]
fn tm_try_build_missing_initial_state_returns_error() {
    use irreducible::machines::TuringMachineBuilder;
    use irreducible::BuilderError;

    let result = TuringMachineBuilder::new()
        .states(vec![0, 1])
        .blank('_')
        .accept_states(vec![1])
        .try_build();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), BuilderError::MissingInitialState);
}

#[test]
fn ntm_try_build_missing_blank_returns_error() {
    use irreducible::machines::Direction;
    use irreducible::{BuilderError, NTMBuilder};

    let result = NTMBuilder::new()
        .states(vec![0, 1])
        .initial_state(0)
        .accept_states(vec![1])
        .transition(0, '0', vec![(1, '0', Direction::Right)])
        .try_build();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), BuilderError::MissingBlank);
}

#[test]
fn ntm_try_build_missing_initial_state_returns_error() {
    use irreducible::{BuilderError, NTMBuilder};

    let result = NTMBuilder::new()
        .states(vec![0, 1])
        .blank('_')
        .accept_states(vec![1])
        .try_build();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), BuilderError::MissingInitialState);
}

// ---------------------------------------------------------------------------
// Internal machine types (Tape, Configuration, Transition)
// ---------------------------------------------------------------------------

#[test]
fn tape_from_input_read_write() {
    use irreducible::machines::{Symbol, Tape};

    let mut tape = Tape::from_input("101", '_');

    // Verify reads at positions
    assert_eq!(tape.read(0), '1');
    assert_eq!(tape.read(1), '0');
    assert_eq!(tape.read(2), '1');
    assert_eq!(tape.read(3), '_'); // beyond input => blank

    // Write a symbol and verify
    tape.write(1, 'X');
    assert_eq!(tape.read(1), 'X');
    assert_eq!(tape.content_string(), "1X1");

    // Verify the Symbol type alias is char
    let s: Symbol = 'A';
    assert_eq!(s, 'A');
}

#[test]
fn configuration_initial_and_fingerprint() {
    use irreducible::machines::Configuration;

    let config = Configuration::initial("101", 0, '_');

    // Fingerprint is deterministic
    let fp1 = config.fingerprint();
    let fp2 = config.fingerprint();
    assert_eq!(fp1, fp2);

    // Display output contains state and head position info
    let display = format!("{config}");
    assert!(
        display.contains("q0"),
        "Display should contain state q0, got: {display}"
    );
    // Head is at position 0, so the first symbol should be bracketed
    assert!(
        display.contains("[1]"),
        "Display should show head position with brackets, got: {display}"
    );
}

#[test]
fn transition_to_interval_and_complexity() {
    use irreducible::machines::{Configuration, Tape, Transition};

    let from = Configuration::new(Tape::from_input("ab", '_'), 0, 0);
    let to = Configuration::new(Tape::from_input("ab", '_'), 1, 1);
    let transition = Transition::new(from, to, 3);

    // to_interval maps step 3 to [3, 4]
    let interval = transition.to_interval();
    assert_eq!(interval.start, 3);
    assert_eq!(interval.end, 4);
    assert_eq!(interval.steps(), 1);

    // complexity is always 1 for an elementary transition
    let complexity = transition.complexity();
    assert!(complexity.as_steps() > 0);
    assert_eq!(complexity.as_steps(), 1);
}

#[test]
fn tape_fingerprint_differs_for_different_content() {
    use irreducible::machines::Tape;

    let tape_a = Tape::from_input("abc", '_');
    let tape_b = Tape::from_input("xyz", '_');

    assert_ne!(
        tape_a.fingerprint(),
        tape_b.fingerprint(),
        "Tapes with different content should have different fingerprints"
    );
}

// ---------------------------------------------------------------------------
// Error / negative path tests
// ---------------------------------------------------------------------------

#[test]
fn discrete_interval_try_new_invalid() {
    // start > end should return Err
    let result = DiscreteInterval::try_new(10, 5);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("start") && msg.contains("end"),
        "Error message should mention start and end, got: {msg}"
    );
}

#[test]
fn computation_state_zero_complexity_interval() {
    // A state with zero complexity should still map to a valid interval.
    // Per the source: `complexity.max(1)` ensures at least [step, step+1].
    let state = ComputationState::new(7, 0);
    let interval = state.to_interval();

    assert_eq!(interval.start, 7);
    assert_eq!(interval.end, 8); // 7 + max(0,1) = 8
    assert_eq!(interval.steps(), 1);
    assert!(!interval.is_identity()); // Not a singleton because end != start
}
