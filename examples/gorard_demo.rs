//! # A Functorial Perspective on (Multi)computational Irreducibility
//!
//! NOTE (v0.4.1): the monoidal-functor section uses deprecated coherence
//! APIs that are tautological for `ParallelIntervals`. See the
//! `src/coherence.rs` module docs. A Phase 2.5 replacement will demonstrate
//! real verification against `catgraph_physics` multiway substrate.
//!
//! This demo implements the key insights from Jonathan Gorard's paper (arXiv:2301.04690):
//!
//! > "Computational irreducibility is equivalent to functoriality of the complexity
//! > map Z': 𝒯 → ℬ from computations to cobordisms."
//!
//! ## Core Idea
//!
//! A computation is **irreducible** if there's no shortcut to predict its outcome —
//! you must run every step. Gorard shows this is equivalent to a categorical property:
//! the functor Z' preserves composition.
//!
//! ## Mathematical Framework
//!
//! - **Category 𝒯**: Computations (objects = states, morphisms = transitions)
//! - **Category ℬ**: Cobordisms (objects = time steps, morphisms = intervals)
//! - **Functor Z'**: Maps transitions to their complexity intervals
//! - **Functoriality**: Z'(g∘f) = Z'(g) ∘ Z'(f) means "no shortcuts exist"

use irreducible::{
    ComputationState,
    // Core types
    DiscreteInterval,
    // Cellular Automata
    ElementaryCA,
    IrreducibilityFunctor,
    NondeterministicTM,
    ParallelIntervals,
    // Multiway systems
    StringRewriteSystem,
    // Turing Machine
    TuringMachine,
    // Corelations (merge partitions)
    evolution_corel,
    // Adjunction
    functor::{AdjunctionVerification, StokesIrreducibility, ZPrimeAdjunction, ZPrimeOps},
    // Hypergraph rewriting + catgraph bridge
    machines::hypergraph::{Hypergraph, HypergraphEvolution, RewriteRule as HypergraphRewriteRule},
    // Multiway analysis
    machines::multiway::{BranchialSummary, OllivierFoliation, extract_branchial_foliation},
    // Multiway coherence
    multiway_coherence::{verify_all_coherence, verify_associator, verify_braiding},
    // Step cospans, Frobenius census, corelations
    multiway_step_cospans,
    step_corels,
    verify_frobenius_preservation,
};

fn main() {
    println!();
    print_header("A FUNCTORIAL PERSPECTIVE ON COMPUTATIONAL IRREDUCIBILITY");
    println!("    Implementation of Gorard's arXiv:2301.04690");
    println!();

    // Demo 1: The Basic Insight
    demo_basic_insight();

    // Demo 2: Turing Machine Irreducibility
    demo_turing_machine();

    // Demo 3: Cellular Automata (Rule 30 vs Rule 0)
    demo_cellular_automata();

    // Demo 4: The Z' ⊣ Z Adjunction
    demo_adjunction();

    // Demo 5: Symmetric Monoidal Structure (Multiway)
    demo_monoidal_structure();

    // Demo 6: Coherence Conditions
    demo_coherence();

    // Demo 7: Stokes Integration (conservation analysis)
    demo_stokes_integration();

    // Demo 8: Hypergraph Rewriting (Wolfram Physics via catgraph)
    demo_hypergraph_rewriting();

    // Demo 9: Multiway Branching Visualization
    demo_multiway_branching();

    // Demo 10: Frobenius Event Census
    demo_frobenius_census();

    // Demo 11: Corelation Merge Partition
    demo_corel_merge_partition();

    // Demo 12: Compact-Closed Witness
    demo_compact_closed_witness();

    // Final Summary
    print_final_summary();
}

// ============================================================================
// Demo 1: The Basic Insight
// ============================================================================

fn demo_basic_insight() {
    print_section("1. THE CORE INSIGHT: Functoriality = Irreducibility");

    println!("  From the paper:");
    println!("  ┌─────────────────────────────────────────────────────────────────┐");
    println!("  │ \"A computation is irreducible if and only if the complexity    │");
    println!("  │  functor Z' preserves composition.\"                            │");
    println!("  └─────────────────────────────────────────────────────────────────┘");
    println!();

    println!("  What does this mean? Let's see with a simple example:");
    println!();

    // Irreducible case: Sequential steps that must all be computed
    println!("  IRREDUCIBLE COMPUTATION (no shortcuts):");
    println!("  ─────────────────────────────────────────");
    let step1 = DiscreteInterval::new(0, 1);
    let step2 = DiscreteInterval::new(1, 2);
    let step3 = DiscreteInterval::new(2, 3);

    print_interval_sequence(&[step1, step2, step3]);

    println!();
    println!("  Composition check:");
    let composed_12 = step1.then(step2).unwrap();
    let composed_123 = composed_12.then(step3).unwrap();
    println!("    Z'(T₁) ∘ Z'(T₂) = [0,1] ∘ [1,2] = [0,2]  ✓ contiguous");
    println!("    [0,2] ∘ Z'(T₃) = [0,2] ∘ [2,3] = [0,3]   ✓ contiguous");
    println!();
    println!(
        "    Total: {} → No gaps, no shortcuts!",
        format_interval(&composed_123)
    );
    println!("    This is IRREDUCIBLE: must compute all steps.");
    println!();

    // Reducible case: A cycle allows skipping
    println!("  REDUCIBLE COMPUTATION (shortcut exists):");
    println!("  ──────────────────────────────────────────");
    println!("  If a computation enters a cycle, we can 'skip ahead':");
    println!();
    println!("    State A → State B → State C → State A (cycle!)");
    println!("                                    ↑");
    println!("                                 shortcut");
    println!();
    println!("    Once we detect the cycle, we know the pattern repeats.");
    println!("    Z'(cycle) ≠ Z'(step)∘Z'(step)∘... — composition breaks!");
    println!();
    println!("    This is REDUCIBLE: we can predict without computing.");
    println!();
}

// ============================================================================
// Demo 2: Turing Machine Irreducibility
// ============================================================================

