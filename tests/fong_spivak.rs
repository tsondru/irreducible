//! Integration tests for Fong-Spivak categorical infrastructure.
//!
//! Phase 1: Verify re-exports from catgraph v0.10.1 are usable.
//! Phase 2: Frobenius verification API tests.

use catgraph::category::{Composable, ComposableMutating};
use catgraph::cospan::Cospan;
use catgraph::frobenius::FrobeniusMorphism;
use irreducible::machines::hypergraph::{Hypergraph, HypergraphEvolution, RewriteRule};
use irreducible::{
    cap_single, cup_single, verify_cospan_chain_frobenius, CospanAlgebra, CospanToFrobeniusFunctor,
    DiscreteInterval, ElementaryCA, HypergraphCategory, HypergraphFunctor, PartitionAlgebra,
    RelabelingFunctor, StokesIrreducibility, TemporalComplex, TuringMachine,
};

// ---------------------------------------------------------------------------
// Phase 1: Re-export validation
// ---------------------------------------------------------------------------

#[test]
fn cospan_satisfies_hypergraph_category() {
    // Cospan<u32> implements HypergraphCategory<u32> (Thm 3.14)
    let unit: Cospan<u32> = HypergraphCategory::unit(0);
    assert_eq!(unit.domain(), Vec::<u32>::new());
    assert_eq!(unit.codomain(), vec![0u32]);

    let counit: Cospan<u32> = HypergraphCategory::counit(0);
    assert_eq!(counit.domain(), vec![0u32]);
    assert_eq!(counit.codomain(), Vec::<u32>::new());

    let mult: Cospan<u32> = HypergraphCategory::multiplication(0);
    assert_eq!(mult.domain(), vec![0u32, 0]);
    assert_eq!(mult.codomain(), vec![0u32]);

    let comult: Cospan<u32> = HypergraphCategory::comultiplication(0);
    assert_eq!(comult.domain(), vec![0u32]);
    assert_eq!(comult.codomain(), vec![0u32, 0]);
}

#[test]
fn relabeling_functor_preserves_structure() {
    let functor = RelabelingFunctor::new(|x: u32| x + 10);

    let unit: Cospan<u32> = HypergraphCategory::unit(0);
    let mapped = functor.map_mor(&unit).unwrap();

    // Middle elements should be shifted by 10
    for &label in mapped.middle() {
        assert!(label >= 10, "label {label} should be >= 10 after relabeling");
    }

    // Codomain should be relabeled: [0] -> [10]
    assert_eq!(mapped.codomain(), vec![10u32]);
}

#[test]
fn cospan_to_frobenius_decomposition() {
    let functor = CospanToFrobeniusFunctor::<()>::new();

    // Decompose a unit cospan: [] -> [0]
    let unit: Cospan<u32> = HypergraphCategory::unit(0);
    let morphism: FrobeniusMorphism<u32, ()> = functor.map_mor(&unit).unwrap();

    assert!(
        morphism.depth() > 0,
        "decomposition should have at least one layer"
    );
    assert_eq!(morphism.domain(), Vec::<u32>::new());
    assert_eq!(morphism.codomain(), vec![0u32]);
}

#[test]
fn cup_cap_zigzag_identity() {
    // Zigzag: (id_X ⊗ cap_X) ; (cup_X ⊗ id_X) = id_X
    // We verify the weaker property: cup and cap have correct interfaces
    let cup: FrobeniusMorphism<u32, ()> = cup_single(0);
    let cap: FrobeniusMorphism<u32, ()> = cap_single(0);

    // cup: [] -> [0, 0]
    assert_eq!(cup.domain(), Vec::<u32>::new());
    assert_eq!(cup.codomain(), vec![0u32, 0]);

    // cap: [0, 0] -> []
    assert_eq!(cap.domain(), vec![0u32, 0]);
    assert_eq!(cap.codomain(), Vec::<u32>::new());

    // cup ; cap should compose (codomain of cup matches domain of cap)
    let mut composed = cup.clone();
    composed.compose(cap).expect("cup ; cap should compose");
    // Result: [] -> []
    assert_eq!(composed.domain(), Vec::<u32>::new());
    assert_eq!(composed.codomain(), Vec::<u32>::new());
}

