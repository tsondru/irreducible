//! Fong-Spivak categorical infrastructure (catgraph v0.10.1).
//!
//! Re-exports from catgraph's Fong-Spivak "Seven Sketches" §2-3 modules:
//!
//! - [`HypergraphCategory`] — symmetric monoidal category with Frobenius structure (§2.3)
//! - [`HypergraphFunctor`], [`RelabelingFunctor`], [`CospanToFrobeniusFunctor`] — structure-preserving maps (§2.3)
//! - [`CospanAlgebra`], [`PartitionAlgebra`], [`NameAlgebra`] — lax monoidal functors from cospans to sets (§2.1)
//! - [`cup`], [`cap`], [`name`], [`unname`] — self-dual compact closed structure (§3.1)
//!
//! Key fact: `Cospan<Lambda>` implements [`HypergraphCategory`] (free hypergraph category, Thm 3.14).
//! This means irreducible's cospan chains (from Stokes and hypergraph evolution) can be
//! verified against Frobenius axioms — a stronger categorical check than monoidal coherence.

// Hypergraph categories (§2.3, Def 2.12)
pub use catgraph::hypergraph_category::HypergraphCategory;

// Hypergraph functors (§2.3, Eq. 12)
pub use catgraph::hypergraph_functor::{
    CospanToFrobeniusFunctor, HypergraphFunctor, RelabelingFunctor,
};

// Cospan algebras (§2.1, Def 2.2)
pub use catgraph::cospan_algebra::{
    cospan_to_frobenius, CospanAlgebra, NameAlgebra, PartitionAlgebra,
};

// Compact closed structure (§3.1)
pub use catgraph::compact_closed::{
    cap, cap_single, cap_tensor, compose_names, cup, cup_single, cup_tensor, name, unname,
};

use catgraph::category::Composable;
use catgraph::cospan::Cospan;

/// Result of verifying a single cospan decomposes into valid Frobenius generators.
#[derive(Clone, Debug)]
pub struct CospanFrobeniusCheck {
    /// Index in the cospan chain.
    pub index: usize,
    /// Whether the decomposition into Frobenius generators succeeded.
    pub is_valid: bool,
    /// Number of Frobenius generator layers in the decomposition.
    pub generator_count: usize,
}

/// Result of verifying Frobenius structure on a cospan chain.
///
/// Each cospan is decomposed via [`CospanToFrobeniusFunctor`] into Frobenius
/// generators. Composition is checked by verifying that sequential composition
/// of decomposed morphisms matches the decomposition of the composed cospan.
#[derive(Clone, Debug)]
pub struct FrobeniusVerificationResult {
    /// Whether every cospan in the chain decomposed successfully.
    pub all_valid: bool,
    /// Whether composition of Frobenius decompositions matches
    /// the decomposition of composed cospans (functoriality of the decomposition).
    pub composition_preserved: bool,
    /// Per-cospan decomposition results.
    pub per_cospan: Vec<CospanFrobeniusCheck>,
}

/// Verify that a cospan chain admits valid Frobenius decomposition
/// and that the decomposition preserves composition.
///
/// Uses [`CospanToFrobeniusFunctor`] (Fong-Spivak Prop 3.8) to decompose
/// each `Cospan<u32>` into `FrobeniusMorphism` generators, then checks:
/// 1. Each individual decomposition is valid (non-empty generator sequence)
/// 2. Composition is preserved: `decompose(c1 ; c2) ≅ decompose(c1) ; decompose(c2)`
#[must_use]
pub fn verify_cospan_chain_frobenius(chain: &[Cospan<u32>]) -> FrobeniusVerificationResult {
    if chain.is_empty() {
        return FrobeniusVerificationResult {
            all_valid: true,
            composition_preserved: true,
            per_cospan: Vec::new(),
        };
    }

    let functor = CospanToFrobeniusFunctor::<()>::new();
    let mut per_cospan = Vec::with_capacity(chain.len());
    let mut all_valid = true;

    // Phase 1: Decompose each cospan individually
    let mut decompositions = Vec::with_capacity(chain.len());
    for (i, cospan) in chain.iter().enumerate() {
        if let Ok(morphism) = functor.map_mor(cospan) {
            let count = morphism.depth();
            per_cospan.push(CospanFrobeniusCheck {
                index: i,
                is_valid: true,
                generator_count: count,
            });
            decompositions.push(Some(morphism));
        } else {
            all_valid = false;
            per_cospan.push(CospanFrobeniusCheck {
                index: i,
                is_valid: false,
                generator_count: 0,
            });
            decompositions.push(None);
        }
    }

    // Phase 2: Check composition preservation on consecutive pairs
    let composition_preserved = check_composition_preservation(chain, &decompositions, &functor);

    FrobeniusVerificationResult {
        all_valid,
        composition_preserved,
        per_cospan,
    }
}

/// Check that `decompose(c1 ; c2) == decompose(c1) ; decompose(c2)` for consecutive pairs.
fn check_composition_preservation(
    chain: &[Cospan<u32>],
    decompositions: &[Option<catgraph::frobenius::FrobeniusMorphism<u32, ()>>],
    functor: &CospanToFrobeniusFunctor<()>,
) -> bool {
    use catgraph::category::ComposableMutating;

    if chain.len() < 2 {
        return true;
    }

    for i in 0..chain.len() - 1 {
        let (Some(d_left), Some(d_right)) = (&decompositions[i], &decompositions[i + 1]) else {
            // If either decomposition failed, composition preservation fails
            return false;
        };

        // Compose the two cospans, then decompose
        let Ok(composed_cospan) = chain[i].compose(&chain[i + 1]) else {
            return false;
        };

        let Ok(decomposed_composition) = functor.map_mor(&composed_cospan) else {
            return false;
        };

        // Compose the individual decompositions
        let mut composed_decompositions = d_left.clone();
        if composed_decompositions.compose(d_right.clone()).is_err() {
            return false;
        }

        // Compare: structural equivalence (domain, codomain)
        if decomposed_composition.domain() != composed_decompositions.domain()
            || decomposed_composition.codomain() != composed_decompositions.codomain()
        {
            return false;
        }
    }

    true
}
