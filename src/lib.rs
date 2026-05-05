//! # Irreducible
//!
//! A library for computational irreducibility based on Jonathan Gorard's
//! "A Functorial Perspective on (Multi)computational Irreducibility" (arXiv:2301.04690).
//!
//! The central insight is that *computational irreducibility is equivalent to functoriality*
//! of a map Z': T -> B from a category of computations to a cobordism category.
//!
//! ## Modules
//!
//! - [`types`] - Core type definitions (`ComputationDomain`, `ComputationContext`, `CausalEffect`)
//! - Category theory types (`Complexity`, `ComputationState`) — local; (`DiscreteInterval`,
//!   `ParallelIntervals`, `TemporalComplex`, `StepTrace`) — re-exported from `catgraph_physics`
//!   via the local `crate::interval` / `crate::temporal_cospan_chain` / `crate::trace` shim
//!   modules introduced in v0.6.3 (shim modules + deprecated aliases removed in v0.7.0).
//! - [`functor`] - The irreducibility functor Z': T -> B, adjunction, monoidal structure
//! - [`machines`] - Computational machines (Turing machines, cellular automata, hypergraph rewriting)
//! - [`multiway_coherence`] - Non-strict SMC coherence over multiway graphs
//! - [`multiway_stokes`] - Discrete exterior calculus on 2D multiway complexes (feature `dec`)
//! - [`temporal_cospan_chain`] - Cospan chain bridge for interval sequences
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

pub mod functor;
pub mod machines;
pub mod types;

// Category theory primitives — moved into irreducible as of v0.4.0
pub mod adjunction;
pub mod bifunctor;
pub mod complexity;
pub mod computation_state;
pub mod interval;
pub mod trace;

// Phase 2.5 modules — real coherence and exterior calculus on multiway substrate
pub mod multiway_coherence;
pub mod multiway_stokes;
pub mod temporal_cospan_chain;

// Phase 0 of the dc_topology substrate port (feature: dc-geometry).
// Bridges multiway/branchial graphs to deep_causality_topology's simplicial
// complex + Regge geometry. See `.claude/plans/2026-04-14-dc-topology-substrate.md`.
#[cfg(feature = "dc-geometry")]
pub mod geometry;

#[cfg(test)]
pub mod test_utils;

// Category theory exports (now local modules)
pub use interval::{DiscreteInterval, ParallelIntervals};
pub use complexity::{Complexity, StepCount};
pub use computation_state::ComputationState;

// Functor exports
pub use functor::IrreducibilityFunctor;

// Adjunction exports
pub use functor::{
    AdjunctionIrreducibility, AdjunctionVerification, ZPrimeAdjunction, ZPrimeOps,
};

// Monoidal functor exports
pub use functor::{MonoidalFunctorResult, TensorCheck};

// Bifunctor / tensor product exports
pub use functor::{
    tensor_bimap, tensor_first, tensor_second, verify_associativity, verify_symmetry,
    verify_unit_laws, IntervalTransform, TensorProduct,
};

// Temporal cospan chain exports (shim into catgraph_physics::temporal_cospan_chain;
// `StokesError` is the deprecated alias for `TemporalComplexError`, dropped in v0.7.0).
#[allow(deprecated)]
pub use temporal_cospan_chain::{
    ConservationResult, StokesError, TemporalComplex, TemporalComplexError,
};

// Stokes integration exports
pub use functor::StokesIrreducibility;

// Type exports
pub use types::{CausalEffect, ComputationContext, ComputationDomain};

// Machine builder exports
pub use machines::BuilderError;

// Turing machine exports
pub use machines::{ExecutionHistory, IrreducibilityAnalysis, TuringMachine};

// Cellular automaton exports (1D)
pub use machines::{CAExecutionHistory, CAIrreducibilityAnalysis, ElementaryCA, Generation};

// Petri net exports
pub use machines::petri::{
    run_multiway_reachability, Marking, PetriBuilder, PetriExecutionHistory, PetriNet,
    PetriNetMachine, PetriTransition, PetriTransitionRecord,
};

// Trace analysis exports (shim into catgraph_physics::trace;
// `IrreducibilityTrace` is the deprecated alias for `StepTrace`, dropped in v0.7.0).
#[allow(deprecated)]
pub use trace::{
    analyze_trace, detect_repeats, is_irreducible, IrreducibilityTrace, RepeatDetection,
    StepTrace, TraceAnalysis,
};

// Multiway system exports
pub use machines::multiway::{
    BranchId, MergePoint, MultiwayCycle, MultiwayEdge, MultiwayEdgeKind, MultiwayEvolutionGraph,
    MultiwayNode, MultiwayNodeId, MultiwayStatistics,
    branchial_to_parallel_intervals, extract_branchial_foliation, find_all_merge_points,
    BranchialGraph, BranchialStepStats, BranchialSummary,
    CurvatureFoliation, DiscreteCurvature,
    OllivierFoliation, OllivierRicciCurvature,
    RewriteApplication, SrsRewriteRule, SRSState, StringRewriteSystem,
    NTMBuilder, NTMTransitionData, NondeterministicTM,
};

// Hypergraph rewriting exports
pub use machines::hypergraph::{
    Hyperedge, Hypergraph, HypergraphEvolution, HypergraphNode, HypergraphStep,
    CausalInvarianceResult, WilsonLoop, RewriteSpan,
    HypergraphRewriteGroup, HypergraphLattice, plaquette_action, total_action,
};

// Fong-Spivak categorical infrastructure (re-exported from catgraph)
pub use functor::{
    cap, cap_single, cap_tensor, compose_names, cospan_to_frobenius, cup, cup_single, cup_tensor,
    name, unname, CospanAlgebra, CospanFrobeniusCheck, CospanToFrobeniusFunctor,
    FrobeniusVerificationResult, HypergraphCategory, HypergraphFunctor, NameAlgebra,
    PartitionAlgebra, RelabelingFunctor, verify_cospan_chain_frobenius,
};

// Multiway coherence exports
pub use multiway_coherence::{
    verify_all_coherence, verify_associator, verify_braiding, verify_unitor,
    AssociatorWitness, BraidingWitness, CoherenceError, UnitorWitness,
};
