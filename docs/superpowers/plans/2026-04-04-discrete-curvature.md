# Discrete Curvature Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the clustering-coefficient curvature heuristic with a trait-based architecture: Ollivier-Ricci (default) + manifold bridge (feature-gated).

**Architecture:** `DiscreteCurvature` trait in `curvature.rs` with two backends — `OllivierRicciCurvature` (always available, custom W₁ solver) and `ManifoldCurvature` (feature = "manifold-curvature", amari-calculus). `CurvatureFoliation` becomes generic over `C: DiscreteCurvature`.

**Tech Stack:** Rust 2024 edition, petgraph-free (BFS on adjacency), amari-calculus (optional)

**Spec:** `docs/superpowers/specs/2026-04-04-discrete-curvature-design.md`

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `src/machines/multiway/curvature.rs` | **Rewrite** | `DiscreteCurvature` trait + `CurvatureFoliation<C>` + type aliases |
| `src/machines/multiway/wasserstein.rs` | **Create** | Wasserstein-1 solver (min-cost flow, shortest-path augmentation) |
| `src/machines/multiway/ollivier_ricci.rs` | **Create** | `OllivierRicciCurvature` implementing `DiscreteCurvature` |
| `src/machines/multiway/manifold_bridge.rs` | **Create** | `ManifoldCurvature` + `BranchialEmbedding` trait (feature-gated) |
| `src/machines/multiway/mod.rs` | **Modify** | Module declarations, re-exports |
| `src/lib.rs` | **Modify** | Top-level re-exports |
| `Cargo.toml` | **Modify** | `manifold-curvature` feature gate + amari-calculus dep |
| `examples/gorard_demo.rs` | **Modify** | Migration: `BranchialCurvature` → `OllivierRicciCurvature` |
| `tests/multiway_evolution.rs` | **Modify** | Migration: same rename + trait method calls |

---

### Task 1: Wasserstein-1 Solver

**Files:**
- Create: `src/machines/multiway/wasserstein.rs`

This is the foundation — the Ollivier-Ricci backend depends on it.

- [ ] **Step 1: Write failing tests for W₁ identity and symmetry**

```rust
// src/machines/multiway/wasserstein.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn w1_identical_distributions_is_zero() {
        // Two identical distributions on 3 points
        let mu = vec![1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0];
        let dist = vec![
            vec![0.0, 1.0, 2.0],
            vec![1.0, 0.0, 1.0],
            vec![2.0, 1.0, 0.0],
        ];
        let result = wasserstein_1(&mu, &mu, &dist);
        assert!(result.abs() < 1e-10, "W1(mu, mu) should be 0, got {result}");
    }

    #[test]
    fn w1_symmetry() {
        let mu = vec![1.0, 0.0, 0.0];
        let nu = vec![0.0, 0.0, 1.0];
        let dist = vec![
            vec![0.0, 1.0, 2.0],
            vec![1.0, 0.0, 1.0],
            vec![2.0, 1.0, 0.0],
        ];
        let forward = wasserstein_1(&mu, &nu, &dist);
        let backward = wasserstein_1(&nu, &mu, &dist);
        assert!(
            (forward - backward).abs() < 1e-10,
            "W1 should be symmetric: {forward} vs {backward}"
        );
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib wasserstein -- --nocapture`
Expected: compilation error — `wasserstein_1` not defined

- [ ] **Step 3: Write the Wasserstein-1 solver**

Implement `wasserstein_1(mu: &[f64], nu: &[f64], distance: &[Vec<f64>]) -> f64` using shortest-path augmentation on the transportation network:

```rust
//! Wasserstein-1 (earth mover's) distance solver.
//!
//! Computes W₁(μ, ν) = min Σ T_ij · d_ij subject to transport constraints.
//! Uses shortest-path augmentation on the residual network (polynomial time,
//! no graph-size cap).

/// Compute Wasserstein-1 distance between two discrete distributions.
///
/// # Arguments
/// * `mu` - Source distribution (non-negative, sums to same total as `nu`)
/// * `nu` - Target distribution (non-negative, sums to same total as `mu`)
/// * `distance` - Pairwise distance matrix d[i][j]
///
/// # Returns
/// The earth mover's distance W₁(μ, ν).
#[must_use]
pub fn wasserstein_1(mu: &[f64], nu: &[f64], distance: &[Vec<f64>]) -> f64 {
    // ... shortest-path augmentation implementation
}
```

