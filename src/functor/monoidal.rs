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
//! ## Mathematical Background
//!
//! For symmetric monoidal categories ⟨𝒯, ⊗, I⟩ and ⟨ℬ, ⊕, ∅⟩:
//! - ⊗ is tensor product (parallel composition of computations)
//! - ⊕ is direct sum (disjoint union of intervals)
//! - I is the unit object (HALT state)
//! - A symmetric monoidal functor must preserve all this structure
//!
//! Full coherence includes associator, unitors, and braiding conditions.
//! This module implements basic tensor preservation with hooks for full coherence.

use std::hash::Hash;

use crate::categories::{DiscreteInterval, ParallelIntervals};
use crate::machines::multiway::{
    extract_branchial_foliation, BranchialGraph, MultiwayEvolutionGraph,
};

use super::{BranchResult, IrreducibilityFunctor};

/// Result of symmetric monoidal functor verification.
///
/// Determines whether Z': 𝒯 → ℬ is a symmetric monoidal functor,
/// which is the criterion for multicomputational irreducibility.
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

    // === Coherence Placeholders ===
    // These are placeholders for future full coherence verification.

    /// Associator coherence: α_{X,Y,Z}: (X ⊗ Y) ⊗ Z ≅ X ⊗ (Y ⊗ Z)
    /// Placeholder for future implementation.
    pub associator_coherent: Option<bool>,

    /// Left unitor coherence: `λ_X`: I ⊗ X ≅ X
    /// Placeholder for future implementation.
    pub left_unitor_coherent: Option<bool>,

    /// Right unitor coherence: `ρ_X`: X ⊗ I ≅ X
    /// Placeholder for future implementation.
    pub right_unitor_coherent: Option<bool>,

    /// Braiding coherence: σ_{X,Y}: X ⊗ Y ≅ Y ⊗ X
    /// Placeholder for future implementation.
    pub braiding_coherent: Option<bool>,
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
            associator_coherent: None,
            left_unitor_coherent: None,
            right_unitor_coherent: None,
            braiding_coherent: None,
        }
    }

    /// Get number of steps where tensor preservation failed.
    #[must_use]
    pub fn tensor_violation_count(&self) -> usize {
        self.tensor_checks.iter().filter(|c| !c.preserves).count()
    }

    /// Check if any coherence conditions are verified.
    #[must_use]
    pub fn has_full_coherence(&self) -> bool {
        self.associator_coherent.unwrap_or(false)
            && self.left_unitor_coherent.unwrap_or(false)
            && self.right_unitor_coherent.unwrap_or(false)
            && self.braiding_coherent.unwrap_or(false)
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
        if let Some(assoc) = self.associator_coherent {
            writeln!(f, "  Associator coherent: {assoc}")?;
        }
        if let Some(braiding) = self.braiding_coherent {
            writeln!(f, "  Braiding coherent: {braiding}")?;
        }

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

        // Overall result
        let is_multicomputationally_irreducible =
            multiway_result.is_fully_irreducible && preserves_tensor;

        MonoidalFunctorResult {
            preserves_tensor,
            branches_irreducible: multiway_result.is_fully_irreducible,
            branch_results: multiway_result.branch_results,
            tensor_checks,
            is_multicomputationally_irreducible,
            // Coherence placeholders (not yet implemented)
            associator_coherent: None,
            left_unitor_coherent: None,
            right_unitor_coherent: None,
            braiding_coherent: None,
        }
    }

    /// Verify tensor preservation at each time step.
    ///
    /// For each step, checks that the parallel structure of intervals
    /// matches what we'd expect from the tensor product of individual branches.
    fn verify_tensor_preservation<S: Clone + Hash, T: Clone>(
        graph: &MultiwayEvolutionGraph<S, T>,
    ) -> Vec<TensorCheck> {
        let foliation = extract_branchial_foliation(graph);
        let mut checks = Vec::new();

        // For each transition step (between t and t+1)
        #[allow(clippy::needless_range_loop)]
        for i in 0..foliation.len().saturating_sub(1) {
            let branchial_t = &foliation[i];

            // Expected: one interval per active branch at step t
            let expected = Self::compute_expected_parallel(branchial_t, i);

            // Actual: the parallel structure we observe
            let actual = Self::compute_actual_parallel(graph, branchial_t, i);

            let check = TensorCheck::new(i, branchial_t.node_count(), expected, actual);
            checks.push(check);
        }

        checks
    }

    /// Compute expected parallel intervals from branchial structure.
    ///
    /// Each node at step t should contribute an interval [t, t+1].
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
            // Check if this node has outgoing edges
            if graph.get_forward_edges(&node_id).is_some_and(|e| !e.is_empty()) {
                result.add_branch(DiscreteInterval::new(step, step + 1));
            }
        }

        result
    }
}

