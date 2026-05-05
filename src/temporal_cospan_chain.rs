//! [DEPRECATED v0.6.3] Re-exports of [`catgraph_physics::temporal_cospan_chain`].
//!
//! Will be removed in irreducible v0.7.0; migrate to
//! `catgraph_physics::temporal_cospan_chain` at your earliest convenience.

pub use catgraph_physics::temporal_cospan_chain::{
    ConservationResult, TemporalComplex, TemporalComplexError,
};

/// Deprecated alias for [`TemporalComplexError`].
///
/// The `Stokes` lineage does not exist in `catgraph-physics`; the error
/// type was renamed at the H.1 port. Will be removed in irreducible v0.7.0.
///
/// `pub use ... as ...` re-exports do not propagate `#[deprecated]` to
/// consumer call sites in current rustc; using a type alias instead so
/// the deprecation warning actually fires when external code references
/// the old name.
#[deprecated(
    since = "0.6.3",
    note = "renamed to `TemporalComplexError` in catgraph-physics v0.3.0; will be removed in irreducible v0.7.0"
)]
pub type StokesError = TemporalComplexError;
