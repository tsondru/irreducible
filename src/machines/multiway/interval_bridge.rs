//! Interval-typed views of a multiway evolution graph.
//!
//! Both helpers report step boundaries as
//! [`catgraph_physics::interval`] values.

use catgraph::CanonicalEncode;
use catgraph_physics::interval::{DiscreteInterval, ParallelIntervals};
use catgraph_physics::multiway::{MultiwayEvolutionGraph, branchial_parallel_step_pairs};

/// For each leaf in `graph`, trace the path back to its root and emit one
/// `DiscreteInterval` per consecutive-step pair along the path.
///
/// Leaves whose path to the root has fewer than two nodes contribute no entry.
#[must_use]
pub fn branch_intervals<S: Clone + CanonicalEncode, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
) -> Vec<Vec<DiscreteInterval>> {
    let mut result = Vec::new();
    for &leaf in graph.leaves() {
        let path = graph.trace_path_to_root(leaf);
        if path.len() > 1 {
            let intervals: Vec<DiscreteInterval> = path
                .windows(2)
                .map(|w| DiscreteInterval::new(w[0].step, w[1].step))
                .collect();
            result.push(intervals);
        }
    }
    result
}

/// Compute `ParallelIntervals` from branchial foliation step boundaries.
///
/// One `ParallelIntervals` per adjacent foliation pair, holding one
/// `DiscreteInterval` per
/// [`catgraph_physics::multiway::branchial_parallel_step_pairs`] entry — i.e.
/// one branch per node at step `t` that has at least one forward transition.
#[must_use]
pub fn branchial_to_parallel_intervals<S: Clone + CanonicalEncode, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
) -> Vec<ParallelIntervals> {
    branchial_parallel_step_pairs(graph)
        .into_iter()
        .map(|pairs| {
            let mut intervals = ParallelIntervals::new();
            for (start, end) in pairs {
                intervals.add_branch(DiscreteInterval::new(start, end));
            }
            intervals
        })
        .collect()
}
