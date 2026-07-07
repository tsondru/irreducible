# irreducible

Computational irreducibility as functoriality in Rust, implementing Jonathan Gorard's ["A Functorial Perspective on (Multi)computational Irreducibility"](https://arxiv.org/pdf/2301.04690) (arXiv:2301.04690).

**Core insight**: A computation is irreducible iff a certain functor Z': T -> B (from computations to cobordisms) preserves composition. No shortcuts exist when Z' is functorial.

irreducible is the **example consumer of the [catgraph](https://github.com/sustia-llc/catgraph) core + physics layer** (v0.2.0): [catgraph](https://github.com/sustia-llc/catgraph) supplies the Fong-Spivak categorical infrastructure (cospans, spans, hypergraph categories, cospan-algebras), catgraph-applied the Petri-net substrate, and catgraph-physics the hypergraph DPO rewriting, multiway evolution graphs, confluence diamond detection, and branchial spectral analysis. irreducible owns the computation-facing layer -- interval algebra, adjunctions, monoidal coherence, discrete exterior calculus, trace analysis -- plus the computation models (TM, CA, SRS, NTM, Petri nets).

354 tests on default features (381 with `manifold-curvature,dec`), zero clippy warnings. Rust 2024 edition, MSRV 1.90.

## Component Index

| Module | Component | Purpose |
|--------|-----------|---------|
| `interval.rs` | `DiscreteInterval`, `ParallelIntervals` | Discrete interval algebra for the cobordism category B (re-export shim into `catgraph_physics::interval` as of v0.6.3) |
| `complexity.rs` | `Complexity`, `StepCount` | Sequential/parallel complexity composition |
| `computation_state.rs` | `ComputationState` | State lifecycle + interval-map bridge |
| `adjunction.rs` | `ZPrimeOps`, `AdjunctionIrreducibility`, `AdjunctionVerification` | Abstract Z' ⊣ Z adjunction traits |
| `bifunctor.rs` | `TensorProduct`, `IntervalTransform` | Bifunctor laws (associativity, unit, symmetry) |
| `multiway_coherence.rs` | `AssociatorWitness`, `BraidingWitness`, `CoherenceError` | Non-strict SMC coherence over multiway graphs |
| `multiway_stokes.rs` | `MultiwayComplex`, `OneForm`, `TwoForm` | Discrete exterior calculus on 2D multiway complexes (feature: `dec`) |
| `temporal_cospan_chain.rs` | `TemporalComplex`, `ConservationResult`, `TemporalComplexError` (`StokesError` deprecated alias) | Cospan chain bridge for interval sequences (re-export shim into `catgraph_physics::temporal_cospan_chain` as of v0.6.3) |
| `trace.rs` | `StepTrace`, `analyze_trace`, `RepeatDetection`, `is_irreducible` (`IrreducibilityTrace` deprecated alias) | Generic trace analysis, repeat detection (re-export shim into `catgraph_physics::trace` as of v0.6.3) |
| `functor/mod.rs` | `IrreducibilityFunctor`, `MultiwayIrreducibilityResult` | Functor Z': T -> B, multiway branch analysis |
| `functor/adjunction.rs` | `ZPrimeAdjunction`, `AdjunctionVerification` | Concrete Z' ⊣ Z adjunction for computation states |
| `functor/monoidal.rs` | `MonoidalFunctorResult`, `TensorCheck` | Symmetric monoidal functor verification |
| `functor/bifunctor.rs` | `TensorProduct`, `IntervalTransform` | Re-exports local bifunctor laws |
| `functor/fong_spivak.rs` | `FrobeniusVerificationResult`, `verify_cospan_chain_frobenius` | Fong-Spivak Frobenius decomposition verification |
| `functor/stokes_integration.rs` | `StokesIrreducibility` | Stokes conservation analysis wrapper |
| `machines/turing.rs` | `TuringMachine`, `ExecutionHistory` | Deterministic Turing machines |
| `machines/cellular_automaton.rs` | `ElementaryCA`, `Generation` | 1D elementary cellular automata (256 rules) |
| `machines/trace.rs` | `StepTrace`, `TraceAnalysis` (`IrreducibilityTrace` deprecated alias) | Generic trace analysis, repeat detection (proxy re-export of `crate::trace` shim) |
| `machines/multiway/string_rewrite.rs` | `StringRewriteSystem`, `SRSState` | Pattern-based multiway string rewriting |
| `machines/multiway/ntm.rs` | `NondeterministicTM`, `NTMBuilder` | Non-deterministic Turing machines |
| `machines/multiway/manifold_bridge.rs` | `ManifoldCurvature`, `BranchialEmbedding` | Regge deficit-angle curvature on branchial complexes via dc_topology (feature-gated) |
| `machines/petri/` | `PetriNetMachine`, `PetriBuilder`, `PetriExecutionHistory`, `run_multiway_reachability` | Place/transition Petri nets — linear trace, multiway reachability, cospan bridge |
| `machines/hypergraph/catgraph_bridge.rs` | `MultiwayCospanExt`, `MultiwayCospanGraph` | Hypergraph evolution cospan analysis |
| `types.rs` | `ComputationDomain`, `ComputationContext`, `CausalEffect` | Domain types for computation models |

## Fong-Spivak Feature Map

Re-exports from catgraph v0.2.0 implementing [Fong & Spivak, *Hypergraph Categories*](https://arxiv.org/abs/1806.08304) SS2-3:

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
| `dc-geometry` | dc_topology Regge + DEC substrate | `deep_causality_topology`, `deep_causality_tensor`, `deep_causality_sparse` |
| `dec` | Discrete exterior calculus on multiway complexes | `dc-geometry`, `nalgebra` |
| `manifold-curvature` | Regge deficit-angle curvature on branchial complexes | `dc-geometry`, `nalgebra` |
| `lapack` | LAPACK-accelerated eigendecomposition for MDS | `nalgebra-lapack` (implies `manifold-curvature`; requires `libopenblas-dev`) |

> The former `persist` feature (SurrealDB evolution-trace storage) was removed pending the catgraph-surreal reboot; see issue [#15](https://github.com/tsondru/irreducible/issues/15) for the restore path.

## Examples

```bash
cargo run --example gorard_demo           # 9-part presentation demo
cargo run --example builders              # TuringMachineBuilder + NTMBuilder
cargo run --example bifunctor_tensor      # Tensor products, monoidal laws
cargo run --example fong_spivak           # Fong-Spivak three-perspective agreement
cargo run --example lattice_gauge         # Wilson loops, plaquette action
cargo run --example multiway_coherence    # Non-confluent fragment failing coherence
cargo run --example multiway_stokes --features dec  # Closed vs non-closed 1-forms
```

## Testing

```bash
cargo test --workspace                               # 354 tests (default features)
cargo test --workspace --features manifold-curvature,dec  # 381 tests (CI feature leg)
cargo test --features dc-geometry                    # dc_topology smoke + bridge tests
cargo clippy --workspace --all-targets -- -D warnings  # CI gate
cargo clippy --workspace -- -W clippy::pedantic      # advisory, zero warnings
```

Integration suites cover: adjunction laws (Z' ⊣ Z triangle identities), the
catgraph cospan bridge, computation-model classification (TM, CA, SRS, NTM,
Petri), Fong-Spivak Frobenius verification, functor laws, hypergraph DPO
rewriting + Wilson loops, multiway coherence (associator/braiding), branchial
evolution + curvature foliation, DEC closed forms (`dec`), and temporal
cospan-chain conservation.

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

- [catgraph](https://github.com/sustia-llc/catgraph) v0.2.0 -- category theory infrastructure (cospans, spans, Fong-Spivak hypergraph categories); workspace tag shared with catgraph-applied (Petri nets) and catgraph-physics (hypergraph DPO rewriting, multiway evolution, confluence diamonds, discrete curvature, branchial spectral analysis)
- `serde` + `serde_json` -- serialization
- Optional: `deep_causality_topology` + `deep_causality_tensor` + `deep_causality_sparse` (Regge curvature + DEC substrate; currently git-rev-pinned pre-release), `nalgebra` (matrix ops), `nalgebra-lapack` (LAPACK)

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
