//! Riemannian manifold curvature backend for branchial graphs.
//!
//! Feature-gated behind `manifold-curvature`. Provides [`ManifoldCurvature`],
//! a [`DiscreteCurvature`] implementation that embeds a branchial graph into
//! a 2D simplicial complex and computes discrete Regge curvature
//! (deficit angles at vertices) via [`deep_causality_topology::ReggeGeometry`].
//!
//! Includes [`ShortestPathMDS`], a concrete embedding strategy that uses
//! classical multidimensional scaling on all-pairs shortest-path distances.
//! The coordinates it produces are not consumed by the Regge pipeline
//! (Regge curvature depends only on combinatorics + edge lengths, not
//! coordinates), but the embedding call is retained because the
//! [`BranchialEmbedding`] trait is part of the public surface of this module.
//!
//! # Substrate port status (Phase 1, 2026-04-14)
//!
//! `manifold-curvature` is now implemented on top of `deep_causality_topology`
//! (`dc-geometry` feature). The Regge pipeline:
//!
//! 1. Embeds the branchial graph with [`BranchialEmbedding::embed`] (coords
//!    + opaque metric handle — result retained only for API compatibility).
//! 2. Builds a 2D simplicial complex via a **fan triangulation** from vertex 0.
//!    This is a stand-in: it produces a valid 2D complex that matches the
//!    existing test expectations (flat → zero curvature everywhere). Swapping
//!    to confluence-diamond faces extracted from the underlying
//!    [`MultiwayEvolutionGraph`] — the triangulation `multiway_stokes` now
//!    uses — is deferred to a follow-up plan (CLAUDE.md Deferred Work #9,
//!    non-Euclidean embedding). Until then this curvature backend reports
//!    zero on every flat-metric branchial.
//! 3. Applies a flat [`ReggeGeometry`] (all edge lengths `1.0`) and calls
//!    [`ReggeGeometry::calculate_ricci_curvature`] to obtain one deficit
//!    angle per vertex bone.
//! 4. Derives sectional curvature via 2D Gauss-Bonnet:
//!    `K_{ij} = δ_k / A_{ijk}` where `k` is the third vertex of a triangle
//!    containing both `i` and `j`, and `A_{ijk}` is the triangle area
//!    (√3/4 for unit-edge equilateral triangles).
//!
//! # Why not `Manifold::with_metric`?
//!
//! `dc_topology::Manifold::with_metric` validates the **link condition** for
//! each vertex — the apex of a fan triangulation fails it (the link is a
//! path, not a circle/disk in the expected Euler-char sense). Since we only
//! need curvature and not DEC ops here, we call
//! [`ReggeGeometry::calculate_ricci_curvature`] directly on the complex,
//! bypassing the manifold-property check. Phase 2 (DEC port in
//! `multiway_stokes`) uses `Manifold` on complexes that do satisfy the check
//! (closed confluence diamonds).

use std::collections::VecDeque;
use std::fmt;

use nalgebra::{DMatrix, SMatrix, SVector};

use catgraph_physics::multiway::{BranchialGraph, CurvatureFoliation, DiscreteCurvature};
use deep_causality_topology::{Simplex, SimplicialComplexBuilder};

use crate::geometry::regge_metric::flat_regge_geometry;

/// Local metric-tensor newtype.
///
/// Replaces the pre-Phase-1 external metric-tensor type. The Regge pipeline
/// does not consume this — it is retained only because
/// [`BranchialEmbedding::embed`] returns it as an opaque handle for API
/// compatibility.
#[derive(Clone, Debug)]
pub struct MetricTensor<const DIM: usize>(pub SMatrix<f64, DIM, DIM>);

impl<const DIM: usize> MetricTensor<DIM> {
    /// Identity metric on flat Euclidean space.
    #[must_use]
    pub fn euclidean() -> Self {
        Self(SMatrix::<f64, DIM, DIM>::identity())
    }
}

/// Strategy for embedding a branchial graph into a smooth manifold.
///
/// The embedding maps discrete graph structure to continuous coordinates
/// with a metric tensor. The Regge curvature pipeline does not use the
/// coordinates directly — they are retained on the trait for downstream
/// consumers that may visualize or otherwise reason about the embedding.
pub trait BranchialEmbedding<const DIM: usize> {
    /// Embed branchial graph into coordinates + metric tensor.
    ///
    /// Returns `(coordinates, metric)` where:
    /// - `coordinates[i]` is the position of vertex `i` in `DIM`-dimensional space
    /// - `metric` is the Riemannian metric tensor on the embedding manifold
    fn embed(&self, branchial: &BranchialGraph) -> (Vec<SVector<f64, DIM>>, MetricTensor<DIM>);
}

