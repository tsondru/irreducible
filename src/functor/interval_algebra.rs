//! Z' as a cospan-algebra: lax monoidal structure on cobordism intervals.
//!
//! Fong-Spivak §2.1 (Def 2.2): a cospan-algebra `(Λ, a)` is a lax symmetric
//! monoidal functor `a: Cospan_Λ → C`. [`IntervalCospanAlgebra`] realizes the
//! cobordism side of Z' as such an algebra:
//!
//! - the carrier `a(x)` for a boundary `x` (a branchial slice with `k`
//!   branches) is a [`ParallelIntervals`] bundle with one branch per
//!   boundary node;
//! - [`map_cospan`](catgraph::cospan_algebra::CospanAlgebra::map_cospan)
//!   transports a bundle across a multiway step cospan: each right-boundary
//!   node receives the hull of the intervals of the left-boundary nodes in
//!   its apex component (Z' of the step event acting on interval bundles);
//! - [`lax_monoidal`](catgraph::cospan_algebra::CospanAlgebra::lax_monoidal)
//!   is [`ParallelIntervals::direct_sum`] — exactly the tensor
//!   `Z'(f) ⊕ Z'(g)` of Gorard's multicomputational condition;
//! - [`unit`](catgraph::cospan_algebra::CospanAlgebra::unit) is the empty
//!   bundle.
//!
//! [`multiway_step_cospans`] encodes a multiway evolution as the chain of
//! per-step cospans this algebra acts on: boundaries are branchial slices,
//! the apex is the set of connected components of the step's parent/child
//! bipartite graph (fork, merge, and sequential events each become one apex
//! vertex), labels are the singleton type `0u32`.
//!
//! ## Evaluation outcome (issue #11)
//!
//! The pre-existing tensor check in
//! [`verify_symmetric_monoidal_functor`](super::IrreducibilityFunctor::verify_symmetric_monoidal_functor)
//! is a branch *survival* (totality) check — "every active branch has a
//! continuation" — which is **not** the lax-monoidal coherence square: for a
//! componentwise transport like `map_cospan`, the square
//! `a(c_x ⊕ c_y)(e_x ⊗ e_y) = a(c_x)(e_x) ⊗ a(c_y)(e_y)` holds by
//! construction. The refactor therefore *delegates the monoidal bookkeeping*
//! (bundle assembly via `lax_monoidal`/`unit`) to this algebra and keeps the
//! survival semantics intact, while `map_cospan` carries the genuinely new
//! content: functorial interval transport across step cospans, with
//! functoriality `a(c₁ ; c₂) = a(c₂) ∘ a(c₁)` testable against pushout
//! composition.

use std::collections::HashMap;
use std::hash::Hash;

use catgraph::cospan::Cospan;
use catgraph::cospan_algebra::CospanAlgebra;
use catgraph::errors::CatgraphError;
use catgraph_physics::multiway::{
    BranchialGraph, MultiwayEvolutionGraph, MultiwayNodeId, extract_branchial_foliation,
};
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use catgraph_physics::interval::{DiscreteInterval, ParallelIntervals};

/// The cobordism side of Z' as a cospan-algebra (F&S Def 2.2).
///
/// See the [module docs](self) for the carrier and the action. All apex
/// labels are the singleton type `0u32` — the algebra is single-typed, and
/// branch identity is carried by the leg structure, not by labels.
#[derive(Default, Clone, Copy, Debug)]
pub struct IntervalCospanAlgebra;

impl CospanAlgebra<u32> for IntervalCospanAlgebra {
    type Elem = ParallelIntervals;

