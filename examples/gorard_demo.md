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
| Functor Z' | `IrreducibilityFunctor`, `IrreducibilityTrace` trait |
| Adjunction Z' ⊣ Z | `ZPrimeAdjunction`, `ZPrimeOps` trait |
| Coherence | `verify_associator_coherence()`, `verify_braiding_coherence()` |
| Stokes integration | `StokesIrreducibility`, `TemporalComplex` |
| Hypergraph rewriting | `Hypergraph`, `HypergraphEvolution`, `RewriteRule` |
| catgraph bridge | `RewriteRule::to_span()`, `HypergraphEvolution::to_cospan_chain()` |
| Multiway branching | `MultiwayEvolutionGraph`, `BranchialGraph`, `CurvatureFoliation` |
| Causal invariance | `analyze_causal_invariance()`, Wilson loop holonomy |

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

## Reference

Gorard, J. (2023). *A Functorial Perspective on (Multi)computational Irreducibility*. [arXiv:2301.04690](https://arxiv.org/abs/2301.04690)
