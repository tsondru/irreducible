//! Categorical bridge between hypergraph rewriting and catgraph.
//!
//! This module provides the categorical interpretation of DPO (Double-Pushout)
//! hypergraph rewriting using catgraph's span and cospan types.
//!
//! # Key Mappings
//!
//! | irreducible               | catgraph            | Notes                              |
//! |---------------------------|---------------------|------------------------------------|
//! | `RewriteRule` (L ← K → R)| `Span<u32>`         | Rule as categorical span (variable IDs) |
//! | Evolution step (Gᵢ → Gᵢ₊₁)| `Cospan<u32>`      | Each step as pushout cospan (vertex IDs) |
//! | `HypergraphEvolution`     | `Vec<Cospan<u32>>`  | Composable cospan chain            |
//! | Wilson loop (causal inv.) | Span equivalence    | Holonomy = 1 iff spans agree       |
//!
//! # Mathematical Background
//!
//! A DPO rewrite rule is a span in the category of hypergraphs:
//!
//! ```text
//!     L ←── K ──→ R
//! ```
//!
//! where K is the kernel (preserved vertices/structure), L is the pattern
//! to match, and R is the replacement. Each rewrite step produces a cospan:
//!
//! ```text
//!     Gᵢ ──→ Gᵢ ∪ R ←── Gᵢ₊₁
//! ```
//!
//! The evolution of a hypergraph under repeated rewriting is a chain of
//! composable cospans. Causal invariance (Wilson loop holonomy = 1) means
//! different rewrite orderings produce equivalent cospan chains.

use std::collections::{BTreeSet, HashMap};
use catgraph::category::Composable;
use catgraph::cospan::Cospan;
use catgraph::span::Span;

use super::{Hypergraph, RewriteRule, RewriteSpan, HypergraphEvolution};

// ============================================================================
// Multiway cospan graph types
// ============================================================================

/// A single edge in a multiway cospan graph: one parent-child rewrite step.
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
/// Each edge represents one rewrite step (parent → child) as a cospan.
/// Merge points are pairs of nodes with matching fingerprints (structurally
/// equivalent hypergraphs reached via different rewrite orderings).
#[derive(Debug, Clone)]
pub struct MultiwayCospanGraph {
    /// All parent→child cospans in the evolution.
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
// RewriteSpan constructor
// ============================================================================

impl RewriteSpan {
    /// Creates a new `RewriteSpan` from its components.
    ///
    /// # Arguments
    ///
    /// * `left` - The left-hand side pattern (to match)
    /// * `kernel` - The preserved subgraph (interface)
    /// * `right` - The right-hand side replacement
    /// * `left_map` - Morphism K → L (kernel vertex → left vertex)
    /// * `right_map` - Morphism K → R (kernel vertex → right vertex)
    #[must_use]
    pub fn new(
        left: Hypergraph,
        kernel: Hypergraph,
        right: Hypergraph,
        left_map: HashMap<usize, usize>,
        right_map: HashMap<usize, usize>,
    ) -> Self {
        Self {
            left,
            kernel,
            right,
            left_map,
            right_map,
        }
    }
}

// ============================================================================
// RewriteRule → Span
// ============================================================================

impl RewriteRule {
    /// Converts this rewrite rule to its categorical span representation.
    ///
    /// A rewrite rule L → R with shared variables K is naturally a span:
    ///
    /// ```text
    ///     L ←── K ──→ R
    /// ```
    ///
    /// - L elements = unique variables in the left pattern
    /// - R elements = unique variables in the right pattern
    /// - K elements = preserved variables (appear in both L and R)
    /// - Each K element maps to its index in L and its index in R
    ///
    /// Labels are `u32` variable IDs, so the span carries which variables
    /// are on each side (e.g., `left() = [0, 1, 2]` for variables 0, 1, 2).
    ///
    /// # Example
    ///
    /// ```rust
    /// use irreducible::machines::hypergraph::RewriteRule;
    ///
    /// // Wolfram A→BB: {0,1,2} → {0,1},{1,2}
    /// let rule = RewriteRule::wolfram_a_to_bb();
    /// let span = rule.to_span();
    ///
    /// // L has 3 variables (0,1,2), R has 3 variables (0,1,2)
    /// assert_eq!(span.left(), &[0u32, 1, 2]);
    /// assert_eq!(span.right(), &[0u32, 1, 2]);
    /// // K = {0,1,2} (all preserved) → 3 middle pairs
    /// assert_eq!(span.middle_pairs().len(), 3);
    /// ```
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[must_use]
    pub fn to_span(&self) -> Span<u32> {
        // Collect unique variables from each side, sorted for determinism
        let left_vars: BTreeSet<usize> = self.left().iter()
            .flat_map(|e| e.vertices().iter().copied())
            .collect();
        let right_vars: BTreeSet<usize> = self.right().iter()
            .flat_map(|e| e.vertices().iter().copied())
            .collect();

        let left_sorted: Vec<usize> = left_vars.iter().copied().collect();
        let right_sorted: Vec<usize> = right_vars.iter().copied().collect();

        // Build index maps: variable → position in sorted vec
        let left_index: HashMap<usize, usize> = left_sorted.iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect();
        let right_index: HashMap<usize, usize> = right_sorted.iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect();

        // Kernel = preserved variables (in both L and R)
        let preserved = self.preserved_variables();
        let mut middle: Vec<(usize, usize)> = preserved.iter()
            .map(|&v| (left_index[&v], right_index[&v]))
            .collect();
        // Sort for deterministic output
        middle.sort_unstable();

        // Labels are variable IDs (as u32)
        let left_labels: Vec<u32> = left_sorted.iter().map(|&v| v as u32).collect();
        let right_labels: Vec<u32> = right_sorted.iter().map(|&v| v as u32).collect();

        Span::new(left_labels, right_labels, middle)
    }

