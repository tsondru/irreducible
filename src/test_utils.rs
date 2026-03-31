//! Shared test utilities and fixtures.
//!
//! This module provides common test helpers that are used across multiple test modules.
//! It is only compiled when running tests.
//!
//! ## Macros
//!
//! ### Display/Debug Assertions
//! - [`assert_display_contains!`] - Assert that Display output contains expected substrings
//! - [`assert_debug_contains!`] - Assert that Debug output contains expected substrings
//!
//! ### Coherence Test Macros
//! - [`intervals!`] - Concise ParallelIntervals creation
//! - [`test_full_coherence!`] - Test all symmetric monoidal coherence conditions
//! - [`test_differential_coherence!`] - Test differential coherence properties
//! - [`test_coherence_condition!`] - Test individual coherence conditions
//!
//! ## Fixtures
//!
//! - [`standard_test_intervals`] - Standard intervals for composition testing
//! - [`standard_parallel_intervals`] - Standard parallel intervals for monoidal testing
//!

use crate::categories::{DiscreteInterval, ParallelIntervals};
use crate::functor::bifunctor::TensorProduct;

// ============================================================================
// Display Testing Macro
// ============================================================================

/// Asserts that a value's Display output contains all expected substrings.
///
/// This macro simplifies testing Display implementations by checking
/// that the formatted output contains each expected substring.
///
/// # Examples
///
/// ```ignore
/// use irreducible::{DiscreteInterval, assert_display_contains};
///
/// let interval = DiscreteInterval::new(5, 10);
/// assert_display_contains!(interval, "5", "10");
///
/// // Multiple assertions in one call
/// let result = some_result();
/// assert_display_contains!(result, "success", "count: 5", "valid");
/// ```
///
/// # Panics
///
/// Panics if any expected substring is not found in the Display output.
#[macro_export]
macro_rules! assert_display_contains {
    ($value:expr, $($expected:expr),+ $(,)?) => {{
        let display = format!("{}", $value);
        $(
            assert!(
                display.contains($expected),
                "Display output does not contain '{}'\nActual output:\n{}",
                $expected, display
            );
        )+
    }};
}

/// Asserts that a value's Debug output contains all expected substrings.
///
/// Similar to [`assert_display_contains!`] but uses the Debug trait.
#[macro_export]
macro_rules! assert_debug_contains {
    ($value:expr, $($expected:expr),+ $(,)?) => {{
        let debug = format!("{:?}", $value);
        $(
            assert!(
                debug.contains($expected),
                "Debug output does not contain '{}'\nActual output:\n{}",
                $expected, debug
            );
        )+
    }};
}

/// Generates a graph creation test for a causal adapter (removed — deep_causality).
///
/// Tests that an adapter can convert execution history to a causal graph
/// with the expected size and frozen state.
///
/// # Arguments
///
/// * `$test_name` - Name for the generated test function
/// * `$adapter` - The adapter type (e.g., `TuringMachineCausalAdapter`)
/// * `$graph_method` - The method to call on adapter (e.g., `execution_to_graph`)
/// * `$setup` - Expression that returns the execution history
/// * `$expected_size` - Expected number of nodes in the graph
///
/// # Example
///
/// ```ignore
/// test_graph_creation!(
///     test_tm_graph_busy_beaver,
///     TuringMachineCausalAdapter,
///     execution_to_graph,
///     { let bb = TuringMachine::busy_beaver_2_2(); bb.run("", 20) },
///     6
/// );
/// ```
#[macro_export]
macro_rules! test_graph_creation {
    ($test_name:ident, $adapter:ty, $graph_method:ident, $setup:expr, $expected_size:expr) => {
        #[test]
        fn $test_name() {
            use deep_causality::CausableGraph;
            let history = $setup;
            let graph = <$adapter>::$graph_method(&history).unwrap();
            assert_eq!(graph.size(), $expected_size,
                "Expected {} nodes, got {}", $expected_size, graph.size());
            assert!(graph.is_frozen(), "Graph should be frozen after creation");
        }
    };
}