fn demo_turing_machine() {
    print_section("2. TURING MACHINE: The Busy Beaver");

    println!("  The Busy Beaver is a classic example of an irreducible computation.");
    println!("  It's designed to run as long as possible before halting.");
    println!();

    let tm = TuringMachine::busy_beaver_2_2();
    let history = tm.run("", 20);

    println!("  Busy Beaver 2-state, 2-symbol:");
    println!("  ─────────────────────────────────");
    println!("    Steps executed: {}", history.step_count());
    println!("    Final tape: {}", history.final_config.tape);
    println!("    Halted: {}", history.halted);
    println!();

    // Show interval mapping
    println!("  Functor Z' maps each transition to an interval:");
    println!();
    let intervals = history.to_intervals();
    for (i, interval) in intervals.iter().take(6).enumerate() {
        println!(
            "    T{} → Z'(T{}) = {}",
            i + 1,
            i + 1,
            format_interval(interval)
        );
    }
    println!();

    // Check irreducibility
    let analysis = history.analyze_irreducibility();
    println!("  Irreducibility Analysis:");
    println!("  ─────────────────────────");
    println!(
        "    Intervals contiguous: {}",
        analysis.is_sequence_contiguous
    );
    println!("    Cycles detected: {}", analysis.shortcuts.len());
    if let Some(ref interval) = analysis.total_interval {
        println!("    Total interval: {}", format_interval(interval));
    }
    println!();

    if analysis.is_irreducible {
        println!("  ✓ IRREDUCIBLE: Z' is functorial — no shortcuts exist!");
    } else {
        println!("  ✗ REDUCIBLE: Shortcuts found — computation can be predicted.");
    }
    println!();

    // Contrast with a cycling machine
    println!("  Contrast: A Cycling Machine");
    println!("  ────────────────────────────");
    let cycling = TuringMachine::infinite_left_mover();
    let cycling_history = cycling.run("111", 20);
    let cycling_analysis = cycling_history.analyze_irreducibility();

    println!(
        "    Steps: {} (stopped at limit)",
        cycling_history.step_count()
    );
    println!("    Halted: {}", cycling_history.halted);
    println!(
        "    Shortcuts (cycles): {}",
        cycling_analysis.shortcuts.len()
    );
    println!();
    if !cycling_analysis.shortcuts.is_empty() {
        println!("  ✗ REDUCIBLE: State repetition detected — we can predict the pattern!");
    }
    println!();
}

// ============================================================================
// Demo 3: Cellular Automata
// ============================================================================

#[allow(clippy::similar_names)]
fn demo_cellular_automata() {
    print_section("3. CELLULAR AUTOMATA: Rule 30 vs Rule 0");

    println!("  Stephen Wolfram's Rule 30 is conjectured to be irreducible —");
    println!("  there's no shortcut to compute generation n without 1..n-1.");
    println!();

    // Rule 30: Chaotic, likely irreducible
    let ca30 = ElementaryCA::rule_30(31);
    let history30 = ca30.run(ca30.single_cell_initial(), 20);
    let analysis30 = history30.analyze_irreducibility();

    println!("  Rule 30 (Chaotic):");
    println!("  ───────────────────");
    print_ca_evolution(&history30, 10);
    println!();
    println!("    Generations: {}", analysis30.step_count);
    println!("    Cycles found: {}", analysis30.cycles.len());
    println!(
        "    Irreducible: {}",
        if analysis30.is_irreducible {
            "YES ✓"
        } else {
            "no"
        }
    );
    println!();

    // Rule 0: Trivially reducible (all cells die)
    let ca0 = ElementaryCA::new(0, 31);
    let initial0 = ca0.from_pattern("000111000111000");
    let history0 = ca0.run(initial0, 20);
    let analysis0 = history0.analyze_irreducibility();

    println!("  Rule 0 (All cells die):");
    println!("  ────────────────────────");
    print_ca_evolution(&history0, 5);
    println!();
    println!("    Generations: {}", analysis0.step_count);
    println!(
        "    Cycles found: {} (fixed point repeats)",
        analysis0.cycles.len()
    );
    println!(
        "    Irreducible: {}",
        if analysis0.is_irreducible {
            "yes"
        } else {
            "NO ✗"
        }
    );
    println!();
    println!("  Rule 0 is REDUCIBLE: once all cells die, we know all future states!");
    println!();
}

// ============================================================================
// Demo 4: The Z' ⊣ Z Adjunction
// ============================================================================

