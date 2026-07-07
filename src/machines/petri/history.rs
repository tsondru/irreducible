//! Execution history for [`super::PetriNetMachine`] and `StepTrace` impl.
//!
//! A [`PetriExecutionHistory`] records one [`PetriTransitionRecord`] per firing
//! step, along with the initial and final [`Marking`]. Because markings are
//! stored sparsely in a `HashMap<usize, Decimal>`, fingerprinting must sort by
//! place index to be order-independent; the [`Hash`] impl on `Marking` already
//! does this, so we reuse it via `DefaultHasher`.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use catgraph_applied::petri_net::Marking;

use crate::interval::DiscreteInterval;
use crate::trace::{self, StepTrace};

/// A single firing step in a Petri-net execution.
///
/// Records which transition fired and the marking before / after the firing.
/// The record is Lambda-agnostic: markings store token counts, not place
/// labels, so there is nothing Lambda-specific to track.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PetriTransitionRecord {
    /// Index of the transition that fired (into `PetriNet::transitions`).
    pub transition_idx: usize,
    /// Marking immediately before the firing.
    pub before: Marking,
    /// Marking immediately after the firing.
    pub after: Marking,
}

impl PetriTransitionRecord {
    /// Construct a transition record.
    #[must_use]
    pub fn new(transition_idx: usize, before: Marking, after: Marking) -> Self {
        Self {
            transition_idx,
            before,
            after,
        }
    }
}

/// Complete execution history of a [`super::PetriNetMachine`] run.
///
/// The history is deadlock-aware: `halted == true` iff no transition was
/// enabled at the final marking. If the run exits due to reaching
/// `max_steps` with transitions still enabled, `halted == false`.
#[derive(Clone, Debug)]
pub struct PetriExecutionHistory {
    /// Marking the run started from.
    pub initial: Marking,
    /// Firings in order.
    pub transitions: Vec<PetriTransitionRecord>,
    /// Marking reached after the last firing (or `initial` if no firings).
    pub final_marking: Marking,
    /// True iff the run ended because no transition was enabled.
    pub halted: bool,
}

impl PetriExecutionHistory {
    /// Check whether this execution is irreducible.
    ///
    /// Mirrors [`crate::ExecutionHistory::is_irreducible`]: delegates to
    /// [`crate::trace::analyze_trace`] and returns the `is_irreducible` verdict.
    #[must_use]
    pub fn is_irreducible(&self) -> bool {
        trace::analyze_trace(self).is_irreducible
    }
}

/// Stable, order-independent fingerprint for a [`Marking`].
///
/// `Marking`'s own [`Hash`] impl sorts entries by place index, so we can
/// feed it into a `DefaultHasher` directly and still get the same `u64` for
/// two markings that were built by inserting tokens in different orders.
fn marking_fingerprint(marking: &Marking) -> u64 {
    let mut hasher = DefaultHasher::new();
    marking.hash(&mut hasher);
    hasher.finish()
}

impl StepTrace for PetriExecutionHistory {
    fn state_fingerprints(&self) -> Vec<u64> {
        let mut fps = Vec::with_capacity(self.transitions.len() + 1);
        fps.push(marking_fingerprint(&self.initial));
        for record in &self.transitions {
            fps.push(marking_fingerprint(&record.after));
        }
        fps
    }

    fn to_intervals(&self) -> Vec<DiscreteInterval> {
        (0..self.transitions.len())
            .map(|i| DiscreteInterval::new(i, i + 1))
            .collect()
    }

    fn step_count(&self) -> usize {
        self.transitions.len()
    }

    fn halted(&self) -> bool {
        self.halted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn d(n: i64) -> Decimal {
        Decimal::from(n)
    }

    #[test]
    fn marking_fingerprint_is_insertion_order_independent() {
        let mut m1 = Marking::new();
        m1.set(0, d(3));
        m1.set(1, d(5));
        m1.set(2, d(7));

        let mut m2 = Marking::new();
        m2.set(2, d(7));
        m2.set(0, d(3));
        m2.set(1, d(5));

        assert_eq!(marking_fingerprint(&m1), marking_fingerprint(&m2));
    }

    #[test]
    fn history_fingerprints_include_initial_plus_after_each_step() {
        let initial = Marking::from_vec(vec![(0, d(1))]);
        let mid = Marking::from_vec(vec![(0, d(0)), (1, d(1))]);
        let final_marking = Marking::from_vec(vec![(1, d(0)), (2, d(1))]);

        let history = PetriExecutionHistory {
            initial: initial.clone(),
            transitions: vec![
                PetriTransitionRecord::new(0, initial.clone(), mid.clone()),
                PetriTransitionRecord::new(1, mid.clone(), final_marking.clone()),
            ],
            final_marking,
            halted: true,
        };

        let fps = history.state_fingerprints();
        assert_eq!(fps.len(), 3);
        assert_eq!(fps[0], marking_fingerprint(&history.initial));
    }

    #[test]
    fn history_intervals_are_contiguous() {
        let initial = Marking::new();
        let history = PetriExecutionHistory {
            initial: initial.clone(),
            transitions: vec![
                PetriTransitionRecord::new(0, initial.clone(), initial.clone()),
                PetriTransitionRecord::new(0, initial.clone(), initial.clone()),
                PetriTransitionRecord::new(0, initial.clone(), initial.clone()),
            ],
            final_marking: initial,
            halted: false,
        };

        let intervals = history.to_intervals();
        assert_eq!(
            intervals,
            vec![
                DiscreteInterval::new(0, 1),
                DiscreteInterval::new(1, 2),
                DiscreteInterval::new(2, 3),
            ]
        );
    }
}
