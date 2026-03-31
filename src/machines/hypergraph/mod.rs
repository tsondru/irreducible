//! Hypergraph rewriting for Wolfram Physics model.
//!
//! This module implements hypergraph rewriting as a computational model,
//! connecting Gorard's irreducibility framework to the Wolfram Physics project.
//!
//! # Key Concepts
//!
//! - **Hypergraph**: Generalization of graphs where edges (hyperedges) can connect
//!   any number of vertices. The fundamental structure in Wolfram Physics.
//!
//! - **Rewrite Rule**: A transformation L → R that replaces a subhypergraph matching
//!   L with the structure R. Rules are represented as spans L ← K → R where K is
//!   the shared "kernel" (vertices preserved across the rewrite).
//!
//! - **Causal Invariance**: A key property where the final result is independent of
//!   the order in which rewrites are applied. Analogous to gauge invariance in physics.
//!
//! - **Wilson Loop**: A closed path in the rewrite history. Holonomy = 1.0 indicates
//!   causal invariance (path-independent evolution).
//!
//! # Connection to Gauge Theory
//!
//! Hypergraph rewrites can be viewed as gauge transformations:
//! - Each rewrite rule is a local gauge transformation
//! - Causal invariance ⟺ gauge invariance
//! - Wilson loops measure holonomy (deviation from flat connection)
//! - Plaquette action provides complexity measure beyond step counting
//!
//! # Example
//!
//! ```rust
//! use irreducible::machines::hypergraph::{Hypergraph, RewriteRule, HypergraphEvolution};
//!
//! // Create initial hypergraph with one hyperedge connecting vertices 0, 1, 2
//! let mut graph = Hypergraph::new();
//! graph.add_hyperedge(vec![0, 1, 2]);
//!
//! // Define rewrite rule: {0, 1, 2} → {0, 1}, {1, 2}
//! let rule = RewriteRule::from_pattern(
//!     vec![vec![0, 1, 2]],           // Left-hand side
//!     vec![vec![0, 1], vec![1, 2]],  // Right-hand side
//! );
//!
//! // Run multiway evolution (explores all possible rule applications)
//! let evolution = HypergraphEvolution::run_multiway(&graph, &[rule], 10, 100);
//!
//! // Check causal invariance via Wilson loops
//! let invariant = evolution.is_causally_invariant();
//! ```
//!
//! # Categorical Bridge (catgraph)
//!
//! The [`catgraph_bridge`] module provides the categorical interpretation
//! of DPO rewriting using catgraph's `Span` and `Cospan` types:
//! - `RewriteRule::to_span()` — rule as categorical span L ← K → R
//! - `HypergraphEvolution::to_cospan_chain()` — evolution as composable cospans

mod hyperedge;
#[allow(clippy::module_inception)]
mod hypergraph;
mod rewrite_rule;
mod evolution;

mod gauge;
pub mod catgraph_bridge;
#[cfg(feature = "persist")]
pub mod persistence;

pub use hyperedge::Hyperedge;
pub use hypergraph::Hypergraph;
pub use rewrite_rule::{RewriteRule, RewriteMatch, RewriteSpan};
pub use evolution::{
    HypergraphEvolution, HypergraphNode, HypergraphStep,
    CausalInvarianceResult, WilsonLoop,
};

pub use gauge::{GaugeGroup, HypergraphRewriteGroup, HypergraphLattice, plaquette_action, total_action};
pub use catgraph_bridge::{
    MultiwayCospan, MultiwayCospanGraph,
    CospanInvarianceResult, CospanMergeDetail,
};
