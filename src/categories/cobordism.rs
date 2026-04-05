//! Cobordism category ℬ: discrete intervals and their parallel composition.
//!
//! Objects of ℬ are natural numbers (time steps); morphisms are discrete
//! intervals `[n, m] ∩ ℕ` with contiguous composition. [`ParallelIntervals`]
//! provides the tensor product ⊗ for multicomputational analysis.
//!
//! Types re-exported from [`catgraph::interval`].

pub use catgraph::interval::{DiscreteInterval, ParallelIntervals};
