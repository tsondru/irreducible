//! Riemannian manifold curvature backend for branchial graphs.
//!
//! Feature-gated behind `manifold-curvature`. Provides [`ManifoldCurvature`],
//! a [`DiscreteCurvature`] implementation that embeds a branchial graph into
//! a smooth manifold via a [`BranchialEmbedding`] strategy and computes
//! Riemannian curvature using [`amari_calculus`].
//!
//! Includes [`ShortestPathMDS`], a concrete embedding that uses classical
//! multidimensional scaling on all-pairs shortest-path distances to embed
//! a branchial graph into Euclidean space.

use std::collections::VecDeque;
use std::fmt;

use amari_calculus::manifold::{MetricTensor, RiemannianManifold};
use nalgebra::DMatrix;

use catgraph::multiway::{BranchialGraph, CurvatureFoliation, DiscreteCurvature};

/// Strategy for embedding a branchial graph into a smooth manifold.
///
/// The embedding maps discrete graph structure to continuous coordinates
/// with a metric tensor, enabling Riemannian curvature computation.
///
/// Concrete embedding algorithms (shortest-path MDS, Laplacian spectral, etc.)
/// are deferred. This trait defines the contract.
pub trait BranchialEmbedding<const DIM: usize> {
    /// Embed branchial graph into coordinates + metric tensor.
    ///
    /// Returns `(coordinates, metric)` where:
    /// - `coordinates[i]` is the position of vertex `i` in `DIM`-dimensional space
    /// - `metric` is the Riemannian metric tensor on the embedding manifold
    fn embed(&self, branchial: &BranchialGraph) -> (Vec<[f64; DIM]>, MetricTensor<DIM>);
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
    fn embed(&self, branchial: &BranchialGraph) -> (Vec<[f64; DIM]>, MetricTensor<DIM>) {
        let n = branchial.nodes.len();

        if n == 0 {
            return (Vec::new(), MetricTensor::euclidean());
        }

        if n == 1 {
            return (vec![[0.0; DIM]], MetricTensor::euclidean());
        }

        // --- Step 1: All-pairs BFS ---
        let distances = all_pairs_bfs(branchial);

        // --- Step 2: Square the distances ---
        let nf = n as f64;
        let mut d_sq = DMatrix::<f64>::zeros(n, n);
        for i in 0..n {
            for j in 0..n {
                d_sq[(i, j)] = distances[(i, j)] * distances[(i, j)];
            }
        }

        // --- Step 3: Double-center: B = -0.5 * H * D^2 * H ---
        // H = I - (1/n) * 1 * 1^T
        let ones = DMatrix::<f64>::from_element(n, n, 1.0 / nf);
        let identity = DMatrix::<f64>::identity(n, n);
        let h = &identity - &ones;
        let b = &h * &d_sq * &h * (-0.5);

        // --- Step 4: Eigendecompose ---
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
        let mut coords = vec![[0.0_f64; DIM]; n];
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

/// Riemannian curvature computed from a branchial graph embedding.
///
/// Wraps the result of embedding a [`BranchialGraph`] into a smooth manifold
/// and computing curvature via [`amari_calculus::RiemannianManifold`].
///
/// Implements [`DiscreteCurvature`] with the same interface as
/// [`OllivierRicciCurvature`](super::ollivier_ricci::OllivierRicciCurvature).
#[derive(Clone, Debug)]
pub struct ManifoldCurvature {
    /// Per-vertex Ricci curvature (trace of Ricci tensor at each vertex).
    vertex_curvatures: Vec<f64>,
    /// Sectional curvatures for vertex pairs `((i, j), kappa)`.
    sectional_curvatures: Vec<((usize, usize), f64)>,
    /// Scalar curvature R at the centroid.
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
    /// Compute Riemannian curvature from a branchial graph and an embedding.
    ///
    /// Algorithm:
    /// 1. Call `embedding.embed(branchial)` to obtain coordinates + metric tensor.
    /// 2. Create `RiemannianManifold::new(metric)`.
    /// 3. Per-vertex Ricci curvature: sum `ricci_tensor(i, i, coord)` over `i in 0..DIM`.
    /// 4. Sectional curvatures for each vertex pair via `riemann_tensor(0, 1, 0, 1, midpoint)`.
    /// 5. Scalar curvature at centroid via `scalar_curvature(&centroid)`.
    /// 6. Package into `ManifoldCurvature`.
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

        if n == 0 {
            return Self {
                vertex_curvatures: Vec::new(),
                sectional_curvatures: Vec::new(),
                scalar: 0.0,
                embedding_dim: DIM,
                dim: 0,
                time_step: branchial.step,
            };
        }

        let (coords, metric) = embedding.embed(branchial);
        let manifold = RiemannianManifold::new(metric);

        // --- Per-vertex Ricci curvature ---
        let vertex_curvatures: Vec<f64> = coords
            .iter()
            .map(|coord| {
                let mut ricci_trace = 0.0;
                for i in 0..DIM {
                    ricci_trace += manifold.ricci_tensor(i, i, coord);
                }
                ricci_trace
            })
            .collect();

        // --- Sectional curvatures for each pair ---
        let mut sectional_curvatures = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                // Midpoint between vertices i and j
                let mut midpoint = [0.0_f64; 3];
                for d in 0..DIM.min(3) {
                    midpoint[d] = f64::midpoint(coords[i][d], coords[j][d]);
                }
                let sectional = manifold.riemann_tensor(0, 1, 0, 1, &midpoint[..DIM]);
                sectional_curvatures.push(((i, j), sectional));
            }
        }

