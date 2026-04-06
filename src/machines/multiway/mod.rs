//! Multiway (non-deterministic) computation systems.
//!
//! Generic multiway infrastructure re-exported from [`catgraph::multiway`].
//! Domain-specific computation models (SRS, NTM) and the manifold curvature
//! bridge remain local to this crate.
//!
//! ## Key Concepts
//!
//! - **Multiway Evolution Graph**: Core data structure capturing branching computation
//! - **Branchial Graphs**: Tensor product structure at each time step (`Sigma_t`)
//! - **Non-deterministic TM**: Turing machine with multiple transitions per state
//! - **String Rewriting System**: Simpler model for multiway evolution

#[cfg(feature = "manifold-curvature")]
pub mod manifold_bridge;
mod ntm;
mod string_rewrite;

// Re-export generic infrastructure from catgraph
pub use catgraph::multiway::{
    run_multiway_bfs, BranchId, MergePoint, MultiwayCycle, MultiwayEdge, MultiwayEdgeKind,
    MultiwayEvolutionGraph, MultiwayNode, MultiwayNodeId, MultiwayStatistics,
    branchial_to_parallel_intervals, extract_branchial_foliation, find_all_merge_points,
    BranchialGraph, BranchialStepStats, BranchialSummary,
    CurvatureFoliation, DiscreteCurvature,
    OllivierFoliation, OllivierRicciCurvature,
    wasserstein_1,
};

// Local computation models
pub use string_rewrite::{RewriteApplication, SrsRewriteRule, SRSState, StringRewriteSystem};
pub use ntm::{NTMBuilder, NTMTransitionData, NondeterministicTM};

// Feature-gated manifold curvature
#[cfg(feature = "manifold-curvature")]
pub use manifold_bridge::{
    BranchialEmbedding, ManifoldCurvature, ManifoldFoliation, ShortestPathMDS,
};
