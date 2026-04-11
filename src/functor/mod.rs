//! The irreducibility functor Z': 𝒯 → ℬ.
//!
//! This module implements the core insight from Gorard's paper:
//! computational irreducibility is equivalent to functoriality of the
//! complexity map from computations to cobordisms.
//!
//! ## Key Concepts
//!
//! - **Category 𝒯**: The category of computations (objects are states, morphisms are transitions)
//! - **Category ℬ**: The cobordism category (objects are time steps, morphisms are intervals)
//! - **Functor Z'**: Maps computations to their complexity intervals
//! - **Functoriality**: Z'(g∘f) = Z'(g) ∘ Z'(f) means "no shortcuts exist"
//!
//! A computation is **irreducible** if Z' is functorial (composition-preserving).
//! A computation is **reducible** if there exists a shortcut: Z'(g∘f) ≠ Z'(g) ∘ Z'(f).

pub mod adjunction;
pub mod bifunctor;
pub mod fong_spivak;
pub mod monoidal;
pub mod stokes_integration;

// Re-export monoidal functor types.
//
// The coherence helpers (`verify_*_coherence`, `CoherenceVerification`,
// `DifferentialCoherence`) are `#[deprecated(since = "0.4.1")]` at their
// definition site because they are tautological for the strict SMC
// structure of `ParallelIntervals`. Consumers using these re-exports will
// see the deprecation warnings on call. Real multiway-based coherence
// lands in v0.4.3 (Phase 2.5).
#[allow(deprecated)]
pub use monoidal::{
    verify_associator_coherence, verify_braiding_coherence, verify_left_unitor_coherence,
    verify_right_unitor_coherence, CoherenceVerification, DifferentialCoherence,
    MonoidalFunctorResult, TensorCheck,
};

// Re-export adjunction types
pub use adjunction::{
    AdjunctionIrreducibility, AdjunctionVerification, ZPrimeAdjunction, ZPrimeOps,
};

// Re-export bifunctor types
pub use bifunctor::{
    tensor_bimap, tensor_first, tensor_second, verify_associativity, verify_symmetry,
    verify_unit_laws, IntervalTransform, TensorProduct,
};

// Re-export Stokes integration types
pub use stokes_integration::{
    ConservationResult, StokesError, StokesIrreducibility, TemporalComplex,
};

// Re-export Fong-Spivak types
pub use fong_spivak::{
    cap, cap_single, cap_tensor, compose_names, cospan_to_frobenius, cup, cup_single, cup_tensor,
    name, unname, CospanAlgebra, CospanFrobeniusCheck, CospanToFrobeniusFunctor,
    FrobeniusVerificationResult, HypergraphCategory, HypergraphFunctor, NameAlgebra,
    PartitionAlgebra, RelabelingFunctor, verify_cospan_chain_frobenius,
};

use crate::{
    complexity::{Complexity, StepCount},
    interval::{DiscreteInterval, ParallelIntervals},
};

/// The irreducibility functor Z': 𝒯 → ℬ.
///
/// This functor maps:
/// - Objects (states) in 𝒯 to time steps in ℬ
/// - Morphisms (transitions) in 𝒯 to discrete intervals in ℬ
///
/// The key property is functoriality: a computation is irreducible iff
/// Z' preserves composition.
#[derive(Clone, Debug, Default)]
pub struct IrreducibilityFunctor;

impl IrreducibilityFunctor {
    /// Create a new irreducibility functor.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Map a single computational step to an interval.
    ///
    /// A single step starting at time `source_step` maps to [`source_step`, `source_step` + 1].
    #[must_use]
    pub fn map_step(source_step: usize) -> DiscreteInterval {
        DiscreteInterval::new(source_step, source_step + 1)
    }

    /// Map a computation with known complexity to an interval.
    ///
    /// A computation starting at `source_step` with `complexity` steps
    /// maps to [`source_step`, `source_step` + complexity].
    pub fn map_morphism<C: Complexity>(source_step: usize, complexity: &C) -> DiscreteInterval {
        DiscreteInterval::new(source_step, source_step + complexity.as_steps())
    }

    /// Verify functoriality: Z'(g∘f) = Z'(g) ∘ Z'(f).
    ///
    /// Returns `true` if the composition is irreducible (no shortcuts).
    /// Returns `false` if a shortcut exists (composition can be compressed).
    ///
    /// # Arguments
    /// - `f_interval`: The interval Z'(f) for morphism f
    /// - `g_interval`: The interval Z'(g) for morphism g
    /// - `composed_interval`: The interval Z'(g∘f) for the composed morphism
    ///
    /// # Theory
    ///
    /// If the intervals are contiguous and their composition equals the
    /// composed interval, the computation is irreducible. Otherwise,
    /// there exists a shortcut (reducibility).
    #[must_use]
    pub fn verify_functoriality(
        f_interval: &DiscreteInterval,
        g_interval: &DiscreteInterval,
        composed_interval: &DiscreteInterval,
    ) -> bool {
        // Try to compose f then g (sequential execution)
        if let Some(expected) = (*f_interval).then(*g_interval) {
            expected == *composed_interval
        } else {
            // Intervals not contiguous = reducible (there's a gap/shortcut)
            false
        }
    }

