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
//! ### Coherence Test Macros (re-exported from catgraph)
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

use catgraph::interval::DiscreteInterval;

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
        catgraph::interval::ParallelIntervals::from_branch(
            catgraph::interval::DiscreteInterval::new($start, $end)
        )
    }};
    // Multiple intervals (tensor product)
    [$(($start:expr, $end:expr)),+ $(,)?] => {{
        #[allow(unused_mut)]
        let mut result = catgraph::interval::ParallelIntervals::new();
        $(
            result.add_branch(catgraph::interval::DiscreteInterval::new($start, $end));
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
