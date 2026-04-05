//! Hypergraph rewriting for Wolfram Physics model.
//!
//! Core types re-exported from [`catgraph::hypergraph`]. This module
//! adds multiway cospan analysis types for evolution graph interpretation.
//!
//! # Key Concepts
//!
//! - **Hypergraph**: Generalization of graphs where edges (hyperedges) can connect
//!   any number of vertices. The fundamental structure in Wolfram Physics.
//!
//! - **Rewrite Rule**: A transformation L -> R that replaces a subhypergraph matching
//!   L with the structure R. Rules are represented as spans L <- K -> R where K is
//!   the shared "kernel" (vertices preserved across the rewrite).
//!
//! - **Causal Invariance**: A key property where the final result is independent of
//!   the order in which rewrites are applied. Analogous to gauge invariance in physics.
//!
//! - **Wilson Loop**: A closed path in the rewrite history. Holonomy = 1.0 indicates
//!   causal invariance (path-independent evolution).
//!
//! # Categorical Bridge (catgraph)
//!
//! The [`catgraph_bridge`] module provides the multiway cospan interpretation
//! using catgraph's `Span` and `Cospan` types. Core conversions (`to_span()`,
//! `to_cospan_chain()`) are native methods on catgraph's hypergraph types.

pub mod catgraph_bridge;
#[cfg(feature = "persist")]
pub mod persistence;

// Re-export core types from catgraph
pub use catgraph::hypergraph::{
    Hyperedge, Hypergraph,
    RewriteRule, RewriteMatch, RewriteSpan,
    HypergraphEvolution, HypergraphNode, HypergraphStep,
    CausalInvarianceResult, WilsonLoop,
    GaugeGroup, HypergraphRewriteGroup, HypergraphLattice,
    plaquette_action, total_action,
};

// Local types (multiway cospan wrappers) and extension trait
pub use catgraph_bridge::{
    MultiwayCospan, MultiwayCospanGraph,
    CospanInvarianceResult, CospanMergeDetail,
    MultiwayCospanExt,
};
