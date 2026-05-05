# irreducible - Computational Irreducibility as Functoriality

## Project Overview

**irreducible** implements Jonathan Gorard's "A Functorial Perspective on (Multi)computational Irreducibility" (arXiv:2301.04690), using [catgraph](https://github.com/tsondru/catgraph) for categorical infrastructure (spans, cospans, symmetric monoidal categories).

**Core insight**: Computational irreducibility is equivalent to functoriality of Z': T -> B, a map from a category of computations (T) to a cobordism category (B). A computation is irreducible iff Z' preserves composition -- no shortcuts exist.

## Workspace Structure

```
irreducible/                            # Workspace root
├── Cargo.toml                          # Workspace root + library package
├── src/
│   ├── lib.rs                          # Library exports (all modules pub)
│   ├── types.rs                        # ComputationDomain, ComputationContext, CausalEffect
│   ├── test_utils.rs                   # Shared test helpers (cfg(test) only)
│   ├── categories/
│   │   ├── cobordism.rs                # Re-exports catgraph::interval (DiscreteInterval, ParallelIntervals)
│   │   ├── complexity.rs               # Re-exports catgraph::complexity (Complexity, StepCount)
│   │   └── computation_state.rs        # Re-exports catgraph::computation_state (ComputationState)
│   ├── multiway_coherence.rs            # Non-strict SMC coherence over multiway graphs
│   ├── multiway_stokes.rs               # Discrete exterior calculus on 2D multiway complex (feature: dec)
│   ├── temporal_cospan_chain.rs          # Cospan chain bridge for interval sequences
│   ├── functor/
│   │   ├── mod.rs                      # IrreducibilityFunctor, MultiwayIrreducibilityResult
│   │   ├── adjunction.rs              # ZPrimeAdjunction + re-exports catgraph::adjunction
│   │   ├── monoidal.rs                # MonoidalFunctorResult, TensorCheck, monoidal functor verification
│   │   ├── bifunctor.rs               # Re-exports catgraph::bifunctor (TensorProduct, etc.)
│   │   ├── fong_spivak.rs             # Fong-Spivak re-exports + FrobeniusVerificationResult, verify_cospan_chain_frobenius
│   │   └── stokes_integration.rs      # StokesIrreducibility wrapper
│   └── machines/
│       ├── mod.rs                      # Machine re-exports, State type alias
│       ├── turing.rs                   # TuringMachine, ExecutionHistory, TuringMachineBuilder
│       ├── cellular_automaton.rs       # ElementaryCA, Generation, CAExecutionHistory
│       ├── trace.rs                    # StepTrace trait (IrreducibilityTrace deprecated alias), TraceAnalysis, RepeatDetection
│       ├── configuration.rs            # Configuration (instantaneous TM description)
│       ├── tape.rs                     # Tape, Symbol
│       ├── transition.rs              # Direction, Transition
│       ├── multiway/
│       │   ├── mod.rs                 # Re-exports from catgraph::multiway + local models
│       │   ├── string_rewrite.rs      # StringRewriteSystem, SrsRewriteRule, SRSState (local)
│       │   ├── ntm.rs                 # NondeterministicTM, NTMBuilder (local)
│       │   └── manifold_bridge.rs     # ManifoldCurvature, BranchialEmbedding (feature-gated, local)
│       └── hypergraph/
│           ├── mod.rs                 # Re-exports from catgraph::hypergraph + local types
│           ├── catgraph_bridge.rs     # MultiwayCospanExt trait, MultiwayCospan/Graph types
│           └── persistence.rs         # EvolutionPersistence (feature = "persist")
├── tests/                              # Integration tests (public API only)
│   ├── adjunction_laws.rs              # Z' ⊣ Z triangle identities, unit/counit
│   ├── catgraph_bridge.rs             # Span/cospan roundtrip, cospan chain composition
│   ├── fong_spivak.rs                 # Fong-Spivak re-exports + Frobenius verification
│   ├── computation_types.rs           # TM/CA domain types, computation context
│   ├── functoriality.rs              # Functor Z' composition preservation
│   ├── hypergraph_rewriting.rs       # DPO rewriting, multiway evolution, gauge theory
│   ├── multiway_coherence.rs         # Non-strict SMC coherence + monoidal functor tests
│   ├── multiway_evolution.rs         # SRS, NTM, branchial analysis, curvature
│   ├── multiway_stokes.rs            # DEC: closed forms, d²=0 (feature: dec)
│   ├── persistence.rs                # SurrealDB persist roundtrips (feature-gated)
│   └── temporal_cospan_chain.rs      # Cospan chain composability
└── examples/
    ├── gorard_demo.rs                 # 9-part presentation demo
    ├── gorard_demo.md                 # Companion documentation
    ├── builders.rs                    # TuringMachineBuilder + NTMBuilder patterns
    ├── bifunctor_tensor.rs            # Tensor products, monoidal law verification
    ├── fong_spivak.rs                 # Fong-Spivak three-perspective agreement demo
    ├── lattice_gauge.rs               # Wilson loops, plaquette action, gauge theory
    ├── multiway_coherence.rs             # Non-confluent fragment failing coherence
    ├── multiway_stokes.rs                # Closed vs non-closed 1-forms (feature: dec)
    └── persist_evolution.rs           # EvolutionPersistence lifecycle (feature-gated)
```

