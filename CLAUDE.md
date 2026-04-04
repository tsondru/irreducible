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
│   ├── functor/
│   │   ├── mod.rs                      # IrreducibilityFunctor, MultiwayIrreducibilityResult
│   │   ├── adjunction.rs              # ZPrimeAdjunction + re-exports catgraph::adjunction
│   │   ├── monoidal.rs                # MonoidalFunctorResult + re-exports catgraph::coherence
│   │   ├── bifunctor.rs               # Re-exports catgraph::bifunctor (TensorProduct, etc.)
│   │   └── stokes_integration.rs      # StokesIrreducibility + re-exports catgraph::stokes
│   └── machines/
│       ├── mod.rs                      # Machine re-exports, State type alias
│       ├── turing.rs                   # TuringMachine, ExecutionHistory, TuringMachineBuilder
│       ├── cellular_automaton.rs       # ElementaryCA, Generation, CAExecutionHistory
│       ├── trace.rs                    # IrreducibilityTrace trait, TraceAnalysis, RepeatDetection
│       ├── configuration.rs            # Configuration (instantaneous TM description)
│       ├── tape.rs                     # Tape, Symbol
│       ├── transition.rs              # Direction, Transition
│       ├── multiway/
│       │   ├── evolution_graph.rs     # MultiwayEvolutionGraph, run_multiway_bfs (generic)
│       │   ├── branchial.rs           # BranchialGraph, extract_branchial_foliation
│       │   ├── curvature.rs           # DiscreteCurvature trait, CurvatureFoliation<C>
│       │   ├── ollivier_ricci.rs      # OllivierRicciCurvature, OllivierFoliation
│       │   ├── wasserstein.rs         # Wasserstein-1 solver (min-cost flow)
│       │   ├── manifold_bridge.rs     # ManifoldCurvature, BranchialEmbedding (feature-gated)
│       │   ├── string_rewrite.rs      # StringRewriteSystem, SRSState
│       │   └── ntm.rs                 # NondeterministicTM, NTMBuilder
│       └── hypergraph/
│           ├── hyperedge.rs           # Hyperedge (n-ary edge)
│           ├── hypergraph.rs          # Hypergraph (vertices + hyperedges)
│           ├── rewrite_rule.rs        # RewriteRule, RewriteMatch, RewriteSpan (DPO)
│           ├── evolution.rs           # HypergraphEvolution, WilsonLoop, causal invariance
│           ├── gauge.rs               # GaugeGroup, HypergraphRewriteGroup, HypergraphLattice
│           ├── catgraph_bridge.rs     # MultiwayCospan, MultiwayCospanGraph, Span/Cospan bridge
│           └── persistence.rs         # EvolutionPersistence (feature = "persist")
├── tests/                              # Integration tests (public API only)
│   ├── adjunction_laws.rs              # Z' ⊣ Z triangle identities, unit/counit
│   ├── catgraph_bridge.rs             # Span/cospan roundtrip, cospan chain composition
│   ├── computation_types.rs           # TM/CA domain types, computation context
│   ├── functoriality.rs              # Functor Z' composition preservation
│   ├── hypergraph_rewriting.rs       # DPO rewriting, multiway evolution, gauge theory
│   ├── monoidal_coherence.rs         # Tensor checks, associator/braiding coherence
│   ├── multiway_evolution.rs         # SRS, NTM, branchial analysis, curvature
│   ├── persistence.rs                # SurrealDB persist roundtrips (feature-gated)
│   ├── property_coherence.rs         # Coherence verification, differential coherence
│   └── stokes_integration.rs         # Conservation laws, Stokes-cospan bridge
└── examples/
    ├── gorard_demo.rs                 # 9-part presentation demo
    ├── gorard_demo.md                 # Companion documentation
    ├── builders.rs                    # TuringMachineBuilder + NTMBuilder patterns
    ├── bifunctor_tensor.rs            # Tensor products, monoidal law verification
    ├── lattice_gauge.rs               # Wilson loops, plaquette action, gauge theory
    └── persist_evolution.rs           # EvolutionPersistence lifecycle (feature-gated)
