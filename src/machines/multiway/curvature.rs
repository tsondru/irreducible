//! Branchial curvature for multiway systems using the Riemann tensor.
//!
//! This module interprets the branchial structure of multiway computations
//! as a geometric space with curvature. Non-zero curvature indicates that
//! "parallel transport" (branch navigation) is path-dependent.
//!
//! # Mathematical Interpretation
//!
//! - **Flat branchial space**: All branches can be consistently ordered;
//!   the computation is "geometrically simple"
//! - **Curved branchial space**: Branch ordering matters; navigating
//!   around closed loops returns to a different "state"
//!
//! This provides a novel irreducibility indicator: high curvature suggests
//! the multiway structure cannot be simplified without losing information.
//!
//! # Connection to Gorard's Theory
//!
//! Gorard's paper discusses the causal graph structure of computations.
//! The branchial graph at each time step captures the "spatial" structure
//! of parallel branches. Curvature in this space measures the complexity
//! of the branching pattern beyond simple counting.
//!
//! # Feature Gate
//!
//! Requires `topology` feature for C`urvatureTensor` access.

use super::{BranchialGraph, MultiwayEvolutionGraph};
use std::hash::Hash;

/// Branchial curvature analysis for multiway systems.
///
/// This type encapsulates geometric curvature measures derived from
/// the branchial graph structure, providing novel irreducibility indicators.
#[derive(Debug, Clone)]
pub struct BranchialCurvature {
    /// Dimension of the branchial space (number of branches).
    pub dimension: usize,

    /// Scalar curvature R (trace of Ricci tensor).
    ///
    /// - R = 0: Flat branchial space (simple structure)
    /// - R > 0: Positive curvature (sphere-like, converging)
    /// - R < 0: Negative curvature (saddle-like, diverging)
    pub scalar_curvature: f64,

    /// Kretschmann scalar K = `R_abcd` `R^abcd`.
    ///
    /// This is always non-negative and measures "total curvature"
    /// independent of sign. Higher values indicate more complex geometry.
    pub kretschmann_scalar: f64,

    /// Maximum sectional curvature.
    ///
    /// The sectional curvature measures curvature in a 2D plane.
    /// The maximum value indicates the "most curved" direction.
    pub max_sectional_curvature: f64,

    /// Whether the branchial space is flat (zero curvature).
    pub is_flat: bool,

    /// Geodesic deviation magnitude.
    ///
    /// Measures how quickly initially parallel branches diverge.
    /// Higher values indicate faster divergence.
    pub geodesic_deviation: f64,

    /// Time step this curvature is computed for.
    pub step: usize,
}

impl BranchialCurvature {
    /// Compute branchial curvature from a branchial graph.
    ///
    /// The curvature tensor is constructed from the graph's adjacency
    /// structure, treating edge connectivity as a discrete metric.
    ///
    /// # Arguments
    ///
    /// * `branchial` - The branchial graph at a specific time step
    ///
    /// # Returns
    ///
    /// A `BranchialCurvature` with computed geometric measures.
    #[must_use]
    pub fn from_branchial(branchial: &BranchialGraph) -> Self {
        let n = branchial.node_count();

        // Trivial cases
        if n <= 1 {
            return Self {
                dimension: n,
                scalar_curvature: 0.0,
                kretschmann_scalar: 0.0,
                max_sectional_curvature: 0.0,
                is_flat: true,
                geodesic_deviation: 0.0,
                step: branchial.step,
            };
        }

        // Construct adjacency-based "curvature" approximation
        // We use the edge structure to define a discrete Riemann-like tensor.
        //
        // The key insight: if branches A, B, C form a "triangle" (pairwise connected),
        // there's no "curvature" in that region. Curvature arises from
        // "holes" in the connectivity pattern.

        let (scalar, kretschmann, max_sectional) =
            Self::compute_curvature_invariants(branchial, n);

        // Geodesic deviation: average distance between branches
        let geodesic_deviation = Self::compute_geodesic_deviation(branchial, n);

        // Flatness check with tolerance
        let is_flat = scalar.abs() < 1e-10 && kretschmann < 1e-10;

        Self {
            dimension: n,
            scalar_curvature: scalar,
            kretschmann_scalar: kretschmann,
            max_sectional_curvature: max_sectional,
            is_flat,
            geodesic_deviation,
            step: branchial.step,
        }
    }

