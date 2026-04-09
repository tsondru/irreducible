//! Stokes integration for computational irreducibility analysis.
//!
//! Core types (`TemporalComplex`, `ConservationResult`, `StokesError`)
//! re-exported from [`catgraph::stokes`]. This module provides the
//! irreducibility-specific `StokesIrreducibility` wrapper.

pub use catgraph::stokes::{ConservationResult, StokesError, TemporalComplex};

use catgraph::interval::DiscreteInterval;

use super::fong_spivak::{verify_cospan_chain_frobenius, FrobeniusVerificationResult};

/// Stokes-theorem perspective on computational irreducibility.
///
/// Wraps a [`TemporalComplex`] (simplicial complex from interval chain)
/// and its [`ConservationResult`] to provide a differential-geometric
/// irreducibility check: a computation is Stokes-irreducible when the
/// integrated complexity equals the expected total (no leakage or inflation).
///
/// This complements the functorial and trace-based approaches.
#[derive(Debug, Clone)]
pub struct StokesIrreducibility {
    /// The temporal complex for the computation.
    pub complex: TemporalComplex,
    /// Conservation analysis result.
    pub conservation: ConservationResult,
    /// Integrated complexity over the full chain.
    pub integrated_complexity: f64,
}

impl StokesIrreducibility {
    /// Analyzes a sequence of intervals using Stokes integration.
    ///
    /// # Errors
    ///
    /// Returns `StokesError::EmptyIntervals` if the interval slice is empty.
    pub fn analyze(intervals: &[DiscreteInterval]) -> Result<Self, StokesError> {
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

    /// Checks if the computation is irreducible from the Stokes perspective.
    ///
    /// A computation is Stokes-irreducible if:
    /// 1. The trajectory is conserved (no leakage)
    /// 2. The integrated complexity equals the expected total
    #[inline]
    #[must_use]
    pub fn is_irreducible(&self) -> bool {
        self.conservation.is_conserved
            && (self.integrated_complexity - self.conservation.total_complexity).abs() < 1e-10
    }

    /// Returns the ratio of integrated to expected complexity.
    ///
    /// - Ratio = 1.0: Perfect conservation (irreducible)
    /// - Ratio < 1.0: Complexity loss (shortcut/leakage)
    /// - Ratio > 1.0: Complexity gain (inflation)
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
    ///
    /// Convenience wrapper around `TemporalComplex::to_cospan_chain()`.
    #[must_use]
    pub fn to_cospan_chain(&self) -> Vec<catgraph::cospan::Cospan<u32>> {
        self.complex.to_cospan_chain()
    }

    /// Verify Frobenius structure on the Stokes cospan chain.
    ///
    /// Decomposes each cospan via [`CospanToFrobeniusFunctor`](super::CospanToFrobeniusFunctor)
    /// and checks that composition is preserved. This provides a Fong-Spivak categorical
    /// verification complementing the differential-geometric Stokes check.
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
        assert!(matches!(result, Err(StokesError::EmptyIntervals)));
    }

    #[test]
    fn test_stokes_irreducibility_cospan_chain() {
        let intervals = vec![
            DiscreteInterval::new(0, 3),
            DiscreteInterval::new(3, 7),
        ];

        let analysis = StokesIrreducibility::analyze(&intervals).unwrap();
        let cospans = analysis.to_cospan_chain();

        assert_eq!(cospans.len(), 2);
        assert!(analysis.is_irreducible());
    }
}