#[allow(clippy::similar_names)]
fn demo_adjunction() {
    print_section("4. THE ADJUNCTION Z' ⊣ Z");

    println!("  From the paper (Section 4.2):");
    println!("  ┌─────────────────────────────────────────────────────────────────┐");
    println!("  │ \"The existence of the adjunction Z' ⊣ Z encodes a kind of      │");
    println!("  │  'quantum duality' between computation and time.\"              │");
    println!("  └─────────────────────────────────────────────────────────────────┘");
    println!();

    println!("  The adjunction relates two functors:");
    println!();
    println!("       Z': 𝒯 → ℬ    (computation states → time intervals)");
    println!("       Z : ℬ → 𝒯    (time intervals → computation states)");
    println!();

    // Demonstrate Z' and Z
    println!("  Example: Mapping between categories");
    println!("  ────────────────────────────────────");

    let state = ComputationState::new(0, 5);
    let interval = ZPrimeAdjunction::zprime(&state);

    println!(
        "    Computation state: step={}, complexity={}",
        state.step, state.complexity
    );
    println!("    Z'(state) = {}", format_interval(&interval));
    println!();

    let interval2 = DiscreteInterval::new(3, 10);
    let state2 = ZPrimeAdjunction::z(&interval2);

    println!("    Interval: {}", format_interval(&interval2));
    println!(
        "    Z(interval) = ComputationState(step={}, complexity={})",
        state2.step, state2.complexity
    );
    println!();

    // Roundtrip
    println!("  Roundtrip verification:");
    println!("  ────────────────────────");
    let original_state = ComputationState::new(5, 10);
    let to_interval = ZPrimeAdjunction::zprime(&original_state);
    let back_to_state = ZPrimeAdjunction::z(&to_interval);

    println!(
        "    Original:  state(step={}, complexity={})",
        original_state.step, original_state.complexity
    );
    println!("    → Z'() →   {}", format_interval(&to_interval));
    println!(
        "    → Z()  →   state(step={}, complexity={})",
        back_to_state.step, back_to_state.complexity
    );
    println!(
        "    Preserved: {} ✓",
        original_state.step == back_to_state.step
            && original_state.complexity == back_to_state.complexity
    );
    println!();

    // Triangle identities
    println!("  Triangle Identities (coherence of adjunction):");
    println!("  ───────────────────────────────────────────────");
    println!("    These ensure Z' and Z are proper adjoint functors:");
    println!();

    let states = vec![
        ComputationState::new(0, 5),
        ComputationState::new(5, 3),
        ComputationState::new(8, 7),
    ];

    let verification = AdjunctionVerification::verify_sequence::<ZPrimeAdjunction>(&states);

    println!(
        "    Triangle 1 (ε ∘ Z'η = id): {} tests, all passed: {}",
        verification.triangle_1_results.len(),
        verification.triangle_1_results.iter().all(|&b| b)
    );
    println!(
        "    Triangle 2 (Zε ∘ η = id): {} tests, all passed: {}",
        verification.triangle_2_results.len(),
        verification.triangle_2_results.iter().all(|&b| b)
    );
    println!();
    println!(
        "    Adjoint pair verified: {} ✓",
        verification.is_adjoint_pair
    );
    println!();
}

// ============================================================================
// Demo 5: Symmetric Monoidal Structure
// ============================================================================

fn demo_monoidal_structure() {
    print_section("5. MULTIWAY SYSTEMS: Tensor Products");

    println!("  From the paper (Section 3):");
    println!("  ┌─────────────────────────────────────────────────────────────────┐");
    println!("  │ \"Multicomputational irreducibility requires Z' to be a         │");
    println!("  │  symmetric monoidal functor: Z'(f ⊗ g) = Z'(f) ⊕ Z'(g)\"        │");
    println!("  └─────────────────────────────────────────────────────────────────┘");
    println!();

    println!("  In multiway systems, computations can branch and run in parallel.");
    println!("  The tensor product ⊗ represents parallel composition.");
    println!();

    // Simple example with ParallelIntervals
    println!("  Example: Parallel Branches");
    println!("  ──────────────────────────");

    let branch1 = ParallelIntervals::from_branch(DiscreteInterval::new(0, 3));
    let branch2 = ParallelIntervals::from_branch(DiscreteInterval::new(0, 5));

    println!("    Branch 1: {}", format_parallel(&branch1));
    println!("    Branch 2: {}", format_parallel(&branch2));

    let tensor = branch1.clone().tensor(branch2.clone());
    println!("    Branch 1 ⊗ Branch 2 = {}", format_parallel(&tensor));
    println!();

    // Show tensor preservation
    println!("  Tensor Preservation Check:");
    println!("  ──────────────────────────");
    println!("    For Z' to be monoidal: Z'(f ⊗ g) must equal Z'(f) ⊕ Z'(g)");
    println!();

    // Demonstrate with a string rewriting system
    let srs = StringRewriteSystem::new(vec![("A", "B"), ("A", "C")]);
    let evolution = srs.run_multiway("A", 3, 10);

    let result = IrreducibilityFunctor::verify_symmetric_monoidal_functor(&evolution);

    println!("    String Rewriting System: A → B | A → C");
    println!(
        "    Steps: {}, Branches: {}",
        evolution.max_step(),
        result.branch_results.len()
    );
    println!(
        "    Tensor preserved: {}",
        if result.preserves_tensor {
            "YES ✓"
        } else {
            "no ✗"
        }
    );
    println!(
        "    All branches irreducible: {}",
        if result.branches_irreducible {
            "YES ✓"
        } else {
            "no ✗"
        }
    );
    println!(
        "    Multicomputationally irreducible: {}",
        if result.is_multicomputationally_irreducible {
            "YES ✓"
        } else {
            "no ✗"
        }
    );
    println!();
}

// ============================================================================
// Demo 6: Coherence Conditions
// ============================================================================

fn demo_coherence() {
    print_section("6. COHERENCE CONDITIONS (Multiway)");

    println!("  Coherence is verified on actual multiway graphs (non-strict SMC).");
    println!("  Confluence = causal commutativity = falsifiable property.");
    println!();

    // Build a confluent 3-fork multiway graph
    use catgraph_physics::multiway::MultiwayEvolutionGraph;

    let mut g: MultiwayEvolutionGraph<&str, &str> = MultiwayEvolutionGraph::new();
    let root = g.add_root("s0");
    let branches = g.add_fork(
        root,
        vec![("s1a", "e1", 0), ("s1b", "e2", 1), ("s1c", "e3", 2)],
    );
    let merge = g.add_sequential_step(branches[0], "s2", "m1");
    g.add_merge_edge(branches[1], merge, "m2");
    g.add_merge_edge(branches[2], merge, "m3");

    println!("  Graph: 3-way fork from s0, all branches merge to s2");
    println!();

    // Associator
    let forks = g.find_fork_points();
    println!("  1. ASSOCIATOR (3+ children at fork, all pairs confluent):");
    let assoc = verify_associator(&g, forks[0]);
    match &assoc {
        Ok(w) => println!(
            "     Verified: {} pairs, {} diamonds",
            w.pairs_verified, w.diamonds_found
        ),
        Err(e) => println!("     FAILED: {e}"),
    }
    println!();

    // Braiding
    let pairs = g.parallel_independent_events(forks[0]);
    println!("  2. BRAIDING (parallel events commute):");
    for (i, (a, b)) in pairs.iter().enumerate() {
        let braid = verify_braiding(&g, *a, *b);
        println!(
            "     Pair {}: {}",
            i,
            if braid.is_ok() { "commutes" } else { "FAILS" }
        );
    }
    println!();

    // Full verification
    let errors = verify_all_coherence(&g);
    println!(
        "  Comprehensive: {} errors (fully coherent: {})",
        errors.len(),
        errors.is_empty()
    );
    println!();
}