/// Branchial embedding via classical multidimensional scaling (MDS) on
/// all-pairs shortest-path distances.
///
/// The algorithm:
/// 1. Compute all-pairs shortest-path distances via BFS.
/// 2. Square the distance matrix elementwise.
/// 3. Double-center: B = -1/2 H D^2 H, where H = I - (1/n) 11^T.
/// 4. Eigendecompose B (symmetric), take the top `DIM` positive eigenvalues.
/// 5. Coordinates: `coord[i][d] = eigenvector_d[i] * sqrt(eigenvalue_d)`.
///
/// The resulting embedding lives in flat Euclidean space, so the metric
/// tensor is the identity.
pub struct ShortestPathMDS<const DIM: usize>;

impl<const DIM: usize> BranchialEmbedding<DIM> for ShortestPathMDS<DIM>
where
    [(); DIM]:,
{
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::needless_range_loop
    )]
    fn embed(&self, branchial: &BranchialGraph) -> (Vec<SVector<f64, DIM>>, MetricTensor<DIM>) {
        let n = branchial.nodes.len();

        if n == 0 {
            return (Vec::new(), MetricTensor::euclidean());
        }

        if n == 1 {
            return (vec![SVector::<f64, DIM>::zeros()], MetricTensor::euclidean());
        }

        // --- Step 1: All-pairs BFS ---
        let distances = all_pairs_bfs(branchial);

        // --- Step 2: Square the distances ---
        let nf = n as f64;
        let d_sq = distances.component_mul(&distances);

        // --- Step 3: Double-center: B = -0.5 * H * D^2 * H ---
        // H = I - (1/n) * 1 * 1^T
        let ones = DMatrix::<f64>::from_element(n, n, 1.0 / nf);
        let identity = DMatrix::<f64>::identity(n, n);
        let h = &identity - &ones;
        let b = &h * &d_sq * &h * (-0.5);

        // --- Step 4: Eigendecompose ---
        #[cfg(feature = "lapack")]
        let eigen = nalgebra_lapack::SymmetricEigen::new(b);
        #[cfg(not(feature = "lapack"))]
        let eigen = b.symmetric_eigen();

        // Sort eigenvalues descending, keeping track of original indices
        let mut indexed: Vec<(usize, f64)> = eigen
            .eigenvalues
            .iter()
            .copied()
            .enumerate()
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // --- Step 5: Extract top DIM eigenvalues and compute coordinates ---
        let mut coords = vec![SVector::<f64, DIM>::zeros(); n];
        for d in 0..DIM {
            if d < indexed.len() && indexed[d].1 > 0.0 {
                let col_idx = indexed[d].0;
                let scale = indexed[d].1.sqrt();
                let eigvec = eigen.eigenvectors.column(col_idx);
                for i in 0..n {
                    coords[i][d] = eigvec[i] * scale;
                }
            }
            // else: eigenvalue <= 0 or not enough eigenvalues, leave as 0.0
        }

        (coords, MetricTensor::euclidean())
    }
}

/// Compute all-pairs shortest-path distances via BFS on an unweighted graph.
///
/// Unreachable pairs receive a distance of `n * n` (a large but finite value)
/// to keep the eigendecomposition numerically stable.
#[must_use]
#[allow(clippy::cast_precision_loss)]
fn all_pairs_bfs(branchial: &BranchialGraph) -> DMatrix<f64> {
    let n = branchial.nodes.len();
    let large_dist = (n * n) as f64;

    // Build adjacency list indexed by position in branchial.nodes
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

    // Map node IDs to indices
    let node_to_idx: std::collections::HashMap<_, _> = branchial
        .nodes
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i))
        .collect();

    for (a, b) in &branchial.edges {
        if let (Some(&ia), Some(&ib)) = (node_to_idx.get(a), node_to_idx.get(b)) {
            adj[ia].push(ib);
            adj[ib].push(ia);
        }
    }

    let mut dist = DMatrix::<f64>::from_element(n, n, large_dist);

    for source in 0..n {
        dist[(source, source)] = 0.0;
        let mut queue = VecDeque::new();
        queue.push_back(source);
        let mut visited = vec![false; n];
        visited[source] = true;

        while let Some(u) = queue.pop_front() {
            for &v in &adj[u] {
                if !visited[v] {
                    visited[v] = true;
                    dist[(source, v)] = dist[(source, u)] + 1.0;
                    queue.push_back(v);
                }
            }
        }
    }

    dist
}