    /// Check if a sequence of steps is irreducible.
    ///
    /// The sequence is irreducible if each consecutive pair of intervals
    /// composes correctly (no gaps, no shortcuts).
    #[must_use]
    pub fn is_sequence_irreducible(intervals: &[DiscreteInterval]) -> bool {
        if intervals.is_empty() {
            return true;
        }

        for window in intervals.windows(2) {
            if !window[0].is_composable_with(&window[1]) {
                return false;
            }
        }
        true
    }

    /// Compute the total interval for a sequence of steps.
    ///
    /// Returns `None` if the sequence is not composable (reducible).
    #[must_use]
    pub fn compose_sequence(intervals: &[DiscreteInterval]) -> Option<DiscreteInterval> {
        if intervals.is_empty() {
            return None;
        }

        let mut result = intervals[0];
        for interval in &intervals[1..] {
            result = result.then(*interval)?;
        }
        Some(result)
    }

    /// Check multicomputational irreducibility for parallel branches.
    ///
    /// In multiway systems, we need to verify functoriality across all branches.
    /// The system is irreducible if:
    /// 1. Each branch is individually irreducible
    /// 2. The tensor structure is preserved
    #[must_use]
    pub fn verify_multiway_functoriality(
        branch_sequences: &[Vec<DiscreteInterval>],
    ) -> MultiwayIrreducibilityResult {
        let mut branch_results = Vec::new();
        let mut all_irreducible = true;

        for (idx, branch) in branch_sequences.iter().enumerate() {
            let is_irreducible = Self::is_sequence_irreducible(branch);
            if !is_irreducible {
                all_irreducible = false;
            }

            let composed = Self::compose_sequence(branch);
            branch_results.push(BranchResult {
                branch_index: idx,
                is_irreducible,
                total_interval: composed,
            });
        }

        MultiwayIrreducibilityResult {
            is_fully_irreducible: all_irreducible,
            branch_results,
        }
    }

    /// Compute complexity ratio: actual / composed.
    ///
    /// A ratio of 1.0 indicates irreducibility.
    /// A ratio < 1.0 indicates the composed path is shorter (shortcut exists).
    /// A ratio > 1.0 indicates overhead in composition.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[must_use]
    pub fn complexity_ratio(
        f_complexity: &StepCount,
        g_complexity: &StepCount,
        composed_complexity: &StepCount,
    ) -> f64 {
        let expected = f_complexity.sequential(g_complexity);
        if expected.as_steps() == 0 {
            1.0
        } else {
            composed_complexity.as_steps() as f64 / expected.as_steps() as f64
        }
    }
}

/// Result of checking multiway irreducibility across parallel branches.
///
/// Each branch is checked independently for interval contiguity;
/// the system is fully irreducible only when every branch passes.
/// [`total_parallel_complexity`](Self::total_parallel_complexity) aggregates
/// the composed intervals into a [`ParallelIntervals`] tensor product.
#[derive(Clone, Debug)]
pub struct MultiwayIrreducibilityResult {
    /// Whether all branches are irreducible
    pub is_fully_irreducible: bool,
    /// Results for each branch
    pub branch_results: Vec<BranchResult>,
}

impl MultiwayIrreducibilityResult {
    /// Get the total complexity across all branches.
    #[must_use]
    pub fn total_parallel_complexity(&self) -> ParallelIntervals {
        let mut result = ParallelIntervals::new();
        for branch in &self.branch_results {
            if let Some(interval) = &branch.total_interval {
                result.add_branch(*interval);
            }
        }
        result
    }

    /// Count reducible branches.
    #[must_use]
    pub fn reducible_branch_count(&self) -> usize {
        self.branch_results.iter().filter(|b| !b.is_irreducible).count()
    }
}

/// Irreducibility verdict for a single branch in a multiway computation.
///
/// If `is_irreducible` is true, the branch's interval sequence under Z'
/// is contiguous and composable into `total_interval`.
#[derive(Clone, Debug)]
pub struct BranchResult {
    /// Index of this branch
    pub branch_index: usize,
    /// Whether this branch is irreducible
    pub is_irreducible: bool,
    /// The composed interval for this branch (if composable)
    pub total_interval: Option<DiscreteInterval>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_step() {
        let interval = IrreducibilityFunctor::map_step(5);
        assert_eq!(interval.start, 5);
        assert_eq!(interval.end, 6);
    }

