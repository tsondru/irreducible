//! The categorical stack across computation models and evolution edge cases
//! (issue #25).
//!
//! `multiway_step_cospans` / `verify_frobenius_preservation` / `step_corels`
//! are model-agnostic: they consume a `MultiwayEvolutionGraph` whatever
//! produced it. These tests drive NTM and Petri reachability graphs through
//! all three, and cover the zero-step, truncated-evolution and interval
//! transport paths.

use catgraph::category::Composable;
use irreducible::machines::Direction;
use irreducible::machines::multiway::extract_branchial_foliation;
use irreducible::machines::petri::{
    Marking, PetriBuilder, PetriNetMachine, run_multiway_reachability,
};
use irreducible::{
    CospanAlgebra, DiscreteInterval, IntervalCospanAlgebra, NondeterministicTM, ParallelIntervals,
    StringRewriteSystem, evolution_corel, multiway_step_cospans, step_corels,
    verify_frobenius_preservation,
};
use rust_decimal::Decimal;

fn d(n: i64) -> Decimal {
    Decimal::from(n)
}

/// The (parents, children) census of each apex vertex of a step cospan, in
/// sorted order — the event-type multiset of the step.
fn apex_census(cospan: &catgraph::cospan::Cospan<u32>) -> Vec<(usize, usize)> {
    let n = cospan.middle().len();
    let mut lefts = vec![0usize; n];
    let mut rights = vec![0usize; n];
    for &a in cospan.left_to_middle() {
        lefts[a] += 1;
    }
    for &a in cospan.right_to_middle() {
        rights[a] += 1;
    }
    let mut census: Vec<(usize, usize)> = lefts.into_iter().zip(rights).collect();
    census.sort_unstable();
    census
}

// ---------------------------------------------------------------------------
// Gap 1: NTM through the three surfaces
// ---------------------------------------------------------------------------

#[test]
fn ntm_fork_flows_through_cospans_frobenius_and_corels() {
    // One fork at step 0 into two accept states, which halt immediately:
    // a two-slice evolution whose single event is a comultiplication.
    let ntm = NondeterministicTM::builder()
        .states(vec![0, 1, 2])
        .initial_state(0)
        .accept_states(vec![1, 2])
        .blank('_')
        .transition(
            0,
            '_',
            vec![(1, 'X', Direction::Right), (2, 'Y', Direction::Left)],
        )
        .build();
    let evolution = ntm.run_multiway("_", 3, 50);

    assert_eq!(evolution.max_step(), 1, "both branches halt after one step");

    // Step cospans: one step, one apex (the fork), 1 parent / 2 children.
    let chain = multiway_step_cospans(&evolution);
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].left_to_middle(), &[0]);
    assert_eq!(chain[0].right_to_middle(), &[0, 0]);
    assert_eq!(chain[0].middle().len(), 1);
    assert_eq!(apex_census(&chain[0]), vec![(1, 2)]);

    // Frobenius: the fork factors through delta.
    let frobenius = verify_frobenius_preservation(&evolution).expect("verification runs");
    assert!(frobenius.all_hold(), "{frobenius:?}");
    assert_eq!(frobenius.per_step.len(), 1);
    assert_eq!(frobenius.per_step[0].components_checked, 1);

    // Corelation: the two children carry distinct configurations, so nothing
    // is glued; parent and both children share the fork's class.
    let corels = step_corels(&evolution).expect("step cospans are jointly surjective");
    assert_eq!(corels.len(), 1);
    let first = &corels[0];
    let dom_len = first.as_cospan().left_to_middle().len();
    let mid_len = first.as_cospan().middle().len();
    assert_eq!((dom_len, mid_len), (1, 1));
    let child_a = dom_len + mid_len;
    assert!(
        first.merges(child_a, child_a + 1),
        "fork children share a class"
    );
    assert!(first.merges(0, child_a), "parent shares the fork event");
}

// ---------------------------------------------------------------------------
// Gap 2: Petri reachability through the three surfaces
// ---------------------------------------------------------------------------

/// Two independent transitions, `A -> X` and `B -> Y`, both enabled at the
/// root: the reachability graph forks at step 0 and reconverges on
/// `{X:1, Y:1}` at step 2.
fn independent_transitions_net() -> PetriNetMachine<char> {
    PetriBuilder::new()
        .place('A')
        .place('B')
        .place('X')
        .place('Y')
        .transition(vec![(0, Decimal::ONE)], vec![(2, Decimal::ONE)])
        .transition(vec![(1, Decimal::ONE)], vec![(3, Decimal::ONE)])
        .build()
}

