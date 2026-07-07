//! Corelations for multiway merge events (F&S 2018 Ex 6.64; issue #14).
//!
//! A [`Corel`] is a jointly-surjective cospan — a partition of its boundary.
//! Multiway step cospans ([`multiway_step_cospans`]) are jointly surjective
//! by construction, so each step lifts to a corelation whose classes are the
//! step's events — and [`step_corels`] additionally **glues
//! fingerprint-coincident states**: the multiway explorer keeps same-state
//! nodes reached along different paths as distinct graph nodes, so merge
//! events exist only at the fingerprint level, and the corelation is exactly
//! the structure that records their identification. A merge therefore
//! appears as one class containing both parent branches; a fork as one class
//! containing both children.
//!
//! [`evolution_corel`] composes the chain into a single corelation from the
//! initial branchial slice to the final one — two positions share a class
//! iff their histories are causally connected through shared or
//! fingerprint-identified events.
//!
//! The raw (merge-blind) event chain remains available via
//! [`multiway_step_cospans`] — it is the substrate of the Frobenius event
//! census (issue #12) and must reflect graph edges only.
//!
//! The hypergraph-evolution counterpart (merge partition over hypergraph
//! *vertex IDs*, cross-run comparison via
//! [`Corel::coarsest_common_refinement`]) lives in
//! [`crate::machines::hypergraph::catgraph_bridge`].

use std::collections::HashMap;
use std::hash::Hash;

pub use catgraph::corel::Corel;

use catgraph::category::Composable;
use catgraph::cospan::Cospan;
use catgraph::errors::CatgraphError;
use catgraph_physics::multiway::{MultiwayEvolutionGraph, extract_branchial_foliation};

use super::interval_algebra::multiway_step_cospans;

/// Lift every multiway step cospan to a corelation, gluing
/// fingerprint-coincident states of the target slice (merge events).
///
/// # Errors
///
/// Returns [`CatgraphError::Corel`] if a quotiented step cospan is not
/// jointly surjective — impossible by construction, so an error indicates a
/// bug in the step-cospan or quotient construction.
pub fn step_corels<S: Clone + Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
) -> Result<Vec<Corel<u32>>, CatgraphError> {
    let foliation = extract_branchial_foliation(graph);
    multiway_step_cospans(graph)
        .into_iter()
        .enumerate()
        .map(|(i, cospan)| {
            let glued = glue_fingerprint_merges(graph, &foliation[i + 1], &cospan);
            Corel::new(glued)
        })
        .collect()
}

/// Compose the whole evolution into one corelation from the initial
/// branchial slice to the final one. Returns `None` for evolutions with
/// fewer than one step.
///
/// # Errors
///
/// Propagates [`CatgraphError`] from corelation composition (pushout).
pub fn evolution_corel<S: Clone + Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
) -> Result<Option<Corel<u32>>, CatgraphError> {
    let mut chain = step_corels(graph)?.into_iter();
    let Some(first) = chain.next() else {
        return Ok(None);
    };
    let mut composite = first;
    for next in chain {
        composite = composite.compose(&next)?;
    }
    Ok(Some(composite))
}

