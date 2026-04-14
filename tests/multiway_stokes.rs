//! Integration tests for discrete exterior calculus on multiway complexes.

#[cfg(feature = "dec")]
mod dec_tests {
    use catgraph_physics::multiway::MultiwayEvolutionGraph;
    use irreducible::multiway_stokes::{MultiwayComplex, OneForm};

    /// Build a graph with one confluence diamond (the minimal 2-simplex).
    fn diamond_graph() -> MultiwayEvolutionGraph<&'static str, &'static str> {
        let mut g = MultiwayEvolutionGraph::new();
        let root = g.add_root("top");
        let branches = g.add_fork(root, vec![("left", "e_left", 0), ("right", "e_right", 1)]);
        let bottom = g.add_sequential_step(branches[0], "bottom", "merge_left");
        g.add_merge_edge(branches[1], bottom, "merge_right");
        g
    }

    /// Build a graph with NO confluence diamonds (pure fork, no merge).
    fn fork_only_graph() -> MultiwayEvolutionGraph<&'static str, &'static str> {
        let mut g = MultiwayEvolutionGraph::new();
        let root = g.add_root("top");
        let branches = g.add_fork(root, vec![("left", "e_left", 0), ("right", "e_right", 1)]);
        g.add_sequential_step(branches[0], "left_end", "step");
        g.add_sequential_step(branches[1], "right_end", "step");
        g
    }

    #[test]
    fn complex_from_confluent_graph_has_2_simplices() {
        let g = diamond_graph();
        let complex = MultiwayComplex::from_evolution(&g);
        assert!(
            complex.num_2_simplices() > 0,
            "expected 2-simplices from diamond"
        );
        assert!(complex.num_1_simplices() > 0);
        assert!(complex.num_0_simplices() > 0);
    }

    #[test]
    fn complex_from_fork_only_has_no_2_simplices() {
        let g = fork_only_graph();
        let complex = MultiwayComplex::from_evolution(&g);
        assert_eq!(complex.num_2_simplices(), 0, "no diamonds = no 2-simplices");
    }

    #[test]
    fn uniform_cost_is_closed() {
        let g = diamond_graph();
        let complex = MultiwayComplex::from_evolution(&g);
        let omega = OneForm::uniform(&complex, 1.0);
        assert!(
            complex.is_closed(&omega),
            "uniform cost should be path-independent"
        );
    }

    #[test]
    fn asymmetric_cost_is_not_closed() {
        let g = diamond_graph();
        let complex = MultiwayComplex::from_evolution(&g);

        if complex.num_2_simplices() > 0 && complex.num_1_simplices() >= 2 {
            let omega =
                OneForm::from_values(&complex, |i| if i % 2 == 0 { 1.0 } else { 3.0 });
            assert!(
                !complex.is_closed(&omega),
                "asymmetric cost should be path-dependent"
            );
        }
    }

    #[test]
    fn d_squared_is_zero() {
        let g = diamond_graph();
        let complex = MultiwayComplex::from_evolution(&g);
        let omega = OneForm::uniform(&complex, 2.5);
        assert!(complex.d_squared_is_zero(&omega));
    }

    #[test]
    fn empty_graph_produces_trivial_complex() {
        let g: MultiwayEvolutionGraph<&str, &str> = MultiwayEvolutionGraph::new();
        let complex = MultiwayComplex::from_evolution(&g);
        assert_eq!(complex.num_0_simplices(), 0);
        assert_eq!(complex.num_1_simplices(), 0);
        assert_eq!(complex.num_2_simplices(), 0);
    }

    #[test]
    fn vacuous_closure_on_no_faces() {
        let g = fork_only_graph();
        let complex = MultiwayComplex::from_evolution(&g);
        let omega = OneForm::uniform(&complex, 42.0);
        assert!(complex.is_closed(&omega), "no faces = vacuously closed");
    }

    #[test]
    fn exterior_derivative_produces_correct_dimension() {
        let g = diamond_graph();
        let complex = MultiwayComplex::from_evolution(&g);
        let omega = OneForm::uniform(&complex, 1.0);
        let d_omega = complex.exterior_derivative(&omega);
        assert_eq!(
            d_omega.coefficients.len(),
            complex.num_2_simplices(),
            "2-form should have one coefficient per face"
        );
    }