// ============================================================================
// Demo 7: Categorical Hypergraph Rewriting (catgraph bridge)
// ============================================================================

fn demo_stokes_integration() {
    print_section("7. CATEGORICAL HYPERGRAPH REWRITING");

    println!("  DPO (Double-Pushout) rewriting has a natural categorical interpretation.");
    println!("  Each rewrite rule L → R with preserved kernel K is a span L ← K → R.");
    println!("  Each rewrite step produces a cospan. The full evolution is a cospan chain.");
    println!();

    // Part A: Rewrite rules as categorical spans
    println!("  A. REWRITE RULES AS SPANS (catgraph::Span)");
    println!("  ────────────────────────────────────────────");
    println!();

    let rules: Vec<(&str, HypergraphRewriteRule)> = vec![
        (
            "A→BB  {{0,1,2}} → {{0,1},{1,2}}",
            HypergraphRewriteRule::wolfram_a_to_bb(),
        ),
        (
            "split {{0,1}} → {{0,2},{2,1}}",
            HypergraphRewriteRule::edge_split(),
        ),
        (
            "collapse {{0,1},{1,2}} → {{0,2}}",
            HypergraphRewriteRule::collapse(),
        ),
    ];

    for (desc, rule) in &rules {
        let span = rule.to_span();
        println!("    {desc}:");
        println!(
            "      |L| = {}, |R| = {}, |K| = {} (preserved)",
            span.left().len(),
            span.right().len(),
            span.middle_pairs().len()
        );
        println!("      Kernel pairs: {:?}", span.middle_pairs());
        println!();
    }

    // Part B: Evolution as cospan chain
    println!("  B. EVOLUTION AS COSPAN CHAIN (catgraph::Cospan)");
    println!("  ────────────────────────────────────────────────");
    println!();

    let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
    let rule = HypergraphRewriteRule::wolfram_a_to_bb();
    let evolution = HypergraphEvolution::run(&initial, &[rule], 4);

    let cospans = evolution.to_cospan_chain();

    println!("    Initial: {{0, 1, 2}} (one ternary hyperedge)");
    println!("    Rule: A→BB (Wolfram Physics fundamental rule)");
    println!("    Steps: {}", cospans.len());
    println!();

    for (i, cospan) in cospans.iter().enumerate() {
        println!(
            "    Step {}: |left| = {}, |apex| = {}, |right| = {}",
            i + 1,
            cospan.left_to_middle().len(),
            cospan.middle().len(),
            cospan.right_to_middle().len()
        );
    }
    println!();

    // Verify composability
    let composable = cospans
        .windows(2)
        .all(|w| w[0].right_to_middle().len() == w[1].left_to_middle().len());
    println!(
        "    Chain composable: {} (boundaries match)",
        if composable { "YES" } else { "NO" }
    );
    println!();

    // Part C: Stokes conservation → cospan composability
    println!("  C. STOKES CONSERVATION = COSPAN COMPOSABILITY");
    println!("  ──────────────────────────────────────────────");
    println!();

    let intervals = vec![
        DiscreteInterval::new(0, 1),
        DiscreteInterval::new(1, 2),
        DiscreteInterval::new(2, 3),
        DiscreteInterval::new(3, 4),
    ];

    match StokesIrreducibility::analyze(&intervals) {
        Ok(analysis) => {
            let stokes_cospans = analysis.to_cospan_chain();

            println!("    Intervals: {} contiguous steps", intervals.len());
            println!(
                "    Stokes conserved: {}",
                analysis.conservation.is_conserved
            );
            println!(
                "    Cospan chain: {} composable cospans",
                stokes_cospans.len()
            );
            println!();
            println!("    Key insight: For dim-1 complexes, dω = 0 always.");
            println!("    Conservation reduces to contiguity + monotonicity,");
            println!("    which is exactly cospan composability in ℬ.");
        }
        Err(e) => println!("    Error: {e}"),
    }
    println!();

    // Part D: Wilson loops and causal invariance
    println!("  D. WILSON LOOPS: Causal Invariance via Span Equivalence");
    println!("  ────────────────────────────────────────────────────────");
    println!();

    let initial_multi = Hypergraph::from_edges(vec![vec![0, 1, 2], vec![1, 2, 3]]);
    let multi_evolution = HypergraphEvolution::run_multiway(
        &initial_multi,
        &[HypergraphRewriteRule::wolfram_a_to_bb()],
        3,
        50,
    );

    let invariance = multi_evolution.analyze_causal_invariance();
    let stats = multi_evolution.statistics();

    println!(
        "    Multiway evolution: {} nodes, {} branches, {} merges",
        stats.total_nodes, stats.branch_count, stats.merge_count
    );
    println!("    Wilson loops analyzed: {}", invariance.loops_analyzed);
    println!(
        "    Causally invariant: {}",
        if invariance.is_invariant {
            "YES (holonomy ≈ 1)"
        } else {
            "NO (non-trivial loops)"
        }
    );
    println!();
    println!("    When all Wilson loops have holonomy = 1, different rewrite");
    println!("    orderings produce equivalent cospan chains — the categorical");
    println!("    manifestation of gauge invariance in Wolfram Physics.");
    println!();
}

// ============================================================================
// Demo 8: Hypergraph Rewriting (Wolfram Physics via catgraph)
// ============================================================================