```

## Dependencies

```toml
[workspace.dependencies]
catgraph = { git = "https://github.com/tsondru/catgraph", tag = "v0.4.0" }  # Category theory (spans, cospans, adjunctions, coherence)
catgraph-surreal = { git = "https://github.com/tsondru/catgraph", tag = "v0.4.0" }  # optional (persist feature)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
surrealdb = { version = "3.0.4", default-features = false, features = ["kv-mem"] }  # optional
tokio = { version = "1", features = ["full"] }                    # optional
amari-calculus = { path = "..." }                                  # optional (manifold-curvature feature)
```

**Note:** During active development, the catgraph dep uses `path = "/home/oryx/Documents/tsondru/catgraph"`. Switch to git tag for releases.

## Feature Flags

| Feature | Gates | Dependencies |
|---------|-------|--------------|
| `persist` | SurrealDB persistence for evolution traces | `catgraph-surreal`, `surrealdb`, `tokio` |
| `manifold-curvature` | Riemannian manifold curvature via amari-calculus | `amari-calculus` |

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
| `NondeterministicTM` | Non-deterministic TM | `machines/multiway/ntm.rs` |

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
| `MonoidalFunctorResult` | Symmetric monoidal functor check | `functor/monoidal.rs` |
| `CoherenceVerification` | Alpha, lambda, rho, sigma conditions | `functor/monoidal.rs` |
| `DifferentialCoherence` | Categorical curvature | `functor/monoidal.rs` |
| `TensorProduct` | Bifunctor trait for tensor | `functor/bifunctor.rs` |
| `TensorCheck` | Per-step tensor verification | `functor/monoidal.rs` |
| `TemporalComplex` | Simplicial complex for Stokes | `functor/stokes_integration.rs` |
| `StokesIrreducibility` | Stokes conservation analysis | `functor/stokes_integration.rs` |
| `ConservationResult` | Integration consistency result | `functor/stokes_integration.rs` |

### Trace Analysis

| Type | Role | Location |
|------|------|----------|
| `IrreducibilityTrace` | Common trait for execution histories | `machines/trace.rs` |
| `TraceAnalysis` | Generic analysis result (contiguity + repeats + ratio) | `machines/trace.rs` |
| `RepeatDetection` | A repeated state (start_step, end_step, cycle_length) | `machines/trace.rs` |
| `analyze_trace()` | Generic analysis function for any `IrreducibilityTrace` | `machines/trace.rs` |
| `detect_repeats()` | Fingerprint-based cycle detection | `machines/trace.rs` |

### Multiway Systems

| Type | Role | Location |
|------|------|----------|
| `MultiwayEvolutionGraph<S,T>` | Generic multiway state graph | `machines/multiway/evolution_graph.rs` |
| `run_multiway_bfs()` | Generic BFS multiway explorer | `machines/multiway/evolution_graph.rs` |
| `BranchialGraph` | Tensor product at each time step | `machines/multiway/branchial.rs` |
| `DiscreteCurvature` | Trait for curvature backends | `machines/multiway/curvature.rs` |
| `CurvatureFoliation<C>` | Generic curvature across time slices | `machines/multiway/curvature.rs` |
| `OllivierRicciCurvature` | Ollivier-Ricci discrete curvature (default) | `machines/multiway/ollivier_ricci.rs` |
| `ManifoldCurvature` | Riemannian curvature via embedding (feature-gated) | `machines/multiway/manifold_bridge.rs` |
| `MultiwayStatistics` | Branch/merge/cycle counts | `machines/multiway/evolution_graph.rs` |

### Hypergraph Rewriting (catgraph bridge)

| Type | Role | Location |
|------|------|----------|
| `Hypergraph` | Vertices + hyperedges | `machines/hypergraph/hypergraph.rs` |
| `Hyperedge` | N-ary edge | `machines/hypergraph/hyperedge.rs` |
| `RewriteRule` | DPO rewrite L -> R | `machines/hypergraph/rewrite_rule.rs` |
| `RewriteSpan` | Explicit span L <- K -> R | `machines/hypergraph/rewrite_rule.rs` |
| `HypergraphEvolution` | Multiway evolution graph | `machines/hypergraph/evolution.rs` |
| `WilsonLoop` | Causal invariance detector | `machines/hypergraph/evolution.rs` |
| `GaugeGroup` | Gauge group trait | `machines/hypergraph/gauge.rs` |
| `HypergraphRewriteGroup` | Lattice gauge theory | `machines/hypergraph/gauge.rs` |
| `HypergraphLattice` | D-dimensional lattice for gauge fields | `machines/hypergraph/gauge.rs` |
| `MultiwayCospan` | Single rewrite step as cospan | `machines/hypergraph/catgraph_bridge.rs` |
| `MultiwayCospanGraph` | Full evolution as cospan graph | `machines/hypergraph/catgraph_bridge.rs` |

### Catgraph Bridge API

| Method | Returns | Purpose |
|--------|---------|---------|
| `RewriteRule::to_span()` | `Span<u32>` | Rule as categorical span |
| `RewriteRule::to_rewrite_span()` | `RewriteSpan` | Full span with kernel hypergraph |
| `HypergraphEvolution::to_cospan_chain()` | `Vec<Cospan<u32>>` | Evolution as composable cospans |
| `TemporalComplex::to_cospan_chain()` | `Vec<Cospan<()>>` | Stokes -> cospan bridge |

## IrreducibilityTrace Trait

```rust
pub trait IrreducibilityTrace {
    fn state_fingerprints(&self) -> Vec<u64>;
    fn to_intervals(&self) -> Vec<DiscreteInterval>;
    fn step_count(&self) -> usize;
    fn halted(&self) -> bool;
}
```

Both `ExecutionHistory` (TM) and `CAExecutionHistory` (CA) implement this trait. Use the generic `analyze_trace(&impl IrreducibilityTrace) -> TraceAnalysis` function for unified irreducibility analysis across all machine types.

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
use irreducible::functor::stokes_integration::{TemporalComplex, StokesIrreducibility};
use irreducible::DiscreteInterval;

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
| `IrreducibilityTrace` implementors | Must provide `state_fingerprints() -> Vec<u64>` |
| Catgraph bridge | Vertex IDs are `u32`, labels are `u32` or `()` |

## Testing

### Running Tests

```bash
cargo test --workspace                    # 416 tests (270 unit + 124 integration + 22 doc), 0 ignored
cargo test -p irreducible                 # Core library unit tests (264)
cargo test --test functoriality           # Single integration test file
cargo test --workspace --features persist # 425 tests (+15 persistence)
cargo run --example gorard_demo           # Run the 9-part demo
cargo run --example builders              # Builder patterns
cargo run --example bifunctor_tensor      # Tensor products, monoidal laws
cargo run --example lattice_gauge         # Wilson loops, gauge theory
cargo run --example persist_evolution --features persist  # SurrealDB persistence
cargo clippy --workspace -- -W clippy::pedantic  # Lint (zero warnings)
```

### Test Categories

| Category | Count | What it covers |
|----------|-------|----------------|
| Unit tests | 270 | All modules: functor, machines, categories, types, trace, curvature |
| Integration tests | 124 | 10 files: adjunction, catgraph bridge, computation types, functoriality, hypergraph, monoidal, multiway, persistence, property coherence, Stokes |
| Doc tests | 22 | Module-level and type-level examples (all enabled, zero ignored) |

### Test Patterns

- Turing machines: `TuringMachine::busy_beaver_2_2()` for quick irreducible example
- Cellular automata: `ElementaryCA::rule_30(21)` (conjectured irreducible), `rule_90(21)` (known reducible)
- Multiway: `StringRewriteSystem::new(vec![("AB", "BA")])` for branching
- Hypergraph: `Hypergraph::new()` + `add_hyperedge()` + `RewriteRule::from_pattern()`

## Clippy Preferences

Rust 2024 edition. Zero pedantic warnings. Patterns to follow:

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
| Visualization | Multiway graphs, branchial structure, curvature heatmaps |
| Lambda calculus | Additional computation model with beta-reduction as morphisms |
| Rule classification | Systematic irreducibility analysis of all 256 elementary CA rules |
| nalgebra bridge | Matrix representations for large tensor product verification |

## API Scope

irreducible implements **computational irreducibility detection through category theory** -- specifically Gorard's functorial perspective mapping computations (Turing machines, cellular automata, string rewriting, hypergraph rewriting) to cobordism intervals and verifying functoriality. It is NOT a general computation framework or a physics simulator.
