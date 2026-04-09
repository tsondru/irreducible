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
