//! Category theory abstractions for computational irreducibility.
//!
//! This module implements the categorical framework from Gorard's paper:
//! - Category 𝒯: computations (states, transitions)
//! - Category ℬ: cobordisms (discrete intervals, step counts)
//! - Complexity algebras for measuring computational cost

mod cobordism;
mod complexity;
mod computation_state;

pub use cobordism::{DiscreteInterval, ParallelIntervals};
pub use complexity::{Complexity, StepCount};
pub use computation_state::ComputationState;
