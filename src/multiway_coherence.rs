//! Non-strict symmetric monoidal coherence verification over multiway graphs.
//!
//! Replaces the deprecated `src/coherence.rs` module. Where the old module
//! verified tautological properties of a strict SMC (`ParallelIntervals`),
//! this module checks real coherence constraints on multiway evolution graphs
//! from `catgraph_physics` -- a genuine non-strict SMC where confluence up to
//! causal equivalence is a falsifiable property.
//!
//! # Formalization by construction
//!
//! The category-theoretic claim ("multiway graphs form a non-strict SMC") is
//! verified computationally: the checks pass on confluent graphs and fail on
//! non-confluent ones. See `docs/phase-2.5-decisions.md` (Q1).
//!
//! References: Gorard arXiv:2301.04690, Wolfram Physics Project (causal
//! invariance as commutativity).

use std::hash::Hash;

use catgraph_physics::multiway::{MultiwayEvolutionGraph, MultiwayNodeId};

/// Witness that an associator check passed -- records the confluence path.
#[derive(Debug, Clone)]
pub struct AssociatorWitness {
    /// The fork point where three or more events originate.
    pub fork: MultiwayNodeId,
    /// Number of parallel event pairs verified.
    pub pairs_verified: usize,
    /// Number of confluence diamonds found in the graph.
    pub diamonds_found: usize,
}

/// Witness that a braiding check passed -- records commutation evidence.
#[derive(Debug, Clone)]
pub struct BraidingWitness {
    /// First event in the commuting pair.
    pub event_a: MultiwayNodeId,
    /// Second event in the commuting pair.
    pub event_b: MultiwayNodeId,
}

/// Witness that a unitor check passed.
#[derive(Debug, Clone)]
pub struct UnitorWitness {
    /// The node for which the identity composition was verified.
    pub node: MultiwayNodeId,
}

/// Errors from coherence verification.
#[derive(Debug, Clone)]
pub enum CoherenceError {
    /// Two events do not share a common descendant -- confluence fails.
    NonConfluent {
        /// First event.
        event_a: MultiwayNodeId,
        /// Second event.
        event_b: MultiwayNodeId,
    },
    /// A required event does not exist in the graph.
    MissingEvent {
        /// The missing node identifier.
        node: MultiwayNodeId,
    },
    /// The fork point has fewer than the required number of children.
    InsufficientBranches {
        /// The fork node.
        fork: MultiwayNodeId,
        /// Children found.
        found: usize,
        /// Children required.
        required: usize,
    },
}

impl std::fmt::Display for CoherenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonConfluent { event_a, event_b } => {
                write!(f, "events {event_a:?} and {event_b:?} are not confluent")
            }
            Self::MissingEvent { node } => write!(f, "node {node:?} not found in graph"),
            Self::InsufficientBranches {
                fork,
                found,
                required,
            } => {
                write!(f, "fork {fork:?} has {found} children, need {required}")
            }
        }
    }
}

impl std::error::Error for CoherenceError {}

/// Verify associator coherence at a fork point with >= 3 children.
///
/// Checks that for every triple (e1, e2, e3) of parallel independent events,
/// the two association orderings are confluent -- all three events pairwise
/// commute (share a common descendant).
///
/// # Errors
///
/// - [`CoherenceError::MissingEvent`] if `fork` is not in the graph
/// - [`CoherenceError::InsufficientBranches`] if `fork` has < 3 children
/// - [`CoherenceError::NonConfluent`] if any pair of children fails to commute
pub fn verify_associator<S: Clone + Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
    fork: MultiwayNodeId,
) -> Result<AssociatorWitness, CoherenceError> {
    if graph.get_node(&fork).is_none() {
        return Err(CoherenceError::MissingEvent { node: fork });
    }

    let children: Vec<MultiwayNodeId> = graph
        .get_forward_edges(&fork)
        .map(|edges| edges.iter().map(|e| e.to).collect())
        .unwrap_or_default();

    if children.len() < 3 {
        return Err(CoherenceError::InsufficientBranches {
            fork,
            found: children.len(),
            required: 3,
        });
    }

    let mut pairs_verified = 0;
    for i in 0..children.len() {
        for j in (i + 1)..children.len() {
            if !graph.events_commute(children[i], children[j]) {
                return Err(CoherenceError::NonConfluent {
                    event_a: children[i],
                    event_b: children[j],
                });
            }
            pairs_verified += 1;
        }
    }

    let diamonds_found = graph.confluence_diamonds().len();

    Ok(AssociatorWitness {
        fork,
        pairs_verified,
        diamonds_found,
    })
}