The algorithm:
1. Compute supply = mu[i] - nu[i] for each node
2. Build a complete bipartite residual graph with costs = distances
3. While there exists a node with positive excess supply:
   - Find shortest augmenting path (Dijkstra with Johnson's reweighting) from a supply node to a demand node
   - Push flow along the path, reducing excess
4. Total cost = W₁

For small instances (n < ~20, typical for branchial graphs), a simpler approach also works:
solve the LP directly via the transportation simplex. Implement whichever is cleaner — the interface is the same.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib wasserstein -- --nocapture`
Expected: 2 tests pass

- [ ] **Step 5: Add known-answer and triangle inequality tests**

```rust
#[test]
fn w1_dirac_masses_equals_distance() {
    // W1 between two Dirac deltas = distance between their supports
    let mu = vec![1.0, 0.0, 0.0];
    let nu = vec![0.0, 0.0, 1.0];
    let dist = vec![
        vec![0.0, 1.0, 3.0],
        vec![1.0, 0.0, 2.0],
        vec![3.0, 2.0, 0.0],
    ];
    let result = wasserstein_1(&mu, &nu, &dist);
    assert!(
        (result - 3.0).abs() < 1e-10,
        "W1 of Dirac deltas should equal distance: got {result}"
    );
}

#[test]
fn w1_triangle_inequality() {
    let dist = vec![
        vec![0.0, 1.0, 2.0],
        vec![1.0, 0.0, 1.0],
        vec![2.0, 1.0, 0.0],
    ];
    let mu = vec![1.0, 0.0, 0.0];
    let nu = vec![0.0, 1.0, 0.0];
    let rho = vec![0.0, 0.0, 1.0];

    let ab = wasserstein_1(&mu, &nu, &dist);
    let bc = wasserstein_1(&nu, &rho, &dist);
    let ac = wasserstein_1(&mu, &rho, &dist);

    assert!(
        ac <= ab + bc + 1e-10,
        "Triangle inequality violated: {ac} > {ab} + {bc}"
    );
}

#[test]
fn w1_uniform_to_skewed() {
    // Hand-computed: uniform [0.5, 0.5] vs [1.0, 0.0] on 2 points at distance 1
    // Optimal: move 0.5 mass from point 1 to point 0, cost = 0.5 * 1.0 = 0.5
    let mu = vec![0.5, 0.5];
    let nu = vec![1.0, 0.0];
    let dist = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
    let result = wasserstein_1(&mu, &nu, &dist);
    assert!(
        (result - 0.5).abs() < 1e-10,
        "Expected W1 = 0.5, got {result}"
    );
}
```

- [ ] **Step 6: Run all wasserstein tests**

Run: `cargo test --lib wasserstein -- --nocapture`
Expected: 6 tests pass

- [ ] **Step 7: Add module declaration (do not re-export yet)**

In `src/machines/multiway/mod.rs`, add:
```rust
mod wasserstein;
```

- [ ] **Step 8: Run cargo check**

Run: `cargo check`
Expected: compiles clean

- [ ] **Step 9: Commit**

```bash
git add src/machines/multiway/wasserstein.rs src/machines/multiway/mod.rs
git commit -m "feat: Wasserstein-1 solver for Ollivier-Ricci curvature"
```

---

### Task 2: DiscreteCurvature Trait and Generic CurvatureFoliation

**Files:**
- Rewrite: `src/machines/multiway/curvature.rs`

Extract the trait and make `CurvatureFoliation` generic. The old `BranchialCurvature` struct and its tests are removed entirely in this task — the Ollivier-Ricci replacement comes in Task 3.

- [ ] **Step 1: Write the DiscreteCurvature trait and generic CurvatureFoliation**

Rewrite `src/machines/multiway/curvature.rs` to contain:

```rust
//! Discrete curvature trait and foliation for multiway systems.
//!
//! Provides a trait-based architecture for computing curvature on branchial
//! graphs, with pluggable backends (Ollivier-Ricci, manifold embedding).

use std::fmt::Debug;
use std::hash::Hash;

use super::{BranchialGraph, MultiwayEvolutionGraph};

/// Discrete curvature on a branchial graph.
///
/// Two backends: Ollivier-Ricci (default) and Riemannian manifold embedding
/// (feature = "manifold-curvature").
pub trait DiscreteCurvature: Clone + Debug {
    /// Scalar curvature R (trace of Ricci). 0 = flat, >0 = sphere-like, <0 = saddle-like.
    fn scalar_curvature(&self) -> f64;

    /// Whether the branchial space is flat (within tolerance).
    fn is_flat(&self) -> bool;

    /// Ricci curvature at vertex i.
    fn ricci_curvature(&self, vertex: usize) -> f64;

    /// Sectional curvature for the 2-plane spanned by vertices i, j.
    fn sectional_curvature(&self, i: usize, j: usize) -> f64;

    /// Irreducibility indicator. Higher = more irreducible.
    fn irreducibility_indicator(&self) -> f64;

    /// Dimension (number of branches).
    fn dimension(&self) -> usize;

    /// Time step this curvature was computed for.
    fn step(&self) -> usize;
}

/// Curvature analysis across all time steps of a multiway evolution.
#[derive(Debug, Clone)]
pub struct CurvatureFoliation<C: DiscreteCurvature> {
    /// Curvature at each time step.
    pub curvatures: Vec<C>,
    /// Total scalar curvature (sum over all steps).
    pub total_scalar_curvature: f64,
    /// Average scalar curvature per step.
    pub average_scalar_curvature: f64,
    /// Maximum curvature magnitude encountered.
    pub max_curvature_magnitude: f64,
    /// Step at which maximum curvature occurs.
    pub max_curvature_step: usize,
}

impl<C: DiscreteCurvature> CurvatureFoliation<C> {
    /// Build a foliation from a precomputed list of per-step curvatures.
    #[must_use]
    pub fn from_curvatures(curvatures: Vec<C>) -> Self {
        let total_scalar: f64 = curvatures.iter().map(|c| c.scalar_curvature()).sum();
        #[allow(clippy::cast_precision_loss)]
        let average_scalar = if curvatures.is_empty() {
            0.0
        } else {
            total_scalar / curvatures.len() as f64
        };

        let (max_curvature_magnitude, max_curvature_step) = curvatures
            .iter()
            .enumerate()
            .map(|(i, c)| (c.scalar_curvature().abs(), i))
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or((0.0, 0));

        Self {
            curvatures,
            total_scalar_curvature: total_scalar,
            average_scalar_curvature: average_scalar,
            max_curvature_magnitude,
            max_curvature_step,
        }
    }

    /// Check if the entire evolution has flat branchial geometry.
    #[must_use]
    pub fn is_globally_flat(&self) -> bool {
        self.curvatures.iter().all(|c| c.is_flat())
    }

    /// Get the irreducibility profile over time.
    #[must_use]
    pub fn irreducibility_profile(&self) -> Vec<f64> {
        self.curvatures
            .iter()
            .map(DiscreteCurvature::irreducibility_indicator)
            .collect()
    }

    /// Compute average irreducibility indicator.
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn average_irreducibility(&self) -> f64 {
        if self.curvatures.is_empty() {
            return 0.0;
        }
        let total: f64 = self
            .curvatures
            .iter()
            .map(DiscreteCurvature::irreducibility_indicator)
            .sum();
        total / self.curvatures.len() as f64
    }
}

impl<C: DiscreteCurvature> std::fmt::Display for CurvatureFoliation<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Curvature Foliation:")?;
        writeln!(f, "  Steps analyzed: {}", self.curvatures.len())?;
        writeln!(f, "  Total scalar curvature: {:.6}", self.total_scalar_curvature)?;
        writeln!(f, "  Average scalar curvature: {:.6}", self.average_scalar_curvature)?;
        writeln!(
            f,
            "  Max curvature magnitude: {:.6} (at step {})",
            self.max_curvature_magnitude, self.max_curvature_step
        )?;
        writeln!(f, "  Globally flat: {}", self.is_globally_flat())?;
        write!(f, "  Average irreducibility: {:.6}", self.average_irreducibility())
    }
}
```

Note: `CurvatureFoliation::from_evolution()` moves to the Ollivier-Ricci module (Task 3) as it needs a concrete type.

- [ ] **Step 2: Run cargo check**

Run: `cargo check`
Expected: Compilation errors — consumers still reference `BranchialCurvature`. This is expected; we fix them in Task 3.

- [ ] **Step 3: Commit (WIP, will fix consumers in Task 3)**

```bash
git add src/machines/multiway/curvature.rs
git commit -m "refactor: DiscreteCurvature trait + generic CurvatureFoliation (WIP)"
```

---

### Task 3: Ollivier-Ricci Backend

**Files:**
- Create: `src/machines/multiway/ollivier_ricci.rs`
- Modify: `src/machines/multiway/mod.rs`

- [ ] **Step 1: Write known-answer tests for Ollivier-Ricci curvature**

```rust
// src/machines/multiway/ollivier_ricci.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::multiway::BranchialGraph;

    /// Build a BranchialGraph from explicit nodes and edges.
    fn make_branchial(step: usize, nodes: Vec<u32>, edges: Vec<(u32, u32)>) -> BranchialGraph {
        BranchialGraph { step, nodes, edges }
    }

    #[test]
    fn complete_graph_k4_has_positive_curvature() {
        // K_4: all edges have κ = 1 (neighbors overlap perfectly)
        let bg = make_branchial(0, vec![0, 1, 2, 3], vec![
            (0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3),
        ]);
        let curv = OllivierRicciCurvature::from_branchial(&bg);

        // All edge curvatures should be positive for complete graph
        for &(_, kappa) in &curv.edge_curvatures {
            assert!(kappa > 0.0, "K4 edge curvature should be positive, got {kappa}");
        }
        assert!(curv.scalar_curvature() > 0.0);
    }

    #[test]
    fn path_graph_has_negative_curvature() {
        // Path P_4: 0-1-2-3, interior edges have κ < 0
        let bg = make_branchial(0, vec![0, 1, 2, 3], vec![
            (0, 1), (1, 2), (2, 3),
        ]);
        let curv = OllivierRicciCurvature::from_branchial(&bg);

        // Interior edge (1,2) should have negative curvature
        let interior_kappa = curv.sectional_curvature(1, 2);
        assert!(interior_kappa < 0.0, "Path interior edge should have κ < 0, got {interior_kappa}");
    }

    #[test]
    fn single_node_is_flat() {
        let bg = make_branchial(0, vec![0], vec![]);
        let curv = OllivierRicciCurvature::from_branchial(&bg);

        assert!(curv.is_flat());
        assert!((curv.scalar_curvature() - 0.0).abs() < 1e-10);
        assert_eq!(curv.dimension(), 1);
        assert_eq!(curv.step(), 0);
    }

    #[test]
    fn two_connected_nodes_curvature() {
        // K_2: single edge, both nodes have degree 1
        // Neighbors of 0 = {1}, neighbors of 1 = {0}
        // μ_0 = Dirac at 1, μ_1 = Dirac at 0
        // W₁(μ_0, μ_1) = d(1, 0) = 1
        // κ(0, 1) = 1 - W₁/d = 1 - 1/1 = 0
        let bg = make_branchial(0, vec![0, 1], vec![(0, 1)]);
        let curv = OllivierRicciCurvature::from_branchial(&bg);

        let kappa = curv.sectional_curvature(0, 1);
        assert!(
            kappa.abs() < 1e-10,
            "K2 edge curvature should be 0, got {kappa}"
        );
    }

    #[test]
    fn dimension_and_step_are_correct() {
        let bg = make_branchial(7, vec![0, 1, 2], vec![(0, 1), (1, 2)]);
        let curv = OllivierRicciCurvature::from_branchial(&bg);
        assert_eq!(curv.dimension(), 3);
        assert_eq!(curv.step(), 7);
    }

    #[test]
    fn irreducibility_indicator_is_non_negative() {
        let bg = make_branchial(0, vec![0, 1, 2, 3], vec![
            (0, 1), (1, 2), (2, 3),
        ]);
        let curv = OllivierRicciCurvature::from_branchial(&bg);
        assert!(curv.irreducibility_indicator() >= 0.0);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib ollivier_ricci -- --nocapture`
Expected: compilation error — `OllivierRicciCurvature` not defined

- [ ] **Step 3: Implement OllivierRicciCurvature**

```rust
//! Ollivier-Ricci curvature for branchial graphs.
//!
//! Computes discrete Ricci curvature using optimal transport:
//! κ(x, y) = 1 - W₁(μ_x, μ_y) / d(x, y)
//!
//! where μ_x is the uniform distribution over neighbors of x.

use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

use super::curvature::{CurvatureFoliation, DiscreteCurvature};
use super::{BranchialGraph, MultiwayEvolutionGraph};
use super::wasserstein::wasserstein_1;

/// Ollivier-Ricci curvature computed from branchial graph structure.
#[derive(Clone, Debug)]
pub struct OllivierRicciCurvature {
    /// Per-edge curvature κ(i, j), stored as ((vertex_idx_i, vertex_idx_j), kappa).
    edge_curvatures: Vec<((usize, usize), f64)>,
    /// Per-vertex Ricci curvature (average of incident edge curvatures).
    vertex_curvatures: Vec<f64>,
    /// Scalar curvature (normalized sum of vertex curvatures).
    scalar: f64,
    /// Number of vertices.
    dim: usize,
    /// Time step.
    time_step: usize,
}

/// Type alias for convenience.
pub type OllivierFoliation = CurvatureFoliation<OllivierRicciCurvature>;
```

The implementation methods:

```rust
impl OllivierRicciCurvature {
    /// Compute Ollivier-Ricci curvature from a branchial graph.
    #[must_use]
    pub fn from_branchial(branchial: &BranchialGraph) -> Self {
        let n = branchial.node_count();

        if n <= 1 {
            return Self {
                edge_curvatures: Vec::new(),
                vertex_curvatures: vec![0.0; n],
                scalar: 0.0,
                dim: n,
                time_step: branchial.step,
            };
        }

        // 1. Build adjacency lists + node index mapping
        let node_to_idx: HashMap<_, _> = branchial.nodes.iter()
            .enumerate()
            .map(|(i, &id)| (id, i))
            .collect();

        let mut adj = vec![vec![]; n];
        let mut edge_indices = Vec::new();

        for &(a, b) in &branchial.edges {
            if let (Some(&i), Some(&j)) = (node_to_idx.get(&a), node_to_idx.get(&b)) {
                adj[i].push(j);
                adj[j].push(i);
                edge_indices.push((i, j));
            }
        }

        // 2. All-pairs shortest paths (BFS, unweighted)
        let dist = Self::all_pairs_bfs(&adj, n);

        // 3. Compute per-edge Ollivier-Ricci curvature
        let edge_curvatures: Vec<((usize, usize), f64)> = edge_indices
            .iter()
            .map(|&(i, j)| {
                let kappa = Self::edge_curvature(i, j, &adj, &dist);
                ((i, j), kappa)
            })
            .collect();

        // 4. Per-vertex Ricci = average of incident edge curvatures
        let mut vertex_curvatures = vec![0.0; n];
        let mut vertex_edge_counts = vec![0usize; n];

        for &((i, j), kappa) in &edge_curvatures {
            vertex_curvatures[i] += kappa;
            vertex_curvatures[j] += kappa;
            vertex_edge_counts[i] += 1;
            vertex_edge_counts[j] += 1;
        }

        for i in 0..n {
            if vertex_edge_counts[i] > 0 {
                vertex_curvatures[i] /= vertex_edge_counts[i] as f64;
            }
        }

        // 5. Scalar curvature = normalized sum of vertex Ricci
        let scalar = vertex_curvatures.iter().sum::<f64>() / n as f64;

        Self {
            edge_curvatures,
            vertex_curvatures,
            scalar,
            dim: n,
            time_step: branchial.step,
        }
    }

    /// Compute Ollivier-Ricci curvature from a multiway graph at a specific step.
    #[must_use]
    pub fn from_evolution_at_step<S: Clone + Hash, T: Clone>(
        graph: &MultiwayEvolutionGraph<S, T>,
        step: usize,
    ) -> Self {
        let branchial = BranchialGraph::from_evolution_at_step(graph, step);
        Self::from_branchial(&branchial)
    }

    /// BFS shortest paths from all sources.
    fn all_pairs_bfs(adj: &[Vec<usize>], n: usize) -> Vec<Vec<f64>> {
        let mut dist = vec![vec![f64::INFINITY; n]; n];
        for s in 0..n {
            dist[s][s] = 0.0;
            let mut queue = VecDeque::new();
            queue.push_back(s);
            while let Some(u) = queue.pop_front() {
                for &v in &adj[u] {
                    if dist[s][v].is_infinite() {
                        dist[s][v] = dist[s][u] + 1.0;
                        queue.push_back(v);
                    }
                }
            }
        }
        dist
    }

    /// Compute κ(x, y) = 1 - W₁(μ_x, μ_y) / d(x, y).
    fn edge_curvature(
        x: usize,
        y: usize,
        adj: &[Vec<usize>],
        dist: &[Vec<f64>],
    ) -> f64 {
        let d_xy = dist[x][y];
        if d_xy == 0.0 {
            return 0.0;
        }

        let mu_x = Self::neighbor_distribution(x, adj, dist[0].len());
        let mu_y = Self::neighbor_distribution(y, adj, dist[0].len());

        let w1 = wasserstein_1(&mu_x, &mu_y, dist);
        1.0 - w1 / d_xy
    }

    /// Uniform distribution over neighbors of vertex v.
    fn neighbor_distribution(v: usize, adj: &[Vec<usize>], n: usize) -> Vec<f64> {
        let mut mu = vec![0.0; n];
        let neighbors = &adj[v];
        if neighbors.is_empty() {
            // Isolated vertex: Dirac at self
            mu[v] = 1.0;
        } else {
            let weight = 1.0 / neighbors.len() as f64;
            for &u in neighbors {
                mu[u] = weight;
            }
        }
        mu
    }

    /// Check if branchial space is geometrically simple.
    #[must_use]
    pub fn is_geometrically_simple(&self) -> bool {
        self.is_flat() && self.dim <= 2
    }

    /// Branchial complexity as a dimensionless ratio in [0, 1].
    #[must_use]
    pub fn branchial_complexity(&self) -> f64 {
        if self.dim <= 1 {
            return 0.0;
        }
        self.scalar.abs().min(1.0)
    }
}

impl DiscreteCurvature for OllivierRicciCurvature {
    fn scalar_curvature(&self) -> f64 {
        self.scalar
    }

    fn is_flat(&self) -> bool {
        self.scalar.abs() < 1e-10
            && self.edge_curvatures.iter().all(|(_, k)| k.abs() < 1e-10)
    }

    fn ricci_curvature(&self, vertex: usize) -> f64 {
        self.vertex_curvatures.get(vertex).copied().unwrap_or(0.0)
    }

    fn sectional_curvature(&self, i: usize, j: usize) -> f64 {
        // Look up edge curvature for this pair
        self.edge_curvatures
            .iter()
            .find(|&&((a, b), _)| (a == i && b == j) || (a == j && b == i))
            .map_or(0.0, |&(_, k)| k)
    }

    fn irreducibility_indicator(&self) -> f64 {
        // Use absolute scalar curvature + variance of edge curvatures
        let variance = if self.edge_curvatures.is_empty() {
            0.0
        } else {
            let mean: f64 = self.edge_curvatures.iter().map(|(_, k)| k).sum::<f64>()
                / self.edge_curvatures.len() as f64;
            self.edge_curvatures.iter().map(|(_, k)| (k - mean).powi(2)).sum::<f64>()
                / self.edge_curvatures.len() as f64
        };
        self.scalar.abs() + variance.sqrt()
    }

    fn dimension(&self) -> usize {
        self.dim
    }

    fn step(&self) -> usize {
        self.time_step
    }
}

impl std::fmt::Display for OllivierRicciCurvature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Ollivier-Ricci Curvature (step {}):", self.time_step)?;
        writeln!(f, "  Dimension: {}", self.dim)?;
        writeln!(f, "  Scalar curvature: {:.6}", self.scalar)?;
        writeln!(f, "  Is flat: {}", self.is_flat())?;
        writeln!(f, "  Edges computed: {}", self.edge_curvatures.len())?;
        writeln!(f, "  Irreducibility indicator: {:.6}", self.irreducibility_indicator())?;
        write!(f, "  Branchial complexity: {:.4}", self.branchial_complexity())
    }
}
```

Also add `OllivierFoliation::from_evolution`:

```rust
impl OllivierFoliation {
    /// Compute Ollivier-Ricci curvature foliation from a multiway evolution graph.
    #[must_use]
    pub fn from_evolution<S: Clone + Hash, T: Clone>(
        graph: &MultiwayEvolutionGraph<S, T>,
    ) -> Self {
        let max_step = graph.max_step();
        let curvatures: Vec<OllivierRicciCurvature> = (0..=max_step)
            .map(|step| OllivierRicciCurvature::from_evolution_at_step(graph, step))
            .collect();
        Self::from_curvatures(curvatures)
    }
}
```

- [ ] **Step 4: Update module declarations in mod.rs**

In `src/machines/multiway/mod.rs`, replace:

```rust
// Old:
pub use curvature::{BranchialCurvature, CurvatureFoliation};
```

With:

```rust
pub mod ollivier_ricci;

