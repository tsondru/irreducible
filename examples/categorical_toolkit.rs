//! The categorical toolkit on one fixture: the diamond string-rewriting
//! system `S → AB | BA`, both `→ Z`.
//!
//! Four stages, each printing what it computed:
//! 1. step cospans — the evolution as a chain of composable events,
//! 2. `IntervalCospanAlgebra` — interval bundles transported along the chain,
//! 3. `verify_frobenius_preservation` — the event census (μ merge, δ fork,
//!    ε death) and the spider factorization per event,
//! 4. `step_corels` / `evolution_corel` — the merge partition, which sees the
//!    reconvergence the merge-blind chain cannot.
//!
//! Every stage ends in assertions, so `cargo run --example categorical_toolkit`
//! is a check and not just a print.

use catgraph::category::Composable;
use catgraph::cospan::Cospan;
use irreducible::{
    CospanAlgebra, DiscreteInterval, IntervalCospanAlgebra, ParallelIntervals, StringRewriteSystem,
    evolution_corel, multiway_step_cospans, step_corels, verify_frobenius_preservation,
};

fn bundle(intervals: &[(usize, usize)]) -> ParallelIntervals {
    let mut p = ParallelIntervals::new();
    for &(s, e) in intervals {
        p.add_branch(DiscreteInterval::new(s, e));
    }
    p
}

fn show(p: &ParallelIntervals) -> String {
    let branches: Vec<String> = p
        .branches
        .iter()
        .map(|b| format!("[{}, {}]", b.start, b.end))
        .collect();
    if branches.is_empty() {
        "∅".to_string()
    } else {
        branches.join(" ⊕ ")
    }
}

/// The (parents, children) census of every apex vertex, sorted.
fn apex_census(cospan: &Cospan<u32>) -> Vec<(usize, usize)> {
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

fn event_name(l: usize, r: usize) -> &'static str {
    match (l, r) {
        (_, 0) => "ε  branch death (counit)",
        (1, 1) => "id sequential step",
        (1, _) => "δ  fork (comultiplication)",
        (_, 1) => "μ  merge (multiplication)",
        _ => "spider (merge then fork)",
    }
}

