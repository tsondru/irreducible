# Discrete Curvature: Trait-Based Dual-Backend Design

**Issue:** [#6 — Curvature heuristic: clustering coefficient vs Ollivier-Ricci/Forman](https://github.com/tsondru/irreducible/issues/6)
**Date:** 2026-04-04
**Status:** Approved

## Summary

Replace the clustering-coefficient curvature heuristic with a trait-based architecture supporting two backends:

1. **Ollivier-Ricci** (always available) — discrete graph curvature via custom Wasserstein-1 solver
2. **Manifold bridge** (feature-gated `manifold-curvature`) — embed branchial graph into smooth manifold, compute Riemannian curvature via amari-calculus

## Decisions

| Decision | Choice |
|----------|--------|
| Backend strategy | Both discrete and continuous, different use cases |
| Type unification | Trait-based (`DiscreteCurvature`) |
| Dependency strategy | Feature-gated for amari-calculus; Ollivier-Ricci always available |
| OT solver | Custom Wasserstein-1 (min-cost flow), no graph-size limits |
| Embedding algorithm | **Deferred** — design specifies `BranchialGraph -> MetricTensor<N>` interface only; the specific embedding strategy (shortest-path MDS, Laplacian spectral, etc.) is pluggable and left to future implementation |
| Trait surface | Rich: `scalar_curvature()`, `is_flat()`, `ricci_curvature(i)`, `sectional_curvature(i, j)`, `irreducibility_indicator()` |
| Migration | `BranchialCurvature` removed (not aliased), replaced by `OllivierRicciCurvature` |

## Trait Definition

```rust
/// Discrete curvature on a branchial graph.
///
/// Two backends: Ollivier-Ricci (default) and Riemannian manifold embedding
/// (feature = "manifold-curvature").
pub trait DiscreteCurvature: Clone + Debug {
    /// Scalar curvature R (trace of Ricci). 0 = flat, >0 = sphere-like, <0 = saddle-like.
    fn scalar_curvature(&self) -> f64;

    /// Whether the branchial space is flat (within tolerance).
    fn is_flat(&self) -> bool;

    /// Ricci curvature at vertex i. For Ollivier-Ricci, this is the
    /// average edge curvature incident to i. For manifold, R_ii.
    fn ricci_curvature(&self, vertex: usize) -> f64;

    /// Sectional curvature for the 2-plane spanned by vertices i, j.
    /// For Ollivier-Ricci, this is the edge curvature kappa(i,j).
    /// For manifold, the sectional curvature of the embedding.
    fn sectional_curvature(&self, i: usize, j: usize) -> f64;

    /// Irreducibility indicator. Higher = more irreducible.
    fn irreducibility_indicator(&self) -> f64;

    /// Dimension (number of branches).
    fn dimension(&self) -> usize;

    /// Time step this curvature was computed for.
    fn step(&self) -> usize;
}
```

## Ollivier-Ricci Backend

Ollivier-Ricci curvature on a graph is defined per-edge:

```
kappa(x, y) = 1 - W_1(mu_x, mu_y) / d(x, y)
```

Where `mu_x` is the uniform probability distribution over neighbors of x, and `W_1` is the Wasserstein-1 (earth mover's) distance.

### Structure

```rust
#[derive(Clone, Debug)]
pub struct OllivierRicciCurvature {
    /// Per-edge curvature kappa(i, j).
    edge_curvatures: Vec<((usize, usize), f64)>,
    /// Per-vertex Ricci curvature (average of incident edge curvatures).
    vertex_curvatures: Vec<f64>,
    /// Scalar curvature (sum of vertex curvatures, normalized).
    scalar: f64,
    dimension: usize,
    step: usize,
}
```

### Wasserstein-1 Solver

Lives in `machines/multiway/wasserstein.rs`.

- Implements min-cost flow on the bipartite transport graph
- No graph-size cap — uses network simplex or shortest-path augmentation (polynomial, scales well)
- Input: two discrete distributions (neighbor histograms) + distance matrix (shortest-path on branchial graph)
- Output: `W_1(mu_x, mu_y)`

### Construction Flow

1. Build adjacency + all-pairs shortest-path distances (BFS, since unweighted)
2. For each edge (x, y): compute neighbor distributions mu_x, mu_y -> solve W_1 -> kappa(x,y)
3. Aggregate: vertex Ricci = mean of incident edges, scalar = normalized sum

## Manifold Bridge Backend

**Feature gate:** `manifold-curvature`, adds dependency on `amari-calculus`.

### Structure

```rust
#[derive(Clone, Debug)]
pub struct ManifoldCurvature {
    /// Per-vertex Ricci curvature R_ii from the Riemannian manifold.
    vertex_curvatures: Vec<f64>,
    /// Sectional curvatures for vertex pairs.
    sectional_curvatures: Vec<((usize, usize), f64)>,
    /// Scalar curvature from amari-calculus.
    scalar: f64,
    /// Embedding dimension (may differ from graph node count).
    embedding_dim: usize,
    dimension: usize,
    step: usize,
}
```

### Embedding Trait

```rust
/// Strategy for embedding a branchial graph into a smooth manifold.
pub trait BranchialEmbedding {
    /// Target dimension of the embedding.
    const DIM: usize;

    /// Embed branchial graph into coordinates + metric tensor.
    fn embed(&self, branchial: &BranchialGraph)
        -> (Vec<[f64; Self::DIM]>, MetricTensor<Self::DIM>);
}
```

**Deferred:** The specific embedding algorithms (shortest-path MDS, Laplacian spectral, etc.) are not part of this design. The trait defines the contract; concrete implementations are future work. We may ship with a placeholder or a simple spectral embedding to validate the pipeline.

### Flow

1. `BranchialEmbedding::embed()` -> coordinates + `MetricTensor<N>`
2. `RiemannianManifold::new(metric)` -> connection, Christoffel symbols
3. Query `riemann_tensor`, `ricci_tensor`, `scalar_curvature` at each embedded vertex position
4. Package into `ManifoldCurvature`

## CurvatureFoliation

Becomes generic over `DiscreteCurvature`:

```rust
pub struct CurvatureFoliation<C: DiscreteCurvature> {
    pub curvatures: Vec<C>,
    pub total_scalar_curvature: f64,
    pub average_scalar_curvature: f64,
    pub max_curvature_magnitude: f64,
    pub max_curvature_step: usize,
}

pub type OllivierFoliation = CurvatureFoliation<OllivierRicciCurvature>;
// Behind feature gate:
pub type ManifoldFoliation = CurvatureFoliation<ManifoldCurvature>;
```

## File Layout

```
src/machines/multiway/
├── curvature.rs           -> trait DiscreteCurvature + CurvatureFoliation<C>
├── ollivier_ricci.rs      -> OllivierRicciCurvature (new)
├── wasserstein.rs         -> W_1 solver (new)
└── manifold_bridge.rs     -> ManifoldCurvature + BranchialEmbedding trait (new, feature-gated)
```

## Migration

| Consumer | Change |
|----------|--------|
| `gorard_demo.rs` | `BranchialCurvature::from_branchial(b)` -> `OllivierRicciCurvature::from_branchial(b)` |
| `tests/multiway_evolution.rs` | Same rename, all assertions stay valid (same trait methods) |
| `CurvatureFoliation::from_evolution()` | Becomes `OllivierFoliation::from_evolution()` |
| `lib.rs` re-exports | `BranchialCurvature` removed, `OllivierRicciCurvature` + `DiscreteCurvature` exported |

`BranchialCurvature` is removed, not aliased. Breaking change (acceptable pre-1.0).

## Testing Strategy

### Ollivier-Ricci Correctness

- Known-answer tests: complete graph K_n has kappa = 1 for all edges, path graph has kappa < 0, cycle graph has kappa = 0
- Property tests: kappa(x,y) in [-1, 1] for unweighted graphs, scalar curvature agrees with sum of vertex Ricci
- Existing tests migrate: same assertions (is_flat, scalar_curvature ~ 0 for linear evolution) should still pass

### Wasserstein-1 Solver

- Unit tests: known transport problems with hand-computed solutions
- Property: W_1(mu, mu) = 0, W_1(mu, nu) = W_1(nu, mu), triangle inequality
- Stress test: graphs up to ~1000 nodes to validate no performance cliff

### Manifold Bridge (feature-gated)

- Validate that flat graph embeddings produce near-zero Riemannian curvature
- Roundtrip: complete graph -> embed -> scalar curvature should be positive
- Trait conformance: `ManifoldCurvature` passes same generic trait tests as `OllivierRicciCurvature`

### Trait-Level Generic Tests

- Tests generic over `C: DiscreteCurvature` that both backends must pass (sign consistency, dimension correctness, flat detection)

### CurvatureFoliation

- Existing foliation tests adapt to `OllivierFoliation` type alias
- `is_globally_flat()` and `irreducibility_profile()` work identically