#[test]
fn petri_forking_reachability_flows_through_cospans_and_frobenius() {
    let machine = independent_transitions_net();
    let initial = Marking::from_vec(vec![(0, Decimal::ONE), (1, Decimal::ONE)]);
    let evolution = run_multiway_reachability(&machine, initial, 3, 16);

    assert_eq!(evolution.max_step(), 2, "both firing orders take two steps");

    let chain = multiway_step_cospans(&evolution);
    assert_eq!(chain.len(), 2);
    // Step 0: one marking forks into two.
    assert_eq!(chain[0].left_to_middle(), &[0]);
    assert_eq!(chain[0].right_to_middle(), &[0, 0]);
    assert_eq!(apex_census(&chain[0]), vec![(1, 2)]);
    // Step 1: each branch advances sequentially — two separate events. The
    // raw chain is merge-blind, so the reconvergence is not visible here.
    assert_eq!(chain[1].left_to_middle(), &[0, 1]);
    assert_eq!(chain[1].right_to_middle(), &[0, 1]);
    assert_eq!(apex_census(&chain[1]), vec![(1, 1), (1, 1)]);

    let frobenius = verify_frobenius_preservation(&evolution).expect("verification runs");
    assert!(frobenius.all_hold(), "{frobenius:?}");
    let checked: Vec<usize> = frobenius
        .per_step
        .iter()
        .map(|s| s.components_checked)
        .collect();
    assert_eq!(checked, vec![1, 2]);
}

#[test]
fn petri_confluent_reachability_merges_in_step_corel() {
    let machine = independent_transitions_net();
    let initial = Marking::from_vec(vec![(0, Decimal::ONE), (1, Decimal::ONE)]);
    let evolution = run_multiway_reachability(&machine, initial, 3, 16);

    // Fixture sanity: both step-2 markings are `{X:1, Y:1}`, kept as distinct
    // graph nodes but fingerprint-equal.
    let final_slice = &extract_branchial_foliation(&evolution)[2];
    let fingerprints: Vec<u64> = final_slice
        .nodes
        .iter()
        .filter_map(|id| evolution.get_node(id).map(|n| n.fingerprint))
        .collect();
    assert_eq!(fingerprints.len(), 2);
    assert_eq!(
        fingerprints[0], fingerprints[1],
        "the two firing orders must reconverge"
    );

    let corels = step_corels(&evolution).expect("step cospans are jointly surjective");
    assert_eq!(corels.len(), 2);
    let merge_step = &corels[1];
    assert_eq!(merge_step.as_cospan().left_to_middle().len(), 2);
    assert_eq!(
        merge_step.as_cospan().middle().len(),
        1,
        "the fingerprint quotient collapses both events into one class"
    );
    assert!(
        merge_step.merges(0, 1),
        "the two parent markings reach one state"
    );
}

#[test]
fn petri_linear_reachability_is_a_single_track_chain() {
    // `R -> D`, three tokens: one enabled transition per marking.
    let machine: PetriNetMachine<char> = PetriBuilder::new()
        .place('R')
        .place('D')
        .transition(vec![(0, Decimal::ONE)], vec![(1, Decimal::ONE)])
        .build();
    let evolution = run_multiway_reachability(&machine, Marking::from_vec(vec![(0, d(3))]), 10, 16);

    assert_eq!(evolution.max_step(), 3, "three tokens, three firings");

    let chain = multiway_step_cospans(&evolution);
    assert_eq!(chain.len(), 3);
    for (i, c) in chain.iter().enumerate() {
        assert_eq!(c.left_to_middle(), &[0], "step {i}");
        assert_eq!(c.right_to_middle(), &[0], "step {i}");
        assert_eq!(apex_census(c), vec![(1, 1)], "step {i}");
    }

    let frobenius = verify_frobenius_preservation(&evolution).expect("verification runs");
    assert!(frobenius.all_hold(), "{frobenius:?}");
    let checked: Vec<usize> = frobenius
        .per_step
        .iter()
        .map(|s| s.components_checked)
        .collect();
    assert_eq!(checked, vec![1, 1, 1]);

    let corels = step_corels(&evolution).expect("step cospans are jointly surjective");
    assert_eq!(corels.len(), 3);
    for (i, c) in corels.iter().enumerate() {
        assert_eq!(c.as_cospan().middle().len(), 1, "step {i} is one event");
    }
}

// ---------------------------------------------------------------------------
// Gap 3: zero-step evolution
// ---------------------------------------------------------------------------

