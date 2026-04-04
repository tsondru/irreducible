//! Symmetric monoidal functor verification for multiway systems.
//!
//! A multiway system is **multicomputationally irreducible** if the functor
//! Z': 𝒯 → ℬ is a **symmetric monoidal functor**, meaning it preserves:
//!
//! 1. **Sequential composition** ∘ (standard computational irreducibility)
//! 2. **Parallel composition** ⊗ (multicomputational irreducibility)
//!
//! The key check is: Z'(f ⊗ g) = Z'(f) ⊕ Z'(g)
//!
//! Coherence verification types re-exported from [`catgraph::coherence`].

use std::hash::Hash;

use crate::categories::{DiscreteInterval, ParallelIntervals};
use crate::machines::multiway::{
    extract_branchial_foliation, BranchialGraph, MultiwayEvolutionGraph,
};

use super::{BranchResult, IrreducibilityFunctor};

// Re-export coherence types from catgraph
pub use catgraph::coherence::{
    verify_associator_coherence, verify_braiding_coherence, verify_left_unitor_coherence,
    verify_right_unitor_coherence, CoherenceVerification, DifferentialCoherence,
};

/// Result of symmetric monoidal functor verification.
///
/// Determines whether Z': 𝒯 → ℬ is a symmetric monoidal functor,
/// which is the criterion for multicomputational irreducibility.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct MonoidalFunctorResult {
    /// Whether Z' preserves tensor products: Z'(f ⊗ g) = Z'(f) ⊕ Z'(g)
    pub preserves_tensor: bool,

    /// Whether each individual branch is irreducible.
    pub branches_irreducible: bool,

    /// Results for each branch.
    pub branch_results: Vec<BranchResult>,

    /// Tensor verification results per step.
    pub tensor_checks: Vec<TensorCheck>,

    /// Overall multicomputational irreducibility.
    pub is_multicomputationally_irreducible: bool,

    // === Coherence Conditions ===

    /// Associator coherence: α_{X,Y,Z}: (X ⊗ Y) ⊗ Z ≅ X ⊗ (Y ⊗ Z)
    pub associator_coherent: bool,

    /// Left unitor coherence: `λ_X`: I ⊗ X ≅ X
    pub left_unitor_coherent: bool,

    /// Right unitor coherence: `ρ_X`: X ⊗ I ≅ X
    pub right_unitor_coherent: bool,

    /// Braiding coherence: σ_{X,Y}: X ⊗ Y ≅ Y ⊗ X
    pub braiding_coherent: bool,
}

impl MonoidalFunctorResult {
    /// Create a result indicating failure to verify.
    #[must_use]
    pub fn failed(_reason: &str) -> Self {
        Self {
            preserves_tensor: false,
            branches_irreducible: false,
            branch_results: Vec::new(),
            tensor_checks: Vec::new(),
            is_multicomputationally_irreducible: false,
            associator_coherent: false,
            left_unitor_coherent: false,
            right_unitor_coherent: false,
            braiding_coherent: false,
        }
    }

    /// Get number of steps where tensor preservation failed.
    #[must_use]
    pub fn tensor_violation_count(&self) -> usize {
        self.tensor_checks.iter().filter(|c| !c.preserves).count()
    }

    /// Check if all coherence conditions are satisfied.
    #[must_use]
    pub fn has_full_coherence(&self) -> bool {
        self.associator_coherent
            && self.left_unitor_coherent
            && self.right_unitor_coherent
            && self.braiding_coherent
    }
}

impl std::fmt::Display for MonoidalFunctorResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Monoidal Functor Verification:")?;
        writeln!(
            f,
            "  Multicomputationally irreducible: {}",
            self.is_multicomputationally_irreducible
        )?;
        writeln!(f, "  Tensor preserved: {}", self.preserves_tensor)?;
        writeln!(f, "  Branches irreducible: {}", self.branches_irreducible)?;
        writeln!(f, "  Branch count: {}", self.branch_results.len())?;
        writeln!(f, "  Steps checked: {}", self.tensor_checks.len())?;

        let violations = self.tensor_violation_count();
        if violations > 0 {
            writeln!(f, "  Tensor violations: {violations}")?;
        }

        // Show coherence status
        writeln!(f, "  Associator coherent: {}", self.associator_coherent)?;
        writeln!(f, "  Left unitor coherent: {}", self.left_unitor_coherent)?;
        writeln!(f, "  Right unitor coherent: {}", self.right_unitor_coherent)?;
        writeln!(f, "  Braiding coherent: {}", self.braiding_coherent)?;

        Ok(())
    }
}

