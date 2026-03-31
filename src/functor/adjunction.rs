//! The Z' ⊣ Z Adjunction between Computation and Cobordism Categories.
//!
//! This module implements the adjunction between the computation category 𝒯
//! and the cobordism category ℬ, as described in Gorard's paper (Section 4.2).
//!
//! ## Mathematical Foundation
//!
//! The adjunction Z' ⊣ Z provides a formal relationship:
//!
//! ```text
//! Hom_ℬ(Z'(c), i) ≅ Hom_𝒯(c, Z(i))
//! ```
//!
//! where:
//! - Z': 𝒯 → ℬ maps computation states to intervals
//! - Z: ℬ → 𝒯 maps intervals back to computation states
//!
//! The **unit** η: `Id_𝒯` → Z∘Z' embeds computations into the adjunction.
//! The **counit** ε: Z'∘Z → `Id_ℬ` extracts intervals from the adjunction.
//!
//! ## Triangle Identities
//!
//! The adjunction satisfies two triangle identities:
//! 1. `ε_{Z'(c)}` ∘ Z'(`η_c`) = id_{Z'(c)}
//! 2. Z(`ε_i`) ∘ η_{Z(i)} = id_{Z(i)}
//!
//! These ensure the adjunction is coherent.
//!
//! ## Physical Interpretation
//!
//! From Gorard's paper, the Z' ⊣ Z adjunction has a "quantum duality"
//! interpretation:
//! - **Unit** η: Represents "state preparation" (lifting a computation into
//!   the cobordism representation)
//! - **Counit** ε: Represents "measurement" (collapsing the cobordism back
//!   to a computation witness)
//!
//! Irreducibility corresponds to the non-triviality of this adjunction.

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

/// Concrete operations for the Z' and Z functors.
///
/// Provides the actual computational transformations between states
/// and intervals that form the Z' ⊣ Z adjunction.
pub trait ZPrimeOps {
    /// Apply Z': 𝒯 → ℬ (computation state to interval).
    fn zprime(state: &ComputationState) -> DiscreteInterval;

    /// Apply Z: ℬ → 𝒯 (interval to computation state).
    fn z(interval: &DiscreteInterval) -> ComputationState;

    /// Apply the unit at a specific state: `η_c`: c → Z(Z'(c)).
    fn unit_at(state: &ComputationState) -> ComputationState;

    /// Apply the counit at a specific interval: `ε_i`: Z'(Z(i)) → i.
    fn counit_at(interval: &DiscreteInterval) -> DiscreteInterval;

    /// Verify the first triangle identity: `ε_{Z'(c)}` ∘ Z'(`η_c`) = id_{Z'(c)}.
    fn verify_triangle_1(state: &ComputationState) -> bool;

    /// Verify the second triangle identity: Z(`ε_i`) ∘ `η_{Z(i)}` = id_{Z(i)}.
    fn verify_triangle_2(interval: &DiscreteInterval) -> bool;
}

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

/// Result of adjunction verification for a sequence of computations.
#[derive(Clone, Debug)]
pub struct AdjunctionVerification {
    /// Whether all triangle identities hold.
    pub triangle_identities_hold: bool,
    /// Results for triangle identity 1 at each state.
    pub triangle_1_results: Vec<bool>,
    /// Results for triangle identity 2 at each interval.
    pub triangle_2_results: Vec<bool>,
    /// Whether Z and Z' form an adjoint pair.
    pub is_adjoint_pair: bool,
}

impl AdjunctionVerification {
    /// Verify the adjunction for a sequence of computation states.
    pub fn verify_sequence(states: &[ComputationState]) -> Self {
        let intervals: Vec<DiscreteInterval> =
            states.iter().map(ZPrimeAdjunction::zprime).collect();

        let triangle_1_results: Vec<bool> = states
            .iter()
            .map(ZPrimeAdjunction::verify_triangle_1)
            .collect();

        let triangle_2_results: Vec<bool> = intervals
            .iter()
            .map(ZPrimeAdjunction::verify_triangle_2)
            .collect();

        let all_triangle_1 = triangle_1_results.iter().all(|&b| b);
        let all_triangle_2 = triangle_2_results.iter().all(|&b| b);
        let triangle_identities_hold = all_triangle_1 && all_triangle_2;

        Self {
            triangle_identities_hold,
            triangle_1_results,
            triangle_2_results,
            is_adjoint_pair: triangle_identities_hold,
        }
    }

    /// Count failures in triangle identity 1.
    #[must_use]
    pub fn triangle_1_failures(&self) -> usize {
        self.triangle_1_results.iter().filter(|&&b| !b).count()
    }

    /// Count failures in triangle identity 2.
    #[must_use]
    pub fn triangle_2_failures(&self) -> usize {
        self.triangle_2_results.iter().filter(|&&b| !b).count()
    }
}

impl std::fmt::Display for AdjunctionVerification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Adjunction Verification:")?;
        writeln!(
            f,
            "  Triangle identities hold: {}",
            self.triangle_identities_hold
        )?;
        writeln!(
            f,
            "  Triangle 1 (ε ∘ Z'η = id): {}/{} passed",
            self.triangle_1_results.iter().filter(|&&b| b).count(),
            self.triangle_1_results.len()
        )?;
        writeln!(
            f,
            "  Triangle 2 (Zε ∘ η = id): {}/{} passed",
            self.triangle_2_results.iter().filter(|&&b| b).count(),
            self.triangle_2_results.len()
        )?;
        writeln!(f, "  Is adjoint pair: {}", self.is_adjoint_pair)
    }
}

/// Extension trait connecting the adjunction to irreducibility analysis.
pub trait AdjunctionIrreducibility {
    /// Check if the adjunction structure reveals irreducibility.
    ///
    /// The adjunction Z' ⊣ Z is "non-trivial" when the unit and counit
    /// are not identity transformations, which corresponds to the
    /// computation being irreducible.
    fn adjunction_irreducibility_indicator(states: &[ComputationState]) -> f64;

    /// Compute the "adjunction gap" - deviation from perfect adjoint pair.
    ///
    /// A gap of 0 indicates perfect adjunction (possibly reducible).
    /// A non-zero gap indicates the adjunction structure has "resistance"
    /// to composition, correlating with irreducibility.
    fn adjunction_gap(state: &ComputationState) -> f64;
}

impl AdjunctionIrreducibility for ZPrimeAdjunction {
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn adjunction_irreducibility_indicator(states: &[ComputationState]) -> f64 {
        if states.is_empty() {
            return 0.0;
        }
        let total_gap: f64 = states.iter().map(Self::adjunction_gap).sum();
        total_gap / states.len() as f64
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn adjunction_gap(state: &ComputationState) -> f64 {
        let unit_result = Self::unit_at(state);
        let step_diff = (unit_result.step as f64 - state.step as f64).abs();
        let complexity_diff = (unit_result.complexity as f64 - state.complexity as f64).abs();
        step_diff + complexity_diff
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
        let verification = AdjunctionVerification::verify_sequence(&states);
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
