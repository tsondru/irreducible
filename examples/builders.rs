//! Builder patterns for Turing machines and non-deterministic TMs.
//!
//! Demonstrates the fluent builder API for constructing deterministic and
//! non-deterministic Turing machines, running them, and inspecting results.
//!
//! Run: `cargo run --example builders`

use irreducible::machines::{Direction, TuringMachine};
use irreducible::NTMBuilder;

fn main() {
    println!("=== Turing Machine Builder ===\n");
    deterministic_builder();

    println!("\n=== Non-deterministic TM Builder ===\n");
    nondeterministic_builder();
}

/// Build a simple TM using the builder pattern.
fn deterministic_builder() {
    let tm = TuringMachine::builder()
        .states(vec![0, 1, 2])
        .initial_state(0)
        .accept_states(vec![2])
        .blank('_')
        // Scan right over 1s
        .transition(0, '1', 0, '1', Direction::Right)
        // Hit blank -> accept
        .transition(0, '_', 2, '_', Direction::Stay)
        .build();

    let history = tm.run("111", 20);

    println!("  Input:       \"111\"");
    println!("  Steps:       {}", history.step_count());
    println!("  Halted:      {}", history.halted);
    println!("  Irreducible: {}", history.is_irreducible());

    // Built-in busy beaver for comparison
    println!("\n  --- Busy Beaver 2-state ---");
    let bb = TuringMachine::busy_beaver_2_2();
    let bb_history = bb.run("", 20);
    let analysis = bb_history.analyze_irreducibility();
    println!("  Steps:       {}", analysis.step_count);
    println!("  Irreducible: {}", analysis.is_irreducible);
}

/// Build a non-deterministic TM with branching transitions.
fn nondeterministic_builder() {
    let ntm = NTMBuilder::new()
        .states(vec![0, 1, 2])
        .initial_state(0)
        .accept_states(vec![2])
        .blank('_')
        // State 0 reading 'a': non-deterministic -- branch left or right
        .transition(0, 'a', vec![
            (1, 'x', Direction::Right),
            (1, 'y', Direction::Left),
        ])
        // State 1: deterministic transitions to accept
        .deterministic_transition(1, '_', 2, '_', Direction::Stay)
        .deterministic_transition(1, 'a', 1, 'a', Direction::Right)
        .build();

    let evolution = ntm.run_multiway("aa", 5, 50);
    let stats = evolution.statistics();

    println!("  Input:        \"aa\"");
    println!("  Total nodes:  {}", stats.total_nodes);
    println!("  Max branches: {}", stats.max_branches);
    println!("  Merge count:  {}", stats.merge_count);
}