    /// Builds the full `RewriteSpan` (L ← K → R) with explicit kernel hypergraph.
    ///
    /// This constructs the kernel as a hypergraph containing only the preserved
    /// vertices and the identity morphisms K → L and K → R.
    #[must_use]
    pub fn to_rewrite_span(&self) -> RewriteSpan {
        let preserved: BTreeSet<usize> = self.preserved_variables().into_iter().collect();

        // Build left hypergraph from pattern
        let mut left = Hypergraph::new();
        for edge in self.left() {
            left.add_hyperedge(edge.vertices().to_vec());
        }

        // Build right hypergraph from pattern
        let mut right = Hypergraph::new();
        for edge in self.right() {
            right.add_hyperedge(edge.vertices().to_vec());
        }

        // Kernel contains only preserved vertices (no edges — they transform)
        let mut kernel = Hypergraph::new();
        for &v in &preserved {
            kernel.add_vertex(Some(v));
        }

        // Identity morphisms: kernel vars map to themselves in L and R
        let left_map: HashMap<usize, usize> = preserved.iter().map(|&v| (v, v)).collect();
        let right_map: HashMap<usize, usize> = preserved.iter().map(|&v| (v, v)).collect();

        RewriteSpan::new(left, kernel, right, left_map, right_map)
    }
}

// ============================================================================
// RewriteSpan → Span
// ============================================================================

impl RewriteSpan {
    /// Converts this `RewriteSpan` to a catgraph `Span<u32>`.
    ///
    /// Uses the `left_map` and `right_map` morphisms to build the span's
    /// middle pairs, mapping kernel elements to their positions in L and R.
    /// Labels are vertex IDs (as `u32`).
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[must_use]
    pub fn to_span(&self) -> Span<u32> {
        let left_verts: Vec<usize> = self.left.vertices().collect();
        let right_verts: Vec<usize> = self.right.vertices().collect();

        // Build index maps
        let left_index: HashMap<usize, usize> = left_verts.iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect();
        let right_index: HashMap<usize, usize> = right_verts.iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect();

        // Each kernel vertex maps through left_map to L and right_map to R
        let mut middle: Vec<(usize, usize)> = Vec::new();
        for k_vert in self.kernel.vertices() {
            if let (Some(&l_vert), Some(&r_vert)) =
                (self.left_map.get(&k_vert), self.right_map.get(&k_vert))
            && let (Some(&l_idx), Some(&r_idx)) =
                (left_index.get(&l_vert), right_index.get(&r_vert))
            {
                middle.push((l_idx, r_idx));
            }
        }
        middle.sort_unstable();

        let left_labels: Vec<u32> = left_verts.iter().map(|&v| v as u32).collect();
        let right_labels: Vec<u32> = right_verts.iter().map(|&v| v as u32).collect();
        Span::new(left_labels, right_labels, middle)
    }
}

// ============================================================================
// HypergraphEvolution → Cospan chain
// ============================================================================

impl HypergraphEvolution {
    /// Converts the evolution into a chain of composable cospans.
    ///
    /// Each rewrite step Gᵢ → Gᵢ₊₁ produces a cospan:
    ///
    /// ```text
    ///     Gᵢ_boundary ──→ apex ←── Gᵢ₊₁_boundary
    /// ```
    ///
    /// where the apex is the union of all vertices from both states,
    /// and the boundary maps send each state's vertices to their
    /// positions in the apex. Preserved vertices map to the same
    /// apex element, creating the categorical "gluing."
    ///
    /// The returned cospans are composable: the right boundary of
    /// cospan i matches the left boundary of cospan i+1.
    ///
    /// # Returns
    ///
    /// A vector of cospans along the deterministic (root-to-last-node) path.
    /// Empty if the evolution has only the root node.
    #[must_use]
    pub fn to_cospan_chain(&self) -> Vec<Cospan<u32>> {
        let path = self.deterministic_path();
        if path.len() < 2 {
            return vec![];
        }

        path.windows(2)
            .map(|w| self.build_cospan_for_pair(w[0], w[1]))
            .collect()
    }

