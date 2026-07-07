//! The Z' ⊣ Z Adjunction between Computation and Cobordism Categories.
//!
//! This module provides the concrete `ZPrimeAdjunction` implementation of the
//! abstract adjunction framework in [`crate::adjunction`], plus the
//! compact-closed witness layer ([`CompactClosedWitness`]) tying the triangle
//! identities to the cup/cap pairing of Fong-Spivak §3.1 (Prop 3.2).
//!
//! See Gorard's paper (Section 4.2) for the mathematical foundation.

pub use crate::adjunction::{AdjunctionIrreducibility, AdjunctionVerification, ZPrimeOps};
use crate::functor::fong_spivak::{
    CospanToFrobeniusFunctor, HypergraphCategory, HypergraphFunctor, name, unname,
};
use crate::{computation_state::ComputationState, interval::DiscreteInterval};
use catgraph::category::{Composable, ComposableMutating, HasIdentity};
use catgraph::cospan::Cospan;
use catgraph::errors::CatgraphError;
use catgraph::monoidal::Monoidal;

/// The Z' ⊣ Z adjunction between computation and cobordism categories.
///
/// # Example
///
/// ```rust
/// use irreducible::functor::{ZPrimeAdjunction, ZPrimeOps};
/// use irreducible::computation_state::ComputationState;
/// use irreducible::interval::DiscreteInterval;
///
/// // Map computation to interval (Z')
/// let state = ComputationState::new(0, 5);
/// let interval = ZPrimeAdjunction::zprime(&state);
/// assert_eq!(interval.start, 0);
/// assert_eq!(interval.end, 5);
///
/// // Map interval to computation (Z)
/// let interval = DiscreteInterval::new(3, 8);
/// let state = ZPrimeAdjunction::z(&interval);
/// assert_eq!(state.step, 3);
/// assert_eq!(state.complexity, 5);
/// ```
pub struct ZPrimeAdjunction;

impl ZPrimeOps for ZPrimeAdjunction {
    fn zprime(state: &ComputationState) -> DiscreteInterval {
        state.to_interval()
    }

    fn z(interval: &DiscreteInterval) -> ComputationState {
        ComputationState::new(interval.start, interval.steps())
    }

    fn unit_at(state: &ComputationState) -> ComputationState {
        let interval = Self::zprime(state);
        Self::z(&interval)
    }

    fn counit_at(interval: &DiscreteInterval) -> DiscreteInterval {
        let state = Self::z(interval);
        Self::zprime(&state)
    }

    fn verify_triangle_1(state: &ComputationState) -> bool {
        let zprime_c = Self::zprime(state);
        let eta_c = Self::unit_at(state);
        let zprime_eta_c = Self::zprime(&eta_c);
        let result = Self::counit_at(&zprime_eta_c);
        result == zprime_c
    }

    fn verify_triangle_2(interval: &DiscreteInterval) -> bool {
        let z_i = Self::z(interval);
        let zprime_z_i = Self::zprime(&z_i);
        let epsilon_i = Self::counit_at(&zprime_z_i);
        let z_epsilon_i = Self::z(&epsilon_i);
        z_epsilon_i == z_i
    }
}

impl AdjunctionIrreducibility for ZPrimeAdjunction {}

/// Compact closed witness for the Z' ⊣ Z adjunction (F&S §3.1 + Prop 3.2).
///
/// In a self-dual compact closed category the unit and counit of the
/// adjunction X ⊣ X are the cup/cap pairing, and the triangle identities
/// are exactly the zigzag (snake) identities. B (cobordism intervals in
/// their cospan encoding) is compact closed; this witness ties the
/// semantic triangle checks in [`ZPrimeOps`] to that categorical
/// structure.
///
/// The snake identities are verified *semantically* — cospan composition
/// via pushout, compared by [`Cospan::structurally_equal`]. The Prop 3.2
/// name/unname round-trip on the Frobenius decomposition is verified at
/// the boundary level only: the free hypergraph category carries no
/// diagram normal form upstream, so full string-diagram equality is not
/// decidable there. The [`ZPrimeOps`] triangle checks therefore remain
/// the equality backstop rather than being replaced.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct CompactClosedWitness {
    /// Right snake `(id ⊗ cup) ; (cap ⊗ id) = id` at every boundary label
    /// of Z'(c)'s cospan.
    pub right_snake_holds: bool,
    /// Left snake `(cup ⊗ id) ; (id ⊗ cap) = id` at every boundary label.
    pub left_snake_holds: bool,
    /// Prop 3.2 round-trip: `unname(name(f))` reproduces the boundaries
    /// (domain and codomain) of the Frobenius decomposition of Z'(c)'s
    /// cospan.
    pub name_roundtrip_boundaries_hold: bool,
}