/// Generates an irreducibility verification test for a causal adapter.
///
/// Tests that an execution produces an irreducible causal graph with
/// valid structure and contiguous intervals.
///
/// # Arguments
///
/// * `$test_name` - Name for the generated test function
/// * `$adapter` - The adapter type
/// * `$graph_method` - Method to create graph (e.g., `execution_to_graph`)
/// * `$verify_method` - Method to verify irreducibility
/// * `$setup` - Expression that returns the execution history
/// * `$expected_nodes` - Expected number of nodes
///
/// # Example
///
/// ```ignore
/// test_irreducibility!(
///     test_tm_irreducible_bb,
///     TuringMachineCausalAdapter,
///     execution_to_graph,
///     verify_graph_irreducibility,
///     { let bb = TuringMachine::busy_beaver_2_2(); bb.run("", 20) },
///     6
/// );
/// ```
#[macro_export]
macro_rules! test_irreducibility {
    ($test_name:ident, $adapter:ty, $graph_method:ident, $verify_method:ident, $setup:expr, $expected_nodes:expr) => {
        #[test]
        fn $test_name() {
            let history = $setup;
            let graph = <$adapter>::$graph_method(&history).unwrap();
            let result = <$adapter>::$verify_method(&history, &graph);

            assert!(result.is_irreducible, "Expected computation to be irreducible");
            assert!(result.structure_valid, "Expected valid graph structure");
            assert!(result.is_sequence_contiguous, "Expected contiguous interval sequence");
            assert_eq!(result.node_count, $expected_nodes,
                "Expected {} nodes, got {}", $expected_nodes, result.node_count);
        }
    };
}

/// Generates a reducibility test for a causal adapter.
///
/// Tests that an execution with cycles produces a reducible result.
/// This is the complement of `test_irreducibility!`.
///
/// # Arguments
///
/// * `$test_name` - Name for the generated test function
/// * `$adapter` - The adapter type
/// * `$graph_method` - Method to create graph
/// * `$verify_method` - Method to verify irreducibility
/// * `$setup` - Expression that returns the execution history (should have cycles)
///
/// # Example
///
/// ```ignore
/// test_reducible!(
///     test_tm_cycling_reducible,
///     TuringMachineCausalAdapter,
///     execution_to_graph,
///     verify_graph_irreducibility,
///     {
///         let tm = TuringMachine::builder()
///             .states(vec![0, 1])
///             .initial_state(0)
///             .blank('_')
///             .transition(0, '_', 1, '_', Direction::Stay)
///             .transition(1, '_', 0, '_', Direction::Stay)
///             .build();
///         tm.run("", 10)
///     }
/// );
/// ```
#[macro_export]
macro_rules! test_reducible {
    ($test_name:ident, $adapter:ty, $graph_method:ident, $verify_method:ident, $setup:expr) => {
        #[test]
        fn $test_name() {
            let history = $setup;
            let graph = <$adapter>::$graph_method(&history).unwrap();
            let result = <$adapter>::$verify_method(&history, &graph);

            assert!(!result.is_irreducible,
                "Expected computation to be REDUCIBLE (has cycles)");
        }
    };
}

/// Generates a chain analysis test for a causal adapter.
///
/// Tests that chain analysis produces expected step counts and linearity.
///
/// # Arguments
///
/// * `$test_name` - Name for the generated test function
/// * `$adapter` - The adapter type
/// * `$graph_method` - Method to create graph
/// * `$analysis_method` - Method to analyze chains
/// * `$setup` - Expression that returns the execution history
/// * `$expected_steps` - Expected total steps
/// * `$is_linear` - Whether the graph should be linear
///
/// # Example
///
/// ```ignore
/// test_chain_analysis!(
///     test_tm_chain_analysis_bb,
///     TuringMachineCausalAdapter,
///     execution_to_graph,
///     analyze_causal_chains,
///     { let bb = TuringMachine::busy_beaver_2_2(); bb.run("", 20) },
///     6,
///     true
/// );
/// ```
#[macro_export]
macro_rules! test_chain_analysis {
    ($test_name:ident, $adapter:ty, $graph_method:ident, $analysis_method:ident, $setup:expr, $expected_steps:expr, $is_linear:expr) => {
        #[test]
        fn $test_name() {
            let history = $setup;
            let graph = <$adapter>::$graph_method(&history).unwrap();
            let analysis = <$adapter>::$analysis_method(&history, &graph);

            assert_eq!(analysis.total_steps, $expected_steps,
                "Expected {} steps, got {}", $expected_steps, analysis.total_steps);
            assert_eq!(analysis.is_linear, $is_linear,
                "Expected is_linear={}, got {}", $is_linear, analysis.is_linear);
        }
    };
}

