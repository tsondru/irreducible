# ADR: Port Curvature + DEC to deep_causality_topology (v0.6.0)

**Date:** 2026-04-14
**Status:** Accepted — shipped in v0.6.0

---

## Context

v0.5.0 shipped the Petri net machine. The next queued item was Deferred Work #9:
non-Euclidean branchial embedding (`BranchialEmbedding` with spherical/hyperbolic metric,
via MDS → smooth manifold → coord-based curvature using `amari-calculus`).

A 2026-04-14 scan of `~/Documents/math` and `~/Documents/category/deep_causality`
identified `deep_causality_topology v0.5.1` as the only practical alternative that is:
- Rust-native and published on crates.io (MIT, MSRV 1.90.0, no async bloat)
- Maintained (active upstream, compatible dep graph)
- Genuinely discrete — Regge deficit-angle curvature on triangulated 2D complexes

`rg amari src tests examples` returned zero at the start of Phase 3 (amari already
purged from call sites in Phases 1–2).

---

## Decision

Port curvature and DEC to `deep_causality_topology`. Drop `amari-calculus` and
`nalgebra-sparse`. Ship as v0.6.0.

### Why dc_topology over amari-calculus

- Branchial graphs are **discrete by construction** — Regge deficit-angle curvature on
  a triangulated branchial complex is the natural discrete model. MDS → smooth manifold
  → coord-based Riemannian curvature was always an approximation that required MDS to
  produce a meaningful embedding.
- **Native DEC operators** (`exterior_derivative`, `hodge_star`, `codifferential`,
  `laplacian`) return `CausalTensor<D>` and replaced ~120 LOC of hand-rolled DEC in
  `src/multiway_stokes.rs`.
- **Real nonzero curvature for free** on non-confluent fragments — the actual goal of
  Deferred Work #9. `cone_point_has_positive_deficit` produces exact π/3 deficit on a
  pentagonal fan (5 equilateral triangles; expected 2π − 5π/3 = π/3 ≈ 1.047).
- **Shared dep ecosystem**: opens the door to the planned `CausalEffect<T>` →
  `PropagatingEffect` wiring when a real caller needs it.

### Why not nalgebra-native Christoffel/Riemann from scratch

~400 LOC of numerical-geometry code with no reference implementation, maintained in
perpetuity, orthogonal to the categorical core of this project.

### Why not keep amari-calculus alongside dc_topology

Parallel-ecosystem maintenance, two curvature backends with different semantics, no
additional capability over the dc_topology path.

---

## What Changed

### Files rewritten

- `src/machines/multiway/manifold_bridge.rs` — full rewrite of `ManifoldCurvature::from_branchial`
  and the `DiscreteCurvature` impl. Now delegates to `ReggeGeometry::calculate_ricci_curvature`
  via the dc_bridge. Per-vertex curvature maps cleanly because bones = vertices in 2D.
- `src/multiway_stokes.rs` — `MultiwayComplex` internals rewritten; `exterior_derivative`
  now delegates to `dc_topology::Manifold::exterior_derivative(1)` via hand-supplied
  Hodge operators. Diamond-coboundary semantics preserved (Gorard-specific; not standard
  simplicial d₁).

### New files

- `src/geometry/mod.rs` — geometry module root
- `src/geometry/dc_bridge.rs` — `branchial_to_simplicial`: walks BranchialGraph nodes →
  0-simplices and edges → 1-simplices; uses `confluence_diamonds()` for 2-simplices
- `src/geometry/regge_metric.rs` — `flat_regge_geometry`: all edge lengths = 1.0 (flat
  baseline, matches prior identity-metric behavior)