## Dependencies

```toml
[workspace.dependencies]
catgraph = { git = "https://github.com/tsondru/catgraph", tag = "v0.12.0" }  # Category theory (spans, cospans, Fong-Spivak, Corel) — slim baseline
catgraph-applied = { git = "https://github.com/tsondru/catgraph", tag = "v0.12.0" }  # Petri nets, wiring diagrams, props
catgraph-physics = { git = "https://github.com/tsondru/catgraph", tag = "v0.12.0" }  # Multiway, hypergraph, curvature, branchial spectral analysis
catgraph-surreal = { git = "https://github.com/tsondru/catgraph-surreal", tag = "v0.10.1" }  # optional (persist feature, Surreal<Any> stores)
deep_causality_topology = "0.5.1"   # optional (dc-geometry feature) — Regge geometry + SimplicialComplex + DEC ops
deep_causality_tensor = "0.4.2"     # optional (dc-geometry feature) — CausalTensor<D> return type for DEC ops
deep_causality_sparse = "0.1.7"     # optional (dc-geometry feature) — CsrMatrix for Hodge operator storage
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
surrealdb = { version = "3.0.5", default-features = false }  # optional (kv-mem provided by catgraph-surreal/native-embedded)
tokio = { version = "1", features = ["full"] }                    # optional
nalgebra = { version = "0.34", optional = true }                   # optional (manifold-curvature + dec features)
nalgebra-lapack = { version = "0.27", features = ["lapack-openblas"] }  # optional (lapack feature)
```

**Note:** During active development, the catgraph dep uses `path = "/home/oryx/Documents/tsondru/catgraph"`. Switch to git tag for releases.

## Feature Flags

| Feature | Gates | Dependencies |
|---------|-------|--------------|
| `dc-geometry` | dc_topology Regge + DEC substrate | `deep_causality_topology`, `deep_causality_tensor`, `deep_causality_sparse` |
| `manifold-curvature` | Regge deficit-angle curvature on branchial complexes | `dc-geometry`, `nalgebra` |
| `dec` | Discrete exterior calculus on multiway complexes | `dc-geometry`, `nalgebra` |
| `lapack` | LAPACK-accelerated eigendecomposition for MDS embedding | `nalgebra-lapack` (implies `manifold-curvature`; requires `libopenblas-dev`) |
| `persist` | SurrealDB persistence for evolution traces | `catgraph-surreal`, `surrealdb`, `tokio` |

Default features: none. Core library is purely computational (no I/O, no async).

## Key Types and Traits

### Category T (Computations)

| Type | Role | Location |
|------|------|----------|
| `ComputationState` | Object in T | `categories/computation_state.rs` |
| `TuringMachine` | Deterministic TM | `machines/turing.rs` |
| `ExecutionHistory` | TM execution trace | `machines/turing.rs` |
| `ElementaryCA` | 1D cellular automaton (256 rules) | `machines/cellular_automaton.rs` |
| `Generation` | A single CA generation (global state) | `machines/cellular_automaton.rs` |
| `CAExecutionHistory` | CA evolution trace | `machines/cellular_automaton.rs` |
| `StringRewriteSystem` | Pattern-based multiway | `machines/multiway/string_rewrite.rs` |
| `SrsRewriteRule` | SRS pattern → replacement | `machines/multiway/string_rewrite.rs` |
| `NondeterministicTM` | Non-deterministic TM | `machines/multiway/ntm.rs` |
| `PetriNetMachine` | Place/transition Petri net (linear trace) | `machines/petri/machine.rs` |
| `PetriExecutionHistory` | Petri firing trace | `machines/petri/history.rs` |
| `PetriBuilder` | Fluent Petri-net construction | `machines/petri/builder.rs` |
| `run_multiway_reachability` | Non-deterministic Petri exploration | `machines/petri/multiway.rs` |
| `BuilderError` | Error from `try_build()` on TM/NTM/Petri builders | `machines/mod.rs` |