    /// Build a multiway with at least 10 vertices and at least 5 confluence
    /// diamonds, then verify that every diamond-coboundary coefficient is
    /// finite under a random-ish 1-form. In 2D the diamond-loop coboundary is
    /// identically zero after a second derivative, so this is a numerical
    /// sanity check on the core pipeline at scale.
    #[test]
    fn d_squared_is_zero_on_large_complex() {
        // Build a graph with >= 5 confluence diamonds by chaining fork/merge
        // patterns. SRS evolutions are unreliable for diamond counts —
        // manual construction keeps the assertion deterministic.
        let mut g: MultiwayEvolutionGraph<String, String> = MultiwayEvolutionGraph::new();
        let root = g.add_root("root".to_string());

        // Chain of 5 diamonds: each stage i forks into left/right, then
        // merges into a single node that becomes the top of diamond i+1.
        let mut top = root;
        for i in 0..5_usize {
            let branches = g.add_fork(
                top,
                vec![
                    (format!("l{i}"), format!("e_l{i}"), 0),
                    (format!("r{i}"), format!("e_r{i}"), 1),
                ],
            );
            let bottom = g.add_sequential_step(
                branches[0],
                format!("m{i}"),
                format!("merge_l{i}"),
            );
            g.add_merge_edge(branches[1], bottom, format!("merge_r{i}"));
            top = bottom;
        }

        let complex = MultiwayComplex::from_evolution(&g);

        assert!(
            complex.num_0_simplices() >= 10,
            "expected >= 10 vertices, got {}",
            complex.num_0_simplices()
        );
        assert!(
            complex.num_2_simplices() >= 5,
            "expected >= 5 diamonds, got {}",
            complex.num_2_simplices()
        );

        // Pseudo-random 1-form: deterministic, varies per edge.
        let omega = OneForm::from_values(&complex, |i| {
            let x = (i as f64).mul_add(1.3, 0.7);
            x.sin() * 2.5 - x.cos()
        });

        let d_omega = complex.exterior_derivative(&omega);
        for (i, &c) in d_omega.coefficients.iter().enumerate() {
            assert!(
                c.is_finite(),
                "d(omega) coefficient {i} is not finite: {c}"
            );
        }

        // Second-derivative sanity: diamond coboundary composed with vertex
        // boundary is identically zero by construction.
        assert!(
            complex.d_squared_is_zero(&omega),
            "d^2 must vanish on a multiway diamond complex"
        );
    }

    /// Build a small 2D simplicial complex by hand, equip it with unit-edge
    /// Hodge star operators, wrap in a `Manifold`, and compute the
    /// Hodge Laplacian on 0-forms via `manifold.laplacian(0)`. Densify the
    /// resulting linear operator and verify that every eigenvalue is
    /// non-negative (within numerical tolerance).
    #[test]
    fn hodge_laplacian_spectrum_is_nonneg() {
        use deep_causality_tensor::CausalTensor;
        use deep_causality_topology::{
            Manifold, Simplex, SimplicialComplex, SimplicialComplexBuilder,
        };
        use irreducible::geometry::hodge::unit_hodge_operators;
        use nalgebra::DMatrix;

        // Single equilateral triangle — passes dc_topology's orientation
        // and link-condition checks (only one 2-simplex → no shared-edge
        // sign conflicts). Two-triangle fans share an interior edge whose
        // boundary sum is 2, which dc_topology rejects as non-oriented
        // under its default SimplicialComplexBuilder sign convention.
        let mut builder = SimplicialComplexBuilder::new(2);
        builder.add_simplex(Simplex::new(vec![0, 1, 2])).unwrap();
        let base: SimplicialComplex<f64> = builder.build().expect("build");

        let hodge_ops = unit_hodge_operators(&base);
        let complex = SimplicialComplex::new(
            base.skeletons().clone(),
            base.boundary_operators().clone(),
            base.coboundary_operators().clone(),
            hodge_ops,
        );

        let total_simplices: usize = complex
            .skeletons()
            .iter()
            .map(|s| s.simplices().len())
            .sum();
        let v_count = complex.skeletons()[0].simplices().len();

        // Probe laplacian(0) column-by-column: apply to e_i (unit 0-form),
        // densify. Result is a v_count × v_count matrix L.
        let mut lap = DMatrix::<f64>::zeros(v_count, v_count);
        for i in 0..v_count {
            let mut probe = vec![0.0_f64; total_simplices];
            probe[i] = 1.0;
            let data = CausalTensor::new(probe, vec![total_simplices]).unwrap();
            let manifold = Manifold::<f64, f64>::with_metric(
                complex.clone(),
                data,
                None,
                0,
            )
            .expect("manifold construction on single triangle");
            let col = manifold.laplacian(0);
            let col_slice = col.as_slice();
            assert_eq!(col_slice.len(), v_count, "laplacian(0) row count");
            for (j, &v) in col_slice.iter().enumerate() {
                lap[(j, i)] = v;
            }
        }

        // Symmetrize numerically — laplacian is self-adjoint under the
        // Hodge inner product; round-off in codifferential may break the
        // invariant by ~1e-15.
        let symmetrized = (lap.clone() + lap.transpose()) * 0.5;
        let eigen = symmetrized.symmetric_eigen();
        let min_eig = eigen.eigenvalues.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_eig = eigen
            .eigenvalues
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);

        assert!(
            min_eig >= -1e-10,
            "Hodge Laplacian on 0-forms must be PSD; min eigenvalue = {min_eig}"
        );
        assert!(
            max_eig.is_finite(),
            "max eigenvalue must be finite: {max_eig}"
        );
        // Observed spectrum on the single-unit-triangle complex:
        // {0, 6, 6}. The 0 eigenvalue is the constant 0-form (nullspace of d),
        // the double 6 is the Hodge-weighted graph Laplacian signature.
    }
}
