//! [DEPRECATED v0.6.3] Re-exports of [`catgraph_physics::trace`].
//!
//! Will be removed in irreducible v0.7.0; migrate to
//! `catgraph_physics::trace` at your earliest convenience.

pub use catgraph_physics::trace::{
    analyze_trace, detect_repeats, is_irreducible, RepeatDetection, StepTrace, TraceAnalysis,
};

/// Deprecated alias for [`StepTrace`].
///
/// Renamed at the H.1 port to neutralize the irreducibility-specific
/// framing — the trait captures *structural* concerns (fingerprints,
/// intervals) that any step-based system can implement, regardless of
/// whether it carries a Wolfram-irreducibility judgment. Will be removed
/// in irreducible v0.7.0.
///
/// Implemented as a sub-trait with a blanket impl over [`StepTrace`] so
/// that bound-position uses (`fn f<T: IrreducibilityTrace>(...)`) compile
/// against any [`StepTrace`] implementor while still firing the
/// `#[deprecated]` warning at consumer call sites — `pub use ... as ...`
/// re-exports do not propagate `#[deprecated]` in current rustc.
#[deprecated(
    since = "0.6.3",
    note = "renamed to `StepTrace` in catgraph-physics v0.3.0; will be removed in irreducible v0.7.0"
)]
pub trait IrreducibilityTrace: StepTrace {}

#[allow(deprecated)]
impl<T: StepTrace + ?Sized> IrreducibilityTrace for T {}
