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
}
