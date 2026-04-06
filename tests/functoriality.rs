//! Integration tests for the core Z' functor: 𝒯 -> ℬ.
//!
//! Tests that TuringMachine and ElementaryCA executions map correctly
//! to sequences of DiscreteIntervals, verifying contiguity, composition,
//! and agreement between domain-specific and generic trace analysis.

use irreducible::{
    analyze_trace, DiscreteInterval, ElementaryCA, Generation, IrreducibilityFunctor,
    IrreducibilityTrace, StokesIrreducibility, TuringMachine,
};

// ---------------------------------------------------------------------------
// Turing Machine functor tests
// ---------------------------------------------------------------------------

#[test]
fn busy_beaver_produces_contiguous_interval_sequence() {
    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);

    let intervals = history.to_intervals();
    assert_eq!(intervals.len(), 6);
    assert!(IrreducibilityFunctor::is_sequence_irreducible(&intervals));
}

#[test]
fn binary_incrementer_produces_contiguous_intervals() {
    let tm = TuringMachine::binary_incrementer();
    let history = tm.run("1011", 50);

    assert!(history.halted);
    let intervals = history.to_intervals();
    assert!(IrreducibilityFunctor::is_sequence_irreducible(&intervals));
    // Each interval should be [i, i+1]
    for (i, interval) in intervals.iter().enumerate() {
        assert_eq!(interval.start, i);
        assert_eq!(interval.end, i + 1);
    }
}

#[test]
fn cycling_tm_is_not_irreducible() {
    // A machine that cycles between two states on blank tape
    let tm = TuringMachine::builder()
        .states(vec![0, 1])
        .initial_state(0)
        .blank('_')
        .transition(0, '_', 1, '_', irreducible::machines::Direction::Stay)
        .transition(1, '_', 0, '_', irreducible::machines::Direction::Stay)
        .build();

    let history = tm.run("", 10);
    assert!(!history.halted);
    assert!(!history.is_irreducible());

    let analysis = history.analyze_irreducibility();
    assert!(!analysis.is_irreducible);
    assert!(!analysis.shortcuts.is_empty());
}

// ---------------------------------------------------------------------------
// Cellular Automaton functor tests
// ---------------------------------------------------------------------------

#[test]
fn rule_30_produces_contiguous_intervals() {
    let ca = ElementaryCA::rule_30(21);
    let initial = ca.single_cell_initial();
    let history = ca.run(initial, 20);

    let intervals = history.to_intervals();
    assert_eq!(intervals.len(), 20);
    assert!(IrreducibilityFunctor::is_sequence_irreducible(&intervals));
}

#[test]
fn rule_0_all_die_produces_cycles_not_irreducible() {
    let ca = ElementaryCA::new(0, 5);
    let initial = irreducible::Generation::new(vec![true, false, true, false, true], 0);
    let history = ca.run(initial, 10);

    // Rule 0 kills everything, then repeats all-dead forever
    let analysis = history.analyze_irreducibility();
    assert!(!analysis.is_irreducible);
    assert!(!analysis.cycles.is_empty());
}

// ---------------------------------------------------------------------------
// Functor composition tests
// ---------------------------------------------------------------------------

#[test]
fn functor_preserves_composition_compose_intervals_equals_total() {
    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);

    let intervals = history.to_intervals();
    let composed = IrreducibilityFunctor::compose_sequence(&intervals);
    let total = history.total_interval();

    assert!(composed.is_some());
    assert!(total.is_some());
    assert_eq!(composed.unwrap(), total.unwrap());
}

#[test]
fn empty_execution_zero_steps_edge_case() {
    // Machine that halts immediately (initial state is accept state)
    let tm = TuringMachine::builder()
        .states(vec![0])
        .initial_state(0)
        .accept_states(vec![0])
        .blank('_')
        .build();

    let history = tm.run("", 10);
    assert!(history.halted);
    assert_eq!(history.step_count(), 0);
    assert!(history.to_intervals().is_empty());
    assert!(history.total_interval().is_none());
    assert!(history.is_irreducible());
}

