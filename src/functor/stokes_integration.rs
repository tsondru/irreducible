//! Stokes integration for computational irreducibility analysis.
//!
//! Core types (`TemporalComplex`, `ConservationResult`, `TemporalComplexError`)
//! re-exported from [`catgraph_physics::temporal_cospan_chain`].
//! This module provides the irreducibility-specific `StokesIrreducibility` wrapper.

pub use catgraph_physics::temporal_cospan_chain::{
    ConservationResult, TemporalComplex, TemporalComplexError,
};

use catgraph_physics::interval::DiscreteInterval;

use super::fong_spivak::{FrobeniusVerificationResult, verify_cospan_chain_frobenius};

/// Stokes conservation analysis of a non-empty interval sequence, kept in
/// input order.
///
/// Holds the [`TemporalComplex`] of the sequence, its [`ConservationResult`],
/// and the integral of its step-count 1-form: the sum of `end - start` over
/// the input intervals.
#[derive(Debug, Clone)]
pub struct StokesIrreducibility {
    /// The temporal complex of the input sequence.
    pub complex: TemporalComplex,
    /// Contiguity, monotonicity and span `last.end - first.start` of the input
    /// sequence.
    pub conservation: ConservationResult,
    /// Sum of `end - start` over the input intervals.
    pub integrated_complexity: f64,
}

impl StokesIrreducibility {
    /// Builds the temporal complex of `intervals` in input order, checks its
    /// conservation, and integrates its step-count 1-form.
    ///
    /// # Errors
    ///
    /// Returns [`TemporalComplexError::EmptyIntervals`] if the interval slice
    /// is empty, or [`TemporalComplexError::InsufficientPoints`] if its
    /// endpoints, with consecutive equal points merged, number fewer than two.
    pub fn analyze(intervals: &[DiscreteInterval]) -> Result<Self, TemporalComplexError> {
        let complex = TemporalComplex::from_intervals(intervals)?;
        let conservation = complex.verify_conservation();
        let form = complex.intervals_to_form();
        let integrated_complexity = complex.integrate(&form);

        Ok(Self {
            complex,
            conservation,
            integrated_complexity,
        })
    }

    /// Returns `true` iff the input sequence is conserved (each interval ends
    /// where the next starts, and starts are non-decreasing) and the
    /// integrated complexity equals the span `last.end - first.start` within
    /// `1e-10`.
    #[inline]
    #[must_use]
    pub fn is_irreducible(&self) -> bool {
        self.conservation.is_conserved
            && (self.integrated_complexity - self.conservation.total_complexity).abs() < 1e-10
    }

    /// Returns integrated complexity divided by the span
    /// `last.end - first.start`, or `1.0` when the span is within `1e-10` of
    /// zero.
    #[inline]
    #[must_use]
    pub fn conservation_ratio(&self) -> f64 {
        if self.conservation.total_complexity.abs() < 1e-10 {
            1.0
        } else {
            self.integrated_complexity / self.conservation.total_complexity
        }
    }

    /// Returns the cospan chain for this analysis.
    #[must_use]
    pub fn to_cospan_chain(&self) -> Vec<catgraph::cospan::Cospan<u32>> {
        self.complex.to_cospan_chain()
    }

    /// Verify Frobenius structure on the Stokes cospan chain.
    #[must_use]
    pub fn verify_frobenius(&self) -> FrobeniusVerificationResult {
        verify_cospan_chain_frobenius(&self.to_cospan_chain())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stokes_irreducibility_simple() {
        let intervals = vec![
            DiscreteInterval::new(0, 2),
            DiscreteInterval::new(2, 4),
            DiscreteInterval::new(4, 6),
        ];
        let analysis = StokesIrreducibility::analyze(&intervals).unwrap();
        assert!(analysis.is_irreducible());
        assert!((analysis.conservation_ratio() - 1.0).abs() < 1e-10);
        assert_eq!(analysis.conservation.total_complexity, 6.0);
    }

    #[test]
    fn test_empty_intervals_error() {
        let result = TemporalComplex::from_intervals(&[]);
        assert!(matches!(result, Err(TemporalComplexError::EmptyIntervals)));
    }

    #[test]
    fn test_stokes_irreducibility_cospan_chain() {
        let intervals = vec![DiscreteInterval::new(0, 3), DiscreteInterval::new(3, 7)];
        let analysis = StokesIrreducibility::analyze(&intervals).unwrap();
        let cospans = analysis.to_cospan_chain();
        assert_eq!(cospans.len(), 2);
        assert!(analysis.is_irreducible());
    }

    #[test]
    fn test_stokes_reducible_on_gap() {
        let intervals = vec![DiscreteInterval::new(0, 2), DiscreteInterval::new(5, 7)];
        let analysis = StokesIrreducibility::analyze(&intervals).unwrap();
        let integrated = analysis.integrated_complexity;
        let total = analysis.conservation.total_complexity;
        assert!(
            (integrated - 4.0).abs() < 1e-10 && (total - 7.0).abs() < 1e-10,
            "gap [0,2],[5,7]: integrated = {integrated}, total_complexity = {total} (expected 4 vs 7)"
        );
        assert!(
            !analysis.is_irreducible(),
            "gap [0,2],[5,7]: is_irreducible = true with integrated = {integrated}, \
             total_complexity = {total}, is_conserved = {} (expected false)",
            analysis.conservation.is_conserved
        );
    }

    #[test]
    fn test_stokes_reducible_on_overlap() {
        let intervals = vec![DiscreteInterval::new(0, 3), DiscreteInterval::new(2, 5)];
        let analysis = StokesIrreducibility::analyze(&intervals).unwrap();
        let integrated = analysis.integrated_complexity;
        let total = analysis.conservation.total_complexity;
        assert!(
            (integrated - 6.0).abs() < 1e-10 && (total - 5.0).abs() < 1e-10,
            "overlap [0,3],[2,5]: integrated = {integrated}, total_complexity = {total} (expected 6 vs 5)"
        );
        assert!(
            !analysis.is_irreducible(),
            "overlap [0,3],[2,5]: is_irreducible = true with integrated = {integrated}, \
             total_complexity = {total}, is_conserved = {} (expected false)",
            analysis.conservation.is_conserved
        );
    }
}
