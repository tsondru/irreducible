//! Discrete exterior calculus on 2D multiway simplicial complexes.
//!
//! Builds a 2D simplicial complex from multiway confluence diamonds and
//! computes `d: Ω¹ → Ω²` such that `d ω = 0` (closedness) is a falsifiable,
//! graph-structural property of path-integrated "cost".
//!
//! # Mathematical Setting
//!
//! Given a [`MultiwayEvolutionGraph`] with confluence diamonds:
//! - **0-simplices** = states (nodes)
//! - **1-simplices** = rewrite events (edges)
//! - **2-simplices** = confluence diamonds (triangulated into pairs of
//!   triangles, see below)
//!
//! A 1-form ω assigns a real coefficient to each rewrite edge. The **diamond
//! exterior derivative** `d ω` evaluates on each diamond by integrating ω
//! around the 4-cycle:
//!
//! `d ω (diamond) = ω(top→left) + ω(left→bottom) − ω(top→right) − ω(right→bottom)`
//!
//! ω is **closed** (`d ω = 0`) iff cost is path-independent across every
//! diamond — the categorical manifestation of causal invariance in the
//! Wolfram Physics sense.
//!
//! # Substrate port (Phase 2, 2026-04-14)
//!
//! The internal skeleton storage is now a
//! [`deep_causality_topology::SimplicialComplex<f64>`]. The
//! [`MultiwayComplex`] type holds:
//!
//! 1. The `dc_topology` complex (vertices + multiway edges + triangulated
//!    2-simplices with populated unit-edge Hodge operators).
//! 2. A parallel table of diamond faces as 4-tuples of multiway-edge indices
//!    — the canonical 4-cycle structure used by [`MultiwayComplex::exterior_derivative`].
//!
//! The diamond-cycle coboundary is deliberately **not** the standard simplicial
//! `d₁` on the triangulated faces: those two semantics differ, and the diamond
//! integral is the one that lines up with the categorical closedness statement
//! (path integrals around confluence diamonds). The triangulated complex is
//! retained because it gives [`Manifold::laplacian`],
//! [`Manifold::hodge_star`], and [`Manifold::codifferential`] something to
//! run against when callers want the full `dc_topology` DEC stack
//! (e.g. the `hodge_laplacian_spectrum_is_nonneg` integration test).
//!
//! Requires feature `dec` (enables `dc-geometry` + `nalgebra`).

#[cfg(feature = "dec")]
mod inner {
    use std::collections::HashMap;
    use std::hash::Hash;

    use nalgebra::DVector;

    use catgraph_physics::multiway::{MultiwayEvolutionGraph, MultiwayNodeId};
    use deep_causality_topology::{
        Simplex, SimplicialComplex, SimplicialComplexBuilder, TopologyError,
    };

    use crate::geometry::hodge::unit_hodge_operators;