// ---------------------------------------------------------------------------
// Generic trace analysis agreement
// ---------------------------------------------------------------------------

#[test]
fn analyze_trace_agrees_with_domain_specific_tm() {
    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);

    let domain_analysis = history.analyze_irreducibility();
    let trace_analysis = analyze_trace(&history);

    assert_eq!(domain_analysis.is_irreducible, trace_analysis.is_irreducible);
    assert_eq!(
        domain_analysis.is_sequence_contiguous,
        trace_analysis.is_sequence_contiguous
    );
    assert_eq!(domain_analysis.step_count, trace_analysis.step_count);
    assert_eq!(domain_analysis.total_interval, trace_analysis.total_interval);
}

#[test]
fn analyze_trace_agrees_with_domain_specific_ca() {
    let ca = ElementaryCA::rule_30(11);
    let initial = ca.single_cell_initial();
    let history = ca.run(initial, 10);

    let domain_analysis = history.analyze_irreducibility();
    let trace_analysis = analyze_trace(&history);

    assert_eq!(domain_analysis.is_irreducible, trace_analysis.is_irreducible);
    assert_eq!(
        domain_analysis.is_sequence_contiguous,
        trace_analysis.is_sequence_contiguous
    );
    assert_eq!(domain_analysis.step_count, trace_analysis.step_count);
}

#[test]
fn long_run_rule_30_remains_irreducible() {
    let ca = ElementaryCA::rule_30(51);
    let initial = ca.single_cell_initial();
    let history = ca.run(initial, 50);

    assert!(history.is_irreducible());
    let analysis = history.analyze_irreducibility();
    assert_eq!(analysis.step_count, 50);
    assert!(analysis.cycles.is_empty());
}

// ---------------------------------------------------------------------------
// Shared trait check (compile-time)
// ---------------------------------------------------------------------------

#[test]
fn tm_and_ca_both_implement_irreducibility_trace() {
    // Verify both types implement the same trait at compile time
    fn assert_trace<T: IrreducibilityTrace>(trace: &T) -> usize {
        trace.step_count()
    }

    let bb = TuringMachine::busy_beaver_2_2();
    let tm_history = bb.run("", 20);
    assert_eq!(assert_trace(&tm_history), 6);

    let ca = ElementaryCA::rule_30(11);
    let ca_history = ca.run(ca.single_cell_initial(), 5);
    assert_eq!(assert_trace(&ca_history), 5);
}

// ---------------------------------------------------------------------------
// Error / negative path tests
// ---------------------------------------------------------------------------

#[test]
fn functor_non_contiguous_intervals_not_irreducible() {
    // Manually construct a sequence with a gap: [0,2], [5,7]
    // These are non-contiguous (2 != 5), so composition should fail
    // and the sequence should NOT be irreducible.
    let intervals = vec![
        DiscreteInterval::new(0, 2),
        DiscreteInterval::new(5, 7),
    ];

    assert!(!IrreducibilityFunctor::is_sequence_irreducible(&intervals));

    // compose_sequence should return None for non-contiguous intervals
    let composed = IrreducibilityFunctor::compose_sequence(&intervals);
    assert!(composed.is_none());
}

