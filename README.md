# Computational Irreducibility as Functoriality

A Rust implementation of Jonathan Gorard's "A Functorial Perspective on (Multi)computational Irreducibility" ([arXiv:2301.04690](https://arxiv.org/pdf/2301.04690)), demonstrating that **computational irreducibility is equivalent to functoriality** of a map from a category of computations to a cobordism category.

Uses [catgraph](https://github.com/tsondru/catgraph) v0.7.0 for categorical infrastructure (spans, cospans, adjunctions, bifunctors, coherence, symmetric monoidal categories, hypergraph DPO rewriting, multiway evolution graphs, discrete curvature). Category theory types, hypergraph rewriting, and multiway infrastructure are defined in catgraph and re-exported transparently — irreducible is a thin domain layer adding computation models (TM, CA, SRS, NTM) and the functorial irreducibility framework.

309 tests (324 with all features), zero clippy warnings. Rust 2024 edition.

## Quick Start

```toml
[dependencies]
irreducible = { git = "https://github.com/tsondru/irreducible" }
# With SurrealDB persistence:
# irreducible = { git = "https://github.com/tsondru/irreducible", features = ["persist"] }
```

```bash
# Run the presentation demo (designed for non-Rust readers)
cargo run --example gorard_demo

# This shows:
# 1. Core insight: Irreducibility = Functoriality
# 2. Turing machines (Busy Beaver vs cycling)
# 3. Cellular automata (Rule 30 vs Rule 0)
# 4. Z' ⊣ Z Adjunction with triangle identities
# 5. Multiway systems with tensor products
# 6. Coherence conditions (α, λ, ρ, σ)
# 7. Stokes integration (conservation → cospan composability)
# 8. Hypergraph rewriting (Wolfram Physics via catgraph)
# 9. Multiway branching (branchial foliation, curvature)

# Run focused examples
cargo run --example builders              # TuringMachineBuilder + NTMBuilder
cargo run --example bifunctor_tensor      # Tensor products, monoidal laws
cargo run --example lattice_gauge         # Wilson loops, plaquette action
cargo run --example persist_evolution --features persist  # SurrealDB persistence

# Run all tests
cargo test --workspace                    # 309 tests (176 unit + 124 integration + 9 doc)
cargo test --workspace --features persist # 324 tests (+15 persistence)

# Run just the library
cargo test -p irreducible                 # Core library tests
```

## Overview

This library formalizes Wolfram's concept of computational irreducibility using category theory:

- **Computational Irreducibility**: A computation is irreducible if it cannot be shortcut — you must trace every step to get the result
- **Functoriality**: A functor preserves compositional structure exactly
- **Core Insight**: These are the same thing. A computation is irreducible iff a certain map Z': 𝒯 → ℬ is a functor

The framework extends naturally to **multicomputational irreducibility** for non-deterministic (multiway) systems via symmetric monoidal categories, and to **hypergraph rewriting** for Wolfram Physics via catgraph's span/cospan types.

## Paper Reference

- **Title**: A Functorial Perspective on (Multi)computational Irreducibility
- **Author**: Jonathan Gorard (Cardiff University & University of Cambridge)
- **Date**: October 2022
- **Link**: <https://arxiv.org/pdf/2301.04690>

---

## Alignment with Gorard's Paper

| Paper Concept | Implementation | Location |
|---|---|---|
| **Cobordism category ℬ** | `DiscreteInterval` [n,m] ⊂ ℕ | `categories/cobordism.rs` |
| **Tensor product (parallel)** | `ParallelIntervals` with `tensor()` | `categories/cobordism.rs` |
| **Functor Z': 𝒯 → ℬ** | `IrreducibilityFunctor` | `functor/mod.rs` |
| **Adjunction Z' ⊣ Z** | `ZPrimeAdjunction`, triangle identities | `functor/adjunction.rs` |
| **Coherence conditions (α,λ,ρ,σ)** | `CoherenceVerification` | `functor/monoidal.rs` |
| **Differential coherence** | `DifferentialCoherence`, categorical curvature | `functor/monoidal.rs` |
| **Bifunctor/Tensor** | `TensorProduct`, `tensor_bimap` | `functor/bifunctor.rs` |
| **Stokes integration** | `TemporalComplex`, `ConservationResult` | `functor/stokes_integration.rs` |
| **Stokes → cospan composability** | `TemporalComplex::to_cospan_chain()` | `functor/stokes_integration.rs` |
| **DPO rewriting as spans** | `RewriteRule::to_span()` → `catgraph::Span<u32>` | `catgraph::hypergraph` |
| **Evolution as cospan chain** | `HypergraphEvolution::to_cospan_chain()` | `catgraph::hypergraph` |
| **Causal invariance** | Wilson loops, holonomy analysis | `catgraph::hypergraph` |
| **Gauge group** | `HypergraphRewriteGroup`, lattice gauge | `catgraph::hypergraph` |
| **Branchial curvature** | `DiscreteCurvature` trait, `OllivierRicciCurvature` | `catgraph::multiway` |
| **Complexity algebra** | `Complexity` trait, `StepCount` | `categories/complexity.rs` |
| **Turing machines** | `TuringMachine`, `ExecutionHistory` | `machines/turing.rs` |
| **Cellular automata (1D)** | `ElementaryCA`, `Generation` | `machines/cellular_automaton.rs` |
| **Multiway systems** | `MultiwayEvolutionGraph`, `BranchialGraph` | `catgraph::multiway` |
| **String rewriting** | `StringRewriteSystem`, `SRSState` | `machines/multiway/string_rewrite.rs` |
| **Non-deterministic TM** | `NondeterministicTM`, `NTMBuilder` | `machines/multiway/ntm.rs` |
| **Symmetric monoidal functor** | `MonoidalFunctorResult`, `CoherenceVerification` | `functor/monoidal.rs` |

---

## Key Concepts

### The Categories

| Category | Objects | Morphisms | Composition |
|---|---|---|---|
| **𝒯** (Computation) | Data structures / states | Computations / transitions | Sequential execution |
| **ℬ** (Cobordism) | Step numbers (ℕ) | Discrete intervals [n,m] ∩ ℕ | Union of contiguous intervals |

### The Functor Z'

```text
Z': 𝒯 → ℬ

Maps:
  - States → Step numbers
  - Computations → Intervals of steps traversed
```

**Theorem**: Z' is a functor ⟺ all computations in 𝒯 are irreducible

### Multiway Extension

For non-deterministic systems, both categories gain symmetric monoidal structure:

- **𝒯**: Tensor product ⊗ represents parallel computation branches
- **ℬ**: Coproduct ⊕ represents disjoint union of intervals

**Theorem**: Z' is a symmetric monoidal functor ⟺ the system is multicomputationally irreducible

---

## Categorical Hypergraph Rewriting (catgraph Bridge)

The hypergraph rewriting module connects Gorard's irreducibility framework to Wolfram Physics via catgraph's categorical types.

### DPO Rewrite Rules as Spans

A Double-Pushout (DPO) rewrite rule is naturally a span L ← K → R:

```text
    L ←── K ──→ R
    │           │
    left        right
    pattern     replacement
```

where K is the kernel (preserved vertices). `RewriteRule::to_span()` produces a `catgraph::Span<()>` encoding this structure:

```rust
use irreducible::machines::hypergraph::RewriteRule;

let rule = RewriteRule::wolfram_a_to_bb();  // {0,1,2} → {0,1},{1,2}
let span = rule.to_span();
// |L| = 3, |R| = 3, |K| = 3 (all preserved)
```

### Evolution as Cospan Chain

Each rewrite step Gᵢ → Gᵢ₊₁ produces a cospan (pushout). The full evolution is a composable chain:

```rust
use irreducible::machines::hypergraph::{Hypergraph, HypergraphEvolution, RewriteRule};

let initial = Hypergraph::from_edges(vec![vec![0, 1, 2]]);
let evolution = HypergraphEvolution::run(&initial, &[RewriteRule::wolfram_a_to_bb()], 5);
let cospans = evolution.to_cospan_chain();  // Vec<catgraph::Cospan<()>>
```

### Causal Invariance via Wilson Loops

Causal invariance — the property that rewrite order doesn't matter — manifests as holonomy = 1 for all Wilson loops. When this holds, different orderings produce equivalent cospan chains, the categorical manifestation of gauge invariance.

---

## Stokes Integration → Cospan Composability

A novel contribution: for 1-dimensional simplicial complexes, the exterior derivative dω is vacuously zero (no 2-simplices). Stokes conservation therefore reduces to:

1. **Contiguity** — intervals connect without gaps
2. **Monotonicity** — time flows forward

These are exactly the conditions for **cospan composability** in the cobordism category ℬ. This bridges differential geometry (Stokes theorem) to category theory (cospan composition) through catgraph:

```rust
use irreducible::functor::StokesIrreducibility;
use irreducible::DiscreteInterval;

let intervals = vec![
    DiscreteInterval::new(0, 2),
    DiscreteInterval::new(2, 5),
    DiscreteInterval::new(5, 7),
];

let analysis = StokesIrreducibility::analyze(&intervals).unwrap();
let cospans = analysis.to_cospan_chain();  // 3 composable cospans in ℬ
assert!(analysis.is_irreducible());
```

---

## The Adjunction Z' ⊣ Z

From the paper (Section 4.2): "The existence of the adjunction Z' ⊣ Z encodes a kind of 'quantum duality' between computation and time."

```text
Z' ⊣ Z

Where:
  Z': 𝒯 → ℬ  (computation states → time intervals)
  Z : ℬ → 𝒯  (time intervals → computation states)

Triangle identities:
  ε_{Z'X} ∘ Z'(η_X) = id_{Z'X}
  Z(ε_Y) ∘ η_{ZY} = id_{ZY}
```

**Key Insight**: Computational irreducibility is *dual/adjoint* to locality of time evolution in quantum mechanics. For multiway systems, Z' is adjoint to the functor defining functorial quantum field theory (Atiyah-Segal axioms).

---

## The Orthogonality Principle

A key insight from Gorard: **computational irreducibility** and **multicomputational irreducibility** are orthogonal:

| Type | Source | Depends On |
|---|---|---|
| **Computational** | State evolution function | Which states follow which |
| **Multicomputational** | State equivalence function | Which states are "the same" |

You can have systems that are computationally irreducible but multicomputationally reducible, or vice versa, or both, or neither.

---

## Architecture

```text
irreducible/
├── src/
│   ├── lib.rs                        # Re-exports
│   ├── types.rs                      # ComputationDomain, ComputationContext, CausalEffect
│   ├── categories/                   # Re-exports from catgraph (intervals, complexity, state)
│   ├── functor/
│   │   ├── mod.rs                    # IrreducibilityFunctor
│   │   ├── adjunction.rs             # ZPrimeAdjunction, triangle identities
│   │   ├── monoidal.rs              # Symmetric monoidal functor verification
│   │   ├── bifunctor.rs             # Re-exports catgraph::bifunctor
│   │   └── stokes_integration.rs    # StokesIrreducibility wrapper
│   └── machines/
│       ├── turing.rs                 # TuringMachine, ExecutionHistory
│       ├── cellular_automaton.rs     # ElementaryCA, Generation
│       ├── trace.rs                  # IrreducibilityTrace trait, TraceAnalysis
│       ├── multiway/                 # Re-exports catgraph::multiway + local models
│       │   ├── string_rewrite.rs     # StringRewriteSystem, SRSState (local)
│       │   ├── ntm.rs               # NondeterministicTM, NTMBuilder (local)
│       │   └── manifold_bridge.rs   # ManifoldCurvature (feature-gated, local)
│       └── hypergraph/              # Re-exports catgraph::hypergraph + local types
│           ├── catgraph_bridge.rs    # MultiwayCospanExt trait (local)
│           └── persistence.rs       # EvolutionPersistence (feature = "persist")
├── tests/                            # Integration tests (124 tests)
└── examples/
    ├── gorard_demo.rs                # 9-part presentation demo
    ├── builders.rs                   # TuringMachineBuilder + NTMBuilder
    ├── bifunctor_tensor.rs           # Tensor products, monoidal laws
    ├── lattice_gauge.rs              # Wilson loops, plaquette action
    └── persist_evolution.rs          # Persistence lifecycle (feature-gated)
```

**catgraph provides:** Hypergraph rewriting (DPO, evolution, Wilson loops, gauge theory), multiway evolution graphs (BFS, branchial foliation, Ollivier-Ricci curvature, Wasserstein transport), intervals, spans, cospans, coherence, adjunctions, Stokes integration, Petri nets.

---

## Testing

```bash
cargo test --workspace                    # 309 tests, zero ignored
cargo test --workspace --features persist # 324 tests (+15 persistence)
cargo clippy -- -W clippy::pedantic       # zero warnings
```

| Suite | Tests | What it covers |
|-------|-------|---------------|
| Unit tests (src/) | 176 | Functor, machines (TM, CA, SRS, NTM, trace), categories, types |
| `adjunction_laws` | 11 | Z' ⊣ Z triangle identities, unit/counit naturality |
| `catgraph_bridge` | 10 | Multiway cospan analysis, composability |
| `computation_types` | 22 | TM, CA, string rewrite, NTM classification |
| `functoriality` | 13 | Functor laws, irreducibility detection |
| `hypergraph_rewriting` | 20 | DPO matching, evolution, Wilson loops, gauge |
| `monoidal_coherence` | 11 | Associator, unitors, braiding, pentagon/hexagon |
| `multiway_evolution` | 14 | Branchial graphs, curvature foliation, NTM |
| `persistence` | 8 | SurrealDB cospan/span roundtrip, multiway, isolation |
| `property_coherence` | 10 | Coherence verification, differential coherence |
| `stokes_integration` | 13 | Temporal complex, Stokes conservation, cospan bridge |
| Doc tests | 9 | Public API examples |
| Persistence unit (feature-gated) | 7 | Unit tests in persistence.rs |
| **Total** | **324** | |

Hypergraph and multiway infrastructure unit tests (~130) now run in catgraph's own test suite.

---

## Dependencies

### Core

- [catgraph](https://github.com/tsondru/catgraph) v0.7.0 -- category theory (cospans, spans, hypergraph rewriting, multiway evolution, discrete curvature, symmetric monoidal categories)
- `serde` + `serde_json` -- serialization

### Optional (`persist` feature)

- [catgraph-surreal](https://github.com/tsondru/catgraph) -- SurrealDB persistence for catgraph types
- `surrealdb` 3.0.4 (kv-mem) -- embedded SurrealDB
- `tokio` -- async runtime for persistence layer

### Optional (`manifold-curvature` feature)

- `amari-calculus` -- Riemannian manifold curvature via branchial graph embedding

---

## Deferred Work

- **Visualization** -- multiway graphs, branchial graphs, curvature heatmaps
- **Lambda calculus** -- computational model for lambda-term reduction
- **Rule classification** -- systematic analysis of all 256 elementary CA rules
- **Proptest functor laws** -- property-based verification of Z'(g∘f) = Z'(g) ∘ Z'(f) over random inputs (coherence/adjunction proptests exist in `property_coherence.rs`)

---

## Contributors

- [tsondru](https://github.com/tsondru)
- [Claude](https://anthropic.com) (Anthropic)

---

## References

### Primary

- Gorard (2022): "A Functorial Perspective on (Multi)computational Irreducibility" — [arXiv:2301.04690](https://arxiv.org/pdf/2301.04690)

### Categorical Infrastructure

- Fong & Spivak: "Hypergraph Categories" — [arXiv:1806.08304](https://arxiv.org/abs/1806.08304)
- Mac Lane (1998): *Categories for the Working Mathematician*
- catgraph: Cospans, spans, wiring diagrams, Frobenius algebras

### Background on Computational Irreducibility

- Wolfram (2002): *A New Kind of Science*
- Gorard (2018): "The Slowdown Theorem"

### Quantum Mechanics Connection

- Abramsky & Coecke (2004): "A categorical semantics of quantum protocols"
- Atiyah (1988): "Topological quantum field theories"

---

## License

[MIT license](LICENSE).
