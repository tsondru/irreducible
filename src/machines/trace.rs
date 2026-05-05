//! Trace analysis for irreducibility.
//!
//! Core trait and analysis functions re-exported from the top-level
//! [`crate::trace`] module (a v0.6.3 shim into `catgraph_physics::trace`).
//!
//! Internal call sites use the new name [`StepTrace`]; the deprecated
//! alias `IrreducibilityTrace` is also re-exported for backwards
//! compatibility through the v0.6.x cycle.

#[allow(deprecated)]
pub use crate::trace::{
    analyze_trace, detect_repeats, IrreducibilityTrace, RepeatDetection, StepTrace, TraceAnalysis,
};