    /// Transport an interval bundle across a step cospan.
    ///
    /// Each right-boundary node receives the hull (min start .. max end) of
    /// the intervals of the left-boundary nodes sharing its apex vertex.
    /// Left branches whose apex component contains no right-boundary node
    /// (dead branches) are dropped — their history leaves the bundle.
    ///
    /// # Errors
    ///
    /// - the element's branch count does not match the cospan's left
    ///   boundary arity;
    /// - a right-boundary node's apex component contains no left-boundary
    ///   node (a spontaneous branch has no interval history to transport;
    ///   multiway step cospans never produce this).
    ///
    /// # Examples
    ///
    /// A merge — both left-boundary nodes share the single apex vertex —
    /// hands the surviving branch the hull of its parents' intervals:
    ///
    /// ```rust
    /// use catgraph::cospan::Cospan;
    /// use irreducible::{CospanAlgebra, DiscreteInterval, IntervalCospanAlgebra, ParallelIntervals};
    ///
    /// let merge = Cospan::new(vec![0, 0], vec![0], vec![0u32]).unwrap();
    ///
    /// let mut parents = ParallelIntervals::new();
    /// parents.add_branch(DiscreteInterval::new(0, 2));
    /// parents.add_branch(DiscreteInterval::new(1, 4));
    ///
    /// let child = IntervalCospanAlgebra.map_cospan(&merge, &parents).unwrap();
    /// assert_eq!(child.branch_count(), 1);
    /// assert_eq!(child.branches[0], DiscreteInterval::new(0, 4));
    /// ```
    fn map_cospan(
        &self,
        cospan: &Cospan<u32>,
        element: &Self::Elem,
    ) -> Result<Self::Elem, CatgraphError> {
        let left = cospan.left_to_middle();
        let right = cospan.right_to_middle();

        if element.branch_count() != left.len() {
            return Err(CatgraphError::Composition {
                message: format!(
                    "element has {} branches but cospan left boundary has {}",
                    element.branch_count(),
                    left.len()
                ),
            });
        }

        // Hull of transported intervals per apex vertex.
        let mut hulls: Vec<Option<(usize, usize)>> = vec![None; cospan.middle().len()];
        for (branch, &apex) in element.branches.iter().zip(left.iter()) {
            let entry = hulls[apex].get_or_insert((branch.start, branch.end));
            entry.0 = entry.0.min(branch.start);
            entry.1 = entry.1.max(branch.end);
        }

        let mut result = ParallelIntervals::new();
        for &apex in right {
            let Some((start, end)) = hulls[apex] else {
                return Err(CatgraphError::Composition {
                    message: format!(
                        "right boundary vertex maps to apex {apex} with no left preimage; \
                         interval transport is undefined for spontaneous branches"
                    ),
                });
            };
            result.add_branch(DiscreteInterval::new(start, end));
        }
        Ok(result)
    }

    fn lax_monoidal(&self, a: &Self::Elem, b: &Self::Elem) -> Self::Elem {
        a.clone().direct_sum(b.clone())
    }

    fn unit(&self) -> Self::Elem {
        ParallelIntervals::new()
    }
}

/// Encode a multiway evolution as its chain of per-step cospans.
///
/// For each step `i`, the cospan's left boundary is the branchial slice at
/// `i`, the right boundary the slice at `i + 1`, and the apex the connected
/// components of the parent/child bipartite graph between them: a sequential
/// event, a fork, and a merge each become a single apex vertex. Dead
/// branches (no forward edges) become apex vertices untouched by the right
/// boundary. Labels are uniformly `0u32` (single-typed).
///
/// Adjacent cospans in the returned chain are composable: the right boundary
/// arity of step `i` equals the left boundary arity of step `i + 1` (both
/// are the branchial slice at `i + 1`, in the same node order).
///
/// # Examples
///
/// The diamond `S → AB | BA`, both `→ Z`, gives a fork at step 0 (one apex
/// vertex, two right-boundary nodes) and two sequential events at step 1:
///
/// ```rust
/// use irreducible::{StringRewriteSystem, multiway_step_cospans};
///
/// let srs = StringRewriteSystem::new(vec![("S", "AB"), ("S", "BA"), ("AB", "Z"), ("BA", "Z")]);
/// let evolution = srs.run_multiway("S", 2, 16);
/// let chain = multiway_step_cospans(&evolution);
///
/// assert_eq!(chain.len(), 2);
/// assert_eq!(chain[0].left_to_middle(), &[0]);
/// assert_eq!(chain[0].right_to_middle(), &[0, 0]);
/// assert_eq!(chain[1].middle().len(), 2);
/// ```
#[must_use]
pub fn multiway_step_cospans<S: Clone + Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
) -> Vec<Cospan<u32>> {
    step_cospans_from_foliation(graph, &extract_branchial_foliation(graph))
}

