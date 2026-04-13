//! Discrete exterior calculus on 2D multiway simplicial complexes.
//!
//! Replaces the deprecated `exterior_derivative` from `src/stokes.rs` (trivial
//! on 1D complexes). This module builds a genuine 2D simplicial complex from
//! multiway confluence diamonds, where `d: Omega^1 -> Omega^2` is non-trivial and
//! `d_omega = 0` (closedness) is a falsifiable property.
//!
//! # Mathematical Setting
//!
//! Given a [`MultiwayEvolutionGraph`] with confluence diamonds:
//! - **0-simplices** = states (nodes)
//! - **1-simplices** = rewrite events (edges)
//! - **2-simplices** = confluence diamonds (two paths reconverging)
//!
//! A 1-form omega assigns a real coefficient to each edge (e.g. "step cost").
//! The exterior derivative `d_omega` evaluates on each diamond:
//!
//!   `d_omega(diamond) = omega(top->left) + omega(left->bottom) - omega(top->right) - omega(right->bottom)`
//!
//! omega is **closed** (`d_omega = 0`) iff cost is path-independent across every
//! diamond. This is exactly causal invariance in the Wolfram Physics sense.
//!
//! Requires feature `dec` (enables `nalgebra` + `nalgebra-sparse`).

#[cfg(feature = "dec")]
mod inner {
    use std::collections::HashMap;
    use std::hash::Hash;

    use nalgebra::DVector;

    use catgraph_physics::multiway::{MultiwayEvolutionGraph, MultiwayNodeId};

    /// A 2-dimensional simplicial complex extracted from a multiway graph.
    #[derive(Debug, Clone)]
    pub struct MultiwayComplex {
        /// Ordered list of 0-simplices (node IDs).
        vertices: Vec<MultiwayNodeId>,
        /// Ordered list of 1-simplices as (from, to) index pairs into `vertices`.
        edges: Vec<(usize, usize)>,
        /// Ordered list of 2-simplices as 4-tuples of edge indices:
        /// `[top->left, left->bottom, top->right, right->bottom]`.
        /// Orientation: top->left->bottom is positive, top->right->bottom is negative.
        faces: Vec<[usize; 4]>,
    }

    /// A 1-form: assigns a coefficient to each 1-simplex (edge).
    #[derive(Debug, Clone)]
    pub struct OneForm {
        /// Coefficients, one per edge.
        pub coefficients: DVector<f64>,
    }

    /// A 2-form: assigns a coefficient to each 2-simplex (face/diamond).
    #[derive(Debug, Clone)]
    pub struct TwoForm {
        /// Coefficients, one per face.
        pub coefficients: DVector<f64>,
    }

    impl MultiwayComplex {
        /// Build the 2D simplicial complex from a multiway evolution graph.
        #[must_use]
        #[allow(clippy::similar_names)]
        pub fn from_evolution<S: Clone + Hash, T: Clone>(
            graph: &MultiwayEvolutionGraph<S, T>,
        ) -> Self {
            let mut vertex_list: Vec<MultiwayNodeId> = Vec::new();
            let mut vertex_index: HashMap<MultiwayNodeId, usize> = HashMap::new();

            for step in 0..=graph.max_step() {
                for node_id in graph.node_ids_at_step(step) {
                    let idx = vertex_list.len();
                    vertex_list.push(node_id);
                    vertex_index.insert(node_id, idx);
                }
            }

            let mut edges: Vec<(usize, usize)> = Vec::new();
            let mut edge_index: HashMap<(usize, usize), usize> = HashMap::new();

            for &v_id in &vertex_list {
                if let Some(fwd) = graph.get_forward_edges(&v_id) {
                    for edge in fwd {
                        if let (Some(&from_idx), Some(&to_idx)) =
                            (vertex_index.get(&edge.from), vertex_index.get(&edge.to))
                        {
                            let key = (from_idx, to_idx);
                            edge_index.entry(key).or_insert_with(|| {
                                let idx = edges.len();
                                edges.push(key);
                                idx
                            });
                        }
                    }
                }
            }

            let diamonds = graph.confluence_diamonds();
            let mut faces: Vec<[usize; 4]> = Vec::new();

            for diamond in &diamonds {
                let top = vertex_index.get(&diamond.top);
                let left = vertex_index.get(&diamond.left);
                let right = vertex_index.get(&diamond.right);
                let bottom = vertex_index.get(&diamond.bottom);

                if let (Some(&t), Some(&l), Some(&r), Some(&b)) = (top, left, right, bottom) {
                    let e_tl = edge_index.get(&(t, l));
                    let e_lb = edge_index.get(&(l, b));
                    let e_tr = edge_index.get(&(t, r));
                    let e_rb = edge_index.get(&(r, b));

                    if let (Some(&tl), Some(&lb), Some(&tr), Some(&rb)) =
                        (e_tl, e_lb, e_tr, e_rb)
                    {
                        faces.push([tl, lb, tr, rb]);
                    }
                }
            }

            Self {
                vertices: vertex_list,
                edges,
                faces,
            }
        }