fn demo_hypergraph_rewriting() {
    print_section("8. HYPERGRAPH REWRITING: Wolfram Physics via catgraph");

    println!("  From the paper (Section 3):");
    println!("  ┌─────────────────────────────────────────────────────────────────┐");
    println!("  │ Hypergraph rewriting is the Wolfram model's core mechanism.     │");
    println!("  │ Each rewrite rule is a categorical span L <- K -> R (DPO),      │");
    println!("  │ and evolution steps are cospans composable in category B.        │");
    println!("  └─────────────────────────────────────────────────────────────────┘");
    println!();

    // Part A: Single rule, deterministic evolution
    println!("  A. DPO REWRITING: Rule A -> BB (Wolfram's canonical example)");
    println!("  ─────────────────────────────────────────────────────────────");
    println!();

    let rule = HypergraphRewriteRule::wolfram_a_to_bb();
    let span = rule.to_span();
    println!("    Rule as categorical span L <- K -> R:");
    println!(
        "      Left (L):  {} elements  (pattern to match)",
        span.left().len()
    );
    println!(
        "      Right (R): {} elements  (replacement)",
        span.right().len()
    );
    println!(
        "      Kernel (K): {} pairs     (preserved structure)",
        span.middle_pairs().len()
    );
    println!();

    let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
    let evolution = HypergraphEvolution::run(&initial, &[rule], 4);

    println!("    Deterministic evolution (4 steps):");
    for step in 0..=evolution.max_step() {
        let node_ids = evolution.nodes_at_step(step);
        for &id in &node_ids {
            if let Some(node) = evolution.get_node(id) {
                println!(
                    "      Step {}: {} vertices, {} hyperedges",
                    step,
                    node.state.vertex_count(),
                    node.state.edge_count()
                );
            }
        }
    }
    println!();

    // Part B: Cospan chain from evolution
    println!("  B. EVOLUTION AS COSPAN CHAIN");
    println!("  ────────────────────────────");
    println!();

    let cospan_chain = evolution.to_cospan_chain();
    println!("    Each step G_i -> G_{{i+1}} yields a cospan in B:");
    for (i, cospan) in cospan_chain.iter().enumerate() {
        println!(
            "      Step {i}: left={}, middle={}, right={}",
            cospan.left_to_middle().len(),
            cospan.middle().len(),
            cospan.right_to_middle().len()
        );
    }
    println!();

    match evolution.compose_cospan_chain() {
        Ok(composed) => {
            println!(
                "    Composed cospan (all steps): middle={} elements",
                composed.middle().len()
            );
            println!("    Chain is composable: boundaries match at every step.");
        }
        Err(e) => println!("    Composition failed: {e}"),
    }
    println!();

    // Part C: Multiple rules, multiway evolution
    println!("  C. MULTIWAY HYPERGRAPH EVOLUTION");
    println!("  ─────────────────────────────────");
    println!();

    let initial_multi = Hypergraph::from_edges(vec![vec![0, 1, 2], vec![1, 2, 3]]);
    let multi_evolution = HypergraphEvolution::run_multiway(
        &initial_multi,
        &[
            HypergraphRewriteRule::wolfram_a_to_bb(),
            HypergraphRewriteRule::edge_split(),
        ],
        3,
        30,
    );

    let stats = multi_evolution.statistics();
    println!("    Two rules applied to overlapping hyperedges:");
    println!("      Nodes explored: {}", stats.total_nodes);
    println!("      Branches:       {}", stats.branch_count);
    println!("      Merges:         {}", stats.merge_count);
    println!("      Max depth:      {}", stats.max_step);
    println!();

    let invariance = multi_evolution.analyze_causal_invariance();
    println!("    Causal invariance (Wilson loop holonomy):");
    println!("      Loops analyzed:    {}", invariance.loops_analyzed);
    println!(
        "      Causally invariant: {}",
        if invariance.is_invariant { "YES" } else { "NO" }
    );
    println!();
    println!("    When causally invariant, different rule application orders");
    println!("    produce equivalent cospan chains — gauge invariance in the");
    println!("    categorical framework.");
    println!();
}

// ============================================================================
// Demo 9: Multiway Branching Visualization
// ============================================================================