### Category B (Cobordisms)

| Type | Role | Location |
|------|------|----------|
| `DiscreteInterval` | Morphism in B (cobordism) | `categories/cobordism.rs` |
| `ParallelIntervals` | Tensor product in B | `categories/cobordism.rs` |
| `Complexity` / `StepCount` | Complexity algebra | `categories/complexity.rs` |

### Functor Z' and Structure

| Type | Role | Location |
|------|------|----------|
| `IrreducibilityFunctor` | Z': T -> B | `functor/mod.rs` |
| `MultiwayIrreducibilityResult` | Multiway branch analysis | `functor/mod.rs` |
| `ZPrimeAdjunction` | Z' ⊣ Z adjunction | `functor/adjunction.rs` |
| `ZPrimeOps` | Adjunction operations trait | `functor/adjunction.rs` |
| `AdjunctionVerification` | Triangle identity checks | `functor/adjunction.rs` |
| `MonoidalFunctorResult` | Symmetric monoidal functor check (`#[non_exhaustive]`) | `functor/monoidal.rs` |
| `TensorProduct` | Bifunctor trait for tensor | `functor/bifunctor.rs` |
| `TensorCheck` | Per-step tensor verification | `functor/monoidal.rs` |
| `TemporalComplex` | Simplicial complex for Stokes | `temporal_cospan_chain.rs` |
| `StokesIrreducibility` | Stokes conservation analysis | `functor/stokes_integration.rs` |
| `ConservationResult` | Integration consistency result | `temporal_cospan_chain.rs` |
| `AssociatorWitness` | Confluence evidence from associator check | `multiway_coherence.rs` |
| `BraidingWitness` | Commutation evidence | `multiway_coherence.rs` |
| `CoherenceError` | Non-confluent / missing event / insufficient branches | `multiway_coherence.rs` |
| `MultiwayComplex` | 2D simplicial complex from multiway graph (feature: dec) | `multiway_stokes.rs` |
| `OneForm` | Coefficient vector on edges (feature: dec) | `multiway_stokes.rs` |
| `TwoForm` | Coefficient vector on faces (feature: dec) | `multiway_stokes.rs` |

### Trace Analysis

| Type | Role | Location |
|------|------|----------|
| `StepTrace` | Common trait for execution histories | `crate::trace` shim → `catgraph_physics::trace::StepTrace` |
| `IrreducibilityTrace` | **Deprecated alias** (since v0.6.3) for `StepTrace`; sub-trait + blanket impl. Removed in v0.7.0. | `crate::trace::IrreducibilityTrace` |
| `TraceAnalysis` | Generic analysis result (contiguity + repeats + ratio) | `crate::trace` shim |
| `RepeatDetection` | A repeated state (start_step, end_step, cycle_length) | `crate::trace` shim |
| `analyze_trace()` | Generic analysis function for any `StepTrace` | `crate::trace` shim |
| `detect_repeats()` | Fingerprint-based cycle detection | `crate::trace` shim |
| `is_irreducible()` | Wolfram-irreducibility judgment over a `StepTrace` (Gorard 2023). Note: approximation — flags shortcuts that manifest as state-fingerprint repeats inside this run; paper Def 1 is the stronger ∃ T* search over alternative TMs (see v0.7.0 plan). | `crate::trace` shim |

### Multiway Systems (re-exported from catgraph::multiway)