/// [`multiway_step_cospans`] over a foliation the caller already extracted.
pub(super) fn step_cospans_from_foliation<S: Clone + Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
    foliation: &[BranchialGraph],
) -> Vec<Cospan<u32>> {
    let mut cospans = Vec::with_capacity(foliation.len().saturating_sub(1));

    for window in foliation.windows(2) {
        let (slice_t, slice_next) = (&window[0], &window[1]);
        let n_left = slice_t.nodes.len();
        let n_right = slice_next.nodes.len();

        // Position of each target-slice node, matching the first occurrence.
        let mut pos_in_next: HashMap<MultiwayNodeId, usize> = HashMap::with_capacity(n_right);
        for (ri, node) in slice_next.nodes.iter().enumerate() {
            pos_in_next.entry(*node).or_insert(ri);
        }

        // Union-find over left ∪ right node positions.
        let mut classes = Partition::new(n_left + n_right);
        for (li, node) in slice_t.nodes.iter().enumerate() {
            if let Some(edges) = graph.get_forward_edges(node) {
                for edge in edges {
                    if let Some(&ri) = pos_in_next.get(&edge.to) {
                        classes.union(li, n_left + ri);
                    }
                }
            }
        }

        let mut apex_of_root: HashMap<usize, usize> = HashMap::new();
        let left: Vec<usize> = (0..n_left)
            .map(|i| renumber_first_seen(&mut classes, i, &mut apex_of_root))
            .collect();
        let right: Vec<usize> = (0..n_right)
            .map(|i| renumber_first_seen(&mut classes, n_left + i, &mut apex_of_root))
            .collect();
        let middle = vec![0u32; apex_of_root.len()];

        // Correct by construction: every leg entry is a `renumber_first_seen`
        // output, i.e. an index into `apex_of_root`, and `middle` has one
        // vertex per class.
        cospans.push(Cospan::new_unchecked(left, right, middle));
    }

    cospans
}

/// Union-find over the dense positions the apex constructions partition.
pub(super) type Partition = QuickUnionUf<UnionBySize>;