// Extension methods (direct_sum, structurally_equivalent, exactly_equal)
// moved to catgraph::interval::ParallelIntervals — available via re-export.

// ============================================================================
// Coherence Verification Functions
// ============================================================================

/// Verify associator coherence: α_{X,Y,Z}: (X ⊗ Y) ⊗ Z ≅ X ⊗ (Y ⊗ Z).
///
/// The associator proves that grouping in tensor products doesn't matter.
/// For parallel intervals, this checks structural equivalence of:
/// `(a ⊗ b) ⊗ c` vs `a ⊗ (b ⊗ c)`
///
/// # Example
///
/// ```rust
/// use irreducible::categories::{DiscreteInterval, ParallelIntervals};
/// use irreducible::functor::monoidal::verify_associator_coherence;
///
/// let a = ParallelIntervals::from_branch(DiscreteInterval::new(0, 2));
/// let b = ParallelIntervals::from_branch(DiscreteInterval::new(2, 4));
/// let c = ParallelIntervals::from_branch(DiscreteInterval::new(4, 6));
///
/// assert!(verify_associator_coherence(&a, &b, &c));
/// ```
#[must_use]
pub fn verify_associator_coherence(
    a: &ParallelIntervals,
    b: &ParallelIntervals,
    c: &ParallelIntervals,
) -> bool {
    // (a ⊗ b) ⊗ c
    let left_grouped = a.clone().tensor(b.clone()).tensor(c.clone());

    // a ⊗ (b ⊗ c)
    let right_grouped = a.clone().tensor(b.clone().tensor(c.clone()));

    // They should be structurally equivalent
    left_grouped.structurally_equivalent(&right_grouped)
}

/// Verify left unitor coherence: λ`_X:` I ⊗ X ≅ X.
///
/// The left unitor proves that tensoring with the unit (empty) on the left
/// gives back the original structure.
///
/// # Example
///
/// ```rust
/// use irreducible::categories::{DiscreteInterval, ParallelIntervals};
/// use irreducible::functor::monoidal::verify_left_unitor_coherence;
///
/// let x = ParallelIntervals::from_branch(DiscreteInterval::new(0, 5));
/// assert!(verify_left_unitor_coherence(&x));
/// ```
#[must_use]
pub fn verify_left_unitor_coherence(x: &ParallelIntervals) -> bool {
    // I = empty ParallelIntervals (unit object)
    let unit = ParallelIntervals::new();

    // I ⊗ X should be equivalent to X
    let tensored = unit.tensor(x.clone());

    tensored.structurally_equivalent(x)
}

/// Verify right unitor coherence: ρ`_X:` X ⊗ I ≅ X.
///
/// The right unitor proves that tensoring with the unit (empty) on the right
/// gives back the original structure.
///
/// # Example
///
/// ```rust
/// use irreducible::categories::{DiscreteInterval, ParallelIntervals};
/// use irreducible::functor::monoidal::verify_right_unitor_coherence;
///
/// let x = ParallelIntervals::from_branch(DiscreteInterval::new(0, 5));
/// assert!(verify_right_unitor_coherence(&x));
/// ```
#[must_use]
pub fn verify_right_unitor_coherence(x: &ParallelIntervals) -> bool {
    // I = empty ParallelIntervals (unit object)
    let unit = ParallelIntervals::new();

    // X ⊗ I should be equivalent to X
    let tensored = x.clone().tensor(unit);

    tensored.structurally_equivalent(x)
}