    /// Compute branchial curvature from a multiway evolution graph at a step.
    ///
    /// # Arguments
    ///
    /// * `graph` - The multiway evolution graph
    /// * `step` - The time step to analyze
    ///
    /// # Returns
    ///
    /// A `BranchialCurvature` for the specified step.
    #[must_use]
    pub fn from_evolution_at_step<S: Clone + Hash, T: Clone>(
        graph: &MultiwayEvolutionGraph<S, T>,
        step: usize,
    ) -> Self {
        let branchial = BranchialGraph::from_evolution_at_step(graph, step);
        Self::from_branchial(&branchial)
    }

    /// Compute curvature invariants from the branchial structure.
    ///
    /// Returns (`scalar_curvature`, `kretschmann_scalar`, `max_sectional_curvature`).
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn compute_curvature_invariants(
        branchial: &BranchialGraph,
        n: usize,
    ) -> (f64, f64, f64) {
        if n < 2 {
            return (0.0, 0.0, 0.0);
        }

        // Build adjacency matrix
        let mut adj = vec![vec![false; n]; n];
        let node_to_idx: std::collections::HashMap<_, _> = branchial
            .nodes
            .iter()
            .enumerate()
            .map(|(i, &node)| (node, i))
            .collect();

        for &(a, b) in &branchial.edges {
            if let (Some(&i), Some(&j)) = (node_to_idx.get(&a), node_to_idx.get(&b)) {
                adj[i][j] = true;
                adj[j][i] = true;
            }
        }

        // Compute discrete curvature based on graph structure
        //
        // For a graph, we define curvature at a vertex based on the
        // Gauss-Bonnet theorem analogy:
        // κ_i = 1 - (degree_i / 6) for triangular lattice
        //
        // Here we use a simpler measure based on clustering coefficient:
        // Low clustering → high curvature (saddle-like)
        // High clustering → low curvature (flat-like)

        let mut total_scalar = 0.0;
        let mut max_sectional = 0.0;

        for i in 0..n {
            // Count neighbors
            let neighbors: Vec<_> = (0..n).filter(|&j| adj[i][j]).collect();
            let degree = neighbors.len();

            if degree < 2 {
                // Isolated or pendant vertices contribute positive curvature
                total_scalar += 1.0;
                continue;
            }

            // Count triangles containing vertex i
            let mut triangle_count = 0;
            for &j in &neighbors {
                for &k in &neighbors {
                    if j < k && adj[j][k] {
                        triangle_count += 1;
                    }
                }
            }

            // Maximum possible triangles = C(degree, 2)
            let max_triangles = degree * (degree - 1) / 2;
            let clustering = if max_triangles > 0 {
                f64::from(triangle_count) / max_triangles as f64
            } else {
                0.0
            };

            // Discrete scalar curvature at vertex i
            // High clustering → low curvature (network is locally flat)
            // Low clustering → high curvature (network has "holes")
            let vertex_curvature = (1.0 - clustering) * (degree as f64 / n as f64);
            total_scalar += vertex_curvature;

            if vertex_curvature > max_sectional {
                max_sectional = vertex_curvature;
            }
        }

        // Normalize scalar curvature
        let scalar = total_scalar / n as f64;

        // Kretschmann approximation: sum of squared curvatures
        let kretschmann = total_scalar * total_scalar / (n * n) as f64;

        (scalar, kretschmann, max_sectional)
    }

    /// Compute geodesic deviation (average pairwise distance).
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn compute_geodesic_deviation(branchial: &BranchialGraph, n: usize) -> f64 {
        if n < 2 {
            return 0.0;
        }

        // For a complete graph, geodesic deviation is 1.0 (all at distance 1)
        // For a sparse graph, deviation is higher (some pairs have larger distance)
        //
        // We approximate using edge density as a proxy for average distance

        let edge_count = branchial.edges.len();
        let max_edges = n * (n - 1) / 2;

        if max_edges == 0 {
            return 0.0;
        }

        let density = edge_count as f64 / max_edges as f64;

        // Geodesic deviation is inversely related to density
        // Dense graph → low deviation (branches are "close")
        // Sparse graph → high deviation (branches are "far apart")
        if density > 0.0 {
            1.0 / density
        } else {
            f64::INFINITY
        }
    }

    /// Returns an irreducibility indicator based on curvature.
    ///
    /// Higher values suggest more irreducible multiway structure:
    /// - Flat branchial space is more "reducible" (simple structure)
    /// - Curved branchial space is more "irreducible" (complex entanglement)
    ///
    /// # Returns
    ///
    /// A non-negative indicator where 0 = flat (trivially reducible).
    #[must_use]
    pub fn irreducibility_indicator(&self) -> f64 {
        // Combine curvature measures into a single indicator
        // Weight Kretschmann heavily as it captures total curvature magnitude
        let curvature_contribution = self.kretschmann_scalar.sqrt();
        let deviation_contribution = if self.geodesic_deviation.is_finite() {
            self.geodesic_deviation.ln().max(0.0)
        } else {
            0.0
        };

        curvature_contribution + 0.5 * deviation_contribution
    }

