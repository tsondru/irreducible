//! Multiway cospan analysis for hypergraph evolution.
//!
//! Core types and extension trait re-exported from
//! [`catgraph::hypergraph::multiway_cospan`], plus the local
//! [`MergesCorelExt`] extension exposing the evolution's merge structure as
//! a [`Corel`] (F&S 2018 Ex 6.64; issue #14).

use std::collections::HashMap;

use catgraph::corel::Corel;
use catgraph::cospan::Cospan;
use catgraph::errors::CatgraphError;

pub use catgraph_physics::hypergraph::multiway_cospan::{
    CospanInvarianceResult, CospanMergeDetail, MultiwayCospan, MultiwayCospanExt,
    MultiwayCospanGraph,
};

/// Extension trait exposing an evolution's merge structure as a corelation
/// over hypergraph vertex IDs.
///
/// Rewrite cospans keep vertex IDs global (legs are injective by ID), so
/// all identification comes from **merge points**: states reached by
/// different rewrite orderings whose fingerprints match. Vertices of merged
/// states are aligned by sorted vertex-ID order — canonical for the
/// fingerprint-equal states the upstream merge detector produces, but an
/// approximation wherever a true isomorphism permutes IDs non-monotonically
/// (recorded here rather than solving graph isomorphism).
pub trait MergesCorelExt {
    /// The merge partition over every vertex ID in the evolution, as a
    /// corelation `x → x` where `x` is the sorted list of distinct vertex
    /// IDs and two IDs share a class iff some chain of merge points
    /// identifies them.
    ///
    /// # Errors
    ///
    /// Returns [`CatgraphError`] if the partition cospan fails corelation
    /// validation (not expected — the construction is jointly surjective).
    fn merges_corel(&self) -> Result<Corel<u32>, CatgraphError>;

    /// The merge partition restricted to a caller-chosen boundary (e.g. the
    /// initial graph's vertex IDs), in the given order.
    ///
    /// Two evolutions of the same initial graph under different rule
    /// orderings restricted to the same boundary share domain and codomain,
    /// so their partitions compare via [`Corel::refines`] /
    /// [`Corel::coarsest_common_refinement`] — a candidate confluence test
    /// stronger than the Wilson-loop check (which only compares composite
    /// boundaries).
    ///
    /// # Errors
    ///
    /// As [`Self::merges_corel`].
    fn merges_corel_over(&self, boundary: &[u32]) -> Result<Corel<u32>, CatgraphError>;
}

impl MergesCorelExt for MultiwayCospanGraph {
    fn merges_corel(&self) -> Result<Corel<u32>, CatgraphError> {
        let mut all: Vec<u32> = self
            .edges
            .iter()
            .flat_map(|e| e.cospan.middle().iter().copied())
            .collect();
        all.sort_unstable();
        all.dedup();
        self.merges_corel_over(&all)
    }

    fn merges_corel_over(&self, boundary: &[u32]) -> Result<Corel<u32>, CatgraphError> {
        let classes = merge_classes(self);

        // Class representative per boundary vertex (vertices not touched by
        // any merge are their own class).
        let mut apex_of_rep: HashMap<u32, usize> = HashMap::new();
        let mut left = Vec::with_capacity(boundary.len());
        let mut middle: Vec<u32> = Vec::new();
        for &v in boundary {
            let rep = classes.get(&v).copied().unwrap_or(v);
            let next = middle.len();
            let apex = *apex_of_rep.entry(rep).or_insert(next);
            if apex == middle.len() {
                middle.push(rep);
            }
            left.push(apex);
        }

        Corel::new(Cospan::new(left.clone(), left, middle))
    }
}

/// Union-find over vertex IDs driven by merge-point alignment; returns the
/// class representative (minimum vertex ID) for every vertex touched by a
/// merge.
fn merge_classes(graph: &MultiwayCospanGraph) -> HashMap<u32, u32> {
    // A node's vertex set, from any incident cospan leg.
    let mut vertices_of_node: HashMap<usize, Vec<u32>> = HashMap::new();
    for edge in &graph.edges {
        vertices_of_node.entry(edge.parent_id).or_insert_with(|| {
            edge.cospan
                .left_to_middle()
                .iter()
                .map(|&m| edge.cospan.middle()[m])
                .collect()
        });
        vertices_of_node.entry(edge.child_id).or_insert_with(|| {
            edge.cospan
                .right_to_middle()
                .iter()
                .map(|&m| edge.cospan.middle()[m])
                .collect()
        });
    }

    fn find(parent: &mut HashMap<u32, u32>, v: u32) -> u32 {
        let mut root = v;
        while let Some(&p) = parent.get(&root) {
            if p == root {
                break;
            }
            root = p;
        }
        // Path compression.
        let mut cur = v;
        while let Some(&p) = parent.get(&cur) {
            if p == root {
                break;
            }
            parent.insert(cur, root);
            cur = p;
        }
        root
    }
    fn union(parent: &mut HashMap<u32, u32>, a: u32, b: u32) {
        let (ra, rb) = (find(parent, a), find(parent, b));
        if ra != rb {
            // Keep the minimum ID as representative.
            let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
            parent.insert(hi, lo);
        }
    }

    let mut parent: HashMap<u32, u32> = HashMap::new();
    for group in &graph.merge_points {
        let Some((first, rest)) = group.split_first() else {
            continue;
        };
        let Some(base) = vertices_of_node.get(first) else {
            continue;
        };
        let mut base_sorted = base.clone();
        base_sorted.sort_unstable();
        for other in rest {
            let Some(vs) = vertices_of_node.get(other) else {
                continue;
            };
            if vs.len() != base_sorted.len() {
                continue; // cannot align differently-sized states
            }
            let mut vs_sorted = vs.clone();
            vs_sorted.sort_unstable();
            for (&a, &b) in base_sorted.iter().zip(vs_sorted.iter()) {
                union(&mut parent, a, b);
            }
        }
    }

    // Resolve every touched vertex to its representative.
    let touched: Vec<u32> = parent.keys().copied().collect();
    let mut out = HashMap::new();
    for v in touched {
        let rep = find(&mut parent, v);
        out.insert(v, rep);
        out.insert(rep, rep);
    }
    out
}