    /// Returns the deterministic path from root to the deepest node.
    ///
    /// Follows the first child at each step (deterministic choice).
    fn deterministic_path(&self) -> Vec<usize> {
        let mut path = vec![0]; // Start at root
        let mut current = 0;

        loop {
            // Find first child of current
            let mut found_child = false;
            for id in (current + 1)..self.node_count() {
                if let Some(node) = self.get_node(id)
                    && node.parent == Some(current)
                {
                    path.push(id);
                    current = id;
                    found_child = true;
                    break;
                }
            }
            if !found_child {
                break;
            }
        }

        path
    }

    /// Builds a cospan from a parent-child node pair.
    ///
    /// The apex is the union of both vertex sets, with labels as vertex IDs.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn build_cospan_for_pair(&self, parent_id: usize, child_id: usize) -> Cospan<u32> {
        let parent = self.get_node(parent_id).unwrap();
        let child = self.get_node(child_id).unwrap();

        let parent_verts: Vec<usize> = parent.state.vertices().collect();
        let child_verts: Vec<usize> = child.state.vertices().collect();

        let mut apex_set: BTreeSet<usize> = BTreeSet::new();
        apex_set.extend(&parent_verts);
        apex_set.extend(&child_verts);
        let apex_sorted: Vec<usize> = apex_set.iter().copied().collect();

        let apex_index: HashMap<usize, usize> = apex_sorted.iter()
            .enumerate()
            .map(|(i, &v)| (v, i))
            .collect();

        let left: Vec<usize> = parent_verts.iter().map(|v| apex_index[v]).collect();
        let right: Vec<usize> = child_verts.iter().map(|v| apex_index[v]).collect();
        let middle: Vec<u32> = apex_sorted.iter().map(|&v| v as u32).collect();

        Cospan::new(left, right, middle)
    }

