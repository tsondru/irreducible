//! Discrete exterior calculus on multiway confluence diamonds.
//!
//! Shows that a uniform-cost 1-form is closed (path-independent)
//! while an asymmetric-cost form is not.

#[cfg(feature = "dec")]
fn main() {
    use catgraph_physics::multiway::MultiwayEvolutionGraph;
    use irreducible::multiway_stokes::{MultiwayComplex, OneForm};

    println!("=== Multiway Discrete Exterior Calculus ===\n");

    // Build a diamond graph: fork into 2, then merge
    let mut g: MultiwayEvolutionGraph<&str, &str> = MultiwayEvolutionGraph::new();
    let root = g.add_root("top");
    let branches = g.add_fork(root, vec![("left", "e_left", 0), ("right", "e_right", 1)]);
    let bottom = g.add_sequential_step(branches[0], "bottom", "merge_left");
    g.add_merge_edge(branches[1], bottom, "merge_right");

    let complex = MultiwayComplex::from_evolution(&g);
    println!(
        "Complex: {} vertices, {} edges, {} faces (diamonds)",
        complex.num_0_simplices(),
        complex.num_1_simplices(),
        complex.num_2_simplices()
    );

    // Uniform cost: every edge = 1.0
    let uniform = OneForm::uniform(&complex, 1.0);
    let d_uniform = complex.exterior_derivative(&uniform);
    println!("\nUniform cost (1.0 per edge):");
    println!("  d_omega = {:?}", d_uniform.coefficients.as_slice());
    println!(
        "  is_closed = {} (path-independent)",
        complex.is_closed(&uniform)
    );

    // Asymmetric cost
    let asymmetric = OneForm::from_values(&complex, |i| if i % 2 == 0 { 1.0 } else { 3.0 });
    let d_asym = complex.exterior_derivative(&asymmetric);
    println!("\nAsymmetric cost (alternating 1.0 / 3.0):");
    println!("  d_omega = {:?}", d_asym.coefficients.as_slice());
    println!(
        "  is_closed = {} (path-dependent)",
        complex.is_closed(&asymmetric)
    );

    // d^2 = 0 sanity check
    println!("\nd^2 = 0 check: {}", complex.d_squared_is_zero(&uniform));
}

#[cfg(not(feature = "dec"))]
fn main() {
    println!("This example requires the `dec` feature:");
    println!("  cargo run --example multiway_stokes --features dec");
}
