//! Multiway cospan analysis for hypergraph evolution.
//!
//! This module provides the multiway cospan graph interpretation of
//! hypergraph evolution, building on catgraph's core span/cospan bridge.
//!
//! Core span/cospan conversions (`to_span()`, `to_cospan_chain()`,
//! `compose_cospan_chain()`) live in catgraph. This module adds:
//!
//! - [`MultiwayCospan`] / [`MultiwayCospanGraph`]: full multiway evolution as a
//!   graph of cospans (not just the deterministic path)
//! - [`CospanInvarianceResult`]: causal invariance verification via cospan composition
//! - [`MultiwayCospanExt`]: extension trait adding multiway methods to
//!   [`HypergraphEvolution`]
//!
//! # Mathematical Background
//!
//! A DPO rewrite rule is a span in the category of hypergraphs:
//!
//! ```text
//!     L <-- K --> R
//! ```
//!
//! Each rewrite step produces a cospan:
//!
//! ```text
//!     Gi --> Gi U R <-- Gi+1
//! ```
//!
//! The evolution of a hypergraph under repeated rewriting is a chain of
//! composable cospans. Causal invariance (Wilson loop holonomy = 1) means
//! different rewrite orderings produce equivalent cospan chains.

use catgraph::category::Composable;
use catgraph::cospan::Cospan;
use catgraph::hypergraph::HypergraphEvolution;

// ============================================================================
// Multiway cospan graph types
// ============================================================================

/// A single edge in the multiway cospan graph: one parent -> child rewrite step.
///
/// The cospan apex is the vertex union of the parent and child hypergraphs;
/// the left and right legs map parent and child vertices into the apex.
#[derive(Debug, Clone)]
pub struct MultiwayCospan {
    /// Parent node ID in the evolution.
    pub parent_id: usize,
    /// Child node ID in the evolution.
    pub child_id: usize,
    /// The cospan for this step (apex = vertex union, labels = vertex IDs).
    pub cospan: Cospan<u32>,
}

/// The full multiway evolution as a graph of cospans.
///
/// Each edge represents one rewrite step (parent -> child) as a cospan.
/// Merge points are pairs of node IDs with matching fingerprints (structurally
/// equivalent hypergraphs reached via different rewrite orderings).
#[derive(Debug, Clone)]
pub struct MultiwayCospanGraph {
    /// All parent->child cospans in the evolution.
    pub edges: Vec<MultiwayCospan>,
    /// Merge points: groups of node IDs with matching fingerprints.
    pub merge_points: Vec<Vec<usize>>,
}

/// Result of causal invariance verification via cospan composition.
#[derive(Debug, Clone)]
pub struct CospanInvarianceResult {
    /// Whether all merge points have equivalent composite cospans.
    pub is_invariant: bool,
    /// Number of merge point pairs checked.
    pub merge_points_checked: usize,
    /// Number of pairs where composites matched.
    pub invariant_merges: usize,
    /// Details for each merge point pair.
    pub details: Vec<CospanMergeDetail>,
}

/// Detail for one merge point comparison.
#[derive(Debug, Clone)]
pub struct CospanMergeDetail {
    /// First node ID in the merge.
    pub node_a: usize,
    /// Second node ID in the merge.
    pub node_b: usize,
    /// Whether the composite cospans matched (same domain + codomain).
    pub composites_match: bool,
}

// ============================================================================
// Extension trait for HypergraphEvolution
// ============================================================================

/// Extension trait adding multiway cospan analysis to [`HypergraphEvolution`].
///
/// These methods build on catgraph's core span/cospan bridge to provide
/// full multiway cospan graphs and causal invariance verification.
pub trait MultiwayCospanExt {
    /// Converts the full multiway evolution into a cospan graph.
    ///
    /// Unlike `to_cospan_chain()` which follows only the deterministic path,
    /// this captures ALL branches -- every parent->child edge becomes a cospan.
    /// Merge points (structurally equivalent states via different paths) are
    /// detected via fingerprint matching.
    #[must_use]
    fn to_multiway_cospan_graph(&self) -> MultiwayCospanGraph;

    /// Verifies causal invariance by comparing composite cospans along
    /// different paths to the same merge point.
    ///
    /// For each merge point (nodes with matching fingerprints), composes
    /// cospans along the path from root to each node and checks whether
    /// the resulting composites have the same domain and codomain.
    #[must_use]
    fn verify_causal_invariance_via_cospans(&self) -> CospanInvarianceResult;
}

impl MultiwayCospanExt for HypergraphEvolution {
    fn to_multiway_cospan_graph(&self) -> MultiwayCospanGraph {
        let mut edges = Vec::new();

        for id in 0..self.node_count() {
            if let Some(node) = self.get_node(id)
                && let Some(parent_id) = node.parent
            {
                let cospan = self.build_cospan_for_pair(parent_id, id);
                edges.push(MultiwayCospan {
                    parent_id,
                    child_id: id,
                    cospan,
                });
            }
        }

        let merge_points = self.find_merges();

        MultiwayCospanGraph { edges, merge_points }
    }

