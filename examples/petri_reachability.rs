//! Petri-net reachability demo.
//!
//! Builds a producer/consumer net (3 tokens in R → 3 tokens in D) and shows:
//! - the deterministic linear trace,
//! - `analyze_trace()` verdict,
//! - multiway reachability statistics,
//! - the cospan representation of transition 0.

use irreducible::machines::petri::{
    run_multiway_reachability, Marking, PetriBuilder, PetriNetMachine,
};
use irreducible::trace::analyze_trace;
use rust_decimal::Decimal;

fn main() {
    let machine: PetriNetMachine<char> = PetriBuilder::new()
        .place('R') // ready
        .place('D') // done
        .transition(vec![(0, Decimal::ONE)], vec![(1, Decimal::ONE)])
        .build();

    let initial = Marking::from_vec(vec![(0, Decimal::from(3))]);
    println!("== Linear trace ==");
    let history = machine.run(initial.clone(), 10);
    for (i, record) in history.transitions.iter().enumerate() {
        println!(
            "step {i}: fire t{} — R={} D={}",
            record.transition_idx,
            record.after.get(0),
            record.after.get(1),
        );
    }
    println!("halted: {}", history.halted);

    println!("\n== Trace analysis ==");
    let analysis = analyze_trace(&history);
    println!("{analysis}");

    println!("== Multiway reachability ==");
    let evolution = run_multiway_reachability(&machine, initial, 5, 64);
    let stats = evolution.statistics();
    println!(
        "nodes: {}, max_branches: {}, merge_count: {}",
        stats.total_nodes, stats.max_branches, stats.merge_count,
    );

    println!("\n== Cospan bridge ==");
    let cospan = machine.transition_as_cospan(0);
    println!(
        "t0 cospan: left={:?} middle={:?} right={:?}",
        cospan.left_to_middle(),
        cospan.middle(),
        cospan.right_to_middle(),
    );
}