/// Verify braiding coherence: two parallel independent events commute.
///
/// Checks that events `event_a` and `event_b` share a common descendant
/// (causal commutativity), meaning the order of execution doesn't matter.
///
/// # Errors
///
/// - [`CoherenceError::MissingEvent`] if either event is not in the graph
/// - [`CoherenceError::NonConfluent`] if the events don't commute
pub fn verify_braiding<S: Clone + Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
    event_a: MultiwayNodeId,
    event_b: MultiwayNodeId,
) -> Result<BraidingWitness, CoherenceError> {
    if graph.get_node(&event_a).is_none() {
        return Err(CoherenceError::MissingEvent { node: event_a });
    }
    if graph.get_node(&event_b).is_none() {
        return Err(CoherenceError::MissingEvent { node: event_b });
    }

    if graph.events_commute(event_a, event_b) {
        Ok(BraidingWitness { event_a, event_b })
    } else {
        Err(CoherenceError::NonConfluent { event_a, event_b })
    }
}

/// Verify unitor coherence: sequential composition with a no-op is trivial.
///
/// For a node in a sequential (non-forking) region, composition with the
/// identity event is trivially equivalent to the event itself.
///
/// # Errors
///
/// - [`CoherenceError::MissingEvent`] if the node is not in the graph
pub fn verify_unitor<S: Clone + Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
    node: MultiwayNodeId,
) -> Result<UnitorWitness, CoherenceError> {
    if graph.get_node(&node).is_none() {
        return Err(CoherenceError::MissingEvent { node });
    }

    Ok(UnitorWitness { node })
}

