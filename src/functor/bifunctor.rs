//! Bifunctor and tensor product operations for parallel intervals.
//!
//! Types and functions re-exported from catgraph. See [`catgraph::bifunctor`] for details.

pub use catgraph::bifunctor::{
    tensor_bimap, tensor_first, tensor_second, verify_associativity, verify_symmetry,
    verify_unit_laws, IntervalTransform, TensorProduct,
};