pub use curvature::{CurvatureFoliation, DiscreteCurvature};
pub use ollivier_ricci::{OllivierFoliation, OllivierRicciCurvature};
```

- [ ] **Step 5: Run unit tests**

Run: `cargo test --lib ollivier_ricci -- --nocapture`
Expected: 7 tests pass

- [ ] **Step 6: Commit**

```bash
git add src/machines/multiway/ollivier_ricci.rs src/machines/multiway/mod.rs
git commit -m "feat: Ollivier-Ricci curvature backend implementing DiscreteCurvature"
```

---

### Task 4: Migration — lib.rs, Examples, Integration Tests

**Files:**
- Modify: `src/lib.rs:96`
- Modify: `examples/gorard_demo.rs:33, 774`
- Modify: `tests/multiway_evolution.rs:7, 160-193`

- [ ] **Step 1: Update lib.rs re-exports**

In `src/lib.rs`, replace line 96:

```rust
// Old:
BranchialCurvature, CurvatureFoliation,
```

With:

```rust
CurvatureFoliation, DiscreteCurvature,
OllivierFoliation, OllivierRicciCurvature,
```

- [ ] **Step 2: Update gorard_demo.rs**

In `examples/gorard_demo.rs`, line 33: replace `curvature::CurvatureFoliation` with `curvature::CurvatureFoliation` and `ollivier_ricci::OllivierFoliation`.

At line 774: replace `CurvatureFoliation::from_evolution` with `OllivierFoliation::from_evolution`.

- [ ] **Step 3: Update tests/multiway_evolution.rs**

Line 7: replace `use irreducible::{BranchialCurvature, ...}` with `use irreducible::{OllivierRicciCurvature, ...}`.

Lines 160-177 (`curvature_computation_on_multiway_graph`):
- Replace `BranchialCurvature::from_branchial` with `OllivierRicciCurvature::from_branchial`
- Replace `curvature.dimension` with `curvature.dimension()`
- Replace `curvature.scalar_curvature` with `curvature.scalar_curvature()`
- Replace `curvature.is_flat` with `curvature.is_flat()`
- Replace `curvature.step` with `curvature.step()`

Lines 180-194 (`curvature_foliation_across_steps`):
- Replace `use irreducible::CurvatureFoliation` with `use irreducible::OllivierFoliation`
- Replace `CurvatureFoliation::from_evolution` with `OllivierFoliation::from_evolution`
- Replace `curvature_foliation.curvatures[0].is_flat` with `curvature_foliation.curvatures[0].is_flat()`

- [ ] **Step 4: Run cargo check**

Run: `cargo check`
Expected: compiles clean (no more references to `BranchialCurvature`)

- [ ] **Step 5: Run full test suite**

Run: `cargo test --workspace`
Expected: all tests pass

- [ ] **Step 6: Run clippy**

Run: `cargo clippy --workspace -- -W clippy::pedantic`
Expected: zero warnings

- [ ] **Step 7: Run the gorard demo**

Run: `cargo run --example gorard_demo`
Expected: demo runs, curvature section shows Ollivier-Ricci output

- [ ] **Step 8: Commit**

```bash
git add src/lib.rs examples/gorard_demo.rs tests/multiway_evolution.rs
git commit -m "refactor: migrate consumers from BranchialCurvature to OllivierRicciCurvature"
```

---

### Task 5: Manifold Bridge Scaffolding (Feature-Gated)

**Files:**
- Create: `src/machines/multiway/manifold_bridge.rs`
- Modify: `Cargo.toml`
- Modify: `src/machines/multiway/mod.rs`

This task creates the `ManifoldCurvature` struct, the `BranchialEmbedding` trait, and the feature gate. No concrete embedding algorithm is implemented — the embedding strategy is explicitly deferred per the spec.

- [ ] **Step 1: Add feature gate and amari-calculus dependency to Cargo.toml**

Add to `[workspace.dependencies]`:
```toml
amari-calculus = { path = "/home/oryx/Documents/industrial-algebra/Amari/amari-calculus" }
```

Add to `[features]`:
```toml
manifold-curvature = ["dep:amari-calculus"]
```

Add to `[dependencies]`:
```toml
amari-calculus = { workspace = true, optional = true }
```

- [ ] **Step 2: Write manifold_bridge.rs**

```rust
//! Manifold bridge for branchial curvature via amari-calculus.
//!
//! Embeds a branchial graph into a smooth Riemannian manifold and
//! computes curvature using differential geometry. The embedding
//! algorithm is pluggable via the [`BranchialEmbedding`] trait.
//!
//! # Feature Gate
//!
//! Requires `manifold-curvature` feature.