| Type | Role | Source |
|------|------|--------|
| `MultiwayEvolutionGraph<S,T>` | Generic multiway state graph | `catgraph::multiway` |
| `run_multiway_bfs()` | Generic BFS multiway explorer | `catgraph::multiway` |
| `BranchialGraph` | Tensor product at each time step | `catgraph::multiway` |
| `DiscreteCurvature` | Trait for curvature backends | `catgraph::multiway` |
| `CurvatureFoliation<C>` | Generic curvature across time slices | `catgraph::multiway` |
| `OllivierRicciCurvature` | Ollivier-Ricci discrete curvature (default) | `catgraph::multiway` |
| `ManifoldCurvature` | Riemannian curvature via embedding (feature-gated) | `machines/multiway/manifold_bridge.rs` (local) |
| `MultiwayStatistics` | Branch/merge/cycle counts | `catgraph::multiway` |

### Hypergraph Rewriting (re-exported from catgraph::hypergraph)

| Type | Role | Source |
|------|------|--------|
| `Hypergraph` | Vertices + hyperedges | `catgraph::hypergraph` |
| `Hyperedge` | N-ary edge | `catgraph::hypergraph` |
| `RewriteRule` | DPO rewrite L -> R | `catgraph::hypergraph` |
| `RewriteSpan` | Explicit span L <- K -> R | `catgraph::hypergraph` |
| `HypergraphEvolution` | Multiway evolution graph | `catgraph::hypergraph` |
| `WilsonLoop` | Causal invariance detector | `catgraph::hypergraph` |
| `GaugeGroup` | Gauge group trait | `catgraph::hypergraph` |
| `HypergraphRewriteGroup` | Lattice gauge theory | `catgraph::hypergraph` |
| `HypergraphLattice` | D-dimensional lattice for gauge fields | `catgraph::hypergraph` |
| `MultiwayCospan` | Single rewrite step as cospan | `machines/hypergraph/catgraph_bridge.rs` (local) |
| `MultiwayCospanGraph` | Full evolution as cospan graph | `machines/hypergraph/catgraph_bridge.rs` (local) |
| `MultiwayCospanExt` | Extension trait for multiway cospan methods | `machines/hypergraph/catgraph_bridge.rs` (local) |

### Fong-Spivak Categorical Infrastructure (catgraph v0.10.0+)

| Type / Trait | Role | Source |
|--------------|------|--------|
| `HypergraphCategory<Lambda>` | Symmetric monoidal category with Frobenius structure (§2.3) | `catgraph::hypergraph_category` |
| `HypergraphFunctor<L1,L2,Src,Tgt>` | Structure-preserving map between hypergraph categories (§2.3) | `catgraph::hypergraph_functor` |
| `RelabelingFunctor` | Free hypergraph functor induced by a set map | `catgraph::hypergraph_functor` |
| `CospanToFrobeniusFunctor` | Decomposes cospans into Frobenius morphisms (Prop 3.8) | `catgraph::hypergraph_functor` |
| `CospanAlgebra<Lambda>` | Lax symmetric monoidal functor `Cospan_Λ → C` (§2.1) | `catgraph::cospan_algebra` |
| `PartitionAlgebra` | Initial cospan-algebra `a(x) = Cospan(0, x)` (Example 2.3) | `catgraph::cospan_algebra` |
| `NameAlgebra` | Named morphisms via compact closed structure (Prop 3.2) | `catgraph::cospan_algebra` |
| `cup_single` / `cap_single` | Cup/cap morphisms for self-dual compact closed (§3.1) | `catgraph::compact_closed` |
| `cup` / `cap` | Multi-type cup/cap | `catgraph::compact_closed` |
| `name` / `unname` | Name bijection `H(X,Y) ≅ H(I, X⊗Y)` (Prop 3.2) | `catgraph::compact_closed` |

**Integration status:** These modules are available in catgraph v0.10.1 but not yet re-exported or used by irreducible. See TODO.md for integration plan.

### Catgraph Bridge API

| Method | Returns | Purpose |
|--------|---------|---------|
| `RewriteRule::to_span()` | `Span<u32>` | Rule as categorical span |
| `RewriteRule::to_rewrite_span()` | `RewriteSpan` | Full span with kernel hypergraph |
| `HypergraphEvolution::to_cospan_chain()` | `Vec<Cospan<u32>>` | Evolution as composable cospans |
| `TemporalComplex::to_cospan_chain()` | `Vec<Cospan<u32>>` | Interval sequence -> cospan bridge (`temporal_cospan_chain.rs`) |

