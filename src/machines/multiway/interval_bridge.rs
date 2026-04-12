//! Interval bridge helpers between `catgraph::multiway` and irreducible's
//! local `interval` module.
//!
//! catgraph v0.10.5 moved `interval` out of catgraph into irreducible, so the
//! two helper functions that used to return `DiscreteInterval` /
//! `ParallelIntervals` (`MultiwayEvolutionGraph::to_branch_intervals` and
//! `branchial_parallel_step_pairs` née `branchial_to_parallel_intervals`) no
//! longer exist in catgraph. This module reimplements them on the irreducible
//! side using catgraph's public multiway API.

use std::hash::Hash;

use catgraph_physics::multiway::{
    extract_branchial_foliation, MultiwayEvolutionGraph, MultiwayNodeId,
};

use crate::interval::{DiscreteInterval, ParallelIntervals};

/// For each leaf in `graph`, trace the path back to its root and emit one
/// `DiscreteInterval` per consecutive-step pair along the path.
///
/// This is the irreducible-side replacement for the
/// `MultiwayEvolutionGraph::to_branch_intervals` method that used to live
/// in catgraph prior to v0.10.5.
#[must_use]
pub fn branch_intervals<S: Clone + Hash, T: Clone>(
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
/// At each adjacent pair `(step_t, step_{t+1})` of the branchial foliation,
/// produce one `DiscreteInterval::new(step_t, step_{t+1})` per node at step
/// `t` that has at least one forward transition — i.e., one interval per
/// parallel branch that is still evolving across the boundary.
///
/// This is the irreducible-side replacement for
/// `catgraph::multiway::branchial_to_parallel_intervals` (removed in
/// catgraph v0.10.5 when `ParallelIntervals` moved to irreducible).
#[must_use]
pub fn branchial_to_parallel_intervals<S: Clone + Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
) -> Vec<ParallelIntervals> {
    let foliation = extract_branchial_foliation(graph);

    foliation
        .windows(2)
        .map(|pair| {
            let mut intervals = ParallelIntervals::new();
            for &node_id in &pair[0].nodes {
                let node: MultiwayNodeId = node_id;
                if graph.get_forward_edges(&node).is_some() {
                    intervals.add_branch(DiscreteInterval::new(pair[0].step, pair[1].step));
                }
            }
            intervals
        })
        .collect()
}