    /// A 2-dimensional simplicial complex extracted from a multiway evolution.
    ///
    /// Internally backed by a [`deep_causality_topology::SimplicialComplex<f64>`]
    /// whose 2-simplices are the triangulation of confluence diamonds
    /// (one diamond → two triangles sharing a synthetic `top→bottom` diagonal).
    /// The parallel [`Self::diamond_faces`] table preserves the 4-cycle
    /// structure used by the diamond exterior derivative.
    #[derive(Debug, Clone)]
    pub struct MultiwayComplex {
        /// `dc_topology` substrate: skeletons + boundary / coboundary / Hodge
        /// operators. Contains the triangulation (including synthetic
        /// diagonals) and is suitable for
        /// [`deep_causality_topology::Manifold`] construction.
        complex: SimplicialComplex<f64>,
        /// Count of multiway edges (excludes synthetic `top↔bottom` diagonals
        /// introduced by the diamond triangulation).
        ///
        /// `OneForm::coefficients.len()` always equals this count.
        multiway_edge_count: usize,
        /// Multiway-edge positions in the `dc_topology` 1-skeleton.
        ///
        /// An edge `(from_idx, to_idx)` is looked up by its directed pair in
        /// the multiway graph; the value is the index of the corresponding
        /// `Simplex::new(vec![from_idx, to_idx])` in `complex.skeletons()[1]`.
        /// `#[allow(dead_code)]` because the field is carried for future
        /// Phase-3 callers (e.g. form round-tripping between multiway and
        /// `dc_topology` spaces); not yet referenced.
        #[allow(dead_code)]
        multiway_edge_to_complex: HashMap<(usize, usize), usize>,
        /// Diamond faces as 4-tuples of multiway-edge indices
        /// `[top→left, left→bottom, top→right, right→bottom]`.
        ///
        /// The indices are into the local 0..`multiway_edge_count` range
        /// (the coefficient positions `OneForm::coefficients` uses), not into
        /// the `dc_topology` 1-skeleton.
        diamond_faces: Vec<[usize; 4]>,
        /// Parallel to `diamond_faces`: the four
        /// `(from_vertex_idx, to_vertex_idx)` tuples for each diamond edge, used by
        /// [`MultiwayComplex::d_squared_is_zero`] to recover the vertex
        /// endpoints without an extra lookup.
        diamond_edge_endpoints: Vec<[(usize, usize); 4]>,
        /// Vertex count (matches `complex.skeletons()[0].simplices().len()`).
        vertex_count: usize,
    }

    /// A 1-form: assigns a coefficient to each multiway edge.
    #[derive(Debug, Clone)]
    pub struct OneForm {
        /// Coefficients, one per multiway edge. Length equals
        /// [`MultiwayComplex::num_1_simplices`].
        pub coefficients: DVector<f64>,
    }

    /// A 2-form: assigns a coefficient to each confluence-diamond face.
    #[derive(Debug, Clone)]
    pub struct TwoForm {
        /// Coefficients, one per diamond face. Length equals
        /// [`MultiwayComplex::num_2_simplices`].
        pub coefficients: DVector<f64>,
    }