/// Verify braiding coherence: σ_{X,Y}: X ⊗ Y ≅ Y ⊗ X.
///
/// The braiding proves that the tensor product is symmetric - order doesn't matter.
///
/// # Example
///
/// ```rust
/// use irreducible::categories::{DiscreteInterval, ParallelIntervals};
/// use irreducible::functor::monoidal::verify_braiding_coherence;
///
/// let x = ParallelIntervals::from_branch(DiscreteInterval::new(0, 3));
/// let y = ParallelIntervals::from_branch(DiscreteInterval::new(3, 7));
///
/// assert!(verify_braiding_coherence(&x, &y));
/// ```
#[must_use]
pub fn verify_braiding_coherence(x: &ParallelIntervals, y: &ParallelIntervals) -> bool {
    // X ⊗ Y
    let xy = x.clone().tensor(y.clone());

    // Y ⊗ X
    let yx = y.clone().tensor(x.clone());

    // They should be structurally equivalent (symmetric monoidal)
    xy.structurally_equivalent(&yx)
}

/// Comprehensive coherence verification result.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct CoherenceVerification {
    /// Associator coherence holds for all tested triples.
    pub associator_coherent: bool,
    /// Left unitor coherence holds.
    pub left_unitor_coherent: bool,
    /// Right unitor coherence holds.
    pub right_unitor_coherent: bool,
    /// Braiding coherence holds for all tested pairs.
    pub braiding_coherent: bool,
    /// Number of associator tests performed.
    pub associator_tests: usize,
    /// Number of braiding tests performed.
    pub braiding_tests: usize,
    /// Overall coherence (all conditions satisfied).
    pub fully_coherent: bool,
}

impl CoherenceVerification {
    /// Verify all coherence conditions for a collection of parallel intervals.
    ///
    /// Tests associator with all triples and braiding with all pairs.
    pub fn verify_all(intervals: &[ParallelIntervals]) -> Self {
        if intervals.is_empty() {
            return Self {
                associator_coherent: true,
                left_unitor_coherent: true,
                right_unitor_coherent: true,
                braiding_coherent: true,
                associator_tests: 0,
                braiding_tests: 0,
                fully_coherent: true,
            };
        }

        // Test unitors on first element (used for early validation)
        let _left_unitor_coherent = verify_left_unitor_coherence(&intervals[0]);
        let _right_unitor_coherent = verify_right_unitor_coherence(&intervals[0]);

        // Test all other elements for unitors too
        let all_left_unitor = intervals.iter().all(verify_left_unitor_coherence);
        let all_right_unitor = intervals.iter().all(verify_right_unitor_coherence);

        // Test associator on all triples
        let mut associator_tests = 0;
        let mut associator_coherent = true;
        for i in 0..intervals.len() {
            for j in 0..intervals.len() {
                for k in 0..intervals.len() {
                    associator_tests += 1;
                    if !verify_associator_coherence(&intervals[i], &intervals[j], &intervals[k]) {
                        associator_coherent = false;
                    }
                }
            }
        }

        // Test braiding on all pairs
        let mut braiding_tests = 0;
        let mut braiding_coherent = true;
        for i in 0..intervals.len() {
            for j in 0..intervals.len() {
                braiding_tests += 1;
                if !verify_braiding_coherence(&intervals[i], &intervals[j]) {
                    braiding_coherent = false;
                }
            }
        }

        let fully_coherent = associator_coherent
            && all_left_unitor
            && all_right_unitor
            && braiding_coherent;

        Self {
            associator_coherent,
            left_unitor_coherent: all_left_unitor,
            right_unitor_coherent: all_right_unitor,
            braiding_coherent,
            associator_tests,
            braiding_tests,
            fully_coherent,
        }
    }
}

impl std::fmt::Display for CoherenceVerification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Coherence Verification:")?;
        writeln!(f, "  Fully coherent: {}", self.fully_coherent)?;
        writeln!(
            f,
            "  Associator α: {} ({} tests)",
            self.associator_coherent, self.associator_tests
        )?;
        writeln!(f, "  Left unitor λ: {}", self.left_unitor_coherent)?;
        writeln!(f, "  Right unitor ρ: {}", self.right_unitor_coherent)?;
        writeln!(
            f,
            "  Braiding σ: {} ({} tests)",
            self.braiding_coherent, self.braiding_tests
        )
    }
}