/// Generates a labeled graph test for a causal adapter.
///
/// Tests that a labeled graph is created with correct size and fingerprint count.
///
/// # Arguments
///
/// * `$test_name` - Name for the generated test function
/// * `$adapter` - The adapter type
/// * `$labeled_method` - Method to create labeled graph (e.g., `execution_to_labeled_graph`)
/// * `$setup` - Expression that returns the execution history
/// * `$expected_graph_size` - Expected number of nodes in graph
/// * `$expected_fp_count` - Expected number of fingerprints
///
/// # Example
///
/// ```ignore
/// test_labeled_graph!(
///     test_tm_labeled_bb,
///     TuringMachineCausalAdapter,
///     execution_to_labeled_graph,
///     { let bb = TuringMachine::busy_beaver_2_2(); bb.run("", 20) },
///     6,
///     7  // initial + 6 transitions
/// );
/// ```
#[macro_export]
macro_rules! test_labeled_graph {
    ($test_name:ident, $adapter:ty, $labeled_method:ident, $setup:expr, $expected_graph_size:expr, $expected_fp_count:expr) => {
        #[test]
        fn $test_name() {
            let history = $setup;
            let (graph, fingerprints) = <$adapter>::$labeled_method(&history).unwrap();

            assert_eq!(graph.size(), $expected_graph_size,
                "Expected {} nodes, got {}", $expected_graph_size, graph.size());
            assert_eq!(fingerprints.len(), $expected_fp_count,
                "Expected {} fingerprints, got {}", $expected_fp_count, fingerprints.len());
        }
    };
}

// ============================================================================
// Coherence Test Macros
// ============================================================================

/// Creates ParallelIntervals from a concise syntax.
///
/// This macro simplifies the creation of `ParallelIntervals` for testing
/// coherence conditions. Each tuple `(start, end)` becomes a branch.
///
/// # Examples
///
/// ```ignore
/// use irreducible::intervals;
///
/// // Single branch
/// let p = intervals![(0, 5)];
///
/// // Multiple branches (tensor product)
/// let p = intervals![(0, 2), (2, 5), (5, 10)];
///
/// // Create a vector of intervals for comprehensive testing
/// let intervals = vec![
///     intervals![(0, 2)],
///     intervals![(2, 5)],
///     intervals![(5, 10)],
/// ];
/// ```
#[macro_export]
macro_rules! intervals {
    // Single interval
    [($start:expr, $end:expr)] => {{
        $crate::categories::ParallelIntervals::from_branch(
            $crate::categories::DiscreteInterval::new($start, $end)
        )
    }};
    // Multiple intervals (tensor product)
    [$(($start:expr, $end:expr)),+ $(,)?] => {{
        #[allow(unused_mut)]
        let mut result = $crate::categories::ParallelIntervals::new();
        $(
            result.add_branch($crate::categories::DiscreteInterval::new($start, $end));
        )+
        result
    }};
}

/// Generates a comprehensive coherence test for symmetric monoidal categories.
///
/// Tests all coherence conditions (associator α, left unitor λ, right unitor ρ,
/// braiding σ) using `CoherenceVerification::verify_all()`.
///
/// # Arguments
///
/// * `$test_name` - Name for the generated test function
/// * `$intervals` - Expression that returns `Vec<ParallelIntervals>`
///
/// # Example
///
/// ```ignore
/// test_full_coherence!(
///     test_standard_intervals_coherent,
///     vec![
///         intervals![(0, 2)],
///         intervals![(2, 5)],
///         intervals![(5, 10)],
///     ]
/// );
/// ```
#[macro_export]
macro_rules! test_full_coherence {
    ($test_name:ident, $intervals:expr) => {
        #[test]
        fn $test_name() {
            use $crate::functor::monoidal::CoherenceVerification;
            let intervals = $intervals;
            let result = CoherenceVerification::verify_all(&intervals);

            assert!(
                result.fully_coherent,
                "Expected full coherence, got:\n\
                 - associator: {}\n\
                 - left unitor: {}\n\
                 - right unitor: {}\n\
                 - braiding: {}",
                result.associator_coherent,
                result.left_unitor_coherent,
                result.right_unitor_coherent,
                result.braiding_coherent
            );
        }
    };
}

