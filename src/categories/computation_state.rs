//! Computation state representation for category 𝒯.
//!
//! A [`ComputationState`] pairs a step number with a complexity value,
//! serving as an object in the category of computations. Converts to/from
//! [`DiscreteInterval`](super::DiscreteInterval) via the functor Z'.
//!
//! Re-exported from [`catgraph::computation_state`].

pub use catgraph::computation_state::ComputationState;