use super::curvature::DiscreteCurvature;
use super::BranchialGraph;
use amari_calculus::manifold::{MetricTensor, RiemannianManifold};

/// Strategy for embedding a branchial graph into a smooth manifold.
///
/// The embedding maps discrete graph structure to continuous coordinates
/// with a metric tensor, enabling Riemannian curvature computation.
///
/// **Note:** Concrete embedding algorithms (shortest-path MDS, Laplacian
/// spectral, etc.) are deferred. This trait defines the contract;
/// implementors provide the specific strategy.
pub trait BranchialEmbedding<const DIM: usize> {
    /// Embed branchial graph into coordinates + metric tensor.
    ///
    /// Returns a coordinate per vertex and a metric tensor for the space.
    fn embed(&self, branchial: &BranchialGraph) -> (Vec<[f64; DIM]>, MetricTensor<DIM>);
}

/// Riemannian curvature of a branchial graph via manifold embedding.
#[derive(Clone, Debug)]
pub struct ManifoldCurvature {
    /// Per-vertex Ricci curvature R_ii from the Riemannian manifold.
    vertex_curvatures: Vec<f64>,
    /// Sectional curvatures for vertex pairs, stored as ((i, j), K_ij).
    sectional_curvatures: Vec<((usize, usize), f64)>,
    /// Scalar curvature from amari-calculus.
    scalar: f64,
    /// Embedding dimension (may differ from graph vertex count).
    embedding_dim: usize,
    /// Number of vertices.
    dim: usize,
    /// Time step.
    time_step: usize,
}

