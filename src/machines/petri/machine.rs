//! [`PetriNetMachine`]: linear-trace wrapper around [`PetriNet`].
//!
//! The machine is intentionally thin — it owns a [`PetriNet`] and provides
//! deterministic stepping semantics suitable for [`crate::trace::StepTrace`].
//! For branching exploration, use [`super::run_multiway_reachability`].

use std::fmt::Debug;

use catgraph::cospan::Cospan;
use catgraph_applied::petri_net::{Marking, PetriNet};

use super::builder::PetriBuilder;
use super::history::{PetriExecutionHistory, PetriTransitionRecord};

/// A Petri net equipped with deterministic linear-trace firing semantics.
///
/// ## Firing rule
///
/// At each step, the enabled transition with the **smallest index** is fired.
/// This is a non-obvious design choice: it produces a single deterministic
/// trajectory suitable for [`crate::trace::StepTrace`], matching
/// the TM/CA shape. If you want to explore every enabled firing (true
/// non-determinism), use [`super::run_multiway_reachability`] instead.
///
/// ## Halting
///
/// A run halts when `net.enabled(&marking).is_empty()`. If `max_steps` is
/// reached while transitions are still enabled, the returned history has
/// `halted == false` — this distinguishes deadlock from step-limit exhaustion.
#[derive(Clone, Debug)]
pub struct PetriNetMachine<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    net: PetriNet<Lambda>,
}

impl<Lambda> PetriNetMachine<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    /// Wrap an existing [`PetriNet`] as a machine.
    #[must_use]
    pub fn new(net: PetriNet<Lambda>) -> Self {
        Self { net }
    }

    /// Start a fluent [`PetriBuilder`] for constructing a machine place-by-place.
    #[must_use]
    pub fn builder() -> PetriBuilder<Lambda> {
        PetriBuilder::new()
    }

    /// Borrow the underlying [`PetriNet`].
    #[must_use]
    pub fn net(&self) -> &PetriNet<Lambda> {
        &self.net
    }

    /// Execute the deterministic linear trace from `initial` for up to `max_steps`.
    ///
    /// Picks the smallest-index enabled transition at each step. Returns
    /// with `halted == true` iff a marking was reached where no transition
    /// was enabled; returns with `halted == false` iff `max_steps` was hit
    /// while transitions were still enabled.
    ///
    /// # Panics
    ///
    /// Panics only if the underlying [`PetriNet::fire`] fails on a transition
    /// that [`PetriNet::enabled`] just reported as enabled — this indicates
    /// a bug in `catgraph-applied`, not user error.
    #[must_use]
    pub fn run(&self, initial: Marking, max_steps: usize) -> PetriExecutionHistory {
        let mut current = initial.clone();
        let mut transitions = Vec::new();
        let mut halted = false;

        for _ in 0..max_steps {
            let enabled = self.net.enabled(&current);
            if enabled.is_empty() {
                halted = true;
                break;
            }
            let idx = enabled[0];
            let next = self
                .net
                .fire(idx, &current)
                .expect("enabled() reported transition must be fireable");
            transitions.push(PetriTransitionRecord::new(
                idx,
                current.clone(),
                next.clone(),
            ));
            current = next;
        }

        // Final halt-check: if we exited the loop normally but nothing is
        // enabled any more, still flag as halted.
        if !halted && self.net.enabled(&current).is_empty() {
            halted = true;
        }

        PetriExecutionHistory {
            initial,
            transitions,
            final_marking: current,
            halted,
        }
    }

    /// Convert transition `idx` to its cospan representation via
    /// [`PetriNet::transition_as_cospan`]. Thin delegate.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is out of bounds or any arc weight is not representable
    /// as `u64`. See [`PetriNet::transition_as_cospan`].
    #[must_use]
    pub fn transition_as_cospan(&self, idx: usize) -> Cospan<Lambda> {
        self.net.transition_as_cospan(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catgraph_applied::petri_net::Transition;
    use rust_decimal::Decimal;

    fn d(n: i64) -> Decimal {
        Decimal::from(n)
    }

    /// Two places, one transition consuming one token from place 0 and
    /// producing one in place 1. Runs 3 steps from a starting marking of 3
    /// tokens in place 0.
    fn producer_consumer() -> PetriNetMachine<char> {
        let net = PetriNet::new(
            vec!['R', 'D'],
            vec![Transition::new(vec![(0, Decimal::ONE)], vec![(1, Decimal::ONE)])],
        );
        PetriNetMachine::new(net)
    }

    #[test]
    fn run_halts_when_no_transition_enabled() {
        let machine = producer_consumer();
        let history = machine.run(Marking::from_vec(vec![(0, d(3))]), 10);

        assert_eq!(history.step_count_trace(), 3);
        assert!(history.halted);
        assert_eq!(history.final_marking.get(0), Decimal::ZERO);
        assert_eq!(history.final_marking.get(1), d(3));
    }

    #[test]
    fn run_picks_smallest_enabled_index() {
        // Two independent transitions both enabled from marking (1, 1).
        // Transition 0 consumes from place 0, transition 1 from place 1.
        // The firing rule must pick index 0 first.
        let net = PetriNet::new(
            vec!['A', 'B', 'C', 'D'],
            vec![
                Transition::new(vec![(0, Decimal::ONE)], vec![(2, Decimal::ONE)]),
                Transition::new(vec![(1, Decimal::ONE)], vec![(3, Decimal::ONE)]),
            ],
        );
        let machine = PetriNetMachine::new(net);
        let history = machine.run(Marking::from_vec(vec![(0, d(1)), (1, d(1))]), 5);

        assert_eq!(history.transitions[0].transition_idx, 0);
        assert_eq!(history.transitions[1].transition_idx, 1);
        assert!(history.halted);
    }

    #[test]
    fn builder_happy_path() {
        let machine: PetriNetMachine<char> = PetriNetMachine::builder()
            .place('R')
            .place('D')
            .transition(vec![(0, Decimal::ONE)], vec![(1, Decimal::ONE)])
            .build();
        assert_eq!(machine.net().place_count(), 2);
        assert_eq!(machine.net().transition_count(), 1);
    }

    /// Helper so `step_count` doesn't clash with the `StepTrace` method.
    trait StepCount {
        fn step_count_trace(&self) -> usize;
    }

    impl StepCount for PetriExecutionHistory {
        fn step_count_trace(&self) -> usize {
            self.transitions.len()
        }
    }
}
