//! Integration tests for the Z' cospan-algebra (issue #11).
//!
//! Acceptance: the `CospanAlgebra`-delegated monoidal verification must agree
//! with the pre-refactor ad-hoc computation on every fixture that
//! `functor::monoidal` exercises. The reference implementation below is the
//! pre-refactor logic, kept verbatim so agreement is checked against the old
//! semantics rather than against the new code's own output.

use std::hash::Hash;

use irreducible::machines::multiway::{MultiwayEvolutionGraph, StringRewriteSystem};
use irreducible::{DiscreteInterval, IrreducibilityFunctor, ParallelIntervals};

use catgraph_physics::multiway::extract_branchial_foliation;

// ---------------------------------------------------------------------------
// Reference: the pre-refactor ad-hoc tensor computation (verbatim semantics)
// ---------------------------------------------------------------------------

struct ReferenceCheck {
    step: usize,
    expected: ParallelIntervals,
    actual: ParallelIntervals,
    preserves: bool,
}

fn reference_tensor_checks<S: Clone + Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
) -> Vec<ReferenceCheck> {
    let foliation = extract_branchial_foliation(graph);
    let mut checks = Vec::new();

    // Same allow as the pre-refactor code carried — the loop is verbatim.
    #[allow(clippy::needless_range_loop)]
    for i in 0..foliation.len().saturating_sub(1) {
        let branchial = &foliation[i];

        let mut expected = ParallelIntervals::new();
        for _ in &branchial.nodes {
            expected.add_branch(DiscreteInterval::new(i, i + 1));
        }

        let mut actual = ParallelIntervals::new();
        for node_id in &branchial.nodes {
            if graph
                .get_forward_edges(node_id)
                .is_some_and(|e| !e.is_empty())
            {
                actual.add_branch(DiscreteInterval::new(i, i + 1));
            }
        }

        let preserves = expected.structurally_equivalent(&actual);
        checks.push(ReferenceCheck {
            step: i,
            expected,
            actual,
            preserves,
        });
    }

    checks
}

fn assert_agreement<S: Clone + Hash, T: Clone>(graph: &MultiwayEvolutionGraph<S, T>, name: &str) {
    let result = IrreducibilityFunctor::verify_symmetric_monoidal_functor(graph);
    let reference = reference_tensor_checks(graph);

    assert_eq!(
        result.tensor_checks.len(),
        reference.len(),
        "{name}: check count diverged"
    );
    for (new, old) in result.tensor_checks.iter().zip(reference.iter()) {
        assert_eq!(new.step, old.step, "{name}: step diverged");
        assert_eq!(
            new.preserves, old.preserves,
            "{name}: preserves diverged at step {}",
            new.step
        );
        assert!(
            new.expected_parallel.exactly_equal(&old.expected),
            "{name}: expected bundle diverged at step {}",
            new.step
        );
        assert!(
            new.actual_parallel.exactly_equal(&old.actual),
            "{name}: actual bundle diverged at step {}",
            new.step
        );
    }

    let reference_preserves_tensor = reference.iter().all(|c| c.preserves);
    assert_eq!(
        result.preserves_tensor, reference_preserves_tensor,
        "{name}: overall preserves_tensor diverged"
    );
}

// ---------------------------------------------------------------------------
// Agreement on every monoidal.rs fixture
// ---------------------------------------------------------------------------

#[test]
fn agreement_swap_system() {
    let srs = StringRewriteSystem::swap_system();
    let evolution = srs.run_multiway("AB", 3, 10);
    assert_agreement(&evolution, "swap_system");
}

#[test]
fn agreement_fibonacci_growth() {
    let srs = StringRewriteSystem::fibonacci_growth();
    let evolution = srs.run_multiway("A", 5, 100);
    assert_agreement(&evolution, "fibonacci_growth");
}

#[test]
fn agreement_branching_system() {
    let srs = StringRewriteSystem::new(vec![("A", "B"), ("A", "C")]);
    let evolution = srs.run_multiway("A", 3, 10);
    assert_agreement(&evolution, "branching A->B|A->C");
}

#[test]
fn agreement_single_branch_graph() {
    let mut graph: MultiwayEvolutionGraph<i32, ()> = MultiwayEvolutionGraph::new();
    let root = graph.add_root(0);
    graph.add_sequential_step(root, 1, ());
    assert_agreement(&graph, "single-branch manual graph");
}

#[test]
fn agreement_wider_srs_sweep() {
    // Beyond the monoidal.rs fixtures: branching + growing + cyclic systems.
    for (name, rules, input, steps, max) in [
        ("ab_ba_a_aa", vec![("AB", "BA"), ("A", "AA")], "AB", 4, 64),
        ("cyclic", vec![("AB", "BA"), ("BA", "AB")], "AB", 4, 64),
        ("growth_fork", vec![("A", "AB"), ("A", "BA")], "A", 4, 64),
    ] {
        let srs = StringRewriteSystem::new(rules);
        let evolution = srs.run_multiway(input, steps, max);
        assert_agreement(&evolution, name);
    }
}
