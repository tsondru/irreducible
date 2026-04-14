//! Bridges between irreducible's multiway/branchial graphs and
//! `deep_causality_topology`'s simplicial complex + Regge geometry substrate.
//!
//! Phase 0 of the `dc_topology` port (`.claude/plans/2026-04-14-dc-topology-substrate.md`).
//!
//! This module is gated behind the `dc-geometry` feature. In Phase 0 it exposes:
//!
//! - [`dc_bridge::branchial_to_simplicial`] — converts a
//!   [`catgraph_physics::multiway::BranchialGraph`] into a 2D
//!   `SimplicialComplex<f64>` containing only 0- and 1-simplices (no faces yet;
//!   face-extraction via confluence diamonds is deferred to Phase 2).
//! - [`regge_metric::flat_regge_geometry`] — constructs a flat baseline
//!   `ReggeGeometry<f64>` with all edge lengths equal to `1.0`.
//!
//! Phases 1 and 2 of the port replace the prior curvature backend (in
//! `machines::multiway::manifold_bridge`) and the hand-rolled DEC (in
//! `multiway_stokes`) with `deep_causality_topology` operations. Phase 0
//! touches neither.

pub mod dc_bridge;
pub mod hodge;
pub mod regge_metric;
