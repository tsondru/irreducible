//! Complexity algebra for measuring computational cost.
//!
//! The [`Complexity`] trait abstracts step counts, and [`StepCount`] provides
//! a concrete wrapper supporting sequential and parallel composition.
//!
//! Types re-exported from [`catgraph::complexity`].

pub use catgraph::complexity::{Complexity, StepCount};
