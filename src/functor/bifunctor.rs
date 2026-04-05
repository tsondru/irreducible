//! Bifunctor and tensor product operations for parallel intervals.
//!
//! Provides the [`TensorProduct`] trait and helper functions (`tensor_bimap`,
//! `tensor_first`, `tensor_second`) for parallel interval composition,
//! plus law verification (`verify_associativity`, `verify_symmetry`,
//! `verify_unit_laws`).
//!
//! Re-exported from [`catgraph::bifunctor`].

pub use catgraph::bifunctor::{
    tensor_bimap, tensor_first, tensor_second, verify_associativity, verify_symmetry,
    verify_unit_laws, IntervalTransform, TensorProduct,
};
