//! Computational machines for irreducibility analysis.
//!
//! This module provides implementations of computational models that integrate with
//! the [`crate::functor::IrreducibilityFunctor`] to verify whether computations are irreducible.
//!
//! ## Turing Machines
//!
//! - [`TuringMachine`] - Deterministic Turing machine with tape and head
//! - [`Configuration`] - Instantaneous description (tape + state + head position)
//! - [`Transition`] - Single step of computation
//! - [`ExecutionHistory`] - Complete execution trace
//!
//! ## Cellular Automata (1D)
//!
//! - [`ElementaryCA`] - 1D elementary cellular automaton (Wolfram rules 0-255)
//! - [`Generation`] - A single generation (global state)
//! - [`CATransition`] - Transition between generations
//! - [`CAExecutionHistory`] - Complete evolution trace
//!
//! ## Hypergraph Rewriting (Wolfram Physics)
//!
//! - [`hypergraph::Hypergraph`] - Hypergraph structure (vertices + hyperedges)
//! - [`hypergraph::Hyperedge`] - N-ary edge connecting multiple vertices
//! - [`hypergraph::RewriteRule`] - L → R pattern-based rewrite rules
//! - [`hypergraph::HypergraphEvolution`] - Multiway evolution with causal invariance
//! - [`hypergraph::HypergraphRewriteGroup`] - Gauge group implementation (topology feature)
//! - [`hypergraph::HypergraphLattice`] - D-dimensional lattice for gauge field analysis
//!
//! ## Example: Turing Machine
//!
//! ```rust
//! use irreducible::machines::{TuringMachine, Direction};
//!
//! let tm = TuringMachine::builder()
//!     .states(vec![0, 1])
//!     .initial_state(0)
//!     .accept_states(vec![1])
//!     .blank('_')
//!     .transition(0, '1', 1, '1', Direction::Right)
//!     .transition(0, '_', 1, '_', Direction::Stay)
//!     .build();
//!
//! let history = tm.run("1", 10);
//! assert!(history.is_irreducible());
//! ```
//!
//! ## Example: Cellular Automaton
//!
//! ```rust
//! use irreducible::machines::ElementaryCA;
//!
//! let ca = ElementaryCA::rule_30(21);
//! let initial = ca.single_cell_initial();
//! let history = ca.run(initial, 20);
//!
//! let analysis = history.analyze_irreducibility();
//! println!("Rule 30 irreducible: {}", analysis.is_irreducible);
//! ```

mod cellular_automaton;
mod configuration;
pub mod multiway;
pub mod hypergraph;
mod tape;
pub mod trace;
mod transition;
mod turing;

// Turing machine exports
pub use configuration::Configuration;
pub use tape::{Symbol, Tape};
pub use transition::{Direction, Transition};
pub use turing::{
    ExecutionHistory, IrreducibilityAnalysis, Shortcut, TransitionFn, TuringMachine,
    TuringMachineBuilder,
};

// Cellular automaton exports
pub use cellular_automaton::{
    CACycle, CAExecutionHistory, CAIrreducibilityAnalysis, CATransition, ElementaryCA, Generation,
};

// Trace analysis exports
pub use trace::{
    analyze_trace, detect_repeats, IrreducibilityTrace, RepeatDetection, TraceAnalysis,
};

/// State identifier for Turing machines.
pub type State = u32;
