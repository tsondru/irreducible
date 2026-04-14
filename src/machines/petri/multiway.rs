//! Non-deterministic multiway reachability for [`super::PetriNetMachine`].
//!
//! Where [`super::PetriNetMachine::run`] picks a single enabled transition per
//! step, [`run_multiway_reachability`] explores **every** enabled firing at
//! every step, producing a branching `MultiwayEvolutionGraph` compatible with
//! the branchial / curvature infrastructure re-used by SRS / NTM / hypergraph.
//!
//! Internally this is a thin wrapper over
//! [`catgraph_physics::multiway::run_multiway_bfs`] with
//! `step_fn = |m| net.enabled(m).map(|i| (net.fire(i, m)?, record, cost))`.

use std::fmt::Debug;

use catgraph_applied::petri_net::Marking;
use catgraph_physics::multiway::{run_multiway_bfs, MultiwayEvolutionGraph};

use super::history::PetriTransitionRecord;
use super::machine::PetriNetMachine;

/// Explore the multiway reachability graph from `initial`.
///
/// At each marking, every enabled transition produces an outgoing edge to the
/// resulting marking. BFS continues to depth `max_steps` or until
/// `max_branches` markings have been visited, whichever comes first.
///
/// Cost per firing is hard-coded to `1` (unit-cost steps), matching the
/// convention of SRS / NTM multiway runs elsewhere in this crate.
#[must_use]
pub fn run_multiway_reachability<Lambda>(
    machine: &PetriNetMachine<Lambda>,
    initial: Marking,
    max_steps: usize,
    max_branches: usize,
) -> MultiwayEvolutionGraph<Marking, PetriTransitionRecord>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    let net = machine.net();
    run_multiway_bfs(
        initial,
        |m| {
            let enabled = net.enabled(m);
            let mut out = Vec::with_capacity(enabled.len());
            for idx in enabled {
                if let Ok(next) = net.fire(idx, m) {
                    let record = PetriTransitionRecord::new(idx, m.clone(), next.clone());
                    out.push((next, record, 1));
                }
            }
            out
        },
        max_steps,
        max_branches,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::PetriBuilder;
    use rust_decimal::Decimal;

    /// Smoke test: a net with no enabled transitions returns a graph whose root
    /// has no children and does not panic inside the closure.
    #[test]
    fn empty_enabled_returns_singleton_graph() {
        let machine: PetriNetMachine<char> = PetriBuilder::new()
            .place('A')
            .transition(vec![(0, Decimal::ONE)], vec![])
            .build();

        let evolution = run_multiway_reachability(&machine, Marking::new(), 5, 16);
        let stats = evolution.statistics();
        assert_eq!(stats.total_nodes, 1, "expected only the root marking");
    }

    /// Two enabled transitions from the root must both appear as outgoing edges.
    #[test]
    fn both_enabled_transitions_produce_edges() {
        let machine: PetriNetMachine<char> = PetriBuilder::new()
            .place('A')
            .place('B')
            .place('X')
            .place('Y')
            .transition(vec![(0, Decimal::ONE)], vec![(2, Decimal::ONE)])
            .transition(vec![(1, Decimal::ONE)], vec![(3, Decimal::ONE)])
            .build();

        let initial = Marking::from_vec(vec![(0, Decimal::ONE), (1, Decimal::ONE)]);
        let evolution = run_multiway_reachability(&machine, initial, 2, 16);
        let stats = evolution.statistics();
        assert!(stats.max_branches >= 2);
    }
}