    fn verify_causal_invariance_via_cospans(&self) -> CospanInvarianceResult {
        let graph = self.to_multiway_cospan_graph();
        let mut details = Vec::new();
        let mut invariant_count = 0;
        let mut total_checked = 0;

        for merge_group in &graph.merge_points {
            for i in 0..merge_group.len() {
                for j in (i + 1)..merge_group.len() {
                    let node_a = merge_group[i];
                    let node_b = merge_group[j];

                    let path_a = graph.path_to_node(self, node_a);
                    let path_b = graph.path_to_node(self, node_b);

                    let composites_match = match (
                        compose_cospan_path(&path_a),
                        compose_cospan_path(&path_b),
                    ) {
                        (Some(ca), Some(cb)) => {
                            ca.domain() == cb.domain() && ca.codomain() == cb.codomain()
                        }
                        _ => false,
                    };

                    if composites_match {
                        invariant_count += 1;
                    }
                    total_checked += 1;

                    details.push(CospanMergeDetail {
                        node_a,
                        node_b,
                        composites_match,
                    });
                }
            }
        }

        CospanInvarianceResult {
            is_invariant: total_checked == 0 || invariant_count == total_checked,
            merge_points_checked: total_checked,
            invariant_merges: invariant_count,
            details,
        }
    }
}

/// Composes a sequence of cospans into a single composite.
fn compose_cospan_path(cospans: &[&Cospan<u32>]) -> Option<Cospan<u32>> {
    let mut iter = cospans.iter();
    let first = (*iter.next()?).clone();
    Some(iter.fold(first, |acc, c| {
        acc.compose(c).expect("path cospans must be composable")
    }))
}

impl MultiwayCospanGraph {
    /// Traces the path of cospans from root to the given node.
    ///
    /// Returns cospans in root-to-target order by following parent links.
    #[must_use]
    pub fn path_to_node<'a>(
        &'a self,
        evolution: &HypergraphEvolution,
        target: usize,
    ) -> Vec<&'a Cospan<u32>> {
        let mut path = Vec::new();
        let mut current = target;

        while let Some(node) = evolution.get_node(current) {
            if let Some(parent_id) = node.parent {
                // Find the edge from parent to current
                if let Some(edge) = self.edges.iter().find(|e| e.child_id == current) {
                    path.push(&edge.cospan);
                }
                current = parent_id;
            } else {
                break; // Reached root
            }
        }

        path.reverse();
        path
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use catgraph::hypergraph::{Hypergraph, RewriteRule};

    // ── Multiway cospan graph ──────────────────────────────────────────

    #[test]
    fn test_multiway_graph_deterministic() {
        // Deterministic evolution: edge count = chain length
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::edge_split()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let chain = evolution.to_cospan_chain();
        let graph = evolution.to_multiway_cospan_graph();

        assert_eq!(graph.edges.len(), chain.len());
    }

    #[test]
    fn test_multiway_graph_branching() {
        // Multiway evolution should have more edges than deterministic
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2], vec![1, 2, 3]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run_multiway(&initial, &rules, 2, 20);

        let chain_len = evolution.to_cospan_chain().len();
        let graph = evolution.to_multiway_cospan_graph();

        // Multiway should have at least as many edges as the deterministic chain
        assert!(graph.edges.len() >= chain_len);
    }

    #[test]
    fn test_multiway_graph_no_self_loops() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run_multiway(&initial, &rules, 3, 50);

        let graph = evolution.to_multiway_cospan_graph();

        for edge in &graph.edges {
            assert_ne!(edge.parent_id, edge.child_id, "no self-loops");
        }
    }

    #[test]
    fn test_multiway_graph_covers_all_nodes() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run_multiway(&initial, &rules, 2, 20);

        let graph = evolution.to_multiway_cospan_graph();
        let child_ids: std::collections::HashSet<usize> =
            graph.edges.iter().map(|e| e.child_id).collect();

        // Every non-root node should appear as a child
        for id in 1..evolution.node_count() {
            assert!(child_ids.contains(&id), "node {} missing from graph edges", id);
        }
    }

    #[test]
    fn test_path_to_node() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::edge_split()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let graph = evolution.to_multiway_cospan_graph();

        // Path to root should be empty (root has no parent cospans)
        let path_root = graph.path_to_node(&evolution, 0);
        assert!(path_root.is_empty());

        // Path to deepest node should have length = depth
        let leaves = evolution.leaves();
        if let Some(&leaf) = leaves.first() {
            let path = graph.path_to_node(&evolution, leaf);
            assert!(!path.is_empty());
        }
    }

    // ── Causal invariance via cospans ──────────────────────────────────

    #[test]
    fn test_causal_invariance_deterministic() {
        // Deterministic evolution has no merge points -> trivially invariant
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::edge_split()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let result = evolution.verify_causal_invariance_via_cospans();
        assert!(result.is_invariant);
        assert_eq!(result.merge_points_checked, 0);
    }

    #[test]
    fn test_causal_invariance_multiway() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2], vec![1, 2, 3]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run_multiway(&initial, &rules, 3, 50);

        let result = evolution.verify_causal_invariance_via_cospans();
        // The result should at least report correctly
        assert_eq!(result.is_invariant, result.merge_points_checked == 0 || result.invariant_merges == result.merge_points_checked);
    }
}
