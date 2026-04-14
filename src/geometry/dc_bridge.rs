//! Bridge from `catgraph_physics::multiway::BranchialGraph` to
//! `deep_causality_topology::SimplicialComplex`.
//!
//! # Phase 0 scope
//!
//! This bridge is intentionally **edge-only** in Phase 0: it emits one
//! 0-simplex per branchial node and one 1-simplex per branchial edge, but no
//! 2-simplices (triangular faces). Face-extraction via `confluence_diamonds()`
//! is deferred to Phase 2, where it lands alongside the DEC port in
//! `multiway_stokes`.
//!
//! A branchial graph at step `t` captures the parallel-branches slice —
//! vertices that share a common ancestor — but it does *not* natively surface
//! the 2-cell confluence structure of the multiway evolution. To build a full
//! 2D complex suitable for Regge curvature, Phase 2 will either (a) accept a
//! `&MultiwayEvolutionGraph<S, T>` and walk it for diamonds, or (b) extend
//! `BranchialGraph` upstream with a faces iterator. That design decision is
//! deferred until the DEC port forces it.

use catgraph_physics::multiway::BranchialGraph;
use deep_causality_topology::{
    Simplex, SimplicialComplex, SimplicialComplexBuilder, TopologyError,
};
use std::collections::HashMap;

/// Convert a [`BranchialGraph`] into a 2D [`SimplicialComplex<f64>`] containing
/// only 0- and 1-simplices.
///
/// The mapping is:
///
/// - Each `branchial.nodes[i]` becomes the 0-simplex with vertex index `i`.
/// - Each `(a, b)` in `branchial.edges` becomes the 1-simplex `{idx(a), idx(b)}`,
///   where `idx(n)` is the index of node `n` in `branchial.nodes`.
///
/// The returned complex has `max_dim = 2` so that Phase 2 can add 2-simplices
/// to the same builder pattern without reconfiguration. No 2-simplices are
/// added here; the 2-skeleton is empty.
///
/// # Errors
///
/// - [`TopologyError::InvalidInput`] if an edge references a node that is not
///   present in `branchial.nodes`.
/// - Propagates [`TopologyError`] from the underlying
///   [`SimplicialComplexBuilder::build`].
#[must_use = "the returned Result carries the constructed complex or a topology error"]
pub fn branchial_to_simplicial(
    branchial: &BranchialGraph,
) -> Result<SimplicialComplex<f64>, TopologyError> {
    let mut builder = SimplicialComplexBuilder::new(2);

    // Index each branchial node by its position in `branchial.nodes`.
    let mut node_to_idx: HashMap<_, usize> = HashMap::with_capacity(branchial.nodes.len());
    for (i, node) in branchial.nodes.iter().enumerate() {
        node_to_idx.insert(*node, i);
        builder.add_simplex(Simplex::new(vec![i]))?;
    }

    // Add each branchial edge as a 1-simplex.
    for (a, b) in &branchial.edges {
        let ia = node_to_idx.get(a).ok_or_else(|| {
            TopologyError::InvalidInput(format!(
                "branchial edge source {a:?} not found in node list"
            ))
        })?;
        let ib = node_to_idx.get(b).ok_or_else(|| {
            TopologyError::InvalidInput(format!(
                "branchial edge target {b:?} not found in node list"
            ))
        })?;
        if ia == ib {
            // Skip self-loops: Simplex::new([v, v]) collapses to [v] which is
            // a 0-simplex, not a 1-simplex, and silently corrupts the complex.
            continue;
        }
        builder.add_simplex(Simplex::new(vec![*ia, *ib]))?;
    }

    builder.build::<f64>()
}