## StepTrace Trait (formerly `IrreducibilityTrace`, deprecated since v0.6.3)

The `StepTrace` trait body lives in `catgraph_physics::trace`; irreducible re-exports it via `crate::trace::StepTrace` (and `irreducible::StepTrace` at the crate root). The body:

```rust
pub trait StepTrace {
    fn state_fingerprints(&self) -> Vec<u64>;
    fn to_intervals(&self) -> Vec<DiscreteInterval>;
    fn step_count(&self) -> usize;
    fn halted(&self) -> bool;
}
```

`ExecutionHistory` (TM), `CAExecutionHistory` (CA), and `PetriExecutionHistory` (Petri) all implement `StepTrace`. Use the generic `analyze_trace(&impl StepTrace) -> TraceAnalysis` function for unified irreducibility analysis across all machine types.

**Deprecated alias `IrreducibilityTrace`.** v0.6.3 introduced the rename and preserved the old name as a `#[deprecated]` *empty sub-trait + blanket impl*:

```rust
#[deprecated(since = "0.6.3", note = "renamed to `StepTrace`; will be removed in v0.7.0")]
pub trait IrreducibilityTrace: StepTrace {}
impl<T: StepTrace> IrreducibilityTrace for T {}
```

Bound positions like `fn f<T: IrreducibilityTrace>(...)` still compile (the blanket impl satisfies them) but emit a deprecation warning at the *bound site*. Do NOT add new methods to `IrreducibilityTrace` — it is empty by design and will be deleted in v0.7.0. Implement `StepTrace` directly.

## Irreducibility Detection

Three perspectives, all equivalent:

1. **Functor**: `IrreducibilityFunctor::is_sequence_irreducible(&intervals)` -- contiguous intervals
2. **Trace**: `analyze_trace(&history)` -- contiguity + no state repetition + complexity ratio
3. **Stokes**: `StokesIrreducibility::analyze(&intervals)?.is_irreducible()` -- conservation laws
4. **Categorical**: `evolution.to_cospan_chain()` -- composable chain iff contiguous

## Common Patterns

### Turing Machine Analysis

```rust
use irreducible::machines::{TuringMachine, Direction};
use irreducible::machines::trace::analyze_trace;

let bb = TuringMachine::busy_beaver_2_2();
let history = bb.run("", 20);
let analysis = analyze_trace(&history);
assert!(analysis.is_irreducible);
assert_eq!(analysis.step_count, 6);
```

### Cellular Automaton Analysis

```rust
use irreducible::machines::ElementaryCA;
use irreducible::machines::trace::analyze_trace;

let ca = ElementaryCA::rule_30(21);
let history = ca.run(ca.single_cell_initial(), 20);
let analysis = analyze_trace(&history);
println!("Rule 30 irreducible: {}", analysis.is_irreducible);
```

### Petri Net Reachability

```rust
use irreducible::machines::petri::{PetriBuilder, Marking, run_multiway_reachability};
use irreducible::trace::analyze_trace;
use rust_decimal::Decimal;

// Producer/consumer: R → D
let machine = PetriBuilder::<char>::new()
    .place('R').place('D')
    .transition(vec![(0, Decimal::ONE)], vec![(1, Decimal::ONE)])
    .build();

// Linear trace: smallest-index-enabled firing rule
let history = machine.run(Marking::from_vec(vec![(0, Decimal::from(3))]), 10);
assert!(history.halted);
assert!(analyze_trace(&history).is_irreducible);

// Non-deterministic multiway reachability
let evolution = run_multiway_reachability(&machine, Marking::from_vec(vec![(0, Decimal::from(3))]), 5, 64);

// Cospan bridge for Z'
let cospan = machine.transition_as_cospan(0);
```

### Multiway SRS Evolution

```rust
use irreducible::StringRewriteSystem;

let srs = StringRewriteSystem::new(vec![
    ("AB", "BA"),
    ("A", "AA"),
]);
let evolution = srs.run_multiway("AB", 5, 100);
let stats = evolution.statistics();
println!("Branches: {}, Merges: {}", stats.max_branches, stats.merge_count);
```

### Generic Multiway BFS

```rust
use irreducible::machines::multiway::run_multiway_bfs;

// step_fn: &S -> Vec<(next_state, transition_data, cost)>
// Requires S: Clone + Hash, T: Clone
let evolution = run_multiway_bfs(initial_state, |s| successors(s), max_steps, max_branches);
```

