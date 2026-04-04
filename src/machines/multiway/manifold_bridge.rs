//! Riemannian manifold curvature backend for branchial graphs.
//!
//! Feature-gated behind `manifold-curvature`. Provides [`ManifoldCurvature`],
//! a [`DiscreteCurvature`] implementation that embeds a branchial graph into
//! a smooth manifold via an [`BranchialEmbedding`] strategy and computes
//! Riemannian curvature using [`amari_calculus`].
//!
//! **No concrete embedding algorithm is included** -- only the trait contract
//! and the curvature struct. Concrete embeddings (shortest-path MDS,
//! Laplacian spectral, etc.) are deferred to downstream crates.

use std::fmt;

use amari_calculus::manifold::{MetricTensor, RiemannianManifold};

use super::branchial::BranchialGraph;
use super::curvature::{CurvatureFoliation, DiscreteCurvature};

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

/// Type alias for a foliation parameterized by the manifold curvature backend.
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