impl ManifoldCurvature {
    /// Compute manifold curvature using a specific embedding strategy.
    #[must_use]
    pub fn from_branchial<const DIM: usize>(
        branchial: &BranchialGraph,
        embedding: &impl BranchialEmbedding<DIM>,
    ) -> Self {
        let n = branchial.node_count();

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

        // Compute per-vertex Ricci curvature
        let vertex_curvatures: Vec<f64> = coords
            .iter()
            .map(|coord| {
                // Ricci curvature at vertex = sum of R_ii components
                (0..DIM)
                    .map(|i| manifold.ricci_tensor(i, i, coord.as_slice()))
                    .sum()
            })
            .collect();

        // Compute sectional curvatures for each pair
        let mut sectional_curvatures = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                // Midpoint between embedded positions
                let midpoint: Vec<f64> = (0..DIM)
                    .map(|d| (coords[i][d] + coords[j][d]) / 2.0)
                    .collect();
                // Use first two coordinate directions for sectional curvature
                let k_ij = if DIM >= 2 {
                    manifold.riemann_tensor(0, 1, 0, 1, &midpoint)
                } else {
                    0.0
                };
                sectional_curvatures.push(((i, j), k_ij));
            }
        }

        // Scalar curvature at centroid
        let centroid: Vec<f64> = (0..DIM)
            .map(|d| coords.iter().map(|c| c[d]).sum::<f64>() / n as f64)
            .collect();
        let scalar = manifold.scalar_curvature(&centroid);

        Self {
            vertex_curvatures,
            sectional_curvatures,
            scalar,
            embedding_dim: DIM,
            dim: n,
            time_step: branchial.step,
        }
    }
}