fn main() {
    let srs = StringRewriteSystem::new(vec![("S", "AB"), ("S", "BA"), ("AB", "Z"), ("BA", "Z")]);
    let evolution = srs.run_multiway("S", 2, 16);

    // -----------------------------------------------------------------
    println!("== 1. Step cospans ==");
    println!("  S forks to AB | BA at step 0; both rewrite to Z at step 1.");
    let chain = multiway_step_cospans(&evolution);
    for (i, c) in chain.iter().enumerate() {
        println!(
            "  step {i}: left={:?} right={:?} apex={} events",
            c.left_to_middle(),
            c.right_to_middle(),
            c.middle().len()
        );
    }
    assert_eq!(chain.len(), 2, "two steps");
    assert_eq!(chain[0].left_to_middle(), &[0]);
    assert_eq!(chain[0].right_to_middle(), &[0, 0]);
    assert_eq!(chain[1].left_to_middle(), &[0, 1]);
    assert_eq!(chain[1].right_to_middle(), &[0, 1]);

    // -----------------------------------------------------------------
    println!("\n== 2. Interval transport (IntervalCospanAlgebra) ==");
    let algebra = IntervalCospanAlgebra;
    let e0 = bundle(&[(0, 1)]);
    let e1 = algebra
        .map_cospan(&chain[0], &e0)
        .expect("the root transports across the fork");
    let e2 = algebra
        .map_cospan(&chain[1], &e1)
        .expect("both branches transport across step 1");
    println!("  Σ₀ = {}", show(&e0));
    println!(
        "  Σ₁ = {}   (the fork duplicates the root's history)",
        show(&e1)
    );
    println!("  Σ₂ = {}", show(&e2));
    assert_eq!(e1.branch_count(), 2);
    assert_eq!(e2.branch_count(), 2);

    // Functoriality: a(c₁ ; c₂) = a(c₂) ∘ a(c₁) on the real chain.
    let composite = chain[0].compose(&chain[1]).expect("adjacent steps compose");
    let direct = algebra
        .map_cospan(&composite, &e0)
        .expect("the composite transports");
    println!(
        "  a(c₀;c₁)(Σ₀) = {}   (equals the staged transport)",
        show(&direct)
    );
    assert!(
        e2.exactly_equal(&direct),
        "Z' must preserve composition: staged {e2:?} vs direct {direct:?}"
    );

    // The merge's hull is visible through the quotiented (corelation) cospan,
    // where both parents share one apex vertex.
    let corels = step_corels(&evolution).expect("step cospans are jointly surjective");
    let merged = algebra
        .map_cospan(corels[1].as_cospan(), &bundle(&[(0, 2), (1, 4)]))
        .expect("the glued step transports");
    println!(
        "  merge hull: [0, 2] ⊕ [1, 4] through the glued step 1 = {}",
        show(&merged)
    );
    assert_eq!(merged.branch_count(), 2);
    assert!(
        merged
            .branches
            .iter()
            .all(|b| *b == DiscreteInterval::new(0, 4)),
        "a merge takes the hull of its parents' intervals"
    );

    // -----------------------------------------------------------------
    println!("\n== 3. Frobenius event census ==");
    let frobenius = verify_frobenius_preservation(&evolution).expect("verification runs");
    println!("  Generator equations (Eq. 12):");
    println!("    η unit           {}", frobenius.unit_preserved);
    println!("    ε counit         {}", frobenius.counit_preserved);
    println!(
        "    μ multiplication {}",
        frobenius.multiplication_preserved
    );
    println!(
        "    δ comultiplication {}",
        frobenius.comultiplication_preserved
    );
    for (i, c) in chain.iter().enumerate() {
        let census = apex_census(c);
        println!("  step {i}: {} event(s)", census.len());
        for (l, r) in census {
            println!("    ({l} parents, {r} children)  {}", event_name(l, r));
        }
    }
    for check in &frobenius.per_step {
        println!(
            "  step {}: {} component(s) factored through their spider recipe: {}",
            check.step, check.components_checked, check.factorizations_hold
        );
    }
    assert!(frobenius.all_hold(), "{frobenius:?}");
    assert_eq!(apex_census(&chain[0]), vec![(1, 2)], "step 0 is a fork");
    assert_eq!(
        apex_census(&chain[1]),
        vec![(1, 1), (1, 1)],
        "step 1 is two sequential events — the chain is merge-blind"
    );

    // -----------------------------------------------------------------
    println!("\n== 4. Corelation merge partition ==");
    println!("  The two step-2 nodes are distinct graph nodes carrying the");
    println!("  same state (\"Z\"). The corelation glues them, so the two");
    println!("  parent branches land in one class.");
    for (i, c) in corels.iter().enumerate() {
        println!(
            "  step {i}: {} class(es) over {} parents / {} children",
            c.as_cospan().middle().len(),
            c.as_cospan().left_to_middle().len(),
            c.as_cospan().right_to_middle().len()
        );
    }
    assert_eq!(corels.len(), 2);
    assert_eq!(
        corels[1].as_cospan().middle().len(),
        1,
        "the fingerprint quotient collapses both step-1 events"
    );
    assert!(
        corels[1].merges(0, 1),
        "AB and BA reach one state, so they share a class"
    );

    let whole = evolution_corel(&evolution)
        .expect("composition succeeds")
        .expect("the evolution has steps");
    let dom_mid = 1 + whole.as_cospan().middle().len();
    println!(
        "  whole evolution: {} initial → {} final position(s), {} class(es)",
        whole.as_cospan().left_to_middle().len(),
        whole.as_cospan().right_to_middle().len(),
        whole.as_cospan().middle().len()
    );
    assert_eq!(whole.as_cospan().left_to_middle().len(), 1);
    assert_eq!(whole.as_cospan().right_to_middle().len(), 2);
    assert!(
        whole.merges(dom_mid, dom_mid + 1),
        "both final positions are the same state"
    );

    println!("\nAll stages checked.");
}