impl CompactClosedWitness {
    /// True when every component witness holds.
    #[must_use]
    pub fn all_hold(&self) -> bool {
        self.right_snake_holds && self.left_snake_holds && self.name_roundtrip_boundaries_hold
    }
}

impl ZPrimeAdjunction {
    /// Z'(c) as a cospan in the temporal-cospan-chain encoding: left
    /// boundary {start}, right boundary {end}, apex {start, end}.
    ///
    /// Matches the per-interval encoding of
    /// `TemporalComplex::to_cospan_chain`, so witnesses computed here are
    /// consistent with the Stokes/cospan-chain perspective.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn zprime_cospan(state: &ComputationState) -> Cospan<u32> {
        let interval = Self::zprime(state);
        Cospan::new(
            vec![0],
            vec![1],
            vec![interval.start as u32, interval.end as u32],
        )
    }

    /// Verify both snake identities for a single boundary label `z`,
    /// returning `(right_snake, left_snake)`.
    fn snake_identities_at(z: u32) -> Result<(bool, bool), CatgraphError> {
        let id: Cospan<u32> = Cospan::identity(&vec![z]);
        let cup = <Cospan<u32> as HypergraphCategory<u32>>::cup(z)?;
        let cap = <Cospan<u32> as HypergraphCategory<u32>>::cap(z)?;

        // Right snake: X ≅ X⊗I → X⊗X⊗X → I⊗X ≅ X.
        let mut first = id.clone();
        first.monoidal(cup.clone());
        let mut second = cap.clone();
        second.monoidal(id.clone());
        let right = first.compose(&second)?;

        // Left snake: X ≅ I⊗X → X⊗X⊗X → X⊗I ≅ X.
        let mut first = cup;
        first.monoidal(id.clone());
        let mut second = id.clone();
        second.monoidal(cap);
        let left = first.compose(&second)?;

        Ok((right.structurally_equal(&id), left.structurally_equal(&id)))
    }

    /// Compute the full compact-closed witness at a state: snake
    /// identities at every boundary label of Z'(c)'s cospan, plus the
    /// Prop 3.2 name/unname boundary round-trip on its Frobenius
    /// decomposition (via [`CospanToFrobeniusFunctor`], Prop 3.8).
    ///
    /// # Errors
    ///
    /// Propagates [`CatgraphError`] from cospan composition or the
    /// Frobenius decomposition; not expected for well-formed states.
    pub fn verify_compact_closed_witness(
        state: &ComputationState,
    ) -> Result<CompactClosedWitness, CatgraphError> {
        let cospan = Self::zprime_cospan(state);

        let mut right_snake_holds = true;
        let mut left_snake_holds = true;
        for &z in cospan.middle() {
            let (right, left) = Self::snake_identities_at(z)?;
            right_snake_holds &= right;
            left_snake_holds &= left;
        }

        let functor = CospanToFrobeniusFunctor::<()>::new();
        let f = functor.map_mor(&cospan)?;
        let named = name(&f)?;
        let unnamed = unname(&named, f.domain().len())?;
        let name_roundtrip_boundaries_hold =
            unnamed.domain() == f.domain() && unnamed.codomain() == f.codomain();

        Ok(CompactClosedWitness {
            right_snake_holds,
            left_snake_holds,
            name_roundtrip_boundaries_hold,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zprime_basic() {
        let state = ComputationState::new(0, 5);
        let interval = ZPrimeAdjunction::zprime(&state);
        assert_eq!(interval.start, 0);
        assert_eq!(interval.end, 5);
    }

    #[test]
    fn test_z_basic() {
        let interval = DiscreteInterval::new(3, 10);
        let state = ZPrimeAdjunction::z(&interval);
        assert_eq!(state.step, 3);
        assert_eq!(state.complexity, 7);
    }

    #[test]
    fn test_z_zprime_roundtrip() {
        let original = ComputationState::new(5, 10);
        let interval = ZPrimeAdjunction::zprime(&original);
        let recovered = ZPrimeAdjunction::z(&interval);
        assert_eq!(recovered.step, original.step);
        assert_eq!(recovered.complexity, original.complexity);
    }

    #[test]
    fn test_zprime_z_roundtrip() {
        let original = DiscreteInterval::new(2, 8);
        let state = ZPrimeAdjunction::z(&original);
        let recovered = ZPrimeAdjunction::zprime(&state);
        assert_eq!(recovered.start, original.start);
        assert_eq!(recovered.end, original.end);
    }

    #[test]
    fn test_unit_at() {
        let state = ComputationState::new(0, 5);
        let unit_result = ZPrimeAdjunction::unit_at(&state);
        assert_eq!(unit_result.step, state.step);
        assert_eq!(unit_result.complexity, state.complexity);
    }

    #[test]
    fn test_counit_at() {
        let interval = DiscreteInterval::new(3, 8);
        let counit_result = ZPrimeAdjunction::counit_at(&interval);
        assert_eq!(counit_result.start, interval.start);
        assert_eq!(counit_result.end, interval.end);
    }

    #[test]
    fn test_triangle_identity_1() {
        let state = ComputationState::new(0, 10);
        assert!(ZPrimeAdjunction::verify_triangle_1(&state));
    }

    #[test]
    fn test_triangle_identity_2() {
        let interval = DiscreteInterval::new(0, 10);
        assert!(ZPrimeAdjunction::verify_triangle_2(&interval));
    }

    #[test]
    fn test_triangle_identities_multiple_states() {
        let states = vec![
            ComputationState::new(0, 5),
            ComputationState::new(5, 3),
            ComputationState::new(8, 7),
        ];
        for state in &states {
            assert!(
                ZPrimeAdjunction::verify_triangle_1(state),
                "Triangle 1 failed for state {:?}",
                state
            );
        }
    }

    #[test]
    fn test_adjunction_verification() {
        let states = vec![
            ComputationState::new(0, 5),
            ComputationState::new(5, 3),
            ComputationState::new(8, 7),
        ];
        let verification = AdjunctionVerification::verify_sequence::<ZPrimeAdjunction>(&states);
        assert!(verification.triangle_identities_hold);
        assert!(verification.is_adjoint_pair);
        assert_eq!(verification.triangle_1_failures(), 0);
        assert_eq!(verification.triangle_2_failures(), 0);
    }

    #[test]
    fn test_adjunction_gap_zero_for_well_formed() {
        let state = ComputationState::new(0, 5);
        let gap = ZPrimeAdjunction::adjunction_gap(&state);
        assert!(gap.abs() < f64::EPSILON, "Expected zero gap, got {}", gap);
    }

    #[test]
    fn compact_closed_witness_holds_for_basic_state() {
        let state = ComputationState::new(0, 5);
        let witness = ZPrimeAdjunction::verify_compact_closed_witness(&state)
            .expect("witness computation on well-formed state");
        assert!(witness.right_snake_holds, "right snake must hold in Cospan");
        assert!(witness.left_snake_holds, "left snake must hold in Cospan");
        assert!(
            witness.name_roundtrip_boundaries_hold,
            "Prop 3.2 name/unname must preserve boundaries"
        );
        assert!(witness.all_hold());
    }

    #[test]
    fn zprime_cospan_matches_temporal_chain_encoding() {
        let state = ComputationState::new(3, 4);
        let cospan = ZPrimeAdjunction::zprime_cospan(&state);
        assert_eq!(cospan.left_to_middle(), &[0]);
        assert_eq!(cospan.right_to_middle(), &[1]);
        assert_eq!(cospan.middle(), &[3, 7]);
    }

    #[test]
    fn compact_closed_witness_zero_complexity_state() {
        // start == end: apex has duplicate labels; witness must still hold.
        let state = ComputationState::new(4, 0);
        let witness = ZPrimeAdjunction::verify_compact_closed_witness(&state)
            .expect("witness on zero-complexity state");
        assert!(witness.all_hold());
    }

    #[test]
    fn test_adjunction_irreducibility_indicator() {
        let states = vec![
            ComputationState::new(0, 5),
            ComputationState::new(5, 3),
            ComputationState::new(8, 7),
        ];
        let indicator = ZPrimeAdjunction::adjunction_irreducibility_indicator(&states);
        assert!(
            indicator.abs() < f64::EPSILON,
            "Expected zero indicator, got {}",
            indicator
        );
    }
}
