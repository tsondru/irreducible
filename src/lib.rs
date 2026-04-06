//! # Irreducible
//!
//! A library for computational irreducibility based on Jonathan Gorard's
//! "A Functorial Perspective on (Multi)computational Irreducibility" (arXiv:2301.04690).
//!
//! The central insight is that *computational irreducibility is equivalent to functoriality*
//! of a map Z': 𝒯 → ℬ from a category of computations to a cobordism category.
//!
//! ## Modules
//!
//! - [`types`] - Core type definitions (`ComputationDomain`, `ComputationContext`, `CausalEffect`)
//! - [`categories`] - Category theory abstractions (`DiscreteInterval`, `Complexity`, `ComputationState`)
//! - [`functor`] - The irreducibility functor Z': 𝒯 → ℬ, adjunction, monoidal structure, Stokes
//! - [`machines`] - Computational machines (Turing machines, cellular automata, hypergraph rewriting)
//!
//! ## Example: Analyzing Turing Machine Irreducibility
//!
//! ```rust
//! use irreducible::machines::{TuringMachine, Direction};
//!
//! // The 2-state Busy Beaver is known to be computationally irreducible
//! let bb = TuringMachine::busy_beaver_2_2();
//! let history = bb.run("", 20);
//!
//! let analysis = history.analyze_irreducibility();
//! assert!(analysis.is_irreducible);
//! assert_eq!(analysis.step_count, 6);
//! ```
//!
//! ## Example: Cellular Automaton Irreducibility
//!
//! ```rust
//! use irreducible::machines::ElementaryCA;
//!
//! // Rule 30 is conjectured to be computationally irreducible
//! let ca = ElementaryCA::rule_30(21);
//! let history = ca.run(ca.single_cell_initial(), 20);
//!
//! let analysis = history.analyze_irreducibility();
//! println!("Rule 30 irreducible: {}", analysis.is_irreducible);
//! ```

pub mod categories;
pub mod functor;
pub mod machines;
pub mod types;

#[cfg(test)]
pub mod test_utils;

// Category theory exports
pub use categories::{
    Complexity, ComputationState, DiscreteInterval, ParallelIntervals, StepCount,
};

// Functor exports
pub use functor::IrreducibilityFunctor;

// Adjunction exports
pub use functor::{
    AdjunctionIrreducibility, AdjunctionVerification, ZPrimeAdjunction, ZPrimeOps,
};

// Monoidal functor exports
pub use functor::{DifferentialCoherence, MonoidalFunctorResult, TensorCheck};

// Bifunctor / tensor product exports
pub use functor::{
    tensor_bimap, tensor_first, tensor_second, verify_associativity, verify_symmetry,
    verify_unit_laws, IntervalTransform, TensorProduct,
};

// Stokes integration exports
pub use functor::{ConservationResult, StokesError, StokesIrreducibility, TemporalComplex};

// Type exports
pub use types::{CausalEffect, ComputationContext, ComputationDomain};

// Machine builder exports
pub use machines::BuilderError;

// Turing machine exports
pub use machines::{ExecutionHistory, IrreducibilityAnalysis, TuringMachine};

// Cellular automaton exports (1D)
pub use machines::{CAExecutionHistory, CAIrreducibilityAnalysis, ElementaryCA, Generation};

// Trace analysis exports
pub use machines::{
    analyze_trace, detect_repeats, IrreducibilityTrace, RepeatDetection, TraceAnalysis,
};

// Multiway system exports
pub use machines::multiway::{
    BranchId, MergePoint, MultiwayCycle, MultiwayEdge, MultiwayEdgeKind, MultiwayEvolutionGraph,
    MultiwayNode, MultiwayNodeId, MultiwayStatistics,
    branchial_to_parallel_intervals, extract_branchial_foliation, find_all_merge_points,
    BranchialGraph, BranchialStepStats, BranchialSummary,
    CurvatureFoliation, DiscreteCurvature,
    OllivierFoliation, OllivierRicciCurvature,
    RewriteApplication, RewriteRule, SRSState, StringRewriteSystem,
    NTMBuilder, NTMTransitionData, NondeterministicTM,
};

// Hypergraph rewriting exports
pub use machines::hypergraph::{
    Hyperedge, Hypergraph, HypergraphEvolution, HypergraphNode, HypergraphStep,
    CausalInvarianceResult, WilsonLoop, RewriteSpan,
    HypergraphRewriteGroup, HypergraphLattice, plaquette_action, total_action,
};