/// Generates a differential coherence test.
///
/// Tests both algebraic coherence and differential form closure using
/// `DifferentialCoherence::verify()`.
///
/// # Arguments
///
/// * `$test_name` - Name for the generated test function
/// * `$intervals` - Expression that returns `Vec<ParallelIntervals>`
/// * `$expected_coherent` - Whether the result should be differentially coherent (default: true)
///
/// # Example
///
/// ```ignore
/// test_differential_coherence!(
///     test_intervals_differentially_coherent,
///     vec![
///         intervals![(0, 2)],
///         intervals![(2, 5)],
///         intervals![(5, 10)],
///     ]
/// );
///
/// // Test expecting incoherence
/// test_differential_coherence!(
///     test_bad_intervals_incoherent,
///     bad_interval_set(),
///     false  // expected not coherent
/// );
/// ```
#[macro_export]
macro_rules! test_differential_coherence {
    ($test_name:ident, $intervals:expr) => {
        $crate::test_differential_coherence!($test_name, $intervals, true);
    };
    ($test_name:ident, $intervals:expr, $expected_coherent:expr) => {
        #[test]
        fn $test_name() {
            use $crate::functor::monoidal::DifferentialCoherence;
            let intervals = $intervals;
            let result = DifferentialCoherence::verify(&intervals);

            if $expected_coherent {
                assert!(
                    result.differential_coherent,
                    "Expected differential coherence, got:\n\
                     - coherence form closed: {}\n\
                     - conservation ratio: {:.4}\n\
                     - non-closure count: {}\n\
                     - defect: {:.4}",
                    result.coherence_form_closed,
                    result.conservation_ratio,
                    result.non_closure_count,
                    result.coherence_defect()
                );
                assert!(!result.has_categorical_curvature(), "Expected flat category (no curvature)");
                assert!(result.coherence_defect() < 0.001, "Expected near-zero coherence defect");
            } else {
                assert!(
                    !result.differential_coherent,
                    "Expected NO differential coherence, but got coherent result"
                );
            }
        }
    };
}

/// Generates a test for an individual coherence condition.
///
/// # Variants
///
/// - `associator` - Tests α: (A ⊗ B) ⊗ C ≅ A ⊗ (B ⊗ C)
/// - `left_unitor` - Tests λ: I ⊗ A ≅ A
/// - `right_unitor` - Tests ρ: A ⊗ I ≅ A
/// - `braiding` - Tests σ: A ⊗ B ≅ B ⊗ A
///
/// # Examples
///
/// ```ignore
/// // Associator test
/// test_coherence_condition!(
///     associator,
///     test_assoc_simple,
///     intervals![(0, 2)],
///     intervals![(2, 5)],
///     intervals![(5, 8)]
/// );
///
/// // Left unitor test
/// test_coherence_condition!(
///     left_unitor,
///     test_left_unit,
///     intervals![(0, 5)]
/// );
///
/// // Braiding test
/// test_coherence_condition!(
///     braiding,
///     test_braid_simple,
///     intervals![(0, 3)],
///     intervals![(3, 7)]
/// );
/// ```
#[macro_export]
macro_rules! test_coherence_condition {
    // Associator: α_{A,B,C}: (A ⊗ B) ⊗ C ≅ A ⊗ (B ⊗ C)
    (associator, $test_name:ident, $a:expr, $b:expr, $c:expr) => {
        #[test]
        fn $test_name() {
            use $crate::functor::monoidal::verify_associator_coherence;
            let a = $a;
            let b = $b;
            let c = $c;
            assert!(
                verify_associator_coherence(&a, &b, &c),
                "Associator coherence failed for (a ⊗ b) ⊗ c ≅ a ⊗ (b ⊗ c)"
            );
        }
    };
    // Left unitor: λ_X: I ⊗ X ≅ X
    (left_unitor, $test_name:ident, $x:expr) => {
        #[test]
        fn $test_name() {
            use $crate::functor::monoidal::verify_left_unitor_coherence;
            let x = $x;
            assert!(
                verify_left_unitor_coherence(&x),
                "Left unitor coherence failed for I ⊗ X ≅ X"
            );
        }
    };
    // Right unitor: ρ_X: X ⊗ I ≅ X
    (right_unitor, $test_name:ident, $x:expr) => {
        #[test]
        fn $test_name() {
            use $crate::functor::monoidal::verify_right_unitor_coherence;
            let x = $x;
            assert!(
                verify_right_unitor_coherence(&x),
                "Right unitor coherence failed for X ⊗ I ≅ X"
            );
        }
    };
    // Braiding: σ_{X,Y}: X ⊗ Y ≅ Y ⊗ X
    (braiding, $test_name:ident, $x:expr, $y:expr) => {
        #[test]
        fn $test_name() {
            use $crate::functor::monoidal::verify_braiding_coherence;
            let x = $x;
            let y = $y;
            assert!(
                verify_braiding_coherence(&x, &y),
                "Braiding coherence failed for X ⊗ Y ≅ Y ⊗ X"
            );
        }
    };
}