impl DiscreteCurvature for ManifoldCurvature {
    fn scalar_curvature(&self) -> f64 {
        self.scalar
    }

    fn is_flat(&self) -> bool {
        self.scalar.abs() < 1e-10
            && self.sectional_curvatures.iter().all(|(_, k)| k.abs() < 1e-10)
    }

    fn ricci_curvature(&self, vertex: usize) -> f64 {
        self.vertex_curvatures.get(vertex).copied().unwrap_or(0.0)
    }

    fn sectional_curvature(&self, i: usize, j: usize) -> f64 {
        self.sectional_curvatures
            .iter()
            .find(|&&((a, b), _)| (a == i && b == j) || (a == j && b == i))
            .map_or(0.0, |&(_, k)| k)
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

impl std::fmt::Display for ManifoldCurvature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Manifold Curvature (step {}):", self.time_step)?;
        writeln!(f, "  Graph dimension: {}", self.dim)?;
        writeln!(f, "  Embedding dimension: {}", self.embedding_dim)?;
        writeln!(f, "  Scalar curvature: {:.6}", self.scalar)?;
        writeln!(f, "  Is flat: {}", self.is_flat())?;
        write!(f, "  Irreducibility indicator: {:.6}", self.irreducibility_indicator())
    }
}

/// Type alias for manifold-based curvature foliation.
pub type ManifoldFoliation = super::curvature::CurvatureFoliation<ManifoldCurvature>;
```

- [ ] **Step 3: Add module declaration in mod.rs**

In `src/machines/multiway/mod.rs`, add:

```rust
#[cfg(feature = "manifold-curvature")]
pub mod manifold_bridge;