    impl MultiwayComplex {
        /// Build the 2D simplicial complex from a multiway evolution graph.
        ///
        /// 0-simplices are multiway nodes, 1-simplices are multiway edges
        /// (plus synthetic `top↔bottom` diagonals from the diamond
        /// triangulation), and 2-simplices are the triangulated diamond
        /// faces. Unit-edge equilateral Hodge operators are attached via
        /// [`unit_hodge_operators`].
        ///
        /// On an empty multiway graph, returns a trivial complex (zero
        /// skeletons, zero faces).
        ///
        /// # Panics
        ///
        /// Does not panic in practice. The internal `expect` covers the
        /// `SimplicialComplexBuilder` contract: `add_simplex` only rejects
        /// simplices exceeding the builder's `max_dim = 2`, and we feed it
        /// only 0-, 1-, and 2-simplices.
        #[must_use]
        #[allow(clippy::similar_names)]
        pub fn from_evolution<S: Clone + Hash, T: Clone>(
            graph: &MultiwayEvolutionGraph<S, T>,
        ) -> Self {
            // --- Step 1: Index vertices contiguously (0..n) in step-major order.
            let mut vertex_list: Vec<MultiwayNodeId> = Vec::new();
            let mut vertex_index: HashMap<MultiwayNodeId, usize> = HashMap::new();

            for step in 0..=graph.max_step() {
                for node_id in graph.node_ids_at_step(step) {
                    let idx = vertex_list.len();
                    vertex_list.push(node_id);
                    vertex_index.insert(node_id, idx);
                }
            }

            if vertex_list.is_empty() {
                // Empty graph → trivial complex with no skeletons.
                return Self {
                    complex: SimplicialComplex::<f64>::default(),
                    multiway_edge_count: 0,
                    multiway_edge_to_complex: HashMap::new(),
                    diamond_faces: Vec::new(),
                    diamond_edge_endpoints: Vec::new(),
                    vertex_count: 0,
                };
            }

            // --- Step 2: Collect multiway edges in insertion order.
            let mut multiway_edges: Vec<(usize, usize)> = Vec::new();
            let mut multiway_edge_index: HashMap<(usize, usize), usize> = HashMap::new();

            for &v_id in &vertex_list {
                if let Some(fwd) = graph.get_forward_edges(&v_id) {
                    for edge in fwd {
                        if let (Some(&from_idx), Some(&to_idx)) =
                            (vertex_index.get(&edge.from), vertex_index.get(&edge.to))
                        {
                            let key = (from_idx, to_idx);
                            multiway_edge_index.entry(key).or_insert_with(|| {
                                let idx = multiway_edges.len();
                                multiway_edges.push(key);
                                idx
                            });
                        }
                    }
                }
            }
            let multiway_edge_count = multiway_edges.len();

            // --- Step 3: Enumerate confluence diamonds → 4-cycle faces.
            let raw_diamonds = graph.confluence_diamonds();
            let mut diamond_faces: Vec<[usize; 4]> = Vec::new();
            let mut diamond_edge_endpoints: Vec<[(usize, usize); 4]> = Vec::new();
            // Collected diamonds in vertex-index form for the triangulation pass.
            let mut diamond_vertices: Vec<[usize; 4]> = Vec::new();

            for diamond in &raw_diamonds {
                let (Some(&t), Some(&l), Some(&r), Some(&b)) = (
                    vertex_index.get(&diamond.top),
                    vertex_index.get(&diamond.left),
                    vertex_index.get(&diamond.right),
                    vertex_index.get(&diamond.bottom),
                ) else {
                    continue;
                };

                let (Some(&tl), Some(&lb), Some(&tr), Some(&rb)) = (
                    multiway_edge_index.get(&(t, l)),
                    multiway_edge_index.get(&(l, b)),
                    multiway_edge_index.get(&(t, r)),
                    multiway_edge_index.get(&(r, b)),
                ) else {
                    continue;
                };

                diamond_faces.push([tl, lb, tr, rb]);
                diamond_edge_endpoints.push([(t, l), (l, b), (t, r), (r, b)]);
                diamond_vertices.push([t, l, r, b]);
            }

            // --- Step 4: Build the `dc_topology` complex.
            // Each diamond triangulates into {t, l, b} and {t, r, b} — sharing
            // the synthetic diagonal {t, b}. Add each multiway edge as a
            // 1-simplex first so it lives in the skeleton regardless of
            // whether any diamond touches it.
            let complex = build_dc_complex(
                vertex_list.len(),
                &multiway_edges,
                &diamond_vertices,
            )
            .expect(
                "dc_topology complex construction cannot fail on valid multiway-derived input",
            );

            let multiway_edge_to_complex =
                build_multiway_edge_to_complex_index(complex_edge_skeleton(&complex), &multiway_edges);

            Self {
                complex,
                multiway_edge_count,
                multiway_edge_to_complex,
                diamond_faces,
                diamond_edge_endpoints,
                vertex_count: vertex_list.len(),
            }
        }

        /// Number of 0-simplices (vertices / multiway states).
        #[inline]
        #[must_use]
        pub fn num_0_simplices(&self) -> usize {
            self.vertex_count
        }

        /// Number of **multiway** 1-simplices (rewrite events).
        ///
        /// Synthetic diagonals introduced by the diamond triangulation are
        /// **not** counted here — the coefficient vector of a [`OneForm`]
        /// runs `0..num_1_simplices`.
        #[inline]
        #[must_use]
        pub fn num_1_simplices(&self) -> usize {
            self.multiway_edge_count
        }

        /// Number of 2-simplices (confluence-diamond faces).
        #[inline]
        #[must_use]
        pub fn num_2_simplices(&self) -> usize {
            self.diamond_faces.len()
        }