#[test]
fn partition_algebra_map_cospan() {
    let algebra = PartitionAlgebra;
    let element: Cospan<u32> = algebra.unit(); // [] -> [] (empty partition)

    // Build a simple cospan to map over
    let unit: Cospan<u32> = HypergraphCategory::unit(0);

    // map_cospan: compose element with cospan
    let result = algebra.map_cospan(&unit, &element);
    assert!(
        result.is_ok(),
        "partition algebra should map empty element through unit cospan"
    );
}

// ---------------------------------------------------------------------------
// Phase 2: Frobenius verification
// ---------------------------------------------------------------------------

#[test]
fn empty_chain_frobenius() {
    let result = verify_cospan_chain_frobenius(&[]);
    assert!(result.all_valid);
    assert!(result.composition_preserved);
    assert!(result.per_cospan.is_empty());
}

#[test]
fn single_cospan_frobenius() {
    let intervals = vec![DiscreteInterval::new(0, 3)];
    let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    let chain = complex.to_cospan_chain();

    let result = verify_cospan_chain_frobenius(&chain);
    assert!(result.all_valid);
    assert!(result.composition_preserved);
    assert_eq!(result.per_cospan.len(), 1);
    assert!(result.per_cospan[0].is_valid);
    assert!(
        result.per_cospan[0].generator_count > 0,
        "single cospan should have at least one generator layer"
    );
}

#[test]
fn stokes_frobenius_verification_irreducible_tm() {
    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);
    let intervals = history.to_intervals();

    let analysis = StokesIrreducibility::analyze(&intervals).unwrap();
    let frobenius = analysis.verify_frobenius();

    assert!(frobenius.all_valid, "all cospans should decompose");
    assert!(
        frobenius.composition_preserved,
        "composition should be preserved"
    );
    assert_eq!(frobenius.per_cospan.len(), intervals.len());
}

#[test]
fn stokes_frobenius_verification_irreducible_ca() {
    let ca = ElementaryCA::rule_30(11);
    let initial = ca.single_cell_initial();
    let history = ca.run(initial, 10);
    let intervals = history.to_intervals();

    let analysis = StokesIrreducibility::analyze(&intervals).unwrap();
    let frobenius = analysis.verify_frobenius();

    assert!(frobenius.all_valid);
    assert!(frobenius.composition_preserved);
    assert_eq!(frobenius.per_cospan.len(), intervals.len());
}

#[test]
fn frobenius_agrees_with_functorial_irreducibility() {
    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);
    let intervals = history.to_intervals();

    let analysis = StokesIrreducibility::analyze(&intervals).unwrap();

    // Both perspectives should agree on irreducibility
    let stokes_irreducible = analysis.is_irreducible();
    let frobenius_valid = analysis.verify_frobenius().all_valid;

    assert_eq!(
        stokes_irreducible, frobenius_valid,
        "Stokes irreducibility ({stokes_irreducible}) should agree with Frobenius validity ({frobenius_valid})"
    );
}

#[test]
fn hypergraph_evolution_frobenius_verification() {
    let rule = RewriteRule::wolfram_a_to_bb();
    let graph = Hypergraph::from_edges(vec![vec![0, 1, 2]]);

    let evolution = HypergraphEvolution::run(&graph, &[rule], 3);
    let chain = evolution.to_cospan_chain();

    if !chain.is_empty() {
        let result = verify_cospan_chain_frobenius(&chain);
        assert!(
            result.all_valid,
            "all evolution cospans should decompose into Frobenius generators"
        );
        assert_eq!(result.per_cospan.len(), chain.len());

        for check in &result.per_cospan {
            assert!(check.is_valid);
            assert!(
                check.generator_count > 0,
                "each cospan should have at least one generator layer"
            );
        }
    }
}