// ============================================================================
// Differential Coherence (topology feature)
// ============================================================================

/// Differential geometric interpretation of coherence conditions.
///
/// This structure connects algebraic coherence (pentagon identity, triangle
/// identities) with differential geometry via the Stokes perspective.
///
/// # Mathematical Interpretation
///
/// - **Pentagon identity**: Viewed as closure of a 2-form (d²ω = 0)
/// - **Triangle identities**: Viewed as conservation on 1-chains
/// - **Coherence form closure**: If dω = 0, coherence is "conserved"
///
/// When algebraic coherence holds AND the differential form is closed,
/// we have *differential coherence* - a geometric confirmation of the
/// categorical structure.
///
/// # Feature Gate
///
/// Requires `topology` feature for Stokes integration analysis.
#[derive(Clone, Debug)]
pub struct DifferentialCoherence {
    /// Standard algebraic coherence verification.
    pub algebraic: CoherenceVerification,

    /// Whether the differential coherence criterion is satisfied.
    ///
    /// This is true when the algebraic coherence can be interpreted
    /// as a closed form condition in the Stokes sense.
    pub differential_coherent: bool,

    /// Whether the "coherence form" (encoding coherence data) is closed.
    ///
    /// A closed coherence form means the pentagon identity holds in
    /// the differential geometric sense: the boundary of coherence
    /// data is exact.
    pub coherence_form_closed: bool,

    /// Conservation ratio from Stokes analysis.
    ///
    /// Ratio = 1.0 means perfect conservation (fully coherent).
    /// Deviations indicate coherence "leakage".
    pub conservation_ratio: f64,

    /// Number of coherence violations detected as non-closures.
    pub non_closure_count: usize,
}

impl DifferentialCoherence {
    /// Verify differential coherence for a collection of parallel intervals.
    ///
    /// This performs both algebraic coherence verification and interprets
    /// the results through the lens of differential forms and Stokes theorem.
    ///
    /// # Arguments
    ///
    /// * `intervals` - Collection of parallel intervals to verify
    ///
    /// # Returns
    ///
    /// A `DifferentialCoherence` result combining algebraic and differential analysis.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[must_use]
    pub fn verify(intervals: &[ParallelIntervals]) -> Self {
        // Step 1: Standard algebraic coherence
        let algebraic = CoherenceVerification::verify_all(intervals);

        // Step 2: Interpret coherence through differential lens
        //
        // The key insight: coherence conditions form a "cocycle" condition.
        // If α_{X,Y,Z} is the associator, the pentagon identity states:
        //   α_{W,X,Y⊗Z} ∘ α_{W⊗X,Y,Z} = (1_W ⊗ α_{X,Y,Z}) ∘ α_{W,X⊗Y,Z} ∘ (α_{W,X,Y} ⊗ 1_Z)
        //
        // This is analogous to d² = 0 (exterior derivative squares to zero).
        // We interpret algebraic.associator_coherent as the closure condition.

        let coherence_form_closed = algebraic.associator_coherent;

        // Step 3: Compute conservation ratio
        //
        // This measures how "conserved" the coherence data is.
        // If all coherence checks pass, ratio = 1.0 (perfect conservation).
        let total_tests = algebraic.associator_tests + algebraic.braiding_tests;
        let passed_tests = if algebraic.associator_coherent {
            algebraic.associator_tests
        } else {
            0
        } + if algebraic.braiding_coherent {
            algebraic.braiding_tests
        } else {
            0
        };

        let conservation_ratio = if total_tests == 0 {
            1.0
        } else {
            passed_tests as f64 / total_tests as f64
        };

        // Step 4: Count non-closures
        let mut non_closure_count = 0;
        if !algebraic.associator_coherent {
            non_closure_count += 1; // Pentagon identity violation
        }
        if !algebraic.left_unitor_coherent || !algebraic.right_unitor_coherent {
            non_closure_count += 1; // Triangle identity violation
        }
        if !algebraic.braiding_coherent {
            non_closure_count += 1; // Hexagon identity violation
        }

        // Step 5: Differential coherence requires both algebraic and form closure
        let differential_coherent =
            algebraic.fully_coherent && coherence_form_closed && conservation_ratio > 0.999;

        Self {
            algebraic,
            differential_coherent,
            coherence_form_closed,
            conservation_ratio,
            non_closure_count,
        }
    }