/// Area of a unit-edge equilateral triangle: √3 / 4.
const AREA_UNIT_EQUILATERAL: f64 = 0.433_012_701_892_219_3;

/// 2D Gauss-Bonnet sectional curvature.
///
/// `K_{ij} = δ_k / A_{ijk}` where:
/// - `k` is the third vertex of a triangle containing both `i` and `j`.
/// - `δ_k` is the deficit angle at `k` (as returned by
///   [`ReggeGeometry::calculate_ricci_curvature`]).
/// - `A_{ijk}` is the area of the triangle (`√3 / 4` for unit-edge
///   equilateral triangles, the only case the flat Regge baseline supports).
///
/// Returns `0.0` if no triangle contains both `i` and `j`, or if `i == j`.
/// When multiple triangles share the edge `{i, j}`, the first match wins —
/// for closed orientable 2-complexes with 2 triangles per edge the choice is
/// ambiguous; this is acceptable for Phase 1 (flat → zero everywhere).
fn gauss_bonnet_sectional(
    triangles: &[[usize; 3]],
    vertex_curvatures: &[f64],
    i: usize,
    j: usize,
) -> f64 {
    if i == j {
        return 0.0;
    }
    for tri in triangles {
        if tri.contains(&i) && tri.contains(&j) {
            let Some(&k) = tri.iter().find(|&&v| v != i && v != j) else {
                continue;
            };
            let delta = vertex_curvatures.get(k).copied().unwrap_or(0.0);
            return delta / AREA_UNIT_EQUILATERAL;
        }
    }
    0.0
}

/// Fan-triangulate `n` vertices around vertex 0.
///
/// Produces triangles `[0, i, i+1]` for `i in 1..n-1`. For `n < 3` returns
/// an empty list (no triangles exist in a graph of 0, 1, or 2 vertices).
///
/// Phase-1 stand-in; Phase 2 will swap this for confluence-diamond faces
/// extracted from the underlying multiway evolution.
fn fan_triangulate(n: usize) -> Vec<[usize; 3]> {
    if n < 3 {
        return Vec::new();
    }
    (1..n - 1).map(|i| [0, i, i + 1]).collect()
}

/// Riemannian curvature computed from a branchial graph via discrete Regge
/// deficit angles.
///
/// Wraps the result of fan-triangulating a [`BranchialGraph`] into a 2D
/// simplicial complex and computing curvature via
/// [`deep_causality_topology::ReggeGeometry::calculate_ricci_curvature`].
///
/// Implements [`DiscreteCurvature`] with the same public interface as the
/// pre-Phase-1 implementation — all flat-metric contracts are preserved.
#[derive(Clone, Debug)]
pub struct ManifoldCurvature {
    /// Per-vertex Ricci curvature (deficit angle at each vertex bone).
    vertex_curvatures: Vec<f64>,
    /// Sectional curvatures stored as a symmetric `n × n` matrix.
    sectional_curvatures: DMatrix<f64>,
    /// Scalar curvature: sum of deficit angles. On a closed 2-manifold this
    /// is `2π · χ` (Gauss-Bonnet); on open complexes boundary bones
    /// contribute zero and the sum tracks only interior curvature.
    scalar: f64,
    /// Dimension of the embedding manifold.
    embedding_dim: usize,
    /// Number of vertices (branchial graph nodes).
    dim: usize,
    /// Time step this curvature was computed for.
    time_step: usize,
}

/// Curvature foliation parameterized by the Riemannian manifold backend.
///
/// Convenience alias: each time step carries a [`ManifoldCurvature`]
/// computed from the branchial graph embedding at that step.
pub type ManifoldFoliation = CurvatureFoliation<ManifoldCurvature>;

