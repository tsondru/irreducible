# irreducible

Computational irreducibility as functoriality in Rust, implementing Jonathan Gorard's ["A Functorial Perspective on (Multi)computational Irreducibility"](https://arxiv.org/pdf/2301.04690) (arXiv:2301.04690).

**Core insight**: A computation is irreducible iff a certain functor Z': T -> B (from computations to cobordisms) preserves composition. No shortcuts exist when Z' is functorial.

Uses [catgraph](https://github.com/tsondru/catgraph) v0.10.1 for categorical infrastructure (cospans, spans, adjunctions, coherence, hypergraph DPO rewriting, multiway evolution, Fong-Spivak hypergraph categories). irreducible adds computation models (TM, CA, SRS, NTM) and the functorial irreducibility framework.

310 tests (325 with all features), zero clippy warnings. Rust 2024 edition.

## Component Index

| Module | Component | Purpose |
|--------|-----------|---------|
| `functor/mod.rs` | `IrreducibilityFunctor`, `MultiwayIrreducibilityResult` | Functor Z': T -> B, multiway branch analysis |
| `functor/adjunction.rs` | `ZPrimeAdjunction`, `AdjunctionVerification` | Z' ⊣ Z adjunction, triangle identities |
| `functor/monoidal.rs` | `MonoidalFunctorResult`, `CoherenceVerification` | Symmetric monoidal functor check (alpha, lambda, rho, sigma) |
| `functor/bifunctor.rs` | `TensorProduct`, `IntervalTransform` | Re-exports catgraph bifunctor laws |
| `functor/fong_spivak.rs` | `FrobeniusVerificationResult`, `verify_cospan_chain_frobenius` | Fong-Spivak Frobenius decomposition verification |
| `functor/stokes_integration.rs` | `StokesIrreducibility`, `TemporalComplex` | Stokes conservation analysis, cospan bridge |
| `machines/turing.rs` | `TuringMachine`, `ExecutionHistory` | Deterministic Turing machines |
| `machines/cellular_automaton.rs` | `ElementaryCA`, `Generation` | 1D elementary cellular automata (256 rules) |
| `machines/trace.rs` | `IrreducibilityTrace`, `TraceAnalysis` | Generic trace analysis, repeat detection |
| `machines/multiway/string_rewrite.rs` | `StringRewriteSystem`, `SRSState` | Pattern-based multiway string rewriting |
| `machines/multiway/ntm.rs` | `NondeterministicTM`, `NTMBuilder` | Non-deterministic Turing machines |
| `machines/multiway/manifold_bridge.rs` | `ManifoldCurvature`, `BranchialEmbedding` | Riemannian curvature via MDS (feature-gated) |
| `machines/hypergraph/catgraph_bridge.rs` | `MultiwayCospanExt`, `MultiwayCospanGraph` | Hypergraph evolution cospan analysis |
| `machines/hypergraph/persistence.rs` | `EvolutionPersistence` | SurrealDB persistence (feature-gated) |
| `types.rs` | `ComputationDomain`, `ComputationContext`, `CausalEffect` | Domain types for computation models |

## Fong-Spivak Feature Map

Re-exports from catgraph v0.10.1 implementing [Fong & Spivak, *Hypergraph Categories*](https://arxiv.org/abs/1806.08304) SS2-3:

| Paper Reference | Re-exported Type | Purpose |
|-----------------|------------------|---------|
| Def 2.12 | `HypergraphCategory` | Symmetric monoidal + Frobenius structure (eta, epsilon, mu, delta) |
| Def 2.2 | `CospanAlgebra`, `PartitionAlgebra`, `NameAlgebra` | Lax monoidal functors Cospan -> Set |
| Def 2.12, Eq 12 | `HypergraphFunctor`, `RelabelingFunctor` | Structure-preserving maps between hypergraph categories |
| Prop 3.8 | `CospanToFrobeniusFunctor` | Decomposes cospans into Frobenius generators |
| SS3.1 | `cup`, `cap`, `name`, `unname` | Self-dual compact closed structure |
| Thm 3.14 | `Cospan<Lambda>: HypergraphCategory` | Free hypergraph category |

**Frobenius verification**: `verify_cospan_chain_frobenius()` decomposes each cospan in a chain via `CospanToFrobeniusFunctor` and checks that composition is preserved -- a stronger categorical check than monoidal coherence.

## Alignment with Gorard's Paper

| Paper Concept | Implementation | Location |
|---|---|---|
| Cobordism category B | `DiscreteInterval`, `ParallelIntervals` | catgraph::interval |
| Functor Z': T -> B | `IrreducibilityFunctor` | functor/mod.rs |
| Adjunction Z' ⊣ Z | `ZPrimeAdjunction`, triangle identities | functor/adjunction.rs |
| Coherence (alpha, lambda, rho, sigma) | `CoherenceVerification`, `DifferentialCoherence` | functor/monoidal.rs |
| Stokes integration | `TemporalComplex`, `ConservationResult` | functor/stokes_integration.rs |
| Frobenius structure | `FrobeniusVerificationResult`, `verify_cospan_chain_frobenius` | functor/fong_spivak.rs |
| DPO rewriting as spans | `RewriteRule::to_span()` | catgraph::hypergraph |
| Evolution as cospan chain | `HypergraphEvolution::to_cospan_chain()` | catgraph::hypergraph |
| Causal invariance | Wilson loops, holonomy analysis | catgraph::hypergraph |
| Branchial curvature | `OllivierRicciCurvature` | catgraph::multiway |
| Complexity algebra | `Complexity`, `StepCount` | catgraph::complexity |

## Quick Start

```toml
[dependencies]
irreducible = { git = "https://github.com/tsondru/irreducible" }
```

```rust
use irreducible::machines::{TuringMachine, Direction};
use irreducible::machines::trace::analyze_trace;

let bb = TuringMachine::busy_beaver_2_2();
let history = bb.run("", 20);
let analysis = analyze_trace(&history);
assert!(analysis.is_irreducible);
assert_eq!(analysis.step_count, 6);
```

### Three Perspectives on Irreducibility

```rust
use irreducible::{
    IrreducibilityFunctor, StokesIrreducibility, TuringMachine,
};

let bb = TuringMachine::busy_beaver_2_2();
let history = bb.run("", 20);
let intervals = history.to_intervals();

// 1. Functorial: contiguous intervals under Z'
let functorial = IrreducibilityFunctor::is_sequence_irreducible(&intervals);

// 2. Stokes: conservation laws hold
let stokes = StokesIrreducibility::analyze(&intervals).unwrap();
let stokes_ok = stokes.is_irreducible();

// 3. Frobenius: valid decomposition into generators (Fong-Spivak)
let frobenius = stokes.verify_frobenius();
let frobenius_ok = frobenius.all_valid && frobenius.composition_preserved;

assert!(functorial && stokes_ok && frobenius_ok); // all agree
```

## Feature Flags

| Feature | Gates | Dependencies |
|---------|-------|--------------|
| *(none)* | Core library (TM, CA, SRS, NTM, functor, cobordism) | `catgraph`, `serde` |
| `manifold-curvature` | Riemannian manifold curvature via MDS embedding | `amari-calculus`, `nalgebra` |
| `lapack` | LAPACK-accelerated eigendecomposition for MDS | `nalgebra-lapack` (implies `manifold-curvature`; requires `libopenblas-dev`) |
| `persist` | SurrealDB persistence for evolution traces | `catgraph-surreal`, `surrealdb`, `tokio` |

## Examples

```bash
cargo run --example gorard_demo           # 9-part presentation demo
cargo run --example builders              # TuringMachineBuilder + NTMBuilder
cargo run --example bifunctor_tensor      # Tensor products, monoidal laws
cargo run --example fong_spivak           # Fong-Spivak three-perspective agreement
cargo run --example lattice_gauge         # Wilson loops, plaquette action
cargo run --example persist_evolution --features persist  # SurrealDB persistence
```

## Testing

```bash
cargo test --workspace                    # 310 tests (152 unit + 149 integration + 9 doc)
cargo test --workspace --features persist # 325 tests (+15 persistence)
cargo clippy --workspace -- -W clippy::pedantic  # zero warnings
```

| Suite | Tests | What it covers |
|-------|-------|---------------|
| Unit tests | 152 | Functor, machines (TM, CA, SRS, NTM, trace), categories, types |
| `adjunction_laws` | 11 | Z' ⊣ Z triangle identities, unit/counit naturality |
| `catgraph_bridge` | 10 | Multiway cospan analysis, composability |
| `computation_types` | 30 | TM, CA, string rewrite, NTM classification |
| `fong_spivak` | 11 | Re-export validation, Frobenius verification, three-way agreement |
| `functoriality` | 16 | Functor laws, irreducibility detection |
| `hypergraph_rewriting` | 21 | DPO matching, evolution, Wilson loops, gauge |
| `monoidal_coherence` | 11 | Associator, unitors, braiding, pentagon/hexagon |
| `multiway_evolution` | 16 | Branchial graphs, curvature foliation, NTM |
| `property_coherence` | 10 | Coherence verification, differential coherence |
| `stokes_integration` | 13 | Temporal complex, Stokes conservation, cospan bridge |
| Doc tests | 9 | Public API examples |

## Key Concepts

### The Categories

| Category | Objects | Morphisms | Composition |
|---|---|---|---|
| T (Computation) | Data structures / states | Computations / transitions | Sequential execution |
| B (Cobordism) | Step numbers (N) | Discrete intervals [n,m] | Union of contiguous intervals |

### The Functor Z'

Z' maps states to step numbers and computations to intervals. **Theorem**: Z' is a functor iff all computations in T are irreducible.

For non-deterministic (multiway) systems, both categories gain symmetric monoidal structure. **Theorem**: Z' is a symmetric monoidal functor iff the system is multicomputationally irreducible.

### The Adjunction Z' ⊣ Z

From the paper (Section 4.2): computational irreducibility is *dual/adjoint* to locality of time evolution in quantum mechanics. For multiway systems, Z' is adjoint to the functor defining functorial quantum field theory (Atiyah-Segal axioms).

### Stokes Integration and Cospan Composability

For 1D simplicial complexes, Stokes conservation reduces to contiguity + monotonicity -- exactly the conditions for cospan composability in B. This bridges differential geometry (Stokes theorem) to category theory (cospan composition).

### Frobenius Decomposition (Fong-Spivak)

`Cospan<Lambda>` implements `HypergraphCategory` (Thm 3.14). `CospanToFrobeniusFunctor` decomposes each cospan into Frobenius generators. If composition is preserved through decomposition, the cospan chain has valid Frobenius structure -- a categorical invariant complementing the Stokes and functorial perspectives.

## Dependencies

- [catgraph](https://github.com/tsondru/catgraph) v0.10.1 -- category theory infrastructure (cospans, spans, Fong-Spivak hypergraph categories, DPO rewriting, multiway evolution, discrete curvature)
- `serde` + `serde_json` -- serialization
- Optional: `amari-calculus` (manifold curvature), `nalgebra` / `nalgebra-lapack` (linear algebra), `catgraph-surreal` + `surrealdb` + `tokio` (persistence)

## References

- Gorard (2022): ["A Functorial Perspective on (Multi)computational Irreducibility"](https://arxiv.org/pdf/2301.04690)
- Fong & Spivak (2019): ["Hypergraph Categories"](https://arxiv.org/abs/1806.08304)
- Mac Lane (1998): *Categories for the Working Mathematician*
- Wolfram (2002): *A New Kind of Science*
- Abramsky & Coecke (2004): "A categorical semantics of quantum protocols"
- Atiyah (1988): "Topological quantum field theories"

## Contributors

- [tsondru](https://github.com/tsondru)
- [Claude](https://anthropic.com) (Anthropic)

## License

[MIT](LICENSE)
