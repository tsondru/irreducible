# Gorard Demo: A Functorial Perspective on Computational Irreducibility

This example provides a comprehensive, presentation-ready demonstration of Jonathan Gorard's paper **"A Functorial Perspective on (Multi)computational Irreducibility"** ([arXiv:2301.04690](https://arxiv.org/abs/2301.04690)).

## Quick Start

```bash
cargo run --example gorard_demo
```

## What This Demo Shows

The demo walks through the key mathematical insights from Gorard's paper with concrete examples:

### 1. The Core Insight: Functoriality = Irreducibility

> "A computation is irreducible if and only if the complexity functor Z' preserves composition."

The demo shows how interval composition reveals whether shortcuts exist:

- **Contiguous intervals** → Must compute every step → **Irreducible**
- **Gaps or cycles** → Can skip computation → **Reducible**

### 2. Turing Machine Examples

- **Busy Beaver**: A classic irreducible computation
  - Z' maps each transition to interval [n, n+1]
  - All intervals contiguous → No shortcuts → Irreducible

- **Cycling Machine**: A reducible computation
  - State repetition creates shortcuts
  - Composition breaks → Reducible

### 3. Cellular Automata

- **Rule 30**: Wolfram's chaotic, conjectured irreducible CA
  - No cycle detection → Must simulate every generation

- **Rule 0**: Trivially reducible (all cells die)
  - Fixed point repeats → Can predict all future states

### 4. The Z' ⊣ Z Adjunction

The paper's "quantum duality" interpretation:

- Z': 𝒯 → ℬ (computation → time intervals)
- Z : ℬ → 𝒯 (time intervals → computation)
- Triangle identities verify proper adjoint functors

### 5. Multiway Systems (Tensor Products)

For parallel/branching computations:

- Tensor product ⊗ represents parallel composition
- Z' must preserve: Z'(f ⊗ g) = Z'(f) ⊕ Z'(g)
- This is **multicomputational irreducibility**

### 6. Coherence Conditions

Symmetric monoidal categories require:

- **Associator α**: (A ⊗ B) ⊗ C ≅ A ⊗ (B ⊗ C)
- **Left unitor λ**: I ⊗ A ≅ A
- **Right unitor ρ**: A ⊗ I ≅ A
- **Braiding σ**: A ⊗ B ≅ B ⊗ A

### 7. Stokes Integration

Conservation laws bridge differential geometry to category theory:

- Temporal complex from interval sequences
- Exterior derivative (boundary operator)
- dω = 0 + contiguity = composable cospans in B
- Wilson loops and causal invariance via span equivalence

### 8. Hypergraph Rewriting (Wolfram Physics via catgraph)

The paper's core physical application — hypergraph rewriting as DPO:

- **DPO rules as spans**: L <- K -> R (pattern, kernel, replacement)
- **Evolution as cospan chains**: each step G_i -> G_{i+1} yields a composable cospan
- **Multiway evolution**: multiple rules on overlapping hyperedges
- **Causal invariance**: Wilson loop holonomy measures gauge invariance

### 9. Multiway Branching Visualization

Text-based visualization of non-deterministic computation structure:

- **Level-by-level tree**: shows all states at each BFS depth
- **Branchial foliation**: Sigma_t hypersurfaces with node/edge/component counts
- **Branchial curvature**: geometric complexity indicator per step
- **NTM branching**: non-deterministic Turing machine fork points

### 10. Frobenius Structure: Multiway Events ARE the Generators

Under the free-hypergraph-category encoding (Thm 3.14), the Frobenius generators of 𝒯 are exactly the multiway event types:

| Generator | Multiway event |
| --------- | -------------- |
| μ multiplication | merge — two branches reach one state |
| δ comultiplication | fork — one state rewrites two ways |
| ε counit | branch death — no continuation |
| η unit | branch birth — the root, never mid-evolution |

On the diamond `S → AB | BA`, both `→ Z`, the demo prints the per-step event census and checks that every event factors through its Frobenius generator recipe (Prop 3.8), alongside Eq. 12 on the four generators.

### 11. Corelation Merge Partition

A corelation is a jointly-surjective cospan — a partition of its boundary. Each step lifts to one whose classes are the step's events.

The multiway explorer keeps same-state nodes reached along different paths as **distinct graph nodes**, so a merge exists only at the fingerprint level. The corelation is what records it: on the diamond, the raw chain sees two separate step-1 events while the corelation glues them into one class. `evolution_corel` composes the chain by pushout into a single partition from the initial branchial slice to the final one.

### 12. Compact Closure

The Z' ⊣ Z adjunction of section 4 has a string-diagram counterpart: in a compact closed category every object is its own dual, witnessed by cup and cap satisfying the snake identities.

