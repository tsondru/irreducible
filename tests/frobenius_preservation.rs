//! Integration tests for Frobenius preservation of Z' (issue #12).
//!
//! Acceptance: at least one TM and one CA test where the
//! unit/counit/multiplication/comultiplication on the source side match
//! those on the target side after applying Z', plus multiway systems with
//! genuine fork/merge events.

use irreducible::machines::multiway::MultiwayEvolutionGraph;
use irreducible::machines::{ElementaryCA, TuringMachine};
use irreducible::trace::StepTrace;
use irreducible::{StringRewriteSystem, verify_frobenius_preservation};

/// Lift a linear execution trace into a (single-track) multiway graph.
fn linear_graph(fingerprints: &[u64]) -> MultiwayEvolutionGraph<u64, ()> {
    let mut graph = MultiwayEvolutionGraph::new();
    let mut node = graph.add_root(fingerprints[0]);
    for &fp in &fingerprints[1..] {
        node = graph.add_sequential_step(node, fp, ());
    }
    graph
}

fn assert_generators_preserved<S: Clone + std::hash::Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
    name: &str,
) {
    let result = verify_frobenius_preservation(graph).expect("verification runs");
    assert!(result.unit_preserved, "{name}: eta not preserved");
    assert!(result.counit_preserved, "{name}: epsilon not preserved");
    assert!(result.multiplication_preserved, "{name}: mu not preserved");
    assert!(
        result.comultiplication_preserved,
        "{name}: delta not preserved"
    );
    assert!(result.all_hold(), "{name}: {result:?}");
}

#[test]
fn turing_machine_execution_preserves_frobenius_structure() {
    let bb = TuringMachine::busy_beaver_2_2();
    let history = bb.run("", 20);
    let graph = linear_graph(&history.state_fingerprints());

    assert_generators_preserved(&graph, "busy beaver 2,2");

    // A single-track trace has exactly one sequential event per step.
    let result = verify_frobenius_preservation(&graph).expect("verification runs");
    for check in &result.per_step {
        assert_eq!(
            check.components_checked, 1,
            "TM trace must be one sequential event per step"
        );
    }
}

#[test]
fn cellular_automaton_execution_preserves_frobenius_structure() {
    let ca = ElementaryCA::rule_30(21);
    let history = ca.run(ca.single_cell_initial(), 20);
    let graph = linear_graph(&history.state_fingerprints());

    assert_generators_preserved(&graph, "rule 30");
}

#[test]
fn forking_srs_preserves_frobenius_structure() {
    let srs = StringRewriteSystem::new(vec![("A", "AB"), ("A", "BA")]);
    let evolution = srs.run_multiway("A", 4, 64);
    assert_generators_preserved(&evolution, "forking SRS");

    // The step-0 fork must appear as a checked event.
    let result = verify_frobenius_preservation(&evolution).expect("verification runs");
    assert!(result.per_step[0].components_checked >= 1);
}

#[test]
fn merging_srs_preserves_frobenius_structure() {
    // Cyclic swap re-converges branches onto shared states (merge events).
    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("BA", "AB")]);
    let evolution = srs.run_multiway("AB", 4, 32);
    assert_generators_preserved(&evolution, "merging SRS");
}

#[test]
fn branching_and_growth_srs_preserves_frobenius_structure() {
    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 4, 64);
    assert_generators_preserved(&evolution, "branch+growth SRS");
}
