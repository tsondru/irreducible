//! Multiway (non-deterministic) computation systems.
//!
//! This module implements systems that exhibit branching behavior,
//! where multiple evolution paths exist simultaneously. This is essential
//! for analyzing **multicomputational irreducibility** per Gorard's paper.
//!
//! ## Key Concepts
//!
//! - **Multiway Evolution Graph**: Core data structure capturing branching computation
//! - **Branchial Graphs**: Tensor product structure at each time step (Σ`_t)`
//! - **Non-deterministic TM**: Turing machine with multiple transitions per state
//! - **String Rewriting System**: Simpler model for multiway evolution
//!
//! ## Symmetric Monoidal Category Structure
//!
//! The category 𝒯 of computations becomes a symmetric monoidal category ⟨𝒯, ⊗, I⟩:
//! - **Objects**: States/configurations
//! - **Morphisms**: Transitions (including non-deterministic branches)
//! - **Tensor product ⊗**: Parallel composition (branches at the same time)
//! - **Unit I**: HALT state
//!
//! Multicomputational irreducibility is characterized by Z': 𝒯 → ℬ being
//! a **symmetric monoidal functor**: preserving both ∘ (sequential) and ⊗ (parallel).
//!
//! ## Example
//!
//! ```rust
//! use irreducible::StringRewriteSystem;
//!
//! // Create a simple string rewriting system
//! let srs = StringRewriteSystem::new(vec![
//!     ("AB", "BA"),  // Swap AB to BA
//!     ("A", "AA"),   // Duplicate A
//! ]);
//!
//! // Run multiway evolution
//! let evolution = srs.run_multiway("AB", 5, 100);
//!
//! // Analyze branchial structure
//! let stats = evolution.statistics();
//! println!("Branches: {}, Merges: {}", stats.max_branches, stats.merge_count);
//! ```

mod branchial;
pub mod curvature;
mod evolution_graph;
mod ntm;
pub mod ollivier_ricci;
mod string_rewrite;
mod wasserstein;

// Core graph structures
pub use evolution_graph::{
    run_multiway_bfs, BranchId, MergePoint, MultiwayCycle, MultiwayEdge, MultiwayEdgeKind,
    MultiwayEvolutionGraph, MultiwayNode, MultiwayNodeId, MultiwayStatistics,
};

// Branchial analysis
pub use branchial::{
    branchial_to_parallel_intervals, extract_branchial_foliation, find_all_merge_points,
    BranchialGraph, BranchialStepStats, BranchialSummary,
};

// String Rewriting System
pub use string_rewrite::{RewriteApplication, RewriteRule, SRSState, StringRewriteSystem};

// Non-deterministic Turing Machine
pub use ntm::{NTMBuilder, NTMTransitionData, NondeterministicTM};

// Discrete curvature trait and backends
pub use curvature::{CurvatureFoliation, DiscreteCurvature};
pub use ollivier_ricci::{OllivierFoliation, OllivierRicciCurvature};
