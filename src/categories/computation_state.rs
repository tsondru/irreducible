//! Computation state representation for category 𝒯.
//!
//! A computation state is an object in the computation category,
//! representing a snapshot at a particular step with associated complexity.

use super::DiscreteInterval;

/// A computation state in category 𝒯.
///
/// Represents an object in the computation category - a snapshot of a
/// computation at a particular step with associated complexity.
///
/// # Mathematical Interpretation
///
/// In Gorard's framework:
/// - Objects in 𝒯 are computation states (TM configurations, CA generations, etc.)
/// - Morphisms in 𝒯 are transitions between states
/// - The functor Z': 𝒯 → ℬ maps states to time steps and transitions to intervals
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComputationState {
    /// The step number in the computation
    pub step: usize,
    /// The complexity (number of steps taken to reach this state)
    pub complexity: usize,
    /// Optional fingerprint for cycle detection
    pub fingerprint: Option<u64>,
}

impl ComputationState {
    /// Create a new computation state.
    #[must_use]
    pub fn new(step: usize, complexity: usize) -> Self {
        Self {
            step,
            complexity,
            fingerprint: None,
        }
    }

    /// Create a computation state with a fingerprint.
    #[must_use]
    pub fn with_fingerprint(step: usize, complexity: usize, fingerprint: u64) -> Self {
        Self {
            step,
            complexity,
            fingerprint: Some(fingerprint),
        }
    }

    /// Create the initial computation state.
    #[must_use]
    pub fn initial() -> Self {
        Self::new(0, 0)
    }

    /// Advance to the next state with one step of complexity.
    #[must_use]
    pub fn next(&self) -> Self {
        Self::new(self.step + 1, self.complexity + 1)
    }

    /// Convert to the corresponding interval in ℬ.
    ///
    /// Maps this computation state to the interval [step, step + complexity].
    #[must_use]
    pub fn to_interval(&self) -> DiscreteInterval {
        DiscreteInterval::new(self.step, self.step + self.complexity.max(1))
    }
}
