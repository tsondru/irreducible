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
//! ### Interval Creation
//! - [`intervals!`] - Concise `ParallelIntervals` creation

use crate::interval::DiscreteInterval;

// ============================================================================
// Display Testing Macro
// ============================================================================

/// Asserts that a value's Display output contains all expected substrings.
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
// Interval Creation Macro
// ============================================================================

/// Creates `ParallelIntervals` from a concise syntax.
///
/// Each tuple `(start, end)` becomes a branch.
///
/// # Examples
///
/// ```ignore
/// // Single branch
/// let p = intervals![(0, 5)];
///
/// // Multiple branches (tensor product)
/// let p = intervals![(0, 2), (2, 5), (5, 10)];
/// ```
#[macro_export]
macro_rules! intervals {
    // Single interval
    [($start:expr, $end:expr)] => {{
        crate::interval::ParallelIntervals::from_branch(
            crate::interval::DiscreteInterval::new($start, $end)
        )
    }};
    // Multiple intervals (tensor product)
    [$(($start:expr, $end:expr)),+ $(,)?] => {{
        #[allow(unused_mut)]
        let mut result = crate::interval::ParallelIntervals::new();
        $(
            result.add_branch(crate::interval::DiscreteInterval::new($start, $end));
        )+
        result
    }};
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