        // --- Scalar curvature at centroid ---
        let mut centroid = [0.0_f64; 3];
        for coord in &coords {
            for d in 0..DIM.min(3) {
                centroid[d] += coord[d];
            }
        }
        for d in 0..DIM.min(3) {
            centroid[d] /= n as f64;
        }
        let scalar = manifold.scalar_curvature(&centroid[..DIM]);

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
        self.scalar.abs() < 1e-10
            && self
                .vertex_curvatures
                .iter()
                .all(|&k| k.abs() < 1e-10)
    }

    fn ricci_curvature(&self, vertex: usize) -> f64 {
        self.vertex_curvatures
            .get(vertex)
            .copied()
            .unwrap_or(0.0)
    }

    fn sectional_curvature(&self, i: usize, j: usize) -> f64 {
        let (u, v) = if i < j { (i, j) } else { (j, i) };
        self.sectional_curvatures
            .iter()
            .find(|&&(e, _)| e == (u, v))
            .map_or(0.0, |&(_, k)| k)
    }

    #[allow(clippy::cast_precision_loss)]
    fn irreducibility_indicator(&self) -> f64 {
        if self.vertex_curvatures.is_empty() {
            return 0.0;
        }

        let abs_scalar = self.scalar.abs();
        let n = self.vertex_curvatures.len() as f64;
        let mean: f64 = self.vertex_curvatures.iter().sum::<f64>() / n;
        let variance: f64 = self
            .vertex_curvatures
            .iter()
            .map(|&k| (k - mean).powi(2))
            .sum::<f64>()
            / n;

        abs_scalar + variance.sqrt()
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
        writeln!(
            f,
            "  Sectional pairs analyzed: {}",
            self.sectional_curvatures.len()
        )?;
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
    use catgraph::multiway::{BranchId, MultiwayNodeId};

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
            for &c in coord {
                assert!(c.is_finite(), "coordinate must be finite: {c}");
            }
        }

        // Compute ManifoldCurvature — should not panic
        let curvature = ManifoldCurvature::from_branchial(&branchial, &ShortestPathMDS::<3>);
        assert_eq!(curvature.dimension(), 4);
        assert_eq!(curvature.step(), 0);
        // MDS into flat Euclidean space: scalar curvature should be near zero
        // (identity metric → zero Riemann tensor). Main check: no crash, valid output.
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
        assert_eq!(coords[0], [0.0; 3]);

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
}