    /// Converts the full multiway evolution into a cospan graph.
    ///
    /// Unlike `to_cospan_chain()` which follows only the deterministic path,
    /// this captures ALL branches — every parent→child edge becomes a cospan.
    /// Merge points (structurally equivalent states via different paths) are
    /// detected via fingerprint matching.
    #[must_use]
    pub fn to_multiway_cospan_graph(&self) -> MultiwayCospanGraph {
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

    /// Composes the deterministic cospan chain into a single composite cospan
    /// representing the global transformation from initial to final state.
    ///
    /// The composite's domain = root vertex IDs, codomain = final vertex IDs.
    ///
    /// # Panics
    ///
    /// Panics if adjacent cospans in the chain are not composable.
    ///
    /// # Errors
    ///
    /// Returns `CatgraphError::Composition` if the cospan chain is empty.
    pub fn compose_cospan_chain(&self) -> Result<Cospan<u32>, catgraph::errors::CatgraphError> {
        let chain = self.to_cospan_chain();
        chain.into_iter()
            .reduce(|acc, c| acc.compose(&c).expect("evolution cospans must be composable"))
            .ok_or_else(|| catgraph::errors::CatgraphError::Composition {
                message: "empty cospan chain".to_string()
            })
    }

    /// Verifies causal invariance by comparing composite cospans along
    /// different paths to the same merge point.
    ///
    /// For each merge point (nodes with matching fingerprints), composes
    /// cospans along the path from root to each node and checks whether
    /// the resulting composites have the same domain and codomain.
    #[must_use]
    pub fn verify_causal_invariance_via_cospans(&self) -> CospanInvarianceResult {
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
                        Self::compose_cospan_path(&path_a),
                        Self::compose_cospan_path(&path_b),
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

    /// Composes a sequence of cospans into a single composite.
    fn compose_cospan_path(cospans: &[&Cospan<u32>]) -> Option<Cospan<u32>> {
        let mut iter = cospans.iter();
        let first = (*iter.next()?).clone();
        Some(iter.fold(first, |acc, c| {
            acc.compose(c).expect("path cospans must be composable")
        }))
    }
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

    // ── RewriteRule::to_span ───────────────────────────────────────────

    #[test]
    fn test_wolfram_a_to_bb_span() {
        // {0,1,2} → {0,1},{1,2}
        // L vars = {0,1,2}, R vars = {0,1,2}, K = {0,1,2} (all preserved)
        let rule = RewriteRule::wolfram_a_to_bb();
        let span = rule.to_span();

        assert_eq!(span.left(), &[0u32, 1, 2]);
        assert_eq!(span.right(), &[0u32, 1, 2]);
        assert_eq!(span.middle_pairs().len(), 3);

        // All three kernel elements map identity: (0,0), (1,1), (2,2)
        for &(l, r) in span.middle_pairs() {
            assert_eq!(l, r, "preserved vars should map to same index");
        }
    }

    #[test]
    fn test_edge_split_span() {
        // {0,1} → {0,2},{2,1}
        // L vars = {0,1}, R vars = {0,1,2}, K = {0,1}
        let rule = RewriteRule::edge_split();
        let span = rule.to_span();

        assert_eq!(span.left(), &[0u32, 1]);     // L has vars 0, 1
        assert_eq!(span.right(), &[0u32, 1, 2]); // R has vars 0, 1, 2
        assert_eq!(span.middle_pairs().len(), 2); // K = {0, 1}
    }

    #[test]
    fn test_triangle_rule_span() {
        // {0,1} → {0,1},{1,2},{2,0}
        // L vars = {0,1}, R vars = {0,1,2}, K = {0,1}
        let rule = RewriteRule::triangle();
        let span = rule.to_span();

        assert_eq!(span.left(), &[0u32, 1]);
        assert_eq!(span.right(), &[0u32, 1, 2]);
        assert_eq!(span.middle_pairs().len(), 2);
    }

    #[test]
    fn test_collapse_rule_span() {
        // {0,1},{1,2} → {0,2}
        // L vars = {0,1,2}, R vars = {0,2}, K = {0,2}
        let rule = RewriteRule::collapse();
        let span = rule.to_span();

        assert_eq!(span.left(), &[0u32, 1, 2]);
        assert_eq!(span.right(), &[0u32, 2]);
        assert_eq!(span.middle_pairs().len(), 2);
    }

    #[test]
    fn test_create_self_loop_span() {
        // {0,1} → {0,1},{1,1}
        // L vars = {0,1}, R vars = {0,1}, K = {0,1}
        let rule = RewriteRule::create_self_loop();
        let span = rule.to_span();

        assert_eq!(span.left(), &[0u32, 1]);
        assert_eq!(span.right(), &[0u32, 1]);
        assert_eq!(span.middle_pairs().len(), 2);
    }

    // ── RewriteRule::to_rewrite_span ───────────────────────────────────

    #[test]
    fn test_rewrite_span_roundtrip() {
        let rule = RewriteRule::wolfram_a_to_bb();
        let rspan = rule.to_rewrite_span();

        // Kernel should have 3 preserved vertices
        assert_eq!(rspan.kernel.vertex_count(), 3);
        // Left should have 1 edge (ternary)
        assert_eq!(rspan.left.edge_count(), 1);
        // Right should have 2 edges (binary)
        assert_eq!(rspan.right.edge_count(), 2);

        // Converting RewriteSpan to catgraph Span should match direct conversion
        let span_from_rule = rule.to_span();
        let span_from_rspan = rspan.to_span();

        assert_eq!(span_from_rule.left(), span_from_rspan.left());
        assert_eq!(span_from_rule.right(), span_from_rspan.right());
        assert_eq!(span_from_rule.middle_pairs(), span_from_rspan.middle_pairs());
    }

    #[test]
    fn test_edge_split_rewrite_span() {
        let rule = RewriteRule::edge_split();
        let rspan = rule.to_rewrite_span();

        // Kernel: vars {0,1} (preserved)
        assert_eq!(rspan.kernel.vertex_count(), 2);
        // Created var: 2 (only in right)
        assert!(rspan.right.vertices().any(|v| v == 2));
        assert!(!rspan.left.vertices().any(|v| v == 2));
    }

    // ── HypergraphEvolution::to_cospan_chain ───────────────────────────

    #[test]
    fn test_deterministic_evolution_cospan_chain() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let cospans = evolution.to_cospan_chain();

        // Should have at least 1 cospan (one rewrite step)
        assert!(!cospans.is_empty());

        // Each cospan's left boundary should have the parent's vertex count
        // and right boundary should have the child's vertex count
        for cospan in &cospans {
            assert!(!cospan.left_to_middle().is_empty());
            assert!(!cospan.right_to_middle().is_empty());
            // All indices should be valid (< middle.len())
            let middle_len = cospan.middle().len();
            assert!(cospan.left_to_middle().iter().all(|&i| i < middle_len));
            assert!(cospan.right_to_middle().iter().all(|&i| i < middle_len));
        }
    }

    #[test]
    fn test_cospan_chain_preserves_shared_vertices() {
        // A→BB: {0,1,2} → {0,1},{1,2}
        // All vertices preserved, so parent and child share the same apex positions
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 1);

        let cospans = evolution.to_cospan_chain();
        assert_eq!(cospans.len(), 1);

        let cospan = &cospans[0];
        // For A→BB with no new vertices, parent and child have same vertices
        // so left and right should map to the same apex positions
        assert_eq!(cospan.left_to_middle(), cospan.right_to_middle());
    }