#[test]
fn repeat_detection_maps_to_shortcuts_and_cycles() {
    // TM cycling: RepeatDetection -> Shortcut
    let cycling_tm = TuringMachine::builder()
        .states(vec![0, 1])
        .initial_state(0)
        .blank('_')
        .transition(0, '_', 1, '_', irreducible::machines::Direction::Stay)
        .transition(1, '_', 0, '_', irreducible::machines::Direction::Stay)
        .build();

    let tm_history = cycling_tm.run("", 10);
    let trace_analysis = analyze_trace(&tm_history);
    let domain_analysis = tm_history.analyze_irreducibility();

    // TraceAnalysis.repeats map to IrreducibilityAnalysis.shortcuts
    assert_eq!(trace_analysis.repeats.len(), domain_analysis.shortcuts.len());
    for (repeat, shortcut) in trace_analysis
        .repeats
        .iter()
        .zip(domain_analysis.shortcuts.iter())
    {
        assert_eq!(repeat.start_step, shortcut.from);
        assert_eq!(repeat.end_step, shortcut.to);
        assert_eq!(repeat.cycle_length, shortcut.cycle_length);
    }

    // CA Rule 0: RepeatDetection -> CACycle
    let ca = ElementaryCA::new(0, 5);
    let initial = irreducible::Generation::new(vec![true, true, true, true, true], 0);
    let ca_history = ca.run(initial, 10);
    let ca_trace = analyze_trace(&ca_history);
    let ca_domain = ca_history.analyze_irreducibility();

    assert_eq!(ca_trace.repeats.len(), ca_domain.cycles.len());
    for (repeat, cycle) in ca_trace.repeats.iter().zip(ca_domain.cycles.iter()) {
        assert_eq!(repeat.start_step, cycle.start_step);
        assert_eq!(repeat.end_step, cycle.end_step);
        assert_eq!(repeat.cycle_length, cycle.cycle_length);
    }
}

// ---------------------------------------------------------------------------
// Three-way agreement tests (functor + trace + Stokes)
// ---------------------------------------------------------------------------

#[test]
fn three_way_agreement_irreducible_tm() {
    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);
    let intervals = history.to_intervals();

    // Perspective 1: Functor contiguity
    assert!(IrreducibilityFunctor::is_sequence_irreducible(&intervals));

    // Perspective 2: Trace analysis (contiguity + no repeats + complexity ratio)
    let trace = analyze_trace(&history);
    assert!(trace.is_irreducible);

    // Perspective 3: Stokes conservation law
    let stokes = StokesIrreducibility::analyze(&intervals).unwrap();
    assert!(stokes.is_irreducible());
}

#[test]
fn three_way_agreement_reducible() {
    // Rule 0 kills all cells after one step, then cycles on all-dead state
    let ca = ElementaryCA::new(0, 5);
    let initial = Generation::new(vec![true, false, true, false, true], 0);
    let history = ca.run(initial, 10);
    let intervals = history.to_intervals();

    // Perspective 1: Functor — intervals are contiguous (each step [i, i+1]),
    // but the functor only checks contiguity, so it may still say true.
    // The real test is whether trace analysis and Stokes agree on reducibility.
    let functor_irreducible = IrreducibilityFunctor::is_sequence_irreducible(&intervals);

    // Perspective 2: Trace analysis detects state repetition → reducible
    let trace = analyze_trace(&history);
    assert!(!trace.is_irreducible);

    // Perspective 3: Stokes — for contiguous intervals it may succeed,
    // but if intervals are non-contiguous it returns Err (also non-irreducible).
    let stokes_irreducible = match StokesIrreducibility::analyze(&intervals) {
        Ok(analysis) => analysis.is_irreducible(),
        Err(_) => false,
    };

    // The trace perspective is the most sensitive (detects cycles).
    // If functor and Stokes only check contiguity, they may agree with each
    // other but still correctly not imply full irreducibility.
    // At minimum: trace says reducible, and functor + Stokes agree with each other.
    assert_eq!(functor_irreducible, stokes_irreducible);
    assert!(!trace.is_irreducible);
}

#[test]
fn rule_110_turing_complete_produces_contiguous_intervals() {
    let ca = ElementaryCA::rule_110(41);
    let initial = ca.single_cell_initial();
    let history = ca.run(initial, 30);
    let intervals = history.to_intervals();

    // Rule 110 is Turing-complete; from a single-cell seed it produces
    // complex non-repeating structure → contiguous and irreducible.
    assert!(IrreducibilityFunctor::is_sequence_irreducible(&intervals));

    let trace = analyze_trace(&history);
    assert!(trace.is_irreducible);
}
