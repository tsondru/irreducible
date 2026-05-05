//! v0.6.3 shim contract — confirm the two deprecated aliases compile,
//! resolve to the renamed types/traits, and round-trip through the
//! catgraph_physics implementation.
//!
//! v0.7.0 deletes this file alongside the aliases themselves.

#![allow(deprecated)]

use irreducible::trace::{IrreducibilityTrace, StepTrace};

/// `IrreducibilityTrace` is a sub-trait of [`StepTrace`] (`pub trait
/// IrreducibilityTrace: StepTrace {}`) with a blanket impl `impl<T:
/// StepTrace> IrreducibilityTrace for T`. The blanket gives the
/// `StepTrace ⇒ IrreducibilityTrace` direction; the supertrait bound
/// gives the `IrreducibilityTrace ⇒ StepTrace` direction. This test
/// confirms a real `StepTrace` implementor (`TuringMachine`'s
/// `ExecutionHistory`) satisfies both bounds at compile time.
#[test]
fn irreducibility_trace_is_blanket_over_step_trace() {
    fn takes_step<T: StepTrace>(_: &T) -> usize {
        1
    }
    fn takes_irr<T: IrreducibilityTrace>(_: &T) -> usize {
        2
    }

    // Use a real implementor — TuringMachine's ExecutionHistory implements
    // StepTrace, so it automatically satisfies IrreducibilityTrace through
    // the blanket impl.
    let bb = irreducible::TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);

    assert_eq!(takes_step(&history), 1);
    assert_eq!(takes_irr(&history), 2);
}