// After existing re-exports:
#[cfg(feature = "manifold-curvature")]
pub use manifold_bridge::{BranchialEmbedding, ManifoldCurvature, ManifoldFoliation};
```

- [ ] **Step 4: Run cargo check (without feature)**

Run: `cargo check`
Expected: compiles clean — manifold code is feature-gated

- [ ] **Step 5: Run cargo check (with feature)**

Run: `cargo check --features manifold-curvature`
Expected: compiles clean with amari-calculus linked

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/machines/multiway/manifold_bridge.rs src/machines/multiway/mod.rs
git commit -m "feat: ManifoldCurvature + BranchialEmbedding trait (feature = manifold-curvature)"
```

---

### Task 6: Trait-Level Generic Tests and Stress Tests

**Files:**
- Modify: `src/machines/multiway/curvature.rs` (add generic test helpers)
- Modify: `src/machines/multiway/ollivier_ricci.rs` (add stress test)

- [ ] **Step 1: Add generic trait conformance tests to curvature.rs**

```rust
// At the bottom of src/machines/multiway/curvature.rs

#[cfg(test)]
pub(crate) mod test_helpers {
    use super::DiscreteCurvature;

    /// Verify trait conformance for any DiscreteCurvature implementation.
    pub fn assert_trait_conformance<C: DiscreteCurvature>(curv: &C, expected_dim: usize, expected_step: usize) {
        assert_eq!(curv.dimension(), expected_dim, "dimension mismatch");
        assert_eq!(curv.step(), expected_step, "step mismatch");
        assert!(curv.irreducibility_indicator() >= 0.0, "indicator must be non-negative");

        // If flat, scalar curvature should be near zero
        if curv.is_flat() {
            assert!(
                curv.scalar_curvature().abs() < 1e-6,
                "flat curvature should have scalar ≈ 0, got {}",
                curv.scalar_curvature()
            );
        }
    }
}
```