#[test]
fn zero_step_evolution_has_no_cospans_and_no_evolution_corel() {
    // A net whose only transition needs a token the empty marking lacks.
    let machine: PetriNetMachine<char> = PetriBuilder::new()
        .place('A')
        .transition(vec![(0, Decimal::ONE)], vec![])
        .build();
    let evolution = run_multiway_reachability(&machine, Marking::new(), 5, 16);

    assert_eq!(evolution.node_count(), 1, "only the root marking");
    assert_eq!(evolution.max_step(), 0);
    assert_eq!(extract_branchial_foliation(&evolution).len(), 1);

    assert!(multiway_step_cospans(&evolution).is_empty());
    assert!(
        step_corels(&evolution)
            .expect("no steps to lift")
            .is_empty()
    );
    assert!(
        evolution_corel(&evolution)
            .expect("composition succeeds")
            .is_none(),
        "fewer than one step has no composite corelation"
    );

    // Generator equations do not depend on the evolution, so they still hold
    // with an empty per-step census.
    let frobenius = verify_frobenius_preservation(&evolution).expect("verification runs");
    assert!(frobenius.per_step.is_empty());
    assert!(frobenius.all_hold(), "{frobenius:?}");
}

// ---------------------------------------------------------------------------
// Gap 4 + 6: truncated evolutions, dead branches, interval transport
// ---------------------------------------------------------------------------

/// `exponential_branching` forks on every symbol and never halts, so a
/// `max_branches` cap truncates it mid-chain. With `max_branches = 3` the BFS
/// expands the root (1 -> 2 branches), expands the first step-1 node
/// (2 -> 3 branches), then stops before the second step-1 node — leaving that
/// node childless.
fn truncated_ntm_evolution() -> irreducible::machines::multiway::MultiwayEvolutionGraph<
    irreducible::machines::Configuration,
    irreducible::NTMTransitionData,
> {
    NondeterministicTM::exponential_branching().run_multiway("", 3, 3)
}

#[test]
fn truncated_evolution_produces_a_counit_event() {
    let evolution = truncated_ntm_evolution();
    let foliation = extract_branchial_foliation(&evolution);

    // Derived from the graph, not from the cospans: the truncation leaves
    // exactly one step-1 node without forward edges, and every other node at
    // steps 0 and 1 forks in two. The cospan literals below follow from this
    // shape, so a drift in the truncated shape — slice widths, or which
    // nodes keep children — fails here first.
    let slice_widths: Vec<usize> = foliation.iter().map(|s| s.nodes.len()).collect();
    assert_eq!(slice_widths, vec![1, 2, 2], "branchial slice widths");
    let children_at_step_1: Vec<usize> = foliation[1]
        .nodes
        .iter()
        .map(|n| evolution.get_forward_edges(n).map_or(0, Vec::len))
        .collect();
    let dead = children_at_step_1.iter().filter(|&&c| c == 0).count();
    assert_eq!(
        dead, 1,
        "one step-1 node is truncated: {children_at_step_1:?}"
    );
    assert_eq!(
        children_at_step_1.iter().sum::<usize>(),
        2,
        "the surviving step-1 node forks in two: {children_at_step_1:?}"
    );

    assert_eq!(evolution.node_count(), 5, "root + 2 + 2");
    assert_eq!(evolution.max_step(), 2);

    let chain = multiway_step_cospans(&evolution);
    assert_eq!(chain.len(), 2);
    assert_eq!(apex_census(&chain[0]), vec![(1, 2)]);
    // Step 1: one branch forks, the truncated one dies — a counit (epsilon).
    assert_eq!(chain[1].left_to_middle().len(), 2);
    assert_eq!(chain[1].right_to_middle().len(), 2);
    assert_eq!(chain[1].middle().len(), 2);
    assert_eq!(apex_census(&chain[1]), vec![(1, 0), (1, 2)]);

    let frobenius = verify_frobenius_preservation(&evolution).expect("verification runs");
    assert!(frobenius.all_hold(), "{frobenius:?}");
    let checked: Vec<usize> = frobenius
        .per_step
        .iter()
        .map(|s| s.components_checked)
        .collect();
    assert_eq!(checked, vec![1, 2]);

    let corels = step_corels(&evolution).expect("step cospans are jointly surjective");
    assert_eq!(corels.len(), 2);
}

#[test]
fn truncated_chain_composes_end_to_end() {
    let evolution = truncated_ntm_evolution();
    let chain = multiway_step_cospans(&evolution);

    let composite = chain[0]
        .compose(&chain[1])
        .expect("adjacent steps of a truncated chain compose");
    assert_eq!(composite.left_to_middle().len(), 1);
    assert_eq!(composite.right_to_middle().len(), 2);

    let corel = evolution_corel(&evolution)
        .expect("composition succeeds")
        .expect("two steps present");
    assert_eq!(corel.as_cospan().left_to_middle().len(), 1);
    assert_eq!(corel.as_cospan().right_to_middle().len(), 2);
    let dom_mid = 1 + corel.as_cospan().middle().len();
    assert!(
        corel.merges(0, dom_mid),
        "the root reaches the surviving branch"
    );
}