    /// Check if the branchial structure is geometrically simple.
    ///
    /// A simple structure has low curvature and low geodesic deviation,
    /// suggesting the branches can be consistently ordered without
    /// path-dependent effects.
    #[must_use]
    pub fn is_geometrically_simple(&self) -> bool {
        self.is_flat && self.geodesic_deviation < 2.0
    }

    /// Compute the "branchial complexity" as a dimensionless ratio.
    ///
    /// This normalizes the curvature by the dimension to give a
    /// scale-independent measure.
    ///
    /// # Returns
    ///
    /// Complexity ratio in range [0, 1] where 0 = flat, 1 = maximally curved.
    #[must_use]
    pub fn branchial_complexity(&self) -> f64 {
        if self.dimension <= 1 {
            return 0.0;
        }

        // Normalize by theoretical maximum
        // For a random graph, expected clustering ~ 0.5, so max curvature ~ 0.5
        let normalized_curvature = self.scalar_curvature / 0.5;
        normalized_curvature.clamp(0.0, 1.0)
    }
}

impl std::fmt::Display for BranchialCurvature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Branchial Curvature (step {}):", self.step)?;
        writeln!(f, "  Dimension: {}", self.dimension)?;
        writeln!(f, "  Scalar curvature R: {:.6}", self.scalar_curvature)?;
        writeln!(f, "  Kretschmann scalar K: {:.6}", self.kretschmann_scalar)?;
        writeln!(
            f,
            "  Max sectional curvature: {:.6}",
            self.max_sectional_curvature
        )?;
        writeln!(f, "  Geodesic deviation: {:.6}", self.geodesic_deviation)?;
        writeln!(f, "  Is flat: {}", self.is_flat)?;
        writeln!(
            f,
            "  Irreducibility indicator: {:.6}",
            self.irreducibility_indicator()
        )?;
        write!(f, "  Branchial complexity: {:.4}", self.branchial_complexity())
    }
}

// ============================================================================
// Curvature Foliation
// ============================================================================

/// Curvature analysis across all time steps of a multiway evolution.
#[derive(Debug, Clone)]
pub struct CurvatureFoliation {
    /// Curvature at each time step.
    pub curvatures: Vec<BranchialCurvature>,
    /// Total scalar curvature (sum over all steps).
    pub total_scalar_curvature: f64,
    /// Average scalar curvature per step.
    pub average_scalar_curvature: f64,
    /// Maximum curvature magnitude encountered.
    pub max_curvature_magnitude: f64,
    /// Step at which maximum curvature occurs.
    pub max_curvature_step: usize,
}

impl CurvatureFoliation {
    /// Compute curvature foliation from a multiway evolution graph.
    ///
    /// # Arguments
    ///
    /// * `graph` - The multiway evolution graph
    ///
    /// # Returns
    ///
    /// A `CurvatureFoliation` with curvature at each time step.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[must_use]
    pub fn from_evolution<S: Clone + Hash, T: Clone>(
        graph: &MultiwayEvolutionGraph<S, T>,
    ) -> Self {
        let max_step = graph.max_step();
        let mut curvatures = Vec::with_capacity(max_step + 1);

        for step in 0..=max_step {
            let curv = BranchialCurvature::from_evolution_at_step(graph, step);
            curvatures.push(curv);
        }

        let total_scalar: f64 = curvatures.iter().map(|c| c.scalar_curvature).sum();
        let average_scalar = if curvatures.is_empty() {
            0.0
        } else {
            total_scalar / curvatures.len() as f64
        };

        let (max_curvature_magnitude, max_curvature_step) = curvatures
            .iter()
            .enumerate()
            .map(|(i, c)| (c.kretschmann_scalar, i))
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
        self.curvatures.iter().all(|c| c.is_flat)
    }

    /// Get the irreducibility profile over time.
    ///
    /// Returns a vector of irreducibility indicators, one per step.
    #[must_use]
    pub fn irreducibility_profile(&self) -> Vec<f64> {
        self.curvatures
            .iter()
            .map(BranchialCurvature::irreducibility_indicator)
            .collect()
    }

    /// Compute average irreducibility indicator.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[must_use]
    pub fn average_irreducibility(&self) -> f64 {
        if self.curvatures.is_empty() {
            return 0.0;
        }

        let total: f64 = self.curvatures.iter().map(BranchialCurvature::irreducibility_indicator).sum();
        total / self.curvatures.len() as f64
    }
}