/// The apex index of `i`'s class, minting indices in order of first request.
///
/// Successive cospans in a chain compose only when both number their shared
/// boundary's classes this way, so both apex constructions
/// ([`step_cospans_from_foliation`] and the corelation quotient) share it.
pub(super) fn renumber_first_seen(
    classes: &mut Partition,
    i: usize,
    apex_of_root: &mut HashMap<usize, usize>,
) -> usize {
    let root = classes.find(i);
    let next = apex_of_root.len();
    *apex_of_root.entry(root).or_insert(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::multiway::StringRewriteSystem;
    use catgraph::category::Composable;

    fn bundle(intervals: &[(usize, usize)]) -> ParallelIntervals {
        let mut p = ParallelIntervals::new();
        for &(s, e) in intervals {
            p.add_branch(DiscreteInterval::new(s, e));
        }
        p
    }

    #[test]
    fn map_cospan_sequential_step_extends_nothing() {
        // 1 → 1 sequential step: single apex vertex.
        let c = Cospan::new(vec![0], vec![0], vec![0u32]).unwrap();
        let alg = IntervalCospanAlgebra;
        let out = alg.map_cospan(&c, &bundle(&[(0, 1)])).unwrap();
        assert_eq!(out.branch_count(), 1);
        assert_eq!(out.branches[0], DiscreteInterval::new(0, 1));
    }

    #[test]
    fn map_cospan_fork_duplicates_history() {
        // 1 → 2 fork: one apex vertex, two right nodes.
        let c = Cospan::new(vec![0], vec![0, 0], vec![0u32]).unwrap();
        let alg = IntervalCospanAlgebra;
        let out = alg.map_cospan(&c, &bundle(&[(0, 3)])).unwrap();
        assert_eq!(out.branch_count(), 2);
        assert!(
            out.branches
                .iter()
                .all(|b| *b == DiscreteInterval::new(0, 3))
        );
    }

    #[test]
    fn map_cospan_merge_takes_hull() {
        // 2 → 1 merge: both left nodes share the apex vertex.
        let c = Cospan::new(vec![0, 0], vec![0], vec![0u32]).unwrap();
        let alg = IntervalCospanAlgebra;
        let out = alg.map_cospan(&c, &bundle(&[(0, 2), (1, 4)])).unwrap();
        assert_eq!(out.branch_count(), 1);
        assert_eq!(out.branches[0], DiscreteInterval::new(0, 4));
    }

    #[test]
    fn map_cospan_dead_branch_drops_interval() {
        // 2 → 1: second left node is an isolated apex vertex (dead branch).
        let c = Cospan::new(vec![0, 1], vec![0], vec![0u32, 0u32]).unwrap();
        let alg = IntervalCospanAlgebra;
        let out = alg.map_cospan(&c, &bundle(&[(0, 2), (0, 5)])).unwrap();
        assert_eq!(out.branch_count(), 1);
        assert_eq!(out.branches[0], DiscreteInterval::new(0, 2));
    }

    #[test]
    fn map_cospan_arity_mismatch_errors() {
        let c = Cospan::new(vec![0], vec![0], vec![0u32]).unwrap();
        let alg = IntervalCospanAlgebra;
        assert!(alg.map_cospan(&c, &bundle(&[(0, 1), (1, 2)])).is_err());
    }

    #[test]
    fn map_cospan_spontaneous_branch_errors() {
        // Right node whose apex vertex has no left preimage.
        let c = Cospan::new(vec![0], vec![1], vec![0u32, 0u32]).unwrap();
        let alg = IntervalCospanAlgebra;
        assert!(alg.map_cospan(&c, &bundle(&[(0, 1)])).is_err());
    }

    #[test]
    fn lax_monoidal_is_direct_sum_with_unit() {
        let alg = IntervalCospanAlgebra;
        let a = bundle(&[(0, 2)]);
        let b = bundle(&[(0, 3), (1, 4)]);

        let ab = alg.lax_monoidal(&a, &b);
        assert_eq!(ab.branch_count(), 3);

        // Unit laws: unit ⊗ a = a = a ⊗ unit.
        let u = alg.unit();
        assert!(alg.lax_monoidal(&u, &a).exactly_equal(&a));
        assert!(alg.lax_monoidal(&a, &u).exactly_equal(&a));
    }

    #[test]
    fn lax_monoidal_naturality_square_componentwise() {
        // a(c_x ⊕ c_y)(e_x ⊗ e_y) = a(c_x)(e_x) ⊗ a(c_y)(e_y).
        let alg = IntervalCospanAlgebra;
        let cx = Cospan::new(vec![0], vec![0, 0], vec![0u32]).unwrap(); // fork
        let cy = Cospan::new(vec![0, 0], vec![0], vec![0u32]).unwrap(); // merge
        let ex = bundle(&[(0, 1)]);
        let ey = bundle(&[(0, 2), (1, 3)]);

        let mut cxy = cx.clone();
        catgraph::monoidal::Monoidal::monoidal(&mut cxy, cy.clone());

        let whole = alg.map_cospan(&cxy, &alg.lax_monoidal(&ex, &ey)).unwrap();
        let parts = alg.lax_monoidal(
            &alg.map_cospan(&cx, &ex).unwrap(),
            &alg.map_cospan(&cy, &ey).unwrap(),
        );
        assert!(whole.exactly_equal(&parts));
    }

    #[test]
    fn functoriality_over_composed_step_cospans() {
        // a(c₁ ; c₂) = a(c₂) ∘ a(c₁) on real multiway step cospans.
        let srs = StringRewriteSystem::new(vec![("A", "AB"), ("A", "BA")]);
        let evolution = srs.run_multiway("A", 3, 32);
        let chain = multiway_step_cospans(&evolution);
        assert!(chain.len() >= 2, "need at least two steps");

        let alg = IntervalCospanAlgebra;
        let k = chain[0].left_to_middle().len();
        let e0 = bundle(&(0..k).map(|_| (0, 1)).collect::<Vec<_>>());

        let staged = alg
            .map_cospan(&chain[1], &alg.map_cospan(&chain[0], &e0).unwrap())
            .unwrap();
        let composite = chain[0].compose(&chain[1]).expect("adjacent steps compose");
        let direct = alg.map_cospan(&composite, &e0).unwrap();

        assert!(
            staged.exactly_equal(&direct),
            "a(c1;c2) must equal a(c2)∘a(c1): staged {staged:?} vs direct {direct:?}"
        );
    }

    #[test]
    fn step_cospan_legs_are_numbered_first_seen() {
        // Apex indices are minted in order of first request over the left
        // boundary then the right. Pinned verbatim: rotating the
        // target-position map gives step 1 `[1, 0, 1, 1]`, and numbering the
        // left boundary in reverse gives `[1, 0, 0, 0]`.
        let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
        let evolution = srs.run_multiway("AB", 4, 64);
        let chain = multiway_step_cospans(&evolution);

        let legs: Vec<(Vec<usize>, Vec<usize>)> = chain
            .iter()
            .take(3)
            .map(|c| (c.left_to_middle().to_vec(), c.right_to_middle().to_vec()))
            .collect();
        assert_eq!(
            legs,
            vec![
                (vec![0], vec![0, 0]),
                (vec![0, 1], vec![0, 1, 1, 1]),
                (
                    vec![0, 1, 2, 3],
                    vec![0, 0, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]
                ),
            ]
        );
    }

    #[test]
    fn step_cospans_share_an_apex_across_every_graph_edge() {
        // Every forward edge from a slice-t node to a slice-(t+1) node puts
        // the two positions in one apex class. The target position is read
        // back with the linear scan the position map replaced.
        let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
        let evolution = srs.run_multiway("AB", 4, 64);
        let foliation = extract_branchial_foliation(&evolution);
        let chain = multiway_step_cospans(&evolution);

        let mut edges_checked = 0;
        for (t, cospan) in chain.iter().enumerate() {
            let (slice_t, slice_next) = (&foliation[t], &foliation[t + 1]);
            for (li, node) in slice_t.nodes.iter().enumerate() {
                let Some(edges) = evolution.get_forward_edges(node) else {
                    continue;
                };
                for edge in edges {
                    let Some(ri) = slice_next.nodes.iter().position(|n| *n == edge.to) else {
                        continue;
                    };
                    edges_checked += 1;
                    assert_eq!(
                        cospan.left_to_middle()[li],
                        cospan.right_to_middle()[ri],
                        "step {t}: edge {li} -> {ri} spans apex {} and {}",
                        cospan.left_to_middle()[li],
                        cospan.right_to_middle()[ri]
                    );
                }
            }
        }
        assert_eq!(edges_checked, 73, "fixture edge census drifted");
    }

    #[test]
    fn step_cospans_boundaries_match_branchial_slices() {
        let srs = StringRewriteSystem::new(vec![("A", "B"), ("A", "C")]);
        let evolution = srs.run_multiway("A", 2, 16);
        let foliation = extract_branchial_foliation(&evolution);
        let chain = multiway_step_cospans(&evolution);

        assert_eq!(chain.len(), foliation.len().saturating_sub(1));
        for (i, c) in chain.iter().enumerate() {
            assert_eq!(c.left_to_middle().len(), foliation[i].nodes.len());
            assert_eq!(c.right_to_middle().len(), foliation[i + 1].nodes.len());
        }
    }
}