### Adjunction Triangle Identities

```rust
use irreducible::{ZPrimeAdjunction, ZPrimeOps};
use irreducible::categories::{ComputationState, DiscreteInterval};

let adj = ZPrimeAdjunction::new();
let state = ComputationState::new(42, 5);
let verification = adj.verify_triangle_identities(&state);
assert!(verification.left_triangle_holds);
assert!(verification.right_triangle_holds);
```

### Tensor Products

```rust
use irreducible::{TensorProduct, DiscreteInterval, ParallelIntervals};

let a = ParallelIntervals::from_interval(DiscreteInterval::new(0, 3));
let b = ParallelIntervals::from_interval(DiscreteInterval::new(0, 5));
let product = a.tensor(b);
```

### Hypergraph Evolution with Catgraph Bridge

```rust
use irreducible::machines::hypergraph::{Hypergraph, RewriteRule, HypergraphEvolution};

let mut graph = Hypergraph::new();
graph.add_hyperedge(vec![0, 1, 2]);

let rule = RewriteRule::from_pattern(
    vec![vec![0, 1, 2]],
    vec![vec![0, 1], vec![1, 2]],
);

let evolution = HypergraphEvolution::run_multiway(&graph, &[rule], 10, 100);
let cospan_chain = evolution.to_cospan_chain();  // Vec<Cospan<u32>>

// Check causal invariance via Wilson loops
let invariant = evolution.is_causally_invariant();
```

### Stokes Integration

```rust
use irreducible::{TemporalComplex, StokesIrreducibility, DiscreteInterval};

let intervals = vec![
    DiscreteInterval::new(0, 2),
    DiscreteInterval::new(2, 5),
    DiscreteInterval::new(5, 7),
];
let complex = TemporalComplex::from_intervals(&intervals).unwrap();
assert!(complex.verify_conservation().is_conserved);

// Convenience wrapper
let analysis = StokesIrreducibility::analyze(&intervals).unwrap();
assert!(analysis.is_irreducible());
```

## Type Constraints

| Context | Required Bounds |
|---------|----------------|
| Multiway states (`S` in `run_multiway_bfs`) | `Clone + Hash` |
| Multiway transitions (`T`) | `Clone` |
| Monoidal functor verification | `S: Clone + Eq + Hash + Debug`, `T: Clone` |
| `StepTrace` implementors | Must provide `state_fingerprints() -> Vec<u64>`, `to_intervals()`, `step_count()`, `halted()` |
| Catgraph bridge | Vertex IDs are `u32`, labels are `u32` or `()` |

## Testing

### Running Tests

```bash
cargo test --workspace                    # 385 tests, 0 ignored
cargo test -p irreducible                 # Core library unit tests
cargo test --test functoriality           # Single integration test file
cargo test --features dc-geometry         # +14 dc_topology smoke + bridge tests
cargo test --features manifold-curvature  # +14 Regge curvature tests
cargo test --features dec                 # +14 DEC tests
cargo test --features lapack              # LAPACK-accelerated eigendecomposition (requires libopenblas-dev)
cargo test --workspace --features persist # +15 persistence tests
cargo test --features "dec manifold-curvature persist"  # 427 total
cargo run --example gorard_demo           # Run the 9-part demo
cargo run --example builders              # Builder patterns
cargo run --example bifunctor_tensor      # Tensor products, monoidal laws
cargo run --example lattice_gauge         # Wilson loops, gauge theory
cargo run --example fong_spivak           # Fong-Spivak three-perspective agreement
cargo run --example multiway_coherence    # Non-confluent fragment failing coherence
cargo run --example multiway_stokes --features dec  # Closed vs non-closed 1-forms
cargo run --example petri_reachability     # Petri-net linear + multiway + cospan
cargo run --example persist_evolution --features persist  # SurrealDB persistence
cargo clippy --workspace -- -W clippy::pedantic  # Lint (zero warnings)
```

### Test Categories

| Category | Count | What it covers |
|----------|-------|----------------|
| Unit tests | ~190 | functor, machines (TM, CA, SRS, NTM, trace), categories, types, multiway coherence, DEC (hypergraph + multiway infra moved to catgraph) |
| Integration tests | ~170 | 12 files: adjunction, catgraph bridge, computation types, fong_spivak, functoriality, hypergraph, manifold curvature, multiway coherence, multiway evolution, multiway stokes, persistence, temporal cospan chain |
| Doc tests | 9 | Module-level and type-level examples (hypergraph + multiway doc tests moved to catgraph) |