    #[test]
    fn test_map_morphism() {
        let complexity = StepCount(3);
        let interval = IrreducibilityFunctor::map_morphism(2, &complexity);
        assert_eq!(interval.start, 2);
        assert_eq!(interval.end, 5);
    }

    #[test]
    fn test_verify_functoriality_irreducible() {
        // f: 0→2, g: 2→5, composed: 0→5
        let f_interval = DiscreteInterval::new(0, 2);
        let g_interval = DiscreteInterval::new(2, 5);
        let composed = DiscreteInterval::new(0, 5);

        assert!(IrreducibilityFunctor::verify_functoriality(
            &f_interval,
            &g_interval,
            &composed
        ));
    }

    #[test]
    fn test_verify_functoriality_reducible_shortcut() {
        // f: 0→2, g: 2→5, but composed is shorter: 0→4 (shortcut!)
        let f_interval = DiscreteInterval::new(0, 2);
        let g_interval = DiscreteInterval::new(2, 5);
        let composed = DiscreteInterval::new(0, 4); // shortcut

        assert!(!IrreducibilityFunctor::verify_functoriality(
            &f_interval,
            &g_interval,
            &composed
        ));
    }

    #[test]
    fn test_verify_functoriality_reducible_gap() {
        // f: 0→2, g: 3→5 (gap at 2-3)
        let f_interval = DiscreteInterval::new(0, 2);
        let g_interval = DiscreteInterval::new(3, 5);
        let composed = DiscreteInterval::new(0, 5);

        assert!(!IrreducibilityFunctor::verify_functoriality(
            &f_interval,
            &g_interval,
            &composed
        ));
    }

    #[test]
    fn test_is_sequence_irreducible() {
        let intervals = vec![
            DiscreteInterval::new(0, 2),
            DiscreteInterval::new(2, 4),
            DiscreteInterval::new(4, 7),
        ];
        assert!(IrreducibilityFunctor::is_sequence_irreducible(&intervals));
    }

    #[test]
    fn test_is_sequence_reducible() {
        let intervals = vec![
            DiscreteInterval::new(0, 2),
            DiscreteInterval::new(3, 5), // gap
            DiscreteInterval::new(5, 7),
        ];
        assert!(!IrreducibilityFunctor::is_sequence_irreducible(&intervals));
    }

    #[test]
    fn test_compose_sequence() {
        let intervals = vec![
            DiscreteInterval::new(0, 2),
            DiscreteInterval::new(2, 4),
            DiscreteInterval::new(4, 7),
        ];
        let composed = IrreducibilityFunctor::compose_sequence(&intervals);
        assert!(composed.is_some());
        let composed = composed.unwrap();
        assert_eq!(composed.start, 0);
        assert_eq!(composed.end, 7);
    }

    #[test]
    fn test_complexity_ratio_irreducible() {
        let f = StepCount(3);
        let g = StepCount(5);
        let composed = StepCount(8);
        let ratio = IrreducibilityFunctor::complexity_ratio(&f, &g, &composed);
        assert!((ratio - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_complexity_ratio_shortcut() {
        let f = StepCount(3);
        let g = StepCount(5);
        let composed = StepCount(6); // shortcut: only 6 instead of 8
        let ratio = IrreducibilityFunctor::complexity_ratio(&f, &g, &composed);
        assert!(ratio < 1.0);
        assert!((ratio - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_multiway_irreducibility() {
        let branch1 = vec![
            DiscreteInterval::new(0, 2),
            DiscreteInterval::new(2, 5),
        ];
        let branch2 = vec![
            DiscreteInterval::new(0, 3),
            DiscreteInterval::new(3, 4),
        ];
        let result = IrreducibilityFunctor::verify_multiway_functoriality(&[branch1, branch2]);
        assert!(result.is_fully_irreducible);
        assert_eq!(result.branch_results.len(), 2);
    }

    #[test]
    fn test_multiway_partial_reducibility() {
        let branch1 = vec![
            DiscreteInterval::new(0, 2),
            DiscreteInterval::new(2, 5),
        ];
        let branch2 = vec![
            DiscreteInterval::new(0, 3),
            DiscreteInterval::new(4, 6), // gap!
        ];
        let result = IrreducibilityFunctor::verify_multiway_functoriality(&[branch1, branch2]);
        assert!(!result.is_fully_irreducible);
        assert_eq!(result.reducible_branch_count(), 1);
    }

}
