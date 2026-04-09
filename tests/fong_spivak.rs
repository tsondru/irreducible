//! Integration tests for Fong-Spivak categorical infrastructure.
//!
//! Phase 1: Verify re-exports from catgraph v0.10.1 are usable.
//! Phase 2: Frobenius verification API tests.

use catgraph::category::{Composable, ComposableMutating};
use catgraph::cospan::Cospan;
use catgraph::frobenius::FrobeniusMorphism;
use irreducible::{
    cap_single, cup_single, CospanAlgebra, CospanToFrobeniusFunctor, HypergraphCategory,
    HypergraphFunctor, PartitionAlgebra, RelabelingFunctor,
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