        /// Number of 0-simplices (vertices/states).
        #[inline]
        #[must_use]
        pub fn num_0_simplices(&self) -> usize {
            self.vertices.len()
        }

        /// Number of 1-simplices (edges/events).
        #[inline]
        #[must_use]
        pub fn num_1_simplices(&self) -> usize {
            self.edges.len()
        }

        /// Number of 2-simplices (faces/diamonds).
        #[inline]
        #[must_use]
        pub fn num_2_simplices(&self) -> usize {
            self.faces.len()
        }

        /// Compute the exterior derivative `d_omega` of a 1-form.
        ///
        /// For each 2-simplex (diamond) `[tl, lb, tr, rb]`:
        ///   `d_omega(face) = omega(tl) + omega(lb) - omega(tr) - omega(rb)`
        ///
        /// Returns a 2-form with one coefficient per face.
        #[must_use]
        pub fn exterior_derivative(&self, omega: &OneForm) -> TwoForm {
            let n_faces = self.faces.len();
            let mut coeffs = DVector::zeros(n_faces);

            for (i, &[tl, lb, tr, rb]) in self.faces.iter().enumerate() {
                coeffs[i] = omega.coefficients[tl] + omega.coefficients[lb]
                    - omega.coefficients[tr]
                    - omega.coefficients[rb];
            }

            TwoForm {
                coefficients: coeffs,
            }
        }

        /// Check whether a 1-form is closed: `d_omega = 0`.
        ///
        /// A closed 1-form means the "cost" is path-independent across every
        /// confluence diamond -- the categorical manifestation of causal invariance.
        #[must_use]
        pub fn is_closed(&self, omega: &OneForm) -> bool {
            if self.faces.is_empty() {
                return true; // vacuously closed (no 2-simplices)
            }
            let d_omega = self.exterior_derivative(omega);
            d_omega.coefficients.iter().all(|&c| c.abs() < 1e-10)
        }

        /// Sanity check: `d^2 = 0` on this complex.
        ///
        /// Verifies that for every 2-simplex, the boundary-of-coboundary
        /// vanishes. This validates the orientation conventions.
        #[must_use]
        #[allow(clippy::similar_names)]
        pub fn d_squared_is_zero(&self, _omega: &OneForm) -> bool {
            if self.faces.is_empty() || self.edges.is_empty() {
                return true;
            }

            // For each face, compute ∂₁(d₂ᵀ row) and check it's zero.
            // d₂ᵀ row for face f = +e_{tl} + e_{lb} - e_{tr} - e_{rb}
            // ∂₁(edge (a,b)) = +vertex_b - vertex_a
            // So ∂₁(d₂ᵀ row) = (+b_tl - a_tl) + (+b_lb - a_lb) - (+b_tr - a_tr) - (+b_rb - a_rb)
            //
            // For a well-formed diamond:
            //   tl: top→left, lb: left→bottom, tr: top→right, rb: right→bottom
            // So: (+left - top) + (+bottom - left) - (+right - top) - (+bottom - right) = 0

            let n_verts = self.vertices.len();

            for &[tl, lb, tr, rb] in &self.faces {
                let mut boundary = vec![0.0_f64; n_verts];
                let (a_tl, b_tl) = self.edges[tl];
                let (a_lb, b_lb) = self.edges[lb];
                let (a_tr, b_tr) = self.edges[tr];
                let (a_rb, b_rb) = self.edges[rb];

                // +e_tl contribution: +b_tl - a_tl
                boundary[b_tl] += 1.0;
                boundary[a_tl] -= 1.0;
                // +e_lb contribution: +b_lb - a_lb
                boundary[b_lb] += 1.0;
                boundary[a_lb] -= 1.0;
                // -e_tr contribution: -(+b_tr - a_tr)
                boundary[b_tr] -= 1.0;
                boundary[a_tr] += 1.0;
                // -e_rb contribution: -(+b_rb - a_rb)
                boundary[b_rb] -= 1.0;
                boundary[a_rb] += 1.0;

                if !boundary.iter().all(|&v| v.abs() < 1e-10) {
                    return false;
                }
            }

            true
        }
    }

    impl OneForm {
        /// Create a uniform 1-form (all edges have the same coefficient).
        #[must_use]
        pub fn uniform(complex: &MultiwayComplex, value: f64) -> Self {
            Self {
                coefficients: DVector::from_element(complex.num_1_simplices(), value),
            }
        }

        /// Create a 1-form from a function of edge index.
        #[must_use]
        pub fn from_values(complex: &MultiwayComplex, f: impl Fn(usize) -> f64) -> Self {
            let n = complex.num_1_simplices();
            Self {
                coefficients: DVector::from_fn(n, |i, _| f(i)),
            }
        }
    }
}

#[cfg(feature = "dec")]
pub use inner::*;