- right snake: `(id ⊗ cup) ; (cap ⊗ id) = id`
- left snake: `(cup ⊗ id) ; (id ⊗ cap) = id`

`verify_compact_closed_witness` checks both at every boundary label of Z'(state)'s cospan, plus the Prop 3.2 name/unname round-trip on its Frobenius decomposition. The round-trip is checked at the boundary level only: the free hypergraph category carries no diagram normal form upstream, so full string-diagram equality is not decidable there.

## Key Concepts from the Paper

```text
Category 𝒯 (Computations)          Functor Z'          Category ℬ (Cobordisms)
─────────────────────────          ──────────          ─────────────────────────
Objects: States/Configs            ────────→           Objects: Time steps (ℕ)
Morphisms: Transitions             ────────→           Morphisms: Intervals [n,m]

                              Z'(g∘f) = Z'(g) ∘ Z'(f)
                                      ↓
                              IRREDUCIBILITY CRITERION
```

## Output

The demo produces formatted output suitable for presentations:

```text
╔══════════════════════════════════════════════════════════════════════╗
║       A FUNCTORIAL PERSPECTIVE ON COMPUTATIONAL IRREDUCIBILITY       ║
╚══════════════════════════════════════════════════════════════════════╝
    Implementation of Gorard's arXiv:2301.04690
```

Each section includes:

- Direct quotes from the paper
- Mathematical notation
- Concrete examples with computed results
- Visual representations (interval diagrams, CA evolution)

## As a Presentation

This demo is designed to be self-explanatory for someone unfamiliar with Rust:

1. **Run the demo**: `cargo run --example gorard_demo`
2. **Output is the presentation**: All mathematical content is in the console output
3. **No Rust knowledge needed**: Results speak for themselves

### Key Implementation Choices

| Paper Concept | Implementation |
| ------------- | -------------- |
| Category 𝒯 | `TuringMachine`, `ElementaryCA`, `StringRewriteSystem`, `NondeterministicTM` |
| Category ℬ | `DiscreteInterval`, `ParallelIntervals` |
| Functor Z' | `IrreducibilityFunctor`, `StepTrace` trait |
| Adjunction Z' ⊣ Z | `ZPrimeAdjunction`, `ZPrimeOps` trait |
| Coherence | `verify_associator_coherence()`, `verify_braiding_coherence()` |
| Stokes integration | `StokesIrreducibility`, `TemporalComplex` |
| Hypergraph rewriting | `Hypergraph`, `HypergraphEvolution`, `RewriteRule` |
| catgraph bridge | `RewriteRule::to_span()`, `HypergraphEvolution::to_cospan_chain()` |
| Multiway branching | `MultiwayEvolutionGraph`, `BranchialGraph`, `CurvatureFoliation` |
| Causal invariance | `analyze_causal_invariance()`, Wilson loop holonomy |
| Step cospans | `multiway_step_cospans()`, `IntervalCospanAlgebra` |
| Frobenius structure | `verify_frobenius_preservation()` |
| Merge partitions | `step_corels()`, `evolution_corel()`, `MergesCorelExt` |
| Compact closure | `ZPrimeAdjunction::verify_compact_closed_witness()` |

## Integration Test Cross-Reference

Each demo section has corresponding assertions in `tests/`:

| Demo Section | Integration Tests |
|---|---|
| 1-3. Functoriality (TM, CA, intervals) | `tests/functoriality.rs` |
| 4. Adjunction Z' ⊣ Z | `tests/adjunction_laws.rs` |
| 5. Monoidal structure | `tests/monoidal_coherence.rs` |
| 6. Coherence conditions | `tests/monoidal_coherence.rs` |
| 7. Stokes integration | `tests/stokes_integration.rs` |
| 8. Hypergraph rewriting | `tests/hypergraph_rewriting.rs`, `tests/catgraph_bridge.rs` |
| 9. Multiway branching | `tests/multiway_evolution.rs` |
| 10. Frobenius structure | `tests/frobenius_preservation.rs`, `tests/categorical_stack.rs` |
| 11. Corelation merge partition | `tests/multiway_evolution.rs`, `tests/categorical_stack.rs` |
| 12. Compact closure | `tests/adjunction_laws.rs` |

## Companion Examples

Two focused examples cover the same surfaces with assertions rather than prose:

- `cargo run --example categorical_toolkit` — the diamond SRS through step cospans, interval transport, the Frobenius census and the merge partition.
- `cargo run --example confluence_corel` — two hypergraph rule orderings compared via `coarsest_common_refinement`, with the Wilson-loop verdict beside it.

## Reference

Gorard, J. (2023). *A Functorial Perspective on (Multi)computational Irreducibility*. [arXiv:2301.04690](https://arxiv.org/abs/2301.04690)