/// Quotient a step cospan's apex by the fingerprint groups of its target
/// slice: right-boundary positions whose nodes carry equal state
/// fingerprints get their apex vertices identified.
fn glue_fingerprint_merges<S: Clone + Hash, T: Clone>(
    graph: &MultiwayEvolutionGraph<S, T>,
    target_slice: &catgraph_physics::multiway::BranchialGraph,
    cospan: &Cospan<u32>,
) -> Cospan<u32> {
    let apex_count = cospan.middle().len();
    let mut parent: Vec<usize> = (0..apex_count).collect();
    fn find(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }

    let mut first_apex_of_fp: HashMap<u64, usize> = HashMap::new();
    for (pos, node_id) in target_slice.nodes.iter().enumerate() {
        let Some(node) = graph.get_node(node_id) else {
            continue;
        };
        let apex = cospan.right_to_middle()[pos];
        if let Some(&seen) = first_apex_of_fp.get(&node.fingerprint) {
            let (a, b) = (find(&mut parent, apex), find(&mut parent, seen));
            if a != b {
                parent[a] = b;
            }
        } else {
            first_apex_of_fp.insert(node.fingerprint, apex);
        }
    }

    // Rebuild with class representatives in first-seen order.
    let mut apex_of_root: HashMap<usize, usize> = HashMap::new();
    let remap = |parent: &mut [usize], old: usize, apex_of_root: &mut HashMap<usize, usize>| {
        let root = find(parent, old);
        let next = apex_of_root.len();
        *apex_of_root.entry(root).or_insert(next)
    };
    let left: Vec<usize> = cospan
        .left_to_middle()
        .iter()
        .map(|&m| remap(&mut parent, m, &mut apex_of_root))
        .collect();
    let right: Vec<usize> = cospan
        .right_to_middle()
        .iter()
        .map(|&m| remap(&mut parent, m, &mut apex_of_root))
        .collect();
    let middle = vec![0u32; apex_of_root.len()];
    Cospan::new(left, right, middle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::multiway::StringRewriteSystem;

    #[test]
    fn step_corels_lift_every_step() {
        let srs = StringRewriteSystem::new(vec![("A", "AB"), ("A", "BA")]);
        let evolution = srs.run_multiway("A", 3, 32);
        let corels = step_corels(&evolution).expect("step cospans are jointly surjective");
        assert_eq!(corels.len(), multiway_step_cospans(&evolution).len());
    }

    #[test]
    fn fork_step_groups_children_in_one_class() {
        // "A" forks to "AB" | "BA": step 0 corelation has one class
        // containing the parent and both children.
        let srs = StringRewriteSystem::new(vec![("A", "AB"), ("A", "BA")]);
        let evolution = srs.run_multiway("A", 1, 8);
        let corels = step_corels(&evolution).expect("corels");
        let first = &corels[0];

        // Flat layout: domain (1 parent) | middle | codomain (2 children).
        let dom_len = first.as_cospan().left_to_middle().len();
        let mid_len = first.as_cospan().middle().len();
        assert_eq!(dom_len, 1);
        let child_a = dom_len + mid_len;
        let child_b = dom_len + mid_len + 1;
        assert!(
            first.merges(child_a, child_b),
            "fork children must share a class"
        );
        assert!(first.merges(0, child_a), "parent and child share the event");
    }

    #[test]
    fn fingerprint_merge_glues_parent_branches() {
        // Diamond: S forks to AB | BA, both rewrite to Z. The two Z nodes
        // are distinct in the graph but fingerprint-equal — the step-1
        // corelation must identify their parent branches.
        let srs =
            StringRewriteSystem::new(vec![("S", "AB"), ("S", "BA"), ("AB", "Z"), ("BA", "Z")]);
        let evolution = srs.run_multiway("S", 2, 16);
        let corels = step_corels(&evolution).expect("corels");
        let merge_step = &corels[1];
        assert_eq!(merge_step.as_cospan().left_to_middle().len(), 2);
        assert!(
            merge_step.merges(0, 1),
            "fingerprint-merged parents must share a class"
        );
    }

    #[test]
    fn evolution_corel_composes_for_linear_trace() {
        let mut graph: MultiwayEvolutionGraph<i32, ()> = MultiwayEvolutionGraph::new();
        let root = graph.add_root(0);
        let a = graph.add_sequential_step(root, 1, ());
        let _ = graph.add_sequential_step(a, 2, ());

        let corel = evolution_corel(&graph)
            .expect("composition succeeds")
            .expect("two steps present");
        // Single track: one initial branch, one final branch, one class.
        assert_eq!(corel.as_cospan().left_to_middle().len(), 1);
        assert_eq!(corel.as_cospan().right_to_middle().len(), 1);
        let dom_mid = 1 + corel.as_cospan().middle().len();
        assert!(corel.merges(0, dom_mid), "root connects to final state");
    }
}
