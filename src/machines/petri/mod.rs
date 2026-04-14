//! Petri-net computation model for irreducibility analysis.
//!
//! A [`PetriNetMachine`] wraps [`catgraph_applied::petri_net::PetriNet`] and joins
//! TM / CA / multiway as the fourth computation model implementing
//! [`crate::trace::IrreducibilityTrace`]. Markings are the objects of `𝒯`,
//! transition firings are the morphisms, and the reachability graph lifts to the
//! multiway evolution substrate used elsewhere in this crate.
//!
//! ## Four perspectives
//!
//! 1. **Linear trace** — [`PetriNetMachine::run`] picks the smallest-index enabled
//!    transition at each step, producing a deterministic [`PetriExecutionHistory`]
//!    that can be analysed with [`crate::trace::analyze_trace`] like any other
//!    [`IrreducibilityTrace`](crate::trace::IrreducibilityTrace) implementor.
//! 2. **Multiway reachability** — [`run_multiway_reachability`] explores every
//!    enabled firing at every step via
//!    [`catgraph_physics::multiway::run_multiway_bfs`], yielding a
//!    `MultiwayEvolutionGraph<Marking, PetriTransitionRecord>` compatible with
//!    branchial analysis and curvature foliations.
//! 3. **Cospan bridge** — each transition has a canonical cospan representation
//!    via [`PetriNetMachine::transition_as_cospan`]; this is the image under Z'
//!    and lets Petri firings compose with the rest of the categorical
//!    infrastructure (Fong-Spivak, Stokes, etc.).
//! 4. **Persistence** — the underlying `PetriNet<Lambda>` can be stored through
//!    catgraph-surreal's `PetriNetStore` (behind the `persist` feature) for
//!    long-running or distributed reachability analyses.

mod builder;
mod history;
mod machine;
mod multiway;

pub use builder::PetriBuilder;
pub use history::{PetriExecutionHistory, PetriTransitionRecord};
pub use machine::PetriNetMachine;
pub use multiway::run_multiway_reachability;

// Re-export the underlying primitives so consumers don't have to depend on
// `catgraph-applied` directly. `Transition` is aliased to `PetriTransition`
// to avoid collision with `crate::machines::Transition` (the TM direction).
pub use catgraph_applied::petri_net::{Marking, PetriNet, Transition as PetriTransition};