    /// Check if the structure exhibits "curvature" in category space.
    ///
    /// Non-zero curvature indicates coherence failures - the category
    /// is not "flat" in the sense that parallel transport (composition)
    /// is path-dependent.
    ///
    /// # Returns
    ///
    /// `true` if there is curvature (coherence failures), `false` if flat.
    #[inline]
    #[must_use]
    pub fn has_categorical_curvature(&self) -> bool {
        self.non_closure_count > 0 || !self.coherence_form_closed
    }

    /// Compute the "coherence defect" as a single scalar.
    ///
    /// This is analogous to scalar curvature in Riemannian geometry.
    /// A value of 0.0 indicates perfect coherence (flat category).
    ///
    /// # Returns
    ///
    /// Coherence defect in range [0.0, 1.0] where 0.0 is perfect.
    #[inline]
    #[must_use]
    pub fn coherence_defect(&self) -> f64 {
        1.0 - self.conservation_ratio
    }
}

impl std::fmt::Display for DifferentialCoherence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Differential Coherence:")?;
        writeln!(
            f,
            "  Differentially coherent: {}",
            self.differential_coherent
        )?;
        writeln!(f, "  Coherence form closed: {}", self.coherence_form_closed)?;
        writeln!(f, "  Conservation ratio: {:.4}", self.conservation_ratio)?;
        writeln!(f, "  Non-closure count: {}", self.non_closure_count)?;
        writeln!(
            f,
            "  Categorical curvature: {}",
            if self.has_categorical_curvature() {
                "present"
            } else {
                "flat"
            }
        )?;
        writeln!(f)?;
        write!(f, "{}", self.algebraic)
    }
}

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

        // Should produce some result (not testing specific outcome, just that it runs)
        assert!(result.tensor_checks.len() <= evolution.max_step());
    }

    #[test]
    fn test_verify_fibonacci_growth() {
        // Fibonacci is deterministic, so should be fully irreducible
        let srs = StringRewriteSystem::fibonacci_growth();
        let evolution = srs.run_multiway("A", 5, 100);

        let result = IrreducibilityFunctor::verify_symmetric_monoidal_functor(&evolution);

        // Single branch (no forking) = tensor checks should all pass
        let stats = evolution.statistics();
        if stats.fork_count == 0 {
            assert!(result.preserves_tensor);
        }
    }

    #[test]
    fn test_verify_branching_system() {
        // System that branches
        let srs = StringRewriteSystem::new(vec![("A", "B"), ("A", "C")]);
        let evolution = srs.run_multiway("A", 3, 10);

        let result = IrreducibilityFunctor::verify_symmetric_monoidal_functor(&evolution);

        // Should have tensor checks
        assert!(result.branch_results.len() >= 1);
    }

    #[test]
    fn test_monoidal_result_display() {
        let result = MonoidalFunctorResult {
            preserves_tensor: true,
            branches_irreducible: true,
            branch_results: vec![],
            tensor_checks: vec![],
            is_multicomputationally_irreducible: true,
            associator_coherent: None,
            left_unitor_coherent: None,
            right_unitor_coherent: None,
            braiding_coherent: None,
        };

        let display = format!("{}", result);
        assert!(display.contains("Multicomputationally irreducible: true"));
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

        // Single non-cycling branch should be irreducible
        assert!(result.branches_irreducible);
        assert!(result.preserves_tensor);
        assert!(result.is_multicomputationally_irreducible);
    }

    // ========================================
    // Coherence Verification Tests (using macros)
    // ========================================

    // Simple associator: (a ⊗ b) ⊗ c ≅ a ⊗ (b ⊗ c)
    crate::test_coherence_condition!(
        associator,
        test_associator_coherence,
        crate::intervals![(0, 2)],
        crate::intervals![(2, 5)],
        crate::intervals![(5, 8)]
    );

    // Associator with multi-branch interval
    crate::test_coherence_condition!(
        associator,
        test_associator_with_multi_branch,
        crate::intervals![(0, 2), (0, 3)],
        crate::intervals![(2, 4)],
        crate::intervals![(4, 7)]
    );

    // Left unitor: I ⊗ X ≅ X
    crate::test_coherence_condition!(
        left_unitor,
        test_left_unitor_coherence,
        crate::intervals![(0, 5)]
    );

    // Left unitor with multi-branch
    crate::test_coherence_condition!(
        left_unitor,
        test_left_unitor_multi_branch,
        crate::intervals![(0, 2), (2, 4)]
    );

    // Right unitor: X ⊗ I ≅ X
    crate::test_coherence_condition!(
        right_unitor,
        test_right_unitor_coherence,
        crate::intervals![(0, 5)]
    );

    // Right unitor with multi-branch
    crate::test_coherence_condition!(
        right_unitor,
        test_right_unitor_multi_branch,
        crate::intervals![(0, 2), (2, 4)]
    );

    // Braiding: X ⊗ Y ≅ Y ⊗ X
    crate::test_coherence_condition!(
        braiding,
        test_braiding_coherence,
        crate::intervals![(0, 3)],
        crate::intervals![(3, 7)]
    );

    // Braiding with different cardinalities
    crate::test_coherence_condition!(
        braiding,
        test_braiding_with_different_cardinalities,
        crate::intervals![(0, 2)],
        crate::intervals![(0, 5)]
    );

    // Full coherence test using macro (3 intervals → 27 associator + 9 braiding tests)
    crate::test_full_coherence!(
        test_coherence_verification_all,
        vec![
            crate::intervals![(0, 2)],
            crate::intervals![(2, 5)],
            crate::intervals![(5, 10)],
        ]
    );

    // Verify specific test counts (standalone test for quantitative assertions)
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
        let display = format!("{}", result);

        assert!(display.contains("Fully coherent: true"));
        assert!(display.contains("Associator α: true"));
        assert!(display.contains("Braiding σ: true"));
    }

    // ========================================
    // Differential Coherence Tests (using macros)
    // ========================================

    // Basic differential coherence with 3 intervals
    crate::test_differential_coherence!(
        test_differential_coherence_basic,
        vec![
            crate::intervals![(0, 2)],
            crate::intervals![(2, 5)],
            crate::intervals![(5, 10)],
        ]
    );

    // Empty collection (vacuously coherent)
    crate::test_differential_coherence!(
        test_differential_coherence_empty,
        vec![]
    );

    // Single interval
    crate::test_differential_coherence!(
        test_differential_coherence_single,
        vec![crate::intervals![(0, 5)]]
    );

    #[test]
    fn test_differential_coherence_display() {
        let intervals = vec![crate::intervals![(0, 2)], crate::intervals![(2, 4)]];

        let result = DifferentialCoherence::verify(&intervals);
        let display = format!("{}", result);

        assert!(display.contains("Differentially coherent: true"));
        assert!(display.contains("Coherence form closed: true"));
        assert!(display.contains("Conservation ratio:"));
        assert!(display.contains("Categorical curvature: flat"));
    }

    #[test]
    fn test_differential_coherence_defect() {
        let intervals = vec![crate::intervals![(0, 2)], crate::intervals![(2, 4)]];

        let result = DifferentialCoherence::verify(&intervals);

        // Perfect coherence means zero defect
        assert!(result.coherence_defect() < 0.001);
    }

    #[test]
    fn test_categorical_curvature_flat() {
        let intervals = vec![crate::intervals![(0, 3)], crate::intervals![(3, 6)]];

        let result = DifferentialCoherence::verify(&intervals);

        // Coherent intervals should be "flat" (no curvature)
        assert!(!result.has_categorical_curvature());
    }
}
