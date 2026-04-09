//! Fong-Spivak categorical verification of computational irreducibility.
//!
//! Demonstrates the connection between three perspectives:
//! 1. Stokes irreducibility (differential-geometric)
//! 2. Frobenius decomposition (Fong-Spivak categorical)
//! 3. Functorial irreducibility (Gorard)
//!
//! Run: `cargo run --example fong_spivak`

use irreducible::machines::hypergraph::{Hypergraph, HypergraphEvolution, RewriteRule};
use irreducible::{
    verify_cospan_chain_frobenius, CospanToFrobeniusFunctor, HypergraphCategory, HypergraphFunctor,
    IrreducibilityFunctor, StokesIrreducibility, TuringMachine,
};

fn main() {
    println!("=== Fong-Spivak Categorical Verification ===\n");

    part1_tm_stokes_frobenius();
    part2_hypergraph_frobenius();
    part3_connection_summary();
}

fn part1_tm_stokes_frobenius() {
    println!("--- Part 1: Turing Machine -> Stokes -> Frobenius ---\n");

    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);
    let intervals = history.to_intervals();

    let stokes = StokesIrreducibility::analyze(&intervals).unwrap();
    println!("Stokes analysis:");
    println!("  Irreducible: {}", stokes.is_irreducible());
    println!("  Conservation ratio: {:.6}", stokes.conservation_ratio());
    println!("  Cospan chain length: {}", stokes.to_cospan_chain().len());

    let frobenius = stokes.verify_frobenius();
    println!("\nFrobenius verification:");
    println!("  All valid: {}", frobenius.all_valid);
    println!(
        "  Composition preserved: {}",
        frobenius.composition_preserved
    );
    for check in &frobenius.per_cospan {
        println!(
            "  Cospan {}: valid={}, generators={}",
            check.index, check.is_valid, check.generator_count
        );
    }
    println!();
}

fn part2_hypergraph_frobenius() {
    println!("--- Part 2: Hypergraph Evolution -> Frobenius ---\n");

    let rule = RewriteRule::wolfram_a_to_bb();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);

    let evolution = HypergraphEvolution::run(&graph, &[rule], 3);
    let chain = evolution.to_cospan_chain();

    println!("Evolution: 3 steps, {} cospans", chain.len());

    if chain.is_empty() {
        println!("  No cospans produced (no matching edges)\n");
        return;
    }

    let result = verify_cospan_chain_frobenius(&chain);
    println!("Frobenius verification:");
    println!("  All valid: {}", result.all_valid);
    println!(
        "  Composition preserved: {}",
        result.composition_preserved
    );
    for check in &result.per_cospan {
        println!(
            "  Cospan {}: valid={}, generators={}",
            check.index, check.is_valid, check.generator_count
        );
    }

    // Demonstrate CospanToFrobeniusFunctor directly
    use catgraph::category::{Composable, ComposableMutating};

    let functor = CospanToFrobeniusFunctor::<()>::new();
    if let Ok(morphism) = functor.map_mor(&chain[0]) {
        println!("\nFirst cospan Frobenius decomposition:");
        println!("  Domain: {:?}", morphism.domain());
        println!("  Codomain: {:?}", morphism.codomain());
        println!("  Depth (layers): {}", morphism.depth());
    }

    // Show HypergraphCategory generators
    let unit: catgraph::cospan::Cospan<u32> = HypergraphCategory::unit(0);
    let comult: catgraph::cospan::Cospan<u32> = HypergraphCategory::comultiplication(0);
    println!("\nHypergraphCategory generators for type 0:");
    println!("  unit:   [] -> {:?}", unit.codomain());
    println!(
        "  comult: {:?} -> {:?}",
        comult.domain(),
        comult.codomain()
    );
    println!();
}

fn part3_connection_summary() {
    println!("--- Part 3: Three Perspectives Agree ---\n");

    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);
    let intervals = history.to_intervals();

    // Perspective 1: Functorial
    let functorial = IrreducibilityFunctor::is_sequence_irreducible(&intervals);

    // Perspective 2: Stokes
    let stokes = StokesIrreducibility::analyze(&intervals).unwrap();
    let stokes_irreducible = stokes.is_irreducible();

    // Perspective 3: Frobenius
    let frobenius = stokes.verify_frobenius();
    let frobenius_valid = frobenius.all_valid && frobenius.composition_preserved;

    println!("Busy Beaver 2,2 --- Three-Way Agreement:");
    println!("  Functorial irreducible:        {functorial}");
    println!("  Stokes irreducible:            {stokes_irreducible}");
    println!("  Frobenius decomposition valid:  {frobenius_valid}");
    println!(
        "  All agree: {}",
        functorial == stokes_irreducible && stokes_irreducible == frobenius_valid
    );
}