        /// Borrow the underlying `dc_topology` complex.
        ///
        /// Exposed for callers that want direct access to the
        /// [`deep_causality_topology::Manifold`] DEC pipeline
        /// (`exterior_derivative`, `hodge_star`, `codifferential`,
        /// `laplacian`) — see the `hodge_laplacian_spectrum_is_nonneg`
        /// integration test for a usage example.
        #[inline]
        #[must_use]
        pub fn simplicial_complex(&self) -> &SimplicialComplex<f64> {
            &self.complex
        }

        /// Compute the exterior derivative `d ω` of a 1-form as the
        /// path-integrated difference around every confluence diamond.
        ///
        /// For each diamond `[tl, lb, tr, rb]`:
        ///
        /// `d ω(face) = ω(tl) + ω(lb) − ω(tr) − ω(rb)`
        ///
        /// Returns a 2-form with one coefficient per diamond.
        ///
        /// This is **not** the standard simplicial `d₁` on the triangulated
        /// faces — see the module-level docs for why the 4-cycle integral is
        /// the correct operator for closedness-as-causal-invariance.
        #[must_use]
        pub fn exterior_derivative(&self, omega: &OneForm) -> TwoForm {
            let n_faces = self.diamond_faces.len();
            let mut coeffs = DVector::zeros(n_faces);

            for (i, &[tl, lb, tr, rb]) in self.diamond_faces.iter().enumerate() {
                coeffs[i] = omega.coefficients[tl] + omega.coefficients[lb]
                    - omega.coefficients[tr]
                    - omega.coefficients[rb];
            }

            TwoForm {
                coefficients: coeffs,
            }
        }

        /// Check whether a 1-form is closed: `d ω = 0`.
        ///
        /// A closed 1-form means "cost" is path-independent across every
        /// confluence diamond — the categorical manifestation of causal
        /// invariance. Returns `true` vacuously when there are no diamonds.
        #[must_use]
        pub fn is_closed(&self, omega: &OneForm) -> bool {
            if self.diamond_faces.is_empty() {
                return true;
            }
            let d_omega = self.exterior_derivative(omega);
            d_omega.coefficients.iter().all(|&c| c.abs() < 1e-10)
        }