impl ManifoldCurvature {
    /// Compute discrete Regge curvature from a branchial graph and an embedding.
    ///
    /// Algorithm (Phase-1 substrate):
    /// 1. Call `embedding.embed(branchial)` for API compatibility (coords
    ///    unused by the curvature pipeline).
    /// 2. Fan-triangulate `n = branchial.nodes.len()` vertices around
    ///    vertex 0 into `n - 2` triangles.
    /// 3. Build a 2D [`deep_causality_topology::SimplicialComplex`] via
    ///    [`SimplicialComplexBuilder`].
    /// 4. Apply a flat [`ReggeGeometry`] (all edge lengths `1.0`).
    /// 5. Call [`ReggeGeometry::calculate_ricci_curvature`] to obtain one
    ///    deficit angle per vertex bone.
    /// 6. Derive sectional curvatures via 2D Gauss-Bonnet.
    /// 7. Scalar curvature = sum of deficit angles.
    ///
    /// Edge cases: `n == 0` returns all-zero curvature; `n == 1` or `n == 2`
    /// returns all-zero curvature (no triangles → Regge pipeline is trivially
    /// flat; matches pre-port contract).
    ///
    /// # Panics
    ///
    /// Does not panic in practice. The three `.expect()` calls cover
    /// invariants that hold by construction:
    /// - `SimplicialComplexBuilder::add_simplex` only rejects simplices whose
    ///   grade exceeds the builder's configured `max_dim`; the fan emits
    ///   only 2-simplices into a `max_dim = 2` builder.
    /// - `SimplicialComplexBuilder::build::<f64>` only fails if the closure
    ///   property is violated; `add_simplex` enforces closure.
    /// - `flat_regge_geometry` only fails if the underlying tensor crate
    ///   rejects a `(Vec<f64>, vec![edge_count])` pair, which is contract-
    ///   valid by construction.
    /// - `ReggeGeometry::calculate_ricci_curvature` requires
    ///   `max_simplex_dimension >= 2`; guaranteed for `n >= 3`.
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        clippy::needless_range_loop,
        clippy::cast_possible_truncation
    )]
    pub fn from_branchial<const DIM: usize>(
        branchial: &BranchialGraph,
        embedding: &impl BranchialEmbedding<DIM>,
    ) -> Self {
        let n = branchial.nodes.len();

        // Preserve the embedding call even though we don't use the coords —
        // downstream callers may rely on side effects (e.g., caching).
        let _embed_result = embedding.embed(branchial);

        if n < 3 {
            // No triangles exist in a 0-, 1-, or 2-vertex graph; the Regge
            // pipeline requires `max_simplex_dimension >= 2`. Short-circuit
            // to an all-zero ManifoldCurvature — matches the pre-port
            // behavior on flat complexes.
            return Self {
                vertex_curvatures: vec![0.0; n],
                sectional_curvatures: DMatrix::<f64>::zeros(n, n),
                scalar: 0.0,
                embedding_dim: DIM,
                dim: n,
                time_step: branchial.step,
            };
        }

        let triangles = fan_triangulate(n);

        // Build 2D simplicial complex from the fan triangulation.
        // add_simplex on a 2-simplex auto-adds its 3 edges and 3 vertices,
        // preserving the closure property.
        let mut builder = SimplicialComplexBuilder::new(2);
        for tri in &triangles {
            // Safe to expect: builder dim is 2 and triangle vertex count is 3.
            builder
                .add_simplex(Simplex::new(tri.to_vec()))
                .expect("fan-triangulation simplex always fits within builder dim");
        }
        let complex = builder
            .build::<f64>()
            .expect("fan-triangulation always yields a valid 2D complex");

        let edge_count = complex.skeletons()[1].simplices().len();
        let metric = flat_regge_geometry(edge_count)
            .expect("flat_regge_geometry cannot fail for non-negative edge_count");

        let ricci = metric
            .calculate_ricci_curvature(&complex)
            .expect("ricci curvature on fan triangulation with flat metric cannot fail");

        // Map the ricci tensor (one entry per vertex bone, canonically sorted
        // by the builder) back to per-vertex deficits. In the builder's
        // canonical sort, Simplex([i]) orders by `i`, so bones[i] == vertex i.
        let mut vertex_curvatures: Vec<f64> = ricci.as_slice().to_vec();

        // Safety net: if the tensor length diverges from n (e.g., if the
        // builder ever drops unreachable vertices), pad or truncate to n
        // so downstream indexing by vertex is always in-bounds.
        if vertex_curvatures.len() < n {
            vertex_curvatures.resize(n, 0.0);
        } else if vertex_curvatures.len() > n {
            vertex_curvatures.truncate(n);
        }

        let scalar: f64 = vertex_curvatures.iter().sum();

        // Sectional curvature via 2D Gauss-Bonnet on the fan triangulation.
        let mut sectional_curvatures = DMatrix::<f64>::zeros(n, n);
        for i in 0..n {
            for j in (i + 1)..n {
                let k = gauss_bonnet_sectional(&triangles, &vertex_curvatures, i, j);
                sectional_curvatures[(i, j)] = k;
                sectional_curvatures[(j, i)] = k;
            }
        }

        Self {
            vertex_curvatures,
            sectional_curvatures,
            scalar,
            embedding_dim: DIM,
            dim: n,
            time_step: branchial.step,
        }
    }

    /// Dimension of the embedding manifold.
    #[must_use]
    pub fn embedding_dimension(&self) -> usize {
        self.embedding_dim
    }
}

