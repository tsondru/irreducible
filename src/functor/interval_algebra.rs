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

use std::hash::Hash;

use catgraph::cospan::Cospan;
use catgraph::cospan_algebra::CospanAlgebra;
use catgraph::errors::CatgraphError;
use catgraph_physics::multiway::{MultiwayEvolutionGraph, extract_branchial_foliation};

use crate::interval::{DiscreteInterval, ParallelIntervals};

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
#[must_use]
pub fn multiway_step_cospans<S: Clone + Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
) -> Vec<Cospan<u32>> {
    let foliation = extract_branchial_foliation(graph);
    let mut cospans = Vec::new();

    for window in foliation.windows(2) {
        let (slice_t, slice_next) = (&window[0], &window[1]);
        let n_left = slice_t.nodes.len();
        let n_right = slice_next.nodes.len();

        // Union-find over left ∪ right node positions.
        let mut parent: Vec<usize> = (0..n_left + n_right).collect();
        fn find(parent: &mut [usize], mut i: usize) -> usize {
            while parent[i] != i {
                parent[i] = parent[parent[i]];
                i = parent[i];
            }
            i
        }

        for (li, node) in slice_t.nodes.iter().enumerate() {
            if let Some(edges) = graph.get_forward_edges(node) {
                for edge in edges {
                    if let Some(ri) = slice_next.nodes.iter().position(|n| *n == edge.to) {
                        let (a, b) = (find(&mut parent, li), find(&mut parent, n_left + ri));
                        parent[a] = b;
                    }
                }
            }
        }

        // Number components in first-seen order for a canonical apex.
        let mut apex_of_root: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();
        let assign =
            |parent: &mut [usize],
             i: usize,
             apex_of_root: &mut std::collections::HashMap<usize, usize>| {
                let root = find(parent, i);
                let next = apex_of_root.len();
                *apex_of_root.entry(root).or_insert(next)
            };

        let left: Vec<usize> = (0..n_left)
            .map(|i| assign(&mut parent, i, &mut apex_of_root))
            .collect();
        let right: Vec<usize> = (0..n_right)
            .map(|i| assign(&mut parent, n_left + i, &mut apex_of_root))
            .collect();
        let middle = vec![0u32; apex_of_root.len()];

        cospans.push(Cospan::new(left, right, middle));
    }

    cospans
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
        let c = Cospan::new(vec![0], vec![0], vec![0u32]);
        let alg = IntervalCospanAlgebra;
        let out = alg.map_cospan(&c, &bundle(&[(0, 1)])).unwrap();
        assert_eq!(out.branch_count(), 1);
        assert_eq!(out.branches[0], DiscreteInterval::new(0, 1));
    }

    #[test]
    fn map_cospan_fork_duplicates_history() {
        // 1 → 2 fork: one apex vertex, two right nodes.
        let c = Cospan::new(vec![0], vec![0, 0], vec![0u32]);
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
        let c = Cospan::new(vec![0, 0], vec![0], vec![0u32]);
        let alg = IntervalCospanAlgebra;
        let out = alg.map_cospan(&c, &bundle(&[(0, 2), (1, 4)])).unwrap();
        assert_eq!(out.branch_count(), 1);
        assert_eq!(out.branches[0], DiscreteInterval::new(0, 4));
    }

    #[test]
    fn map_cospan_dead_branch_drops_interval() {
        // 2 → 1: second left node is an isolated apex vertex (dead branch).
        let c = Cospan::new(vec![0, 1], vec![0], vec![0u32, 0u32]);
        let alg = IntervalCospanAlgebra;
        let out = alg.map_cospan(&c, &bundle(&[(0, 2), (0, 5)])).unwrap();
        assert_eq!(out.branch_count(), 1);
        assert_eq!(out.branches[0], DiscreteInterval::new(0, 2));
    }

    #[test]
    fn map_cospan_arity_mismatch_errors() {
        let c = Cospan::new(vec![0], vec![0], vec![0u32]);
        let alg = IntervalCospanAlgebra;
        assert!(alg.map_cospan(&c, &bundle(&[(0, 1), (1, 2)])).is_err());
    }

    #[test]
    fn map_cospan_spontaneous_branch_errors() {
        // Right node whose apex vertex has no left preimage.
        let c = Cospan::new(vec![0], vec![1], vec![0u32, 0u32]);
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
        let cx = Cospan::new(vec![0], vec![0, 0], vec![0u32]); // fork
        let cy = Cospan::new(vec![0, 0], vec![0], vec![0u32]); // merge
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