- [ ] **Step 2: Call generic conformance from ollivier_ricci tests**

Add to the existing tests in `ollivier_ricci.rs`:

```rust
#[test]
fn trait_conformance_flat() {
    use super::super::curvature::test_helpers::assert_trait_conformance;
    let bg = make_branchial(3, vec![0], vec![]);
    let curv = OllivierRicciCurvature::from_branchial(&bg);
    assert_trait_conformance(&curv, 1, 3);
}

#[test]
fn trait_conformance_nontrivial() {
    use super::super::curvature::test_helpers::assert_trait_conformance;
    let bg = make_branchial(0, vec![0, 1, 2, 3], vec![
        (0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3),
    ]);
    let curv = OllivierRicciCurvature::from_branchial(&bg);
    assert_trait_conformance(&curv, 4, 0);
}
```

- [ ] **Step 3: Add W₁ stress test**

Add to `wasserstein.rs` tests:

```rust
#[test]
fn stress_test_100_nodes() {
    // 100-node path graph: distance matrix is |i - j|
    let n = 100;
    let dist: Vec<Vec<f64>> = (0..n)
        .map(|i| (0..n).map(|j| (i as f64 - j as f64).abs()).collect())
        .collect();

    // Uniform vs shifted uniform
    let mu: Vec<f64> = (0..n).map(|i| if i < 50 { 1.0 / 50.0 } else { 0.0 }).collect();
    let nu: Vec<f64> = (0..n).map(|i| if i >= 50 { 1.0 / 50.0 } else { 0.0 }).collect();

    let result = wasserstein_1(&mu, &nu, &dist);
    assert!(result > 0.0, "W1 should be positive for disjoint supports");
    assert!(result.is_finite(), "W1 should be finite");
}
```

- [ ] **Step 4: Run all tests**

Run: `cargo test --workspace`
Expected: all tests pass

- [ ] **Step 5: Run clippy**

Run: `cargo clippy --workspace -- -W clippy::pedantic`
Expected: zero warnings

- [ ] **Step 6: Commit**

```bash
git add src/machines/multiway/curvature.rs src/machines/multiway/ollivier_ricci.rs src/machines/multiway/wasserstein.rs
git commit -m "test: generic trait conformance tests + W1 stress test"
```

---

### Task 7: CLAUDE.md and Issue Cleanup

**Files:**
- Modify: `CLAUDE.md`
- Close: GitHub issue #6

- [ ] **Step 1: Update CLAUDE.md**

Update the workspace structure to reflect new files:

```
│       ├── multiway/
│       │   ├── ...
│       │   ├── curvature.rs           # DiscreteCurvature trait, CurvatureFoliation<C>
│       │   ├── ollivier_ricci.rs      # OllivierRicciCurvature, OllivierFoliation
│       │   ├── wasserstein.rs         # Wasserstein-1 solver (min-cost flow)
│       │   └── manifold_bridge.rs     # ManifoldCurvature, BranchialEmbedding (feature-gated)
```

Update the Key Types table: replace `BranchialCurvature` with `OllivierRicciCurvature` and add `DiscreteCurvature`, `ManifoldCurvature`.

Update the Feature Flags table: add `manifold-curvature` row.

Update test counts after verifying final count.

- [ ] **Step 2: Run final test count**

Run: `cargo test --workspace 2>&1 | tail -20`

Update the test count in CLAUDE.md accordingly.

- [ ] **Step 3: Commit CLAUDE.md**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md for discrete curvature refactor"
```

- [ ] **Step 4: Close issue #6**

```bash
gh issue close 6 --repo tsondru/irreducible --comment "Resolved: Ollivier-Ricci replaces clustering coefficient. Manifold bridge (amari-calculus) available via feature gate. See docs/superpowers/specs/2026-04-04-discrete-curvature-design.md"
```
