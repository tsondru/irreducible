//! v0.6.3 shim contract — confirm the two deprecated aliases compile,
//! resolve to the renamed types/traits, and round-trip through the
//! catgraph_physics implementation.
//!
//! v0.7.0 deletes this file alongside the aliases themselves.

#![allow(deprecated)]

use irreducible::trace::{IrreducibilityTrace, StepTrace};

/// `IrreducibilityTrace` is now a sub-trait of [`StepTrace`] with a
/// blanket impl, so any [`StepTrace`] implementor automatically satisfies
/// `IrreducibilityTrace` (and vice versa via the blanket). This test
/// confirms the bound-position equivalence at compile time.
#[test]
fn irreducibility_trace_is_blanket_over_step_trace() {
    fn takes_step<T: StepTrace + ?Sized>(_: &T) -> usize {
        1
    }
    fn takes_irr<T: IrreducibilityTrace + ?Sized>(_: &T) -> usize {
        2
    }

    // Use a real implementor — TuringMachine's ExecutionHistory implements
    // StepTrace via the catgraph_physics-side trait surface, and so
    // automatically satisfies IrreducibilityTrace through the blanket.
    let bb = irreducible::TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);

    assert_eq!(takes_step(&history), 1);
    assert_eq!(takes_irr(&history), 2);
}
