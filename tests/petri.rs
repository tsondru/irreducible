//! Integration tests for the Petri-net computation model.
//!
//! Covers the four perspectives: linear trace, multiway reachability, cospan
//! bridge round-trip, deadlock detection, and step-limit exhaustion.

use irreducible::machines::petri::{
    run_multiway_reachability, Marking, PetriBuilder, PetriNet, PetriNetMachine, PetriTransition,
};
use irreducible::trace::analyze_trace;
use rust_decimal::Decimal;

fn d(n: i64) -> Decimal {
    Decimal::from(n)
}

/// Two places: `R` (ready) and `D` (done). One transition consuming one token
/// from R and producing one in D. Start with 3 tokens in R.
fn producer_consumer() -> PetriNetMachine<char> {
    PetriBuilder::new()
        .place('R')
        .place('D')
        .transition(vec![(0, Decimal::ONE)], vec![(1, Decimal::ONE)])
        .build()
}

#[test]
fn producer_consumer_linear() {
    let machine = producer_consumer();
    let history = machine.run(Marking::from_vec(vec![(0, d(3))]), 10);

    assert_eq!(history.step_count_trace(), 3);
    assert!(history.halted);

    let analysis = analyze_trace(&history);
    assert!(analysis.is_irreducible);
    assert!(analysis.is_sequence_contiguous);

    assert_eq!(history.final_marking.get(0), Decimal::ZERO);
    assert_eq!(history.final_marking.get(1), d(3));
}

#[test]
fn multiway_branching() {
    // Two independent transitions both enabled from the same marking:
    // t0 consumes from place 0, t1 consumes from place 1. Starting with
    // one token in each place yields a branching factor of 2 at the root.
    let machine: PetriNetMachine<char> = PetriBuilder::new()
        .place('A')
        .place('B')
        .place('X')
        .place('Y')
        .transition(vec![(0, Decimal::ONE)], vec![(2, Decimal::ONE)])
        .transition(vec![(1, Decimal::ONE)], vec![(3, Decimal::ONE)])
        .build();

    let initial = Marking::from_vec(vec![(0, d(1)), (1, d(1))]);
    let evolution = run_multiway_reachability(&machine, initial, 3, 16);

    let stats = evolution.statistics();
    assert!(
        stats.max_branches >= 2,
        "expected branching factor ≥ 2, got {}",
        stats.max_branches
    );
}

#[test]
fn cospan_bridge_roundtrip() {
    // 2 H + O → W: two tokens from place 0, one from place 1, produce two
    // tokens at place 2. The cospan round-trip should preserve arc weights.
    let machine: PetriNetMachine<char> = PetriNetMachine::new(PetriNet::new(
        vec!['H', 'O', 'W'],
        vec![PetriTransition::new(
            vec![(0, d(2)), (1, Decimal::ONE)],
            vec![(2, d(2))],
        )],
    ));

    let cospan = machine.transition_as_cospan(0);
    let roundtrip = PetriNet::from_cospan(&cospan);

    assert_eq!(roundtrip.place_count(), machine.net().place_count());
    assert_eq!(
        roundtrip.arc_weight_pre(0, 0),
        machine.net().arc_weight_pre(0, 0)
    );
    assert_eq!(
        roundtrip.arc_weight_pre(1, 0),
        machine.net().arc_weight_pre(1, 0)
    );
    assert_eq!(
        roundtrip.arc_weight_post(2, 0),
        machine.net().arc_weight_post(2, 0)
    );
}

#[test]
fn deadlock_before_step_limit() {
    // One transition requiring a token in place 0 — but the initial marking
    // is empty, so nothing is ever enabled. Expect immediate halt at step 0.
    let machine: PetriNetMachine<char> = PetriBuilder::new()
        .place('A')
        .place('B')
        .transition(vec![(0, Decimal::ONE)], vec![(1, Decimal::ONE)])
        .build();

    let history = machine.run(Marking::new(), 10);
    assert!(history.halted);
    assert_eq!(history.step_count_trace(), 0);
    assert!(history.transitions.is_empty());
}

#[test]
fn step_limit_with_more_enabled() {
    // Transition consumes one from place 0 and produces one back in place 0
    // — always re-enabling itself. With max_steps=5 and infinite supply, we
    // should exit with halted=false.
    let machine: PetriNetMachine<char> = PetriBuilder::new()
        .place('A')
        .transition(vec![(0, Decimal::ONE)], vec![(0, Decimal::ONE)])
        .build();

    let history = machine.run(Marking::from_vec(vec![(0, d(1))]), 5);
    assert!(!history.halted);
    assert_eq!(history.step_count_trace(), 5);
}

/// Inline extension trait so we can assert on firing-step count without
/// shadowing the `StepTrace::step_count` method name.
trait StepCount {
    fn step_count_trace(&self) -> usize;
}

impl StepCount for irreducible::machines::petri::PetriExecutionHistory {
    fn step_count_trace(&self) -> usize {
        self.transitions.len()
    }
}