### Test Patterns

- Turing machines: `TuringMachine::busy_beaver_2_2()` for quick irreducible example
- Cellular automata: `ElementaryCA::rule_30(21)` (conjectured irreducible), `rule_90(21)` (known reducible)
- Multiway: `StringRewriteSystem::new(vec![("AB", "BA")])` for branching
- Hypergraph: `Hypergraph::new()` + `add_hyperedge()` + `RewriteRule::from_pattern()`
- Three-way agreement: `IrreducibilityFunctor` + `analyze_trace()` + `StokesIrreducibility` on same execution

## Clippy Preferences

Rust 2024 edition, MSRV 1.90 (dc_topology requirement). Zero pedantic warnings. Patterns to follow:

- `#[must_use]` on all value-returning methods
- `matches!` macro instead of match expressions returning bool
- Collapse nested `if let` with `&&` (let chains)
- Use `#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]` only where f64/usize conversion is intentional

## Async Integration

The core library is **purely computational** (no I/O, no async). Two integration paths:

### Feature `persist`

Enables `EvolutionPersistence` for storing evolution traces in SurrealDB via catgraph-surreal V2 hub-node reification:

```rust
// Cargo.toml: irreducible = { features = ["persist"] }
use irreducible::machines::hypergraph::persistence::EvolutionPersistence;

let persist = EvolutionPersistence::new(&db);
let hub_ids = persist.persist_cospan_chain(&evolution, "chain_name").await?;
let span_id = persist.persist_span(&rule, "rule_name").await?;
```

### tokio-rayon for CPU-bound Work

For calling from async contexts, use the tokio-rayon executor pattern (not `spawn_blocking`):

```rust
use std::sync::LazyLock;
use tokio_rayon::AsyncThreadPool;

static EXEC: LazyLock<Executor> = LazyLock::new(|| Executor::new());

let result = EXEC.run(move || {
    let evolution = HypergraphEvolution::run_multiway(&graph, &rules, 100, 1000);
    evolution.to_cospan_chain()
}).await;
```

## Deferred Work

| Area | Notes |
|------|-------|
| Fong-Spivak integration | Re-export and use catgraph v0.10.1 Fong-Spivak modules (`HypergraphCategory`, `CospanAlgebra`, `HypergraphFunctor`, `compact_closed`). See TODO.md for phased plan |
| Non-Euclidean embedding (partial) | `BranchialEmbedding` with non-flat metric (spherical, hyperbolic) + confluence-diamond face extraction in `manifold_bridge.rs`. The discrete-Regge substrate is in place as of v0.6.0; what remains is (a) non-flat edge-length assignment for systematically nonzero curvature from the branchial graph structure alone (vs. exposed boundary vertices), and (b) swapping the fan triangulation in `manifold_bridge` for the confluence-diamond face extraction that `multiway_stokes` already uses. See `docs/decisions/2026-04-14-dc-topology-substrate.md` |
| Visualization | Multiway graphs, branchial structure, curvature heatmaps |
| Lambda calculus | Additional computation model with beta-reduction as morphisms |
| Rule classification | Systematic irreducibility analysis of all 256 elementary CA rules |
| deep_causality integration | Wire `CausalEffect<T>` into `deep_causality::PropagatingEffect` for causal computation; currently no production consumers but planned |
| Spectral coherence | nalgebra `SymmetricEigen` on densified `DMatrix` for Hodge Laplacian eigendecomposition at ≤1000 simplices. Above ~5000 simplices, Lanczos / ARPACK required (no nalgebra iterative eigensolvers). Deferred beyond Phase 2.5. |

**Intentionally kept**: `CausalEffect<T>` in `types.rs` has zero internal consumers currently but is retained for the planned deep_causality integration. Do not flag as dead code.

## API Scope

irreducible implements **computational irreducibility detection through category theory** -- specifically Gorard's functorial perspective mapping computations (Turing machines, cellular automata, string rewriting, hypergraph rewriting) to cobordism intervals and verifying functoriality. It is NOT a general computation framework or a physics simulator.
