//! [DEPRECATED v0.6.3] Re-exports of [`catgraph_physics::trace`].
//!
//! Will be removed in irreducible v0.7.0; migrate to
//! `catgraph_physics::trace` at your earliest convenience.

pub use catgraph_physics::trace::{
    RepeatDetection, StepTrace, TraceAnalysis, analyze_trace, detect_repeats, is_irreducible,
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
/// `#[deprecated]` warning **at the bound site** — `pub use ... as ...`
/// re-exports do not propagate `#[deprecated]` in current rustc.
///
/// Note that the warning fires when the trait *name* appears (a generic
/// bound, a `dyn IrreducibilityTrace`, a path), but **not** at method-call
/// sites: a method call on a `T: IrreducibilityTrace` value resolves
/// through the supertrait to `StepTrace::method` and emits no warning.
/// One deprecation hit per bound is the intended trade-off of the
/// sub-trait pattern over duplicating method signatures.
#[deprecated(
    since = "0.6.3",
    note = "renamed to `StepTrace` in catgraph-physics v0.3.0; will be removed in irreducible v0.7.0"
)]
pub trait IrreducibilityTrace: StepTrace {}

#[allow(deprecated)]
impl<T: StepTrace> IrreducibilityTrace for T {}