#[allow(clippy::similar_names)]
fn demo_multiway_branching() {
    print_section("9. MULTIWAY BRANCHING: Visualizing Non-Determinism");

    println!("  From the paper (Section 2.2):");
    println!("  ┌─────────────────────────────────────────────────────────────────┐");
    println!("  │ The multiway evolution graph captures ALL possible computation  │");
    println!("  │ paths. Branchial graphs at each step encode the tensor product  │");
    println!("  │ structure, and curvature measures geometric complexity.          │");
    println!("  └─────────────────────────────────────────────────────────────────┘");
    println!();

    // Part A: SRS multiway tree
    println!("  A. STRING REWRITING: Multiway Evolution Tree");
    println!("  ─────────────────────────────────────────────");
    println!();

    let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
    let evolution = srs.run_multiway("AB", 4, 30);
    let stats = evolution.statistics();

    println!("    Rules: AB -> BA, A -> AA");
    println!("    Initial state: \"AB\"");
    println!();

    // Level-by-level visualization
    for step in 0..=stats.max_depth.min(4) {
        let nodes = evolution.nodes_at_step(step);
        let states: Vec<String> = nodes.iter().map(|n| format!("{}", n.state)).collect();

        let prefix = if step == 0 { "    root" } else { "        " };
        let branch_marker = if nodes.len() > 1 {
            format!("  ({} branches)", nodes.len())
        } else {
            String::new()
        };
        println!(
            "{prefix} Step {step}: {}{branch_marker}",
            states.join("  |  ")
        );
    }
    println!();

    println!("    Statistics:");
    println!("      Total nodes: {}", stats.total_nodes);
    println!(
        "      Fork points: {} (non-deterministic choices)",
        stats.fork_count
    );
    println!(
        "      Merge points: {} (confluent paths)",
        stats.merge_count
    );
    println!("      Max depth: {}", stats.max_depth);
    println!("      Leaf nodes: {} (terminal states)", stats.leaf_count);
    println!();

    // Part B: NTM multiway tree
    println!("  B. NON-DETERMINISTIC TURING MACHINE: Branching Computation");
    println!("  ────────────────────────────────────────────────────────────");
    println!();

    let ntm = NondeterministicTM::simple_branching_example();
    let ntm_evolution = ntm.run_multiway("0", 4, 20);
    let ntm_stats = ntm_evolution.statistics();

    println!("    NTM: on '0', can go left OR right (non-deterministic)");
    println!();

    for step in 0..=ntm_stats.max_depth.min(3) {
        let nodes = ntm_evolution.nodes_at_step(step);
        let is_fork = ntm_evolution
            .find_fork_points()
            .iter()
            .any(|fp| fp.step == step);
        let marker = if is_fork { " <-- FORK" } else { "" };
        println!("      Step {step}: {} state(s){marker}", nodes.len());
    }
    println!();

    // Part C: Branchial foliation and curvature
    println!("  C. BRANCHIAL FOLIATION: Geometry of Branching");
    println!("  ──────────────────────────────────────────────");
    println!();

    let srs2 = StringRewriteSystem::new(vec![("A", "AB"), ("A", "BA"), ("B", "A")]);
    let evolution2 = srs2.run_multiway("A", 5, 50);
    let foliation = extract_branchial_foliation(&evolution2);
    let summary = BranchialSummary::from_foliation(&foliation);

    println!("    Rules: A -> AB, A -> BA, B -> A");
    println!("    Branchial hypersurfaces Sigma_t at each step:");
    println!();
    println!("      Step  Nodes  Edges  Components");
    println!("      ────  ─────  ─────  ──────────");

    for (step, bg) in foliation.iter().enumerate() {
        let components = bg.connected_components();
        let bar = "|".repeat(bg.node_count().min(30));
        println!(
            "      {step:>4}  {:>5}  {:>5}  {:>10}  {bar}",
            bg.node_count(),
            bg.edge_count(),
            components
        );
    }
    println!();

    println!("    Summary:");
    println!(
        "      Max parallel branches: {}",
        summary.max_parallel_branches
    );
    println!("      Average branching: {:.2}", summary.average_branches);
    println!(
        "      Total branchial edges: {}",
        summary.total_branchial_edges
    );
    println!();

    // Part D: Curvature
    println!("  D. BRANCHIAL CURVATURE: Geometric Complexity");
    println!("  ─────────────────────────────────────────────");
    println!();

    let curvature_foliation = OllivierFoliation::from_evolution(&evolution2);
    let profile = curvature_foliation.irreducibility_profile();

    println!("    Curvature measures how \"geometrically complex\" branching is.");
    println!("    Flat (0.0) = uniform branching. High = irregular structure.");
    println!();
    println!("      Step  Irreducibility   Visual");
    println!("      ────  ──────────────   ──────");

    for (step, &indicator) in profile.iter().enumerate() {
        let bar_len = (indicator * 30.0).min(30.0) as usize;
        let bar = "#".repeat(bar_len);
        println!("      {step:>4}  {indicator:>14.4}   {bar}");
    }
    println!();

    let avg = curvature_foliation.average_irreducibility();
    let flat = curvature_foliation.is_globally_flat();
    println!("    Average irreducibility indicator: {avg:.4}");
    println!(
        "    Globally flat: {} (uniform branching = simpler structure)",
        if flat { "YES" } else { "NO" }
    );
    println!();
}

// ============================================================================
// Demo 10: Frobenius Event Census
// ============================================================================

fn demo_frobenius_census() {
    print_section("10. FROBENIUS STRUCTURE: Multiway Events ARE the Generators");

    println!("  Fong-Spivak §2.3 (Eq. 12) + Prop 3.8:");
    println!("  ┌─────────────────────────────────────────────────────────────────┐");
    println!("  │ Under the free-hypergraph-category encoding, the Frobenius     │");
    println!("  │ generators of 𝒯 ARE the multiway event types.                  │");
    println!("  └─────────────────────────────────────────────────────────────────┘");
    println!();
    println!("    μ  multiplication   ←→  merge  (two branches reach one state)");
    println!("    δ  comultiplication ←→  fork   (one state rewrites two ways)");
    println!("    ε  counit           ←→  death  (a branch has no continuation)");
    println!("    η  unit             ←→  birth  (the root; never mid-evolution)");
    println!();

    // Diamond: S forks to AB | BA, both rewrite to Z.
    let srs = StringRewriteSystem::new(vec![("S", "AB"), ("S", "BA"), ("AB", "Z"), ("BA", "Z")]);
    let evolution = srs.run_multiway("S", 2, 16);

    println!("  Fixture: S → AB | BA, both → Z");
    println!("  ────────────────────────────────");
    let chain = multiway_step_cospans(&evolution);
    for (i, cospan) in chain.iter().enumerate() {
        let apex_count = cospan.middle().len();
        let mut parents = vec![0usize; apex_count];
        let mut children = vec![0usize; apex_count];
        for &a in cospan.left_to_middle() {
            parents[a] += 1;
        }
        for &a in cospan.right_to_middle() {
            children[a] += 1;
        }
        println!("    step {i}: {apex_count} event(s)");
        for (l, r) in parents.iter().zip(children.iter()) {
            let generator = match (l, r) {
                (_, 0) => "ε  branch death",
                (1, 1) => "id sequential step",
                (1, _) => "δ  fork",
                (_, 1) => "μ  merge",
                _ => "spider (merge then fork)",
            };
            println!("      ({l} parents → {r} children)   {generator}");
        }
    }
    println!();

    let result = verify_frobenius_preservation(&evolution).expect("verification runs");
    println!("  Eq. 12 on the generators (Z' preserves each):");
    println!("    η  {}", result.unit_preserved);
    println!("    ε  {}", result.counit_preserved);
    println!("    μ  {}", result.multiplication_preserved);
    println!("    δ  {}", result.comultiplication_preserved);
    println!();
    println!("  Spider factorization per event (Prop 3.8):");
    for check in &result.per_step {
        println!(
            "    step {}: {} component(s), factorizations hold: {}",
            check.step, check.components_checked, check.factorizations_hold
        );
    }
    println!();
    println!(
        "  All Frobenius checks hold: {}  ← every event is its generator recipe",
        result.all_hold()
    );
    println!();

    assert!(result.all_hold(), "{result:?}");
    assert_eq!(chain.len(), 2, "the diamond runs two steps");
    assert_eq!(
        chain[0].middle().len(),
        1,
        "step 0 is one event: the fork out of S"
    );
    assert_eq!(
        result.per_step[1].components_checked, 2,
        "step 1 is two events in the merge-blind chain"
    );
}