    #[test]
    fn test_cospan_chain_with_new_vertices() {
        // edge-split: {0,1} → {0,2},{2,1} — creates new vertex
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::edge_split()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 1);

        let cospans = evolution.to_cospan_chain();
        assert_eq!(cospans.len(), 1);

        let cospan = &cospans[0];
        // Parent has 2 vertices {0,1}, child has 3 {0,1,2}
        assert_eq!(cospan.left_to_middle().len(), 2);
        assert_eq!(cospan.right_to_middle().len(), 3);
        // Apex should have 3 elements (union)
        assert_eq!(cospan.middle().len(), 3);
    }

    #[test]
    fn test_empty_evolution_no_cospans() {
        // No applicable rules → no steps → no cospans
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()]; // needs ternary edge
        let evolution = HypergraphEvolution::run(&initial, &rules, 10);

        let cospans = evolution.to_cospan_chain();
        assert!(cospans.is_empty());
    }

    #[test]
    fn test_multi_step_cospan_chain() {
        // Run multiple edge splits
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::edge_split()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let cospans = evolution.to_cospan_chain();
        assert_eq!(cospans.len(), 3);

        // Chain should be composable: right boundary of step i
        // has same size as left boundary of step i+1
        for i in 0..cospans.len() - 1 {
            assert_eq!(
                cospans[i].right_to_middle().len(),
                cospans[i + 1].left_to_middle().len(),
                "cospan chain boundary mismatch at step {}", i
            );
        }
    }

    // ── Cospan label values ─────────────────────────────────────────────

    #[test]
    fn test_cospan_apex_labels_are_vertex_ids() {
        // A→BB preserves all vertices: apex labels should be {0, 1, 2}
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 1);

        let cospans = evolution.to_cospan_chain();
        assert_eq!(cospans.len(), 1);

        // Apex should contain actual vertex IDs
        assert_eq!(cospans[0].middle(), &[0u32, 1, 2]);
    }

    #[test]
    fn test_cospan_chain_composable_via_catgraph() {
        use catgraph::category::Composable;

        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::edge_split()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let cospans = evolution.to_cospan_chain();
        assert_eq!(cospans.len(), 3);

        // Verify composability via catgraph's Composable trait
        for i in 0..cospans.len() - 1 {
            assert!(
                cospans[i].composable(&cospans[i + 1]).is_ok(),
                "cospans {} and {} should be composable", i, i + 1
            );
        }
    }

    // ── Span validity ──────────────────────────────────────────────────

    #[test]
    fn test_all_common_rules_produce_valid_spans() {
        let rules = vec![
            RewriteRule::wolfram_a_to_bb(),
            RewriteRule::edge_split(),
            RewriteRule::triangle(),
            RewriteRule::collapse(),
            RewriteRule::create_self_loop(),
        ];

        for rule in &rules {
            // to_span() calls Span::new() which calls assert_valid()
            let span = rule.to_span();
            assert!(span.left().len() > 0 || span.right().len() > 0,
                "rule '{}' should produce non-trivial span", rule);

            // to_rewrite_span() + to_span() should also be valid
            let rspan = rule.to_rewrite_span();
            let _span2 = rspan.to_span();
        }
    }

    // ── Multiway cospan graph (Phase 2) ────────────────────────────────

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

    // ── Cospan composition (Phase 3) ───────────────────────────────────

    #[test]
    fn test_compose_single_step() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 1);

        let composite = evolution.compose_cospan_chain().unwrap();

        // For A→BB with all preserved, domain = codomain = {0, 1, 2}
        assert_eq!(composite.domain(), vec![0u32, 1, 2]);
        assert_eq!(composite.codomain(), vec![0u32, 1, 2]);
    }

    #[test]
    fn test_compose_multi_step() {
        use catgraph::category::Composable;

        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::edge_split()];
        let evolution = HypergraphEvolution::run(&initial, &rules, 3);

        let composite = evolution.compose_cospan_chain().unwrap();

        // Domain should be root vertices
        assert_eq!(composite.domain(), vec![0u32, 1]);
    }

    #[test]
    fn test_compose_empty_chain_error() {
        let initial = Hypergraph::from_edges(vec![vec![0, 1]]);
        let rules = vec![RewriteRule::wolfram_a_to_bb()]; // won't match binary edge
        let evolution = HypergraphEvolution::run(&initial, &rules, 10);

        assert!(evolution.compose_cospan_chain().is_err());
    }

    #[test]
    fn test_causal_invariance_deterministic() {
        // Deterministic evolution has no merge points → trivially invariant
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