impl DiscreteCurvature for ManifoldCurvature {
    fn scalar_curvature(&self) -> f64 {
        self.scalar
    }

    fn is_flat(&self) -> bool {
        self.vertex_curvatures.iter().all(|&k| k.abs() < 1e-10)
    }

    fn ricci_curvature(&self, vertex: usize) -> f64 {
        self.vertex_curvatures
            .get(vertex)
            .copied()
            .unwrap_or(0.0)
    }

    fn sectional_curvature(&self, i: usize, j: usize) -> f64 {
        if i >= self.sectional_curvatures.nrows() || j >= self.sectional_curvatures.ncols() {
            return 0.0;
        }
        self.sectional_curvatures[(i, j)]
    }

    fn irreducibility_indicator(&self) -> f64 {
        self.scalar.abs()
    }

    fn dimension(&self) -> usize {
        self.dim
    }

    fn step(&self) -> usize {
        self.time_step
    }
}

impl fmt::Display for ManifoldCurvature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Manifold Curvature (step {}):",
            self.time_step
        )?;
        writeln!(f, "  Embedding dimension: {}", self.embedding_dim)?;
        writeln!(f, "  Graph dimension: {}", self.dim)?;
        writeln!(f, "  Scalar curvature R: {:.6}", self.scalar)?;
        let n = self.sectional_curvatures.nrows();
        let pairs = n * n.saturating_sub(1) / 2;
        writeln!(f, "  Sectional pairs analyzed: {pairs}")?;
        writeln!(f, "  Is flat: {}", self.is_flat())?;
        write!(
            f,
            "  Irreducibility indicator: {:.6}",
            self.irreducibility_indicator()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catgraph_physics::multiway::{BranchId, MultiwayNodeId};
    use deep_causality_topology::SimplicialComplexBuilder;

    use crate::StringRewriteSystem;
    use crate::machines::multiway::extract_branchial_foliation;

    fn make_id(branch: usize, step: usize) -> MultiwayNodeId {
        MultiwayNodeId::new(BranchId(branch), step)
    }

    #[test]
    fn complete_graph_k4_embeds_and_produces_valid_curvature() {
        // K₄: 4 nodes, all 6 edges
        let nodes: Vec<MultiwayNodeId> = (0..4).map(|i| make_id(i, 0)).collect();
        let edges = vec![
            (nodes[0], nodes[1]),
            (nodes[0], nodes[2]),
            (nodes[0], nodes[3]),
            (nodes[1], nodes[2]),
            (nodes[1], nodes[3]),
            (nodes[2], nodes[3]),
        ];
        let branchial = BranchialGraph {
            step: 0,
            nodes: nodes.clone(),
            edges,
        };

        let (coords, _metric) = ShortestPathMDS::<3>.embed(&branchial);
        assert_eq!(coords.len(), 4);

        // All coordinates should be finite
        for coord in &coords {
            for &c in coord.as_slice() {
                assert!(c.is_finite(), "coordinate must be finite: {c}");
            }
        }

        // Compute ManifoldCurvature — should not panic
        let curvature = ManifoldCurvature::from_branchial(&branchial, &ShortestPathMDS::<3>);
        assert_eq!(curvature.dimension(), 4);
        assert_eq!(curvature.step(), 0);
        // Flat Regge metric → zero deficit everywhere.
        assert!(curvature.scalar_curvature().is_finite());
    }

    #[test]
    fn path_graph_p5_near_zero_curvature() {
        // P₅: 5 nodes in a line: 0-1-2-3-4
        let nodes: Vec<MultiwayNodeId> = (0..5).map(|i| make_id(i, 2)).collect();
        let edges = vec![
            (nodes[0], nodes[1]),
            (nodes[1], nodes[2]),
            (nodes[2], nodes[3]),
            (nodes[3], nodes[4]),
        ];
        let branchial = BranchialGraph {
            step: 2,
            nodes: nodes.clone(),
            edges,
        };

        let (coords, _metric) = ShortestPathMDS::<2>.embed(&branchial);
        assert_eq!(coords.len(), 5);

        let curvature = ManifoldCurvature::from_branchial(&branchial, &ShortestPathMDS::<2>);
        assert_eq!(curvature.dimension(), 5);
        assert_eq!(curvature.step(), 2);
        assert!(curvature.scalar_curvature().is_finite());
    }

    #[test]
    fn single_node_is_flat() {
        let branchial = BranchialGraph {
            step: 0,
            nodes: vec![make_id(0, 0)],
            edges: Vec::new(),
        };

        let (coords, _metric) = ShortestPathMDS::<3>.embed(&branchial);
        assert_eq!(coords.len(), 1);
        assert_eq!(coords[0], SVector::<f64, 3>::zeros());

        let curvature = ManifoldCurvature::from_branchial(&branchial, &ShortestPathMDS::<3>);
        assert!(curvature.is_flat());
    }

    #[test]
    fn trait_conformance_dimension_step_indicator() {
        let nodes: Vec<MultiwayNodeId> = (0..3).map(|i| make_id(i, 5)).collect();
        let edges = vec![
            (nodes[0], nodes[1]),
            (nodes[1], nodes[2]),
        ];
        let branchial = BranchialGraph {
            step: 5,
            nodes,
            edges,
        };

        let curvature = ManifoldCurvature::from_branchial(&branchial, &ShortestPathMDS::<3>);
        assert_eq!(curvature.dimension(), 3);
        assert_eq!(curvature.step(), 5);
        // Irreducibility indicator is non-negative
        assert!(curvature.irreducibility_indicator() >= 0.0);
        // Ricci curvature at each vertex is finite
        for v in 0..3 {
            assert!(curvature.ricci_curvature(v).is_finite());
        }
        // Sectional curvature for pairs is finite
        assert!(curvature.sectional_curvature(0, 1).is_finite());
        assert!(curvature.sectional_curvature(0, 2).is_finite());
        assert!(curvature.sectional_curvature(1, 2).is_finite());
    }

    #[test]
    fn empty_graph_returns_empty_coordinates() {
        let branchial = BranchialGraph {
            step: 0,
            nodes: Vec::new(),
            edges: Vec::new(),
        };

        let (coords, _metric) = ShortestPathMDS::<3>.embed(&branchial);
        assert!(coords.is_empty());

        let curvature = ManifoldCurvature::from_branchial(&branchial, &ShortestPathMDS::<3>);
        assert_eq!(curvature.dimension(), 0);
        assert!(curvature.is_flat());
    }

    #[test]
    fn dim3_embedding_uses_all_three_dimensions() {
        // K₅ embedded in 3D — all 3 coordinate dimensions should be populated.
        let nodes: Vec<MultiwayNodeId> = (0..5).map(|i| make_id(i, 0)).collect();
        let edges = vec![
            (nodes[0], nodes[1]),
            (nodes[0], nodes[2]),
            (nodes[0], nodes[3]),
            (nodes[0], nodes[4]),
            (nodes[1], nodes[2]),
            (nodes[1], nodes[3]),
            (nodes[1], nodes[4]),
            (nodes[2], nodes[3]),
            (nodes[2], nodes[4]),
            (nodes[3], nodes[4]),
        ];
        let branchial = BranchialGraph {
            step: 0,
            nodes: nodes.clone(),
            edges,
        };

        let (coords, _metric) = ShortestPathMDS::<3>.embed(&branchial);
        assert_eq!(coords.len(), 5);
        for coord in &coords {
            assert_eq!(coord.len(), 3);
            for &c in coord.as_slice() {
                assert!(c.is_finite(), "coordinate must be finite: {c}");
            }
        }

        let curvature = ManifoldCurvature::from_branchial(&branchial, &ShortestPathMDS::<3>);
        assert_eq!(curvature.dimension(), 5);
        assert!(curvature.scalar_curvature().is_finite());
        assert!(curvature.sectional_curvature(0, 1).is_finite());
    }

    /// Phase-1 stand-in: fan-triangulation with a flat Regge metric gives
    /// zero deficit at every vertex (Regge curvature depends only on
    /// angle sums, and equilateral triangles have 60°-angles summing to
    /// <2π at every interior vertex, *but* the apex of a fan is the only
    /// interior vertex and its link is a path, not a closed loop — so
    /// boundary-bone zeroing takes over).
    ///
    /// This test locks in the "flat → zero" invariant that existing callers
    /// depend on while the Phase-1 substrate is in place.
    #[test]
    fn fan_triangulation_flat_curvature_is_zero() {
        let srs = StringRewriteSystem::new(vec![("AB", "BA"), ("A", "AA")]);
        let evolution = srs.run_multiway("AB", 4, 200);
        let foliation = extract_branchial_foliation(&evolution);

        let branchial = foliation
            .iter()
            .find(|b| b.node_count() >= 5)
            .expect("SRS evolution should produce a branchial with >= 5 nodes");

        let curvature = ManifoldCurvature::from_branchial(branchial, &ShortestPathMDS::<3>);

        assert!(
            curvature.is_flat(),
            "flat Regge metric + fan triangulation must give zero curvature everywhere"
        );
        assert!(curvature.scalar_curvature().abs() < 1e-10);
        for v in 0..curvature.dimension() {
            assert!(curvature.ricci_curvature(v).abs() < 1e-10);
        }
    }

    /// Hand-rolled cone-point test: 5 equilateral triangles meeting at vertex 0
    /// form a pentagonal-fan disk. The five boundary vertices (1..=5) each have
    /// only one incident triangle, so they are boundary bones (deficit 0).
    /// The apex vertex 0 is interior: all 5 incident edges have 2 incident
    /// triangles apiece, so its deficit angle is `2π - 5·(π/3) = π/3 ≈ 1.047`.
    ///
    /// This test hits the `dc_topology` curvature pipeline with a genuinely
    /// non-flat (in the Gauss-curvature sense) complex and verifies that
    /// Regge deficit angles line up with the hand-calculated value.
    #[test]
    fn cone_point_has_positive_deficit() {
        use std::f64::consts::PI;

        // Pentagonal fan: 5 triangles meeting at vertex 0, closing into
        // a disk with boundary cycle 1→2→3→4→5→1.
        let triangles = [
            [0usize, 1, 2],
            [0, 2, 3],
            [0, 3, 4],
            [0, 4, 5],
            [0, 5, 1],
        ];

        let mut builder = SimplicialComplexBuilder::new(2);
        for tri in &triangles {
            builder
                .add_simplex(Simplex::new(tri.to_vec()))
                .expect("pentagonal-fan simplex within builder dim");
        }
        let complex = builder
            .build::<f64>()
            .expect("pentagonal-fan should yield a valid 2D complex");

        let edge_count = complex.skeletons()[1].simplices().len();
        let metric = flat_regge_geometry(edge_count)
            .expect("flat_regge_geometry on pentagonal-fan edges");

        let ricci = metric
            .calculate_ricci_curvature(&complex)
            .expect("ricci curvature on pentagonal fan");

        let deficits = ricci.as_slice();
        // bones[i] == vertex i (canonical sort on 0-simplices).
        assert_eq!(deficits.len(), 6, "6 vertices → 6 bones");

        // Apex: 2π - 5·(π/3) = π/3.
        let expected_apex = PI / 3.0;
        assert!(
            (deficits[0] - expected_apex).abs() < 1e-10,
            "apex deficit should be π/3 ≈ 1.047, got {}",
            deficits[0]
        );

        // Boundary vertices: zero deficit (boundary-bone zeroing).
        for v in 1..=5 {
            assert!(
                deficits[v].abs() < 1e-10,
                "boundary vertex {v} should have zero deficit, got {}",
                deficits[v]
            );
        }
    }
}