/// Result of tensor preservation check at a single step.
#[derive(Clone, Debug)]
pub struct TensorCheck {
    /// The time step.
    pub step: usize,

    /// Branches active at this step.
    pub active_branch_count: usize,

    /// Expected: tensor product of individual branch intervals.
    pub expected_parallel: ParallelIntervals,

    /// Actual: computed from multiway graph.
    pub actual_parallel: ParallelIntervals,

    /// Whether tensor preservation holds at this step.
    pub preserves: bool,
}

impl TensorCheck {
    /// Create a check result.
    #[must_use]
    pub fn new(
        step: usize,
        active_branch_count: usize,
        expected: ParallelIntervals,
        actual: ParallelIntervals,
    ) -> Self {
        let preserves = expected.structurally_equivalent(&actual);
        Self {
            step,
            active_branch_count,
            expected_parallel: expected,
            actual_parallel: actual,
            preserves,
        }
    }
}

impl IrreducibilityFunctor {
    /// Verify that a multiway evolution satisfies symmetric monoidal functor properties.
    ///
    /// Checks:
    /// 1. Each branch is individually irreducible (sequential composition)
    /// 2. Tensor product is preserved: Z'(f ⊗ g) = Z'(f) ⊕ Z'(g)
    ///
    /// This is the criterion for **multicomputational irreducibility**.
    #[must_use]
    pub fn verify_symmetric_monoidal_functor<S: Clone + Hash, T: Clone>(
        graph: &MultiwayEvolutionGraph<S, T>,
    ) -> MonoidalFunctorResult {
        // Step 1: Check individual branch irreducibility
        let branch_intervals = graph.to_branch_intervals();
        let multiway_result = Self::verify_multiway_functoriality(&branch_intervals);

        // Step 2: Check tensor preservation at each step
        let tensor_checks = Self::verify_tensor_preservation(graph);
        let preserves_tensor = tensor_checks.iter().all(|c| c.preserves);

        // Step 3: Verify coherence conditions via catgraph
        let parallel_intervals: Vec<ParallelIntervals> = branch_intervals
            .iter()
            .map(|branch| {
                let mut pi = ParallelIntervals::new();
                for interval in branch {
                    pi.add_branch(*interval);
                }
                pi
            })
            .collect();
        let coherence = CoherenceVerification::verify_all(&parallel_intervals);

        // Overall result: symmetric monoidal functor requires all three
        let is_multicomputationally_irreducible =
            multiway_result.is_fully_irreducible && preserves_tensor && coherence.fully_coherent;

        MonoidalFunctorResult {
            preserves_tensor,
            branches_irreducible: multiway_result.is_fully_irreducible,
            branch_results: multiway_result.branch_results,
            tensor_checks,
            is_multicomputationally_irreducible,
            associator_coherent: coherence.associator_coherent,
            left_unitor_coherent: coherence.left_unitor_coherent,
            right_unitor_coherent: coherence.right_unitor_coherent,
            braiding_coherent: coherence.braiding_coherent,
        }
    }

    /// Verify tensor preservation at each time step.
    fn verify_tensor_preservation<S: Clone + Hash, T: Clone>(
        graph: &MultiwayEvolutionGraph<S, T>,
    ) -> Vec<TensorCheck> {
        let foliation = extract_branchial_foliation(graph);
        let mut checks = Vec::new();

        #[allow(clippy::needless_range_loop)]
        for i in 0..foliation.len().saturating_sub(1) {
            let branchial_t = &foliation[i];

            let expected = Self::compute_expected_parallel(branchial_t, i);
            let actual = Self::compute_actual_parallel(graph, branchial_t, i);

            let check = TensorCheck::new(i, branchial_t.node_count(), expected, actual);
            checks.push(check);
        }

        checks
    }

    /// Compute expected parallel intervals from branchial structure.
    fn compute_expected_parallel(branchial: &BranchialGraph, step: usize) -> ParallelIntervals {
        let mut result = ParallelIntervals::new();

        for _ in &branchial.nodes {
            result.add_branch(DiscreteInterval::new(step, step + 1));
        }

        result
    }