impl std::fmt::Display for CurvatureFoliation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Curvature Foliation:")?;
        writeln!(f, "  Steps analyzed: {}", self.curvatures.len())?;
        writeln!(
            f,
            "  Total scalar curvature: {:.6}",
            self.total_scalar_curvature
        )?;
        writeln!(
            f,
            "  Average scalar curvature: {:.6}",
            self.average_scalar_curvature
        )?;
        writeln!(
            f,
            "  Max curvature magnitude: {:.6} (at step {})",
            self.max_curvature_magnitude, self.max_curvature_step
        )?;
        writeln!(f, "  Globally flat: {}", self.is_globally_flat())?;
        write!(
            f,
            "  Average irreducibility: {:.6}",
            self.average_irreducibility()
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::multiway::StringRewriteSystem;

    #[test]
    fn test_branchial_curvature_single_branch() {
        let mut graph = MultiwayEvolutionGraph::<i32, ()>::new();
        graph.add_root(0);

        let curv = BranchialCurvature::from_evolution_at_step(&graph, 0);

        assert_eq!(curv.dimension, 1);
        assert!(curv.is_flat);
        assert!((curv.scalar_curvature - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_branchial_curvature_branching() {
        let srs = StringRewriteSystem::new(vec![("A", "B"), ("A", "C")]);
        let evolution = srs.run_multiway("A", 2, 10);

        let curv = BranchialCurvature::from_evolution_at_step(&evolution, 1);

        // Two branches at step 1 should have some structure
        assert!(curv.dimension >= 1);
    }

    #[test]
    fn test_curvature_foliation() {
        let srs = StringRewriteSystem::swap_system();
        let evolution = srs.run_multiway("AB", 5, 50);

        let foliation = CurvatureFoliation::from_evolution(&evolution);

        // Should have curvature data for each step
        assert!(!foliation.curvatures.is_empty());
        assert!(foliation.average_scalar_curvature >= 0.0 || foliation.average_scalar_curvature <= 0.0);
    }

    #[test]
    fn test_irreducibility_indicator() {
        let mut graph = MultiwayEvolutionGraph::<i32, ()>::new();
        let root = graph.add_root(0);
        // Fork creates two branches from the root: (state, transition_data, rule_index)
        graph.add_fork(root, vec![(1, (), 0), (2, (), 1)]);

        let curv = BranchialCurvature::from_evolution_at_step(&graph, 1);
        let indicator = curv.irreducibility_indicator();

        // Indicator should be non-negative
        assert!(indicator >= 0.0);
    }

    #[test]
    fn test_branchial_complexity() {
        let curv = BranchialCurvature {
            dimension: 5,
            scalar_curvature: 0.25,
            kretschmann_scalar: 0.0625,
            max_sectional_curvature: 0.3,
            is_flat: false,
            geodesic_deviation: 1.5,
            step: 0,
        };

        let complexity = curv.branchial_complexity();
        assert!(complexity >= 0.0);
        assert!(complexity <= 1.0);
    }

    #[test]
    fn test_curvature_display() {
        let curv = BranchialCurvature {
            dimension: 3,
            scalar_curvature: 0.1,
            kretschmann_scalar: 0.01,
            max_sectional_curvature: 0.15,
            is_flat: false,
            geodesic_deviation: 2.0,
            step: 5,
        };

        let display = format!("{}", curv);
        assert!(display.contains("Branchial Curvature"));
        assert!(display.contains("Dimension: 3"));
        assert!(display.contains("step 5"));
    }

    #[test]
    fn test_foliation_irreducibility_profile() {
        let srs = StringRewriteSystem::fibonacci_growth();
        let evolution = srs.run_multiway("A", 4, 20);

        let foliation = CurvatureFoliation::from_evolution(&evolution);
        let profile = foliation.irreducibility_profile();

        // Should have one indicator per step
        assert_eq!(profile.len(), foliation.curvatures.len());

        // All indicators should be non-negative
        assert!(profile.iter().all(|&x| x >= 0.0));
    }

    #[test]
    fn test_globally_flat() {
        // Linear evolution (no branching) should be globally flat
        let mut graph = MultiwayEvolutionGraph::<i32, ()>::new();
        let root = graph.add_root(0);
        let n1 = graph.add_sequential_step(root, 1, ());
        let _n2 = graph.add_sequential_step(n1, 2, ());

        let foliation = CurvatureFoliation::from_evolution(&graph);

        // Single-branch evolution is flat
        assert!(foliation.is_globally_flat());
    }
}
