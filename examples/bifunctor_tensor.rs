//! Tensor products and bifunctor verification for parallel intervals.
//!
//! Demonstrates the symmetric monoidal structure of the cobordism category B.
//!
//! Run: `cargo run --example bifunctor_tensor`

use irreducible::{
    DiscreteInterval, ParallelIntervals, TensorProduct,
    verify_associativity, verify_symmetry, verify_unit_laws,
};

fn main() {
    println!("=== Bifunctor & Tensor Products ===\n");
    basic_tensor();

    println!("\n=== Monoidal Laws ===\n");
    monoidal_laws();
}

/// Tensor product of parallel intervals.
fn basic_tensor() {
    let a = ParallelIntervals::from_branch(DiscreteInterval::new(0, 3));
    let b = ParallelIntervals::from_branch(DiscreteInterval::new(0, 5));

    println!("  a = interval [0, 3]");
    println!("  b = interval [0, 5]");

    let product = a.clone().tensor(b.clone());
    println!("  a ⊗ b = {} parallel intervals", product.branches.len());

    // Unit: tensoring with the identity
    let unit = ParallelIntervals::unit();
    let a_unit = a.clone().tensor(unit.clone());
    println!("  a ⊗ I = {} intervals (same as a)", a_unit.branches.len());

    // Symmetry: a ⊗ b has same total span as b ⊗ a
    let ba = b.tensor(a);
    println!(
        "  a ⊗ b intervals: {}, b ⊗ a intervals: {} (symmetric)",
        product.branches.len(),
        ba.branches.len()
    );
}

/// Verify the monoidal category laws.
fn monoidal_laws() {
    let a = ParallelIntervals::from_branch(DiscreteInterval::new(0, 2));
    let b = ParallelIntervals::from_branch(DiscreteInterval::new(0, 3));
    let c = ParallelIntervals::from_branch(DiscreteInterval::new(0, 4));

    let assoc = verify_associativity(&a, &b, &c);
    println!("  Associativity (a⊗b)⊗c = a⊗(b⊗c): {assoc}");

    let sym = verify_symmetry(&a, &b);
    println!("  Symmetry a⊗b = b⊗a:               {sym}");

    let units = verify_unit_laws(&a);
    println!("  Unit laws a⊗I = I⊗a = a:           {units}");
}
