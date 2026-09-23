# irreducible

Computational irreducibility as functoriality in Rust, implementing Jonathan Gorard's ["A Functorial Perspective on (Multi)computational Irreducibility"](https://arxiv.org/pdf/2301.04690) (arXiv:2301.04690).

**Core insight**: A computation is irreducible iff a certain functor Z': T -> B (from computations to cobordisms) preserves composition. No shortcuts exist when Z' is functorial.

irreducible is the **example consumer of the [catgraph](https://github.com/sustia-llc/catgraph) core + physics layer** (v0.24.0): [catgraph](https://github.com/sustia-llc/catgraph) supplies the Fong-Spivak categorical infrastructure (cospans, spans, hypergraph categories, cospan-algebras), catgraph-applied the Petri-net substrate, and catgraph-physics the hypergraph DPO rewriting, multiway evolution graphs, confluence diamond detection, and branchial spectral analysis. irreducible owns the computation-facing layer -- interval algebra, adjunctions, monoidal coherence, discrete exterior calculus, trace analysis -- plus the computation models (TM, CA, SRS, NTM, Petri nets).

Zero clippy warnings at `-D warnings`. Rust 2024 edition, MSRV 1.98 (measured cross-feature maximum; the default feature set builds on 1.90, the `persist` tiers need 1.94, the `dc-geometry` tiers 1.98).

## Component Index

| Module | Component | Purpose |
|--------|-----------|---------|
| `complexity.rs` | `Complexity`, `StepCount` | Sequential/parallel complexity composition |
| `computation_state.rs` | `ComputationState` | State lifecycle + interval-map bridge |
| `adjunction.rs` | `ZPrimeOps`, `AdjunctionIrreducibility`, `AdjunctionVerification` | Abstract Z' ⊣ Z adjunction traits |
| `bifunctor.rs` | `TensorProduct`, `IntervalTransform` | Bifunctor laws (associativity, unit, symmetry) |
| `multiway_coherence.rs` | `AssociatorWitness`, `BraidingWitness`, `CoherenceError` | Non-strict SMC coherence over multiway graphs |
| `multiway_stokes.rs` | `MultiwayComplex`, `OneForm`, `TwoForm` | Discrete exterior calculus on 2D multiway complexes (feature: `dec`) |
| `functor/mod.rs` | `IrreducibilityFunctor`, `MultiwayIrreducibilityResult` | Functor Z': T -> B, multiway branch analysis |
| `functor/adjunction.rs` | `ZPrimeAdjunction`, `AdjunctionVerification`, `CompactClosedWitness` | Concrete Z' ⊣ Z adjunction + compact-closed (cup/cap zigzag, Prop 3.2) witnesses |
| `functor/monoidal.rs` | `MonoidalFunctorResult`, `TensorCheck` | Symmetric monoidal functor verification |
| `functor/interval_algebra.rs` | `IntervalCospanAlgebra`, `multiway_step_cospans` | Z' as a cospan-algebra (F&S Def 2.2): interval-bundle transport over multiway step cospans |
| `functor/frobenius_preservation.rs` | `FrobeniusPreservationResult`, `verify_frobenius_preservation` | Z' as a hypergraph functor: Eq. 12 generator preservation + per-event spider factorization (μ = merge, δ = fork, ε = death) |
| `functor/corel.rs` | `Corel`, `step_corels`, `evolution_corel` | Merge partitions as corelations (F&S 2018 Ex 6.64) with fingerprint gluing; hypergraph-side `MergesCorelExt` adds `coarsest_common_refinement` confluence comparison |
| `functor/bifunctor.rs` | `TensorProduct`, `IntervalTransform` | Re-exports local bifunctor laws |
| `functor/fong_spivak.rs` | `FrobeniusVerificationResult`, `verify_cospan_chain_frobenius` | Fong-Spivak Frobenius decomposition verification |
| `functor/stokes_integration.rs` | `StokesIrreducibility` | Stokes conservation analysis wrapper |
| `machines/turing.rs` | `TuringMachine`, `ExecutionHistory` | Deterministic Turing machines |
| `machines/cellular_automaton.rs` | `ElementaryCA`, `Generation` | 1D elementary cellular automata (256 rules) |
| `machines/multiway/string_rewrite.rs` | `StringRewriteSystem`, `SRSState` | Pattern-based multiway string rewriting |
| `machines/multiway/ntm.rs` | `NondeterministicTM`, `NTMBuilder` | Non-deterministic Turing machines |
| `machines/multiway/manifold_bridge.rs` | `ManifoldCurvature`, `BranchialEmbedding` | Regge deficit-angle curvature on branchial complexes via dc_topology (feature-gated) |
| `machines/petri/` | `PetriNetMachine`, `PetriBuilder`, `PetriExecutionHistory`, `run_multiway_reachability` | Place/transition Petri nets — linear trace, multiway reachability, cospan bridge |
| `machines/hypergraph/catgraph_bridge.rs` | `MultiwayCospanExt`, `MultiwayCospanGraph` | Hypergraph evolution cospan analysis |
| `types.rs` | `ComputationDomain`, `ComputationContext`, `CausalEffect` | Domain types for computation models |

## Fong-Spivak Feature Map

Re-exports from catgraph v0.24.0 implementing [Fong & Spivak, *Hypergraph Categories*](https://arxiv.org/abs/1806.08304) SS2-3:

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
| Cobordism category B | `DiscreteInterval`, `ParallelIntervals` | catgraph_physics::interval |
| Functor Z': T -> B | `IrreducibilityFunctor` | functor/mod.rs |
| Adjunction Z' ⊣ Z | `ZPrimeAdjunction`, triangle identities | functor/adjunction.rs |
| Coherence (alpha, lambda, rho, sigma) | `verify_associator`, `verify_braiding`, `verify_all_coherence` | multiway_coherence.rs |
| Stokes integration | `TemporalComplex`, `ConservationResult` | catgraph_physics::temporal_cospan_chain |
| Discrete exterior calculus | `MultiwayComplex`, `OneForm`, `TwoForm` | multiway_stokes.rs (feature: dec) |
| Frobenius structure | `FrobeniusVerificationResult`, `verify_cospan_chain_frobenius` | functor/fong_spivak.rs |
| DPO rewriting as spans | `RewriteRule::to_span()` | catgraph_physics::hypergraph |
| Evolution as cospan chain | `HypergraphEvolution::to_cospan_chain()` | catgraph_physics::hypergraph |
| Causal invariance | Wilson loops, holonomy analysis | catgraph_physics::hypergraph |
| Branchial curvature | `OllivierRicciCurvature` | catgraph_physics::multiway |
| Complexity algebra | `Complexity`, `StepCount` | irreducible::complexity |

## Quick Start

```toml
[dependencies]
irreducible = { git = "https://github.com/tsondru/irreducible" }
```

```rust
use irreducible::machines::{TuringMachine, Direction};
use irreducible::analyze_trace;

let bb = TuringMachine::busy_beaver_2_2();
let history = bb.run("", 20);
let analysis = analyze_trace(&history);
assert!(analysis.is_contiguous_without_repeats);
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

// 1. Functorial: consecutive intervals compose under Eq 12
let functorial = IrreducibilityFunctor::is_composable_chain(&intervals);

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
| `dc-geometry` | dc_topology Regge + DEC substrate | `deep_causality_topology`, `deep_causality_tensor`, `deep_causality_linear` |
| `dec` | Discrete exterior calculus on multiway complexes | `dc-geometry`, `nalgebra` |
| `manifold-curvature` | Regge deficit-angle curvature on branchial complexes | `dc-geometry`, `nalgebra` |
| `lapack` | LAPACK-accelerated eigendecomposition for MDS | `nalgebra-lapack` (implies `manifold-curvature`; requires `libopenblas-dev`) |
| `persist` | SurrealDB storage for cospan chains (`machines::hypergraph::persistence`) | `catgraph-surreal` (no engine — add one of the two below) |
| `persist-mem` | `persist` on the in-memory engine, endpoint `memory` | `catgraph-surreal/mem` |
| `persist-rocksdb` | `persist` on the durable RocksDB engine, endpoint `rocksdb://path` | `catgraph-surreal/rocksdb` |

`persist` brings in `EvolutionPersistence`, which stores a `Vec<Cospan<u32>>`
— from `HypergraphEvolution::to_cospan_chain` or `TemporalComplex::to_cospan_chain`
— and loads it back. It selects no storage engine on its own, so a build needs
`persist-mem` (endpoint `memory`, the store lives and dies with the process) or
`persist-rocksdb` (endpoint `rocksdb://<path>`). The surface is async; the
library takes no runtime dependency, so the caller supplies one.

## Examples

```bash
cargo run --example gorard_demo           # 12-part presentation demo
cargo run --example builders              # TuringMachineBuilder + NTMBuilder
cargo run --example bifunctor_tensor      # Tensor products, monoidal laws
cargo run --example fong_spivak           # Fong-Spivak three-perspective agreement
cargo run --example lattice_gauge         # Wilson loops, plaquette action
cargo run --example multiway_coherence    # Non-confluent fragment failing coherence
cargo run --example multiway_stokes --features dec  # Closed vs non-closed 1-forms
cargo run --example persist_evolution --features persist-mem  # Store and reload a cospan chain
```

## Testing

```bash
cargo test --workspace                               # default features
cargo test --workspace --features manifold-curvature,dec  # CI feature leg
cargo test --features dc-geometry                    # dc_topology smoke + bridge tests
cargo test --workspace --features persist-mem        # persistence against the in-memory engine
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

- [catgraph](https://github.com/sustia-llc/catgraph) v0.24.0 -- category theory infrastructure (cospans, spans, Fong-Spivak hypergraph categories); workspace tag shared with catgraph-applied (Petri nets) and catgraph-physics (hypergraph DPO rewriting, multiway evolution, confluence diamonds, discrete curvature, branchial spectral analysis)
- `serde` + `serde_json` -- serialization
- Optional: `deep_causality_topology` 0.10 + `deep_causality_tensor` 0.5 + `deep_causality_linear` 0.1 (Regge curvature + DEC substrate), `nalgebra` (matrix ops), `nalgebra-lapack` (LAPACK)

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
