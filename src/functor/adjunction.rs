//! The Z' ⊣ Z Adjunction between Computation and Cobordism Categories.
//!
//! This module provides the concrete `ZPrimeAdjunction` implementation of the
//! abstract adjunction framework from [`catgraph::adjunction`].
//!
//! See Gorard's paper (Section 4.2) for the mathematical foundation.

pub use catgraph::adjunction::{
    AdjunctionIrreducibility, AdjunctionVerification, ZPrimeOps,
};
use crate::categories::{ComputationState, DiscreteInterval};

/// The Z' ⊣ Z adjunction between computation and cobordism categories.
///
/// # Example
///
/// ```rust
/// use irreducible::functor::{ZPrimeAdjunction, ZPrimeOps};
/// use irreducible::categories::{ComputationState, DiscreteInterval};
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
        let _eta_z_i = Self::unit_at(&z_i);
        let zprime_z_i = Self::zprime(&z_i);
        let epsilon_i = Self::counit_at(&zprime_z_i);
        let z_epsilon_i = Self::z(&epsilon_i);
        z_epsilon_i.step == z_i.step && z_epsilon_i.complexity == z_i.complexity
    }
}

impl AdjunctionIrreducibility for ZPrimeAdjunction {}

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