// ============================================================================
// Demo 11: Corelation Merge Partition
// ============================================================================

fn demo_corel_merge_partition() {
    print_section("11. CORELATIONS: Where the Branches Reconverge");

    println!("  Fong-Spivak 2018, Ex 6.64:");
    println!("  ┌─────────────────────────────────────────────────────────────────┐");
    println!("  │ A corelation is a jointly-surjective cospan — a partition of   │");
    println!("  │ its boundary. Each multiway step lifts to one, whose classes    │");
    println!("  │ are the step's events.                                          │");
    println!("  └─────────────────────────────────────────────────────────────────┘");
    println!();
    println!("  The multiway explorer keeps same-state nodes reached along");
    println!("  different paths as DISTINCT graph nodes, so a merge exists only");
    println!("  at the fingerprint level. The corelation is what records it.");
    println!();

    let srs = StringRewriteSystem::new(vec![("S", "AB"), ("S", "BA"), ("AB", "Z"), ("BA", "Z")]);
    let evolution = srs.run_multiway("S", 2, 16);

    println!("  Diamond: S → AB | BA, both → Z");
    println!("  ───────────────────────────────");
    let raw = multiway_step_cospans(&evolution);
    let corels = step_corels(&evolution).expect("step cospans are jointly surjective");
    for (i, (raw_step, corel)) in raw.iter().zip(corels.iter()).enumerate() {
        println!(
            "    step {i}: raw chain sees {} event(s); the corelation sees {} class(es)",
            raw_step.middle().len(),
            corel.as_cospan().middle().len()
        );
    }
    println!();
    println!(
        "    Step 1's two parent branches share a class: {}",
        corels[1].merges(0, 1)
    );
    println!("    ← AB and BA both reach \"Z\": the reconvergence the raw,");
    println!("      merge-blind chain cannot see.");
    println!();

    let whole = evolution_corel(&evolution)
        .expect("composition succeeds")
        .expect("the evolution has steps");
    let dom_mid = 1 + whole.as_cospan().middle().len();
    println!("  Composed over the whole evolution (pushout of the chain):");
    println!(
        "    {} initial position → {} final position(s), {} class(es)",
        whole.as_cospan().left_to_middle().len(),
        whole.as_cospan().right_to_middle().len(),
        whole.as_cospan().middle().len()
    );
    println!(
        "    Both final positions are one state: {}",
        whole.merges(dom_mid, dom_mid + 1)
    );
    println!();

    assert_eq!(raw[1].middle().len(), 2, "the raw chain sees two events");
    assert_eq!(
        corels[1].as_cospan().middle().len(),
        1,
        "the corelation glues them into one class"
    );
    assert!(
        corels[1].merges(0, 1),
        "AB and BA must share the merged class"
    );
    assert_eq!(whole.as_cospan().left_to_middle().len(), 1);
    assert_eq!(whole.as_cospan().right_to_middle().len(), 2);
    assert!(whole.merges(dom_mid, dom_mid + 1));
}

// ============================================================================
// Demo 12: Compact-Closed Witness
// ============================================================================

fn demo_compact_closed_witness() {
    print_section("12. COMPACT CLOSURE: The Snake Identities Behind Z' ⊣ Z");

    println!("  The adjunction of Demo 4 has a string-diagram counterpart: in a");
    println!("  compact closed category every object is its own dual, witnessed");
    println!("  by cup and cap satisfying the snake (zigzag) identities.");
    println!();
    println!("      right snake:  (id ⊗ cup) ; (cap ⊗ id) = id");
    println!("      left snake:   (cup ⊗ id) ; (id ⊗ cap) = id");
    println!();
    println!("  Drawn, the composite is a bent wire pulled straight:");
    println!();
    println!("        ╭───╮              ");
    println!("     ───╯   ╰───    ═══>    ─────────");
    println!();

    let state = ComputationState::new(0, 5);
    let cospan = ZPrimeAdjunction::zprime_cospan(&state);
    println!("  Z'(state) as a cospan in the temporal encoding:");
    println!(
        "    left={:?} apex={:?} right={:?}",
        cospan.left_to_middle(),
        cospan.middle(),
        cospan.right_to_middle()
    );
    println!();

    let witness = ZPrimeAdjunction::verify_compact_closed_witness(&state)
        .expect("the witness computes for a well-formed state");
    println!("  Witness at every boundary label of Z'(state):");
    println!(
        "    right snake holds:              {}",
        witness.right_snake_holds
    );
    println!(
        "    left snake holds:               {}",
        witness.left_snake_holds
    );
    println!(
        "    name/unname boundary round-trip: {}",
        witness.name_roundtrip_boundaries_hold
    );
    println!();
    println!(
        "  All compact-closed witnesses hold: {}",
        witness.all_hold()
    );
    println!();
    println!("  The round-trip is Prop 3.2's name/unname on the Frobenius");
    println!("  decomposition, checked at the boundary level: the free");
    println!("  hypergraph category carries no diagram normal form upstream,");
    println!("  so full string-diagram equality is not decidable there.");
    println!();

    assert!(witness.right_snake_holds, "right snake");
    assert!(witness.left_snake_holds, "left snake");
    assert!(
        witness.name_roundtrip_boundaries_hold,
        "name/unname boundary round-trip"
    );
    assert!(witness.all_hold());
    assert_eq!(
        cospan.middle().len(),
        2,
        "Z'(state)'s apex is {{start, end}}"
    );
}

// ============================================================================
// Final Summary
// ============================================================================