- `src/geometry/hodge.rs` — `unit_hodge_operators`: hand-supplies diagonal Hodge star
  operators for 2D unit-equilateral triangles (dc_topology's builder does not compute them)
- `tests/dc_topology_smoke.rs` — Phase 0 spike tests (feature: dc-geometry)

### Deps added

- `deep_causality_topology = "0.5.1"` (Regge geometry + SimplicialComplex + DEC ops)
- `deep_causality_tensor = "0.4.2"` (CausalTensor<D> return type for DEC ops)
- `deep_causality_sparse = "0.1.7"` (CsrMatrix for Hodge operator storage)

### Deps removed

- `amari-calculus` (git dep, no longer referenced anywhere)
- `nalgebra-sparse` (replaced by `deep_causality_sparse`)

### MSRV bump

1.85 → 1.90 (dc_topology requirement). Already set in `[workspace.package]` during
Phase 0; Cargo.toml `rust-version = "1.90"` confirmed.

### Feature flags updated

- `manifold-curvature = ["dc-geometry", "dep:nalgebra"]` (was: `["dep:amari-calculus", "dep:nalgebra"]`)
- `dec = ["dc-geometry", "dep:nalgebra"]` (was: `["dep:nalgebra", "dep:nalgebra-sparse"]`)
- `dc-geometry = ["dep:deep_causality_topology", "dep:deep_causality_tensor", "dep:deep_causality_sparse"]` (new)

---

## What Was Kept

- **`BranchialEmbedding` trait** — opaque metric handle preserved via local `MetricTensor`
  newtype. Public trait API unchanged; callers are unaffected.
- **`ShortestPathMDS` embedder** — produces coordinates. Coordinates are no longer consumed
  by Regge curvature (Regge works on edge lengths, not embedding coordinates) but
  `ShortestPathMDS` is retained for API stability and potential future use in non-flat
  edge-length assignment from a non-Euclidean embedding.
- **Diamond-coboundary semantics** in `MultiwayComplex::exterior_derivative` — the
  Gorard-specific 4-cycle invariant is not captured by the standard simplicial d₁.
  `multiway_stokes` uses dc_topology DEC ops on the underlying manifold but routes the
  diamond boundary computation through our own coboundary logic before handing off.

---

## Trade-offs

### Hodge operators hand-supplied

`dc_topology::SimplicialComplexBuilder` computes boundary/coboundary automatically but
does not compute Hodge star operators. For a 2D unit-equilateral-triangle complex the
Hodge star is a diagonal matrix (formulae in `src/geometry/hodge.rs`). These are
hand-supplied once at construction. Consequence: non-equilateral or non-unit-length
complexes would need a different Hodge computation; for now all our test complexes are
unit-equilateral and the formula is exact.

### `Manifold::with_metric` orientation + link condition

dc_topology validates manifold orientation and link condition. Fan triangulations
(multiple triangles sharing a single center vertex with no boundary closure) and
two-triangle patches fail this validation. Both callers bypass `Manifold::with_metric`
where the full manifold abstraction is not needed:
- Regge works on the raw `SimplicialComplex` (no `Manifold` needed for curvature)
- DEC uses `Manifold::with_metric` only on complexes that pass the link condition;
  for complexes that do not, we fall back to direct coboundary math

### Spectral coherence (Hodge Laplacian eigendecomposition)

`hodge_laplacian_spectrum_is_nonneg` uses `nalgebra::SymmetricEigen` on the densified
`DMatrix` representation of the Hodge Laplacian. This is practical for ≤1000 simplices.
Above ~5000 simplices, Lanczos/ARPACK would be required (no nalgebra iterative
eigensolvers). Deferred; consistent with the plan's original Deferred Work note.

---

## Evidence

### Tests passing (v0.6.0)

- Workspace baseline: 385 tests
- `--features dc-geometry`: +14 dc_topology smoke + bridge tests
- `--features manifold-curvature`: +14 (10 original + 4 new: non-confluent-fragment
  nonzero curvature, cone-point positive deficit, and numerical canaries)
- `--features dec`: +15 (8 original + 7 new: d²=0 on large complex, Hodge Laplacian
  nonneg spectrum, and integration canaries)
- `--features persist`: +15 persistence tests (unchanged)
- `--features "dec manifold-curvature persist"`: 427 total, zero failures

### Key numerical results

- `cone_point_has_positive_deficit`: pentagonal fan (5 equilateral triangles),
  expected deficit 2π − 5·(π/3) = π/3 ≈ 1.047 rad. Actual: 1.047 rad (exact).
- `hodge_laplacian_spectrum_is_nonneg`: single triangle complex. Spectrum {0.0, 6.0, 6.0}.
  Kernel = constant 0-form; twice-degenerate 6.0 from edge Hodge scaling (⋆₁ = 2/√3 on
  unit-equilateral edge, composed through d₀ᵀ ⋆₁ d₀). Nonneg confirmed to 1e-10.
- `rg amari src tests examples` returns zero.
- `cargo tree -p irreducible --all-features | rg amari` returns zero.

### Clippy

Zero warnings under `cargo clippy --workspace --all-features -- -W clippy::pedantic`.

---

## Follow-ups (deferred, not in this plan)

1. **Non-flat Regge edge-lengths** — spherical/hyperbolic edge-length assignment from
   branchial graph structure (shortest-path distances, Ollivier-Ricci weights) to produce
   systematically nonzero curvature from the graph alone, not just from exposed boundary
   vertices. This is the remaining open piece of Deferred Work #9.
2. **Swap `manifold_bridge.rs` fan triangulation for confluence-diamond face extraction**
   — `multiway_stokes.rs` already uses `confluence_diamonds()` for face extraction;
   `manifold_bridge.rs` still uses a fan triangulation from the center vertex. Unifying
   these would make face extraction consistent across both modules.
3. **`CausalEffect<T>` → `PropagatingEffect` wiring** — the dc monadic API (`bind`,
   `pure`, structured logging) has a different shape than irreducible's simple
   `{value, has_error, log_entries}`. Deferred until there is a real caller.
4. **`EvolutionPersistence` for Manifold objects** — currently only stores cospan chains
   via catgraph-surreal. Manifold persistence is a separate feature.