#[test]
fn map_cospan_over_a_truncated_chain_drops_the_dead_branch() {
    let evolution = truncated_ntm_evolution();
    let chain = multiway_step_cospans(&evolution);
    let algebra = IntervalCospanAlgebra;

    // Step 1 has one live apex (1 parent, 2 children) and one dead apex
    // (1 parent, no children). Give the dead branch a distinctive interval:
    // transport must forget it, and both children inherit the live hull.
    let step = &chain[1];
    let mut children_of_apex = vec![0usize; step.middle().len()];
    for &a in step.right_to_middle() {
        children_of_apex[a] += 1;
    }
    let dead_apex = children_of_apex
        .iter()
        .position(|&r| r == 0)
        .expect("the truncated step has a dead branch");

    let mut bundle = ParallelIntervals::new();
    for &apex in step.left_to_middle() {
        if apex == dead_apex {
            bundle.add_branch(DiscreteInterval::new(5, 9));
        } else {
            bundle.add_branch(DiscreteInterval::new(0, 1));
        }
    }

    let out = algebra
        .map_cospan(step, &bundle)
        .expect("transport succeeds");
    assert_eq!(out.branch_count(), 2);
    assert!(
        out.branches
            .iter()
            .all(|b| *b == DiscreteInterval::new(0, 1)),
        "the dead branch's interval must not survive: {out:?}"
    );
}

#[test]
fn map_cospan_over_a_truncated_chain_rejects_the_wrong_arity() {
    let evolution = truncated_ntm_evolution();
    let chain = multiway_step_cospans(&evolution);
    let algebra = IntervalCospanAlgebra;

    // Step 1 has a two-node left boundary; a one-branch bundle is an arity
    // mismatch.
    let mut too_small = ParallelIntervals::new();
    too_small.add_branch(DiscreteInterval::new(0, 1));
    assert_eq!(chain[1].left_to_middle().len(), 2);
    assert!(algebra.map_cospan(&chain[1], &too_small).is_err());

    // Every step of a real evolution transports a correctly sized bundle:
    // multiway step cospans never produce a spontaneous branch.
    for (i, c) in chain.iter().enumerate() {
        let mut bundle = ParallelIntervals::new();
        for _ in c.left_to_middle() {
            bundle.add_branch(DiscreteInterval::new(i, i + 1));
        }
        assert!(
            algebra.map_cospan(c, &bundle).is_ok(),
            "step {i} must transport"
        );
    }
}

// ---------------------------------------------------------------------------
// Gap 7 (multiway half): the new perspectives on the shared fixtures
// ---------------------------------------------------------------------------

#[test]
fn diamond_srs_frobenius_census_and_merge_partition_agree() {
    let srs = StringRewriteSystem::new(vec![("S", "AB"), ("S", "BA"), ("AB", "Z"), ("BA", "Z")]);
    let evolution = srs.run_multiway("S", 2, 16);

    // Event census: a fork at step 0, two sequential events at step 1.
    let chain = multiway_step_cospans(&evolution);
    assert_eq!(chain.len(), 2);
    assert_eq!(apex_census(&chain[0]), vec![(1, 2)]);
    assert_eq!(apex_census(&chain[1]), vec![(1, 1), (1, 1)]);

    let frobenius = verify_frobenius_preservation(&evolution).expect("verification runs");
    assert!(frobenius.all_hold(), "{frobenius:?}");

    // Merge partition: the reconvergence the merge-blind census cannot see.
    // The step-1 corelation is where gluing is observable — in the composite
    // the two final positions already share the root's class through the
    // fork, so a merge there does not witness the fingerprint quotient.
    let corels = step_corels(&evolution).expect("step cospans are jointly surjective");
    assert_eq!(
        corels[1].as_cospan().middle().len(),
        1,
        "the two step-1 events are glued into one class"
    );
    assert!(
        corels[1].merges(0, 1),
        "the two parent branches reach one state"
    );

    let corel = evolution_corel(&evolution)
        .expect("composition succeeds")
        .expect("two steps present");
    assert_eq!(corel.as_cospan().left_to_middle().len(), 1);
    assert_eq!(corel.as_cospan().right_to_middle().len(), 2);
    let dom_mid = 1 + corel.as_cospan().middle().len();
    assert!(
        corel.merges(dom_mid, dom_mid + 1),
        "the two final positions are one state"
    );
}
