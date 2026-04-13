# irreducible

Computational irreducibility as functoriality in Rust, implementing Jonathan Gorard's ["A Functorial Perspective on (Multi)computational Irreducibility"](https://arxiv.org/pdf/2301.04690) (arXiv:2301.04690).

**Core insight**: A computation is irreducible iff a certain functor Z': T -> B (from computations to cobordisms) preserves composition. No shortcuts exist when Z' is functorial.

Uses [catgraph](https://github.com/tsondru/catgraph) v0.10.6 for the Fong-Spivak categorical infrastructure (cospans, spans, hypergraph DPO rewriting, multiway evolution, hypergraph categories, cospan-algebras, Thm 1.2 equivalence) and [catgraph-physics](https://github.com/tsondru/catgraph) v0.10.6 for multiway evolution graphs and confluence diamond detection. irreducible owns the computation-facing layer -- interval algebra, adjunctions, monoidal coherence, discrete exterior calculus, trace analysis -- plus the computation models (TM, CA, SRS, NTM).

369 tests (+ 8 with `dec` feature), zero clippy warnings. Rust 2024 edition.

## Component Index

| Module | Component | Purpose |
|--------|-----------|---------|
| `interval.rs` | `DiscreteInterval`, `ParallelIntervals` | Discrete interval algebra for the cobordism category B |
| `complexity.rs` | `Complexity`, `StepCount` | Sequential/parallel complexity composition |
| `computation_state.rs` | `ComputationState` | State lifecycle + interval-map bridge |
| `adjunction.rs` | `ZPrimeOps`, `AdjunctionIrreducibility`, `AdjunctionVerification` | Abstract Z' ⊣ Z adjunction traits |
| `bifunctor.rs` | `TensorProduct`, `IntervalTransform` | Bifunctor laws (associativity, unit, symmetry) |
| `multiway_coherence.rs` | `AssociatorWitness`, `BraidingWitness`, `CoherenceError` | Non-strict SMC coherence over multiway graphs |
| `multiway_stokes.rs` | `MultiwayComplex`, `OneForm`, `TwoForm` | Discrete exterior calculus on 2D multiway complexes (feature: `dec`) |
| `temporal_cospan_chain.rs` | `TemporalComplex`, `ConservationResult`, `StokesError` | Cospan chain bridge for interval sequences |
| `trace.rs` | `IrreducibilityTrace`, `analyze_trace`, `RepeatDetection` | Generic trace analysis, repeat detection |
| `functor/mod.rs` | `IrreducibilityFunctor`, `MultiwayIrreducibilityResult` | Functor Z': T -> B, multiway branch analysis |
| `functor/adjunction.rs` | `ZPrimeAdjunction`, `AdjunctionVerification` | Concrete Z' ⊣ Z adjunction for computation states |
| `functor/monoidal.rs` | `MonoidalFunctorResult`, `TensorCheck` | Symmetric monoidal functor verification |
| `functor/bifunctor.rs` | `TensorProduct`, `IntervalTransform` | Re-exports local bifunctor laws |
| `functor/fong_spivak.rs` | `FrobeniusVerificationResult`, `verify_cospan_chain_frobenius` | Fong-Spivak Frobenius decomposition verification |
| `functor/stokes_integration.rs` | `StokesIrreducibility` | Stokes conservation analysis wrapper |
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

Re-exports from catgraph v0.10.6 implementing [Fong & Spivak, *Hypergraph Categories*](https://arxiv.org/abs/1806.08304) SS2-3:

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
| Cobordism category B | `DiscreteInterval`, `ParallelIntervals` | irreducible::interval |
| Functor Z': T -> B | `IrreducibilityFunctor` | functor/mod.rs |
| Adjunction Z' ⊣ Z | `ZPrimeAdjunction`, triangle identities | functor/adjunction.rs |
| Coherence (alpha, lambda, rho, sigma) | `verify_associator`, `verify_braiding`, `verify_all_coherence` | multiway_coherence.rs |
| Stokes integration | `TemporalComplex`, `ConservationResult` | temporal_cospan_chain.rs |
| Discrete exterior calculus | `MultiwayComplex`, `OneForm`, `TwoForm` | multiway_stokes.rs (feature: dec) |
| Frobenius structure | `FrobeniusVerificationResult`, `verify_cospan_chain_frobenius` | functor/fong_spivak.rs |
| DPO rewriting as spans | `RewriteRule::to_span()` | catgraph::hypergraph |
| Evolution as cospan chain | `HypergraphEvolution::to_cospan_chain()` | catgraph::hypergraph |
| Causal invariance | Wilson loops, holonomy analysis | catgraph::hypergraph |
| Branchial curvature | `OllivierRicciCurvature` | catgraph::multiway |
| Complexity algebra | `Complexity`, `StepCount` | irreducible::complexity |

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
| *(none)* | Core library (TM, CA, SRS, NTM, functor, cobordism) | `catgraph`, `catgraph-physics`, `serde` |
| `dec` | Discrete exterior calculus on multiway complexes | `nalgebra`, `nalgebra-sparse` |
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
cargo run --example multiway_coherence    # Non-confluent fragment failing coherence
cargo run --example multiway_stokes --features dec  # Closed vs non-closed 1-forms
cargo run --example persist_evolution --features persist  # SurrealDB persistence
```

## Testing

```bash
cargo test --workspace                    # 369 tests, 0 ignored
cargo test --features dec                 # +8 DEC tests
cargo test --workspace --features persist # +15 persistence tests
cargo clippy --workspace -- -W clippy::pedantic  # zero warnings
```

| Suite | Tests | What it covers |
|-------|-------|---------------|
| Unit tests | ~190 | Functor, machines (TM, CA, SRS, NTM, trace), categories, types, multiway coherence |
| `adjunction_laws` | 11 | Z' ⊣ Z triangle identities, unit/counit naturality |
| `catgraph_bridge` | 10 | Multiway cospan analysis, composability |
| `computation_types` | 30 | TM, CA, string rewrite, NTM classification |
| `fong_spivak` | 11 | Re-export validation, Frobenius verification, three-way agreement |
| `functoriality` | 16 | Functor laws, irreducibility detection |
| `hypergraph_rewriting` | 21 | DPO matching, evolution, Wilson loops, gauge |
| `multiway_coherence` | 10 | Associator, braiding, unitor on real multiway graphs + monoidal functor |
| `multiway_evolution` | 16 | Branchial graphs, curvature foliation, NTM |
| `multiway_stokes` | 8 | DEC: closed forms, d^2=0, diamond detection (feature: dec) |
| `temporal_cospan_chain` | 11 | Cospan chain composability, conservation |
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

### Multiway Coherence (v0.4.3)

Coherence verification now operates on real multiway evolution graphs from catgraph-physics -- a genuine non-strict symmetric monoidal category where confluence up to causal equivalence is a falsifiable property. `verify_associator` checks that parallel independent events at a fork point are pairwise confluent; `verify_braiding` checks that two events commute (share a common descendant). Non-confluent fragments produce `CoherenceError::NonConfluent`.

### Discrete Exterior Calculus (v0.4.3, feature: `dec`)

Confluence diamonds in a multiway graph are 2-simplices. A 1-form assigns costs to edges; the exterior derivative `d: Omega^1 -> Omega^2` evaluates on each diamond. A closed 1-form (`d_omega = 0`) means cost is path-independent across every diamond -- the categorical manifestation of causal invariance.

### Stokes Integration and Cospan Composability

For 1D simplicial complexes, Stokes conservation reduces to contiguity + monotonicity -- exactly the conditions for cospan composability in B. This bridges differential geometry (Stokes theorem) to category theory (cospan composition).

### Frobenius Decomposition (Fong-Spivak)

`Cospan<Lambda>` implements `HypergraphCategory` (Thm 3.14). `CospanToFrobeniusFunctor` decomposes each cospan into Frobenius generators. If composition is preserved through decomposition, the cospan chain has valid Frobenius structure -- a categorical invariant complementing the Stokes and functorial perspectives.

## Dependencies

- [catgraph](https://github.com/tsondru/catgraph) v0.10.6 -- category theory infrastructure (cospans, spans, Fong-Spivak hypergraph categories, DPO rewriting)
- [catgraph-physics](https://github.com/tsondru/catgraph) v0.10.6 -- multiway evolution, confluence diamonds, discrete curvature
- `serde` + `serde_json` -- serialization
- Optional: `nalgebra` + `nalgebra-sparse` (DEC), `amari-calculus` (manifold curvature), `nalgebra-lapack` (LAPACK), `catgraph-surreal` + `surrealdb` + `tokio` (persistence)

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