        /// Sanity check: `d² = 0` on this complex.
        ///
        /// Verifies that for every diamond, the boundary-of-coboundary
        /// vanishes at every vertex. Validates that the orientation
        /// convention baked into [`Self::exterior_derivative`] is internally
        /// consistent; in 2D this is identically true by construction.
        #[must_use]
        #[allow(clippy::similar_names)]
        pub fn d_squared_is_zero(&self, _omega: &OneForm) -> bool {
            if self.diamond_faces.is_empty() || self.multiway_edge_count == 0 {
                return true;
            }

            let n_verts = self.vertex_count;

            for endpoints in &self.diamond_edge_endpoints {
                let mut boundary = vec![0.0_f64; n_verts];
                let [(a_tl, b_tl), (a_lb, b_lb), (a_tr, b_tr), (a_rb, b_rb)] = *endpoints;

                // +e_tl: +b_tl − a_tl
                boundary[b_tl] += 1.0;
                boundary[a_tl] -= 1.0;
                // +e_lb: +b_lb − a_lb
                boundary[b_lb] += 1.0;
                boundary[a_lb] -= 1.0;
                // −e_tr: −(b_tr − a_tr)
                boundary[b_tr] -= 1.0;
                boundary[a_tr] += 1.0;
                // −e_rb: −(b_rb − a_rb)
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

        /// Create a 1-form from a function of multiway-edge index.
        #[must_use]
        pub fn from_values(complex: &MultiwayComplex, f: impl Fn(usize) -> f64) -> Self {
            let n = complex.num_1_simplices();
            Self {
                coefficients: DVector::from_fn(n, |i, _| f(i)),
            }
        }
    }

    /// Build a `dc_topology` `SimplicialComplex<f64>` with the given vertices,
    /// multiway edges, and triangulated diamond faces; populate Hodge stars.
    ///
    /// Each diamond `[t, l, r, b]` triangulates into `{t, l, b}` and
    /// `{t, r, b}`, sharing the synthetic diagonal `{t, b}`. The
    /// [`SimplicialComplexBuilder`] enforces closure, so adding the
    /// 2-simplices implicitly adds every edge (multiway or synthetic) they
    /// touch.
    ///
    /// We add the multiway 1-simplices explicitly first so they remain in the
    /// 1-skeleton even if no diamond touches them (e.g. in a fork-only graph).
    ///
    /// # Panics
    ///
    /// Does not panic in practice: the only failure mode of `add_simplex` is
    /// exceeding the builder's `max_dim = 2`, and we only feed it 0-, 1-, and
    /// 2-simplices.
    fn build_dc_complex(
        vertex_count: usize,
        multiway_edges: &[(usize, usize)],
        diamond_vertices: &[[usize; 4]],
    ) -> Result<SimplicialComplex<f64>, TopologyError> {
        let mut builder = SimplicialComplexBuilder::new(2);

        // 0-simplices — ensures isolated vertices are retained.
        for v in 0..vertex_count {
            builder.add_simplex(Simplex::new(vec![v]))?;
        }

        // 1-simplices: multiway edges. Skip self-loops: Simplex::new collapses
        // them to 0-simplices, silently corrupting the 1-skeleton.
        for &(a, b) in multiway_edges {
            if a == b {
                continue;
            }
            builder.add_simplex(Simplex::new(vec![a, b]))?;
        }

        // 2-simplices: two triangles per diamond.
        for &[t, l, r, b] in diamond_vertices {
            // Guard against degenerate diamonds where multiple corners share
            // a vertex — a collapsed `Simplex::new` silently loses a vertex.
            let mut uniq: Vec<usize> = vec![t, l, r, b];
            uniq.sort_unstable();
            uniq.dedup();
            if uniq.len() < 4 {
                continue;
            }
            builder.add_simplex(Simplex::new(vec![t, l, b]))?;
            builder.add_simplex(Simplex::new(vec![t, r, b]))?;
        }

        let mut complex = builder.build::<f64>()?;

        // Populate Hodge stars for DEC ops (`codifferential`, `laplacian`).
        let hodge_ops = unit_hodge_operators(&complex);
        let (skeletons, boundary_ops, coboundary_ops) = (
            complex.skeletons().clone(),
            complex.boundary_operators().clone(),
            complex.coboundary_operators().clone(),
        );
        complex = SimplicialComplex::new(skeletons, boundary_ops, coboundary_ops, hodge_ops);

        Ok(complex)
    }

    /// Borrow the 1-skeleton of a `dc_topology` complex. Returns `None` if the
    /// complex has no 1-stratum (empty multiway → zero edges → zero-length
    /// skeleton list after the 0-skeleton).
    fn complex_edge_skeleton(
        complex: &SimplicialComplex<f64>,
    ) -> Option<&deep_causality_topology::Skeleton> {
        complex.skeletons().get(1)
    }

    /// Index each multiway edge by its `(vertex_a, vertex_b)` pair into the
    /// `dc_topology` 1-skeleton. Useful for callers who want to map a
    /// multiway-edge coefficient into the full `dc_topology` data tensor.
    fn build_multiway_edge_to_complex_index(
        edge_skeleton: Option<&deep_causality_topology::Skeleton>,
        multiway_edges: &[(usize, usize)],
    ) -> HashMap<(usize, usize), usize> {
        let Some(edge_skeleton) = edge_skeleton else {
            return HashMap::new();
        };
        let mut map = HashMap::with_capacity(multiway_edges.len());
        for &(a, b) in multiway_edges {
            if a == b {
                continue;
            }
            let edge = Simplex::new(vec![a, b]);
            if let Some(idx) = edge_skeleton.get_index(&edge) {
                map.insert((a, b), idx);
            }
        }
        map
    }
}

#[cfg(feature = "dec")]
pub use inner::*;