// ============================================================================
// Standard Test Fixtures
// ============================================================================

/// Returns standard test intervals for composition testing.
///
/// Provides a set of contiguous intervals that can be used to test
/// interval composition, contiguity checks, and functor operations.
///
/// Returns intervals: [0,1], [1,3], [3,5], [5,10]
pub fn standard_test_intervals() -> Vec<DiscreteInterval> {
    vec![
        DiscreteInterval::new(0, 1),
        DiscreteInterval::new(1, 3),
        DiscreteInterval::new(3, 5),
        DiscreteInterval::new(5, 10),
    ]
}

/// Returns standard non-contiguous intervals for gap detection testing.
///
/// Returns intervals: [0,2], [5,7], [10,12] (gaps at 2-5 and 7-10)
pub fn non_contiguous_intervals() -> Vec<DiscreteInterval> {
    vec![
        DiscreteInterval::new(0, 2),
        DiscreteInterval::new(5, 7),
        DiscreteInterval::new(10, 12),
    ]
}

/// Returns standard parallel intervals for monoidal testing.
///
/// Provides a set of parallel intervals suitable for testing tensor
/// products and symmetric monoidal structure.
pub fn standard_parallel_intervals() -> Vec<ParallelIntervals> {
    vec![
        ParallelIntervals::from_branch(DiscreteInterval::new(0, 5)),
        ParallelIntervals::from_branch(DiscreteInterval::new(10, 15)),
        ParallelIntervals::from_branch(DiscreteInterval::new(20, 25)),
    ]
}

/// Returns a pair of parallel intervals for braiding tests.
pub fn braiding_test_pair() -> (ParallelIntervals, ParallelIntervals) {
    (
        ParallelIntervals::from_branch(DiscreteInterval::new(0, 5)),
        ParallelIntervals::from_branch(DiscreteInterval::new(10, 15)),
    )
}

/// Returns parallel intervals representing a unit (identity element).
pub fn unit_parallel_interval() -> ParallelIntervals {
    ParallelIntervals::unit()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_intervals_contiguous() {
        let intervals = standard_test_intervals();
        assert_eq!(intervals.len(), 4);

        // Verify contiguity
        for window in intervals.windows(2) {
            assert_eq!(window[0].end, window[1].start);
        }
    }

    #[test]
    fn test_non_contiguous_intervals_have_gaps() {
        let intervals = non_contiguous_intervals();
        assert_eq!(intervals.len(), 3);

        // Verify gaps exist
        assert_ne!(intervals[0].end, intervals[1].start);
        assert_ne!(intervals[1].end, intervals[2].start);
    }

    #[test]
    fn test_standard_parallel_intervals() {
        let intervals = standard_parallel_intervals();
        assert_eq!(intervals.len(), 3);

        // Each should be a single-branch parallel interval
        for pi in &intervals {
            assert_eq!(pi.branch_count(), 1);
        }
    }

    #[test]
    fn test_braiding_pair() {
        let (a, b) = braiding_test_pair();
        assert_eq!(a.branch_count(), 1);
        assert_eq!(b.branch_count(), 1);
    }

    #[test]
    fn test_unit_interval() {
        let unit = unit_parallel_interval();
        assert!(unit.is_unit());
    }

    #[test]
    fn test_assert_display_contains_macro() {
        let interval = DiscreteInterval::new(5, 10);
        assert_display_contains!(interval, "5", "10");
    }

    #[test]
    fn test_assert_debug_contains_macro() {
        let interval = DiscreteInterval::new(5, 10);
        assert_debug_contains!(interval, "5", "10");
    }
}