fn print_final_summary() {
    print_section("SUMMARY: Gorard's Key Insights Demonstrated");

    println!("  ┌─────────────────────────────────────────────────────────────────┐");
    println!("  │                                                                 │");
    println!("  │  1. IRREDUCIBILITY = FUNCTORIALITY                              │");
    println!("  │     A computation is irreducible iff Z' preserves composition   │");
    println!("  │     Z'(g∘f) = Z'(g) ∘ Z'(f)                                      │");
    println!("  │                                                                 │");
    println!("  │  2. THE FUNCTOR Z': 𝒯 → ℬ                                       │");
    println!("  │     Maps computations to cobordisms (time intervals)            │");
    println!("  │     Contiguous intervals = no shortcuts = irreducible           │");
    println!("  │                                                                 │");
    println!("  │  3. THE ADJUNCTION Z' ⊣ Z                                       │");
    println!("  │     Encodes 'quantum duality' between computation and time      │");
    println!("  │     Triangle identities ensure coherence                        │");
    println!("  │                                                                 │");
    println!("  │  4. SYMMETRIC MONOIDAL STRUCTURE                                │");
    println!("  │     For multiway systems: Z'(f ⊗ g) = Z'(f) ⊕ Z'(g)             │");
    println!("  │     Coherence conditions (α, λ, ρ, σ) must hold                 │");
    println!("  │                                                                 │");
    println!("  │  5. CATEGORICAL HYPERGRAPH REWRITING (catgraph)                  │");
    println!("  │     DPO rules as spans L ← K → R, evolution as cospan chains  │");
    println!("  │     Wilson loop holonomy = gauge invariance                     │");
    println!("  │                                                                 │");
    println!("  │  6. STOKES → COSPAN COMPOSABILITY                              │");
    println!("  │     For dim-1: dω=0 + contiguity = composable cospans in ℬ    │");
    println!("  │     Bridges differential geometry to category theory            │");
    println!("  │                                                                 │");
    println!("  │  7. HYPERGRAPH REWRITING (Wolfram Physics)                      │");
    println!("  │     DPO rules, cospan chains, multiway causal invariance       │");
    println!("  │                                                                 │");
    println!("  │  8. MULTIWAY BRANCHING VISUALIZATION                            │");
    println!("  │     Branchial foliation, curvature, geometric complexity        │");
    println!("  │                                                                 │");
    println!("  │  9. FROBENIUS STRUCTURE                                         │");
    println!("  │     Multiway events ARE the generators: μ merge, δ fork,        │");
    println!("  │     ε death, η birth — each factors through its spider recipe   │");
    println!("  │                                                                 │");
    println!("  │ 10. CORELATION MERGE PARTITION                                  │");
    println!("  │     Jointly-surjective cospans record the fingerprint-level     │");
    println!("  │     reconvergence the raw event chain cannot see                │");
    println!("  │                                                                 │");
    println!("  │ 11. COMPACT CLOSURE                                             │");
    println!("  │     Snake identities + Prop 3.2 name/unname round-trip          │");
    println!("  │     behind the Z' ⊣ Z adjunction                                │");
    println!("  │                                                                 │");
    println!("  └─────────────────────────────────────────────────────────────────┘");
    println!();
    println!("  Reference: Gorard, J. (2023). A Functorial Perspective on");
    println!("             (Multi)computational Irreducibility. arXiv:2301.04690");
    println!();
}

// ============================================================================
// Helper Functions for Formatting
// ============================================================================

fn print_header(title: &str) {
    let width = 70;
    println!("╔{}╗", "═".repeat(width));
    let padding = (width - title.len()) / 2;
    println!(
        "║{}{}{:>width$}║",
        " ".repeat(padding),
        title,
        "",
        width = width - padding - title.len()
    );
    println!("╚{}╝", "═".repeat(width));
}

fn print_section(title: &str) {
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  {title}");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
}

fn format_interval(interval: &DiscreteInterval) -> String {
    format!("[{}, {}]", interval.start, interval.end)
}

fn format_parallel(p: &ParallelIntervals) -> String {
    let branches: Vec<String> = p.branches.iter().map(format_interval).collect();
    if branches.is_empty() {
        "∅ (empty)".to_string()
    } else {
        branches.join(" ⊕ ")
    }
}

fn print_interval_sequence(intervals: &[DiscreteInterval]) {
    println!();
    println!("    Time:  0     1     2     3     4     5");
    println!("           │     │     │     │     │     │");
    println!("           ├─────┼─────┼─────┼─────┼─────┤");

    for (i, interval) in intervals.iter().enumerate() {
        let start = interval.start;
        let end = interval.end;
        let mut line = [' '; 30];

        // Mark the interval
        let start_pos = start * 6;
        let end_pos = end * 6;
        if start_pos < line.len() && end_pos <= line.len() {
            for ch in &mut line[start_pos..end_pos] {
                *ch = '─';
            }
            line[start_pos] = '├';
            if end_pos > 0 && end_pos - 1 < line.len() {
                line[end_pos - 1] = '┤';
            }
        }

        let line_str: String = line.iter().collect();
        println!(
            "    T{}: {}  {}",
            i + 1,
            line_str.trim_end(),
            format_interval(interval)
        );
    }
    println!();
}

fn print_ca_evolution(history: &irreducible::CAExecutionHistory, max_rows: usize) {
    let generations = history.all_generations();
    let width = generations.first().map_or(31, |g| g.width());
    let half_width = 15.min(width / 2);

    for (i, generation) in generations.iter().take(max_rows).enumerate() {
        // Build display from cells directly, centered on the middle
        let mid = width / 2;
        let start = mid.saturating_sub(half_width);
        let end = (mid + half_width + 1).min(width);

        let display: String = (start..end)
            .map(|x| {
                if generation.get(x.cast_signed()) {
                    '█'
                } else {
                    '·'
                }
            })
            .collect();

        println!("    {i:>3}: {display}");
    }

    if generations.len() > max_rows {
        println!(
            "    ... ({} more generations)",
            generations.len() - max_rows
        );
    }
}