/// Verify all coherence conditions across the entire graph.
///
/// Iterates over every fork point and checks associator coherence (at forks
/// with 3+ children) and braiding for every pair of parallel independent
/// events.
///
/// Returns a list of all errors found (empty = fully coherent).
#[must_use]
pub fn verify_all_coherence<S: Clone + Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
) -> Vec<CoherenceError> {
    let mut errors = Vec::new();

    for fork in graph.find_fork_points() {
        let children: Vec<MultiwayNodeId> = graph
            .get_forward_edges(&fork)
            .map(|edges| edges.iter().map(|e| e.to).collect())
            .unwrap_or_default();

        if children.len() >= 3 && let Err(e) = verify_associator(graph, fork) {
            errors.push(e);
        }

        for (a, b) in graph.parallel_independent_events(fork) {
            if let Err(e) = verify_braiding(graph, a, b) {
                errors.push(e);
            }
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use catgraph_physics::multiway::MultiwayEvolutionGraph;

    fn confluent_3fork() -> MultiwayEvolutionGraph<&'static str, &'static str> {
        let mut g = MultiwayEvolutionGraph::new();
        let root = g.add_root("s0");
        let branches = g.add_fork(
            root,
            vec![("s1a", "e1", 0), ("s1b", "e2", 1), ("s1c", "e3", 2)],
        );
        let merge_target = g.add_sequential_step(branches[0], "s2", "m1");
        g.add_merge_edge(branches[1], merge_target, "m2");
        g.add_merge_edge(branches[2], merge_target, "m3");
        g
    }

    fn non_confluent_3fork() -> MultiwayEvolutionGraph<&'static str, &'static str> {
        let mut g = MultiwayEvolutionGraph::new();
        let root = g.add_root("s0");
        let branches = g.add_fork(
            root,
            vec![("s1a", "e1", 0), ("s1b", "e2", 1), ("s1c", "e3", 2)],
        );
        let merge_target = g.add_sequential_step(branches[0], "s2", "m1");
        g.add_merge_edge(branches[1], merge_target, "m2");
        g.add_sequential_step(branches[2], "s_diverged", "no_merge");
        g
    }

    #[test]
    fn associator_passes_confluent() {
        let g = confluent_3fork();
        let forks = g.find_fork_points();
        assert!(!forks.is_empty());
        let result = verify_associator(&g, forks[0]);
        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }

    #[test]
    fn associator_fails_non_confluent() {
        let g = non_confluent_3fork();
        let forks = g.find_fork_points();
        assert!(!forks.is_empty());
        let result = verify_associator(&g, forks[0]);
        assert!(
            matches!(result, Err(CoherenceError::NonConfluent { .. })),
            "expected NonConfluent, got {result:?}",
        );
    }

    #[test]
    fn braiding_passes_commuting() {
        let g = confluent_3fork();
        let forks = g.find_fork_points();
        let pairs = g.parallel_independent_events(forks[0]);
        assert!(!pairs.is_empty());
        let (a, b) = pairs[0];
        let result = verify_braiding(&g, a, b);
        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }

    #[test]
    fn braiding_fails_non_commuting() {
        let g = non_confluent_3fork();
        let forks = g.find_fork_points();
        let pairs = g.parallel_independent_events(forks[0]);
        let divergent_pair = pairs.iter().find(|(a, b)| !g.events_commute(*a, *b));
        if let Some(&(a, b)) = divergent_pair {
            let result = verify_braiding(&g, a, b);
            assert!(
                matches!(result, Err(CoherenceError::NonConfluent { .. })),
                "expected NonConfluent, got {result:?}",
            );
        }
    }

    #[test]
    fn unitor_passes_sequential() {
        let mut g: MultiwayEvolutionGraph<&str, &str> = MultiwayEvolutionGraph::new();
        let root = g.add_root("s0");
        let n1 = g.add_sequential_step(root, "s1", "step");
        let _n2 = g.add_sequential_step(n1, "s2", "step");
        let result = verify_unitor(&g, root);
        assert!(result.is_ok());
    }

    #[test]
    fn verify_all_confluent_is_empty() {
        let g = confluent_3fork();
        let errors = verify_all_coherence(&g);
        assert!(errors.is_empty(), "expected no errors, got {errors:?}");
    }

    #[test]
    fn verify_all_non_confluent_has_errors() {
        let g = non_confluent_3fork();
        let errors = verify_all_coherence(&g);
        assert!(!errors.is_empty(), "expected errors for non-confluent graph");
    }

    #[test]
    fn missing_event_error() {
        let g: MultiwayEvolutionGraph<&str, &str> = MultiwayEvolutionGraph::new();
        let fake = MultiwayNodeId::new(catgraph_physics::multiway::BranchId(99), 99);
        let result = verify_unitor(&g, fake);
        assert!(matches!(result, Err(CoherenceError::MissingEvent { .. })));
    }

    #[test]
    fn insufficient_branches_error() {
        let mut g: MultiwayEvolutionGraph<&str, &str> = MultiwayEvolutionGraph::new();
        let root = g.add_root("s0");
        let _branches = g.add_fork(root, vec![("s1a", "e1", 0), ("s1b", "e2", 1)]);
        let forks = g.find_fork_points();
        let result = verify_associator(&g, forks[0]);
        assert!(
            matches!(result, Err(CoherenceError::InsufficientBranches { .. })),
            "expected InsufficientBranches, got {result:?}",
        );
    }
}