    /// Compute actual parallel intervals from the graph.
    fn compute_actual_parallel<S: Clone + Hash, T: Clone>(
        graph: &MultiwayEvolutionGraph<S, T>,
        branchial: &BranchialGraph,
        step: usize,
    ) -> ParallelIntervals {
        let mut result = ParallelIntervals::new();

        for &node_id in &branchial.nodes {
            if graph.get_forward_edges(&node_id).is_some_and(|e| !e.is_empty()) {
                result.add_branch(DiscreteInterval::new(step, step + 1));
            }
        }

        result
    }
}

// Extension methods (direct_sum, structurally_equivalent, exactly_equal)
// moved to catgraph::interval::ParallelIntervals — available via re-export.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::multiway::{MultiwayEvolutionGraph, StringRewriteSystem};

    #[test]
    fn test_parallel_intervals_direct_sum() {
        let p1 = crate::intervals![(0, 2)];
        let p2 = crate::intervals![(0, 3)];

        let sum = p1.direct_sum(p2);
        assert_eq!(sum.branch_count(), 2);
        assert_eq!(sum.total_complexity(), 3 + 4); // 3 + 4 = 7
    }

    #[test]
    fn test_parallel_intervals_structurally_equivalent() {
        let p1 = crate::intervals![(0, 2), (0, 2)]; // Two branches of cardinality 3
        let p2 = crate::intervals![(1, 3), (2, 4)]; // Two branches of cardinality 3

        // Both have two intervals of cardinality 3
        assert!(p1.structurally_equivalent(&p2));

        let p3 = crate::intervals![(0, 1)];
        assert!(!p1.structurally_equivalent(&p3)); // Different branch count
    }

    #[test]
    fn test_verify_simple_multiway() {
        let srs = StringRewriteSystem::swap_system();
        let evolution = srs.run_multiway("AB", 3, 10);

        let result = IrreducibilityFunctor::verify_symmetric_monoidal_functor(&evolution);

        assert!(result.tensor_checks.len() <= evolution.max_step());
    }

    #[test]
    fn test_verify_fibonacci_growth() {
        let srs = StringRewriteSystem::fibonacci_growth();
        let evolution = srs.run_multiway("A", 5, 100);

        let result = IrreducibilityFunctor::verify_symmetric_monoidal_functor(&evolution);

        let stats = evolution.statistics();
        if stats.fork_count == 0 {
            assert!(result.preserves_tensor);
        }
    }

    #[test]
    fn test_verify_branching_system() {
        let srs = StringRewriteSystem::new(vec![("A", "B"), ("A", "C")]);
        let evolution = srs.run_multiway("A", 3, 10);

        let result = IrreducibilityFunctor::verify_symmetric_monoidal_functor(&evolution);

        assert!(!result.branch_results.is_empty());
    }

    #[test]
    fn test_monoidal_result_display() {
        let result = MonoidalFunctorResult {
            preserves_tensor: true,
            branches_irreducible: true,
            branch_results: vec![],
            tensor_checks: vec![],
            is_multicomputationally_irreducible: true,
            associator_coherent: true,
            left_unitor_coherent: true,
            right_unitor_coherent: true,
            braiding_coherent: true,
        };

        let display = format!("{result}");
        assert!(display.contains("Multicomputationally irreducible: true"));
        assert!(display.contains("Associator coherent: true"));
        assert!(display.contains("Left unitor coherent: true"));
        assert!(display.contains("Right unitor coherent: true"));
        assert!(display.contains("Braiding coherent: true"));
    }

    #[test]
    fn test_tensor_check_creation() {
        let expected = crate::intervals![(0, 1)];
        let actual = crate::intervals![(0, 1)];

        let check = TensorCheck::new(0, 1, expected, actual);
        assert!(check.preserves);
        assert_eq!(check.step, 0);
    }

    #[test]
    fn test_single_branch_is_multicomputationally_irreducible() {
        let mut graph: MultiwayEvolutionGraph<i32, ()> = MultiwayEvolutionGraph::new();
        let root = graph.add_root(0);
        graph.add_sequential_step(root, 1, ());

        let result = IrreducibilityFunctor::verify_symmetric_monoidal_functor(&graph);

        assert!(result.branches_irreducible);
        assert!(result.preserves_tensor);
        assert!(result.is_multicomputationally_irreducible);
    }

    // ========================================
    // Coherence Verification Tests (using macros)
    // ========================================

    crate::test_coherence_condition!(
        associator,
        test_associator_coherence,
        crate::intervals![(0, 2)],
        crate::intervals![(2, 5)],
        crate::intervals![(5, 8)]
    );

    crate::test_coherence_condition!(
        associator,
        test_associator_with_multi_branch,
        crate::intervals![(0, 2), (0, 3)],
        crate::intervals![(2, 4)],
        crate::intervals![(4, 7)]
    );

    crate::test_coherence_condition!(
        left_unitor,
        test_left_unitor_coherence,
        crate::intervals![(0, 5)]
    );

    crate::test_coherence_condition!(
        left_unitor,
        test_left_unitor_multi_branch,
        crate::intervals![(0, 2), (2, 4)]
    );

    crate::test_coherence_condition!(
        right_unitor,
        test_right_unitor_coherence,
        crate::intervals![(0, 5)]
    );

    crate::test_coherence_condition!(
        right_unitor,
        test_right_unitor_multi_branch,
        crate::intervals![(0, 2), (2, 4)]
    );

    crate::test_coherence_condition!(
        braiding,
        test_braiding_coherence,
        crate::intervals![(0, 3)],
        crate::intervals![(3, 7)]
    );

    crate::test_coherence_condition!(
        braiding,
        test_braiding_with_different_cardinalities,
        crate::intervals![(0, 2)],
        crate::intervals![(0, 5)]
    );

    crate::test_full_coherence!(
        test_coherence_verification_all,
        vec![
            crate::intervals![(0, 2)],
            crate::intervals![(2, 5)],
            crate::intervals![(5, 10)],
        ]
    );

    #[test]
    fn test_coherence_verification_counts() {
        let intervals = vec![
            crate::intervals![(0, 2)],
            crate::intervals![(2, 5)],
            crate::intervals![(5, 10)],
        ];

        let result = CoherenceVerification::verify_all(&intervals);
        assert_eq!(result.associator_tests, 27); // 3^3
        assert_eq!(result.braiding_tests, 9); // 3^2
    }

    #[test]
    fn test_coherence_verification_empty() {
        let result = CoherenceVerification::verify_all(&[]);

        assert!(result.fully_coherent);
        assert_eq!(result.associator_tests, 0);
        assert_eq!(result.braiding_tests, 0);
    }

    #[test]
    fn test_coherence_verification_display() {
        let intervals = vec![crate::intervals![(0, 2)], crate::intervals![(2, 4)]];

        let result = CoherenceVerification::verify_all(&intervals);
        let display = format!("{result}");

        assert!(display.contains("Fully coherent: true"));
        assert!(display.contains("Associator α: true"));
        assert!(display.contains("Braiding σ: true"));
    }

    // ========================================
    // Differential Coherence Tests (using macros)
    // ========================================

    crate::test_differential_coherence!(
        test_differential_coherence_basic,
        vec![
            crate::intervals![(0, 2)],
            crate::intervals![(2, 5)],
            crate::intervals![(5, 10)],
        ]
    );

    crate::test_differential_coherence!(
        test_differential_coherence_empty,
        vec![]
    );

    crate::test_differential_coherence!(
        test_differential_coherence_single,
        vec![crate::intervals![(0, 5)]]
    );

    #[test]
    fn test_differential_coherence_display() {
        let intervals = vec![crate::intervals![(0, 2)], crate::intervals![(2, 4)]];

        let result = DifferentialCoherence::verify(&intervals);
        let display = format!("{result}");

        assert!(display.contains("Differentially coherent: true"));
        assert!(display.contains("Coherence form closed: true"));
        assert!(display.contains("Conservation ratio:"));
        assert!(display.contains("Categorical curvature: flat"));
    }

    #[test]
    fn test_differential_coherence_defect() {
        let intervals = vec![crate::intervals![(0, 2)], crate::intervals![(2, 4)]];

        let result = DifferentialCoherence::verify(&intervals);

        assert!(result.coherence_defect() < 0.001);
    }

    #[test]
    fn test_categorical_curvature_flat() {
        let intervals = vec![crate::intervals![(0, 3)], crate::intervals![(3, 6)]];

        let result = DifferentialCoherence::verify(&intervals);

        assert!(!result.has_categorical_curvature());
    }
}
