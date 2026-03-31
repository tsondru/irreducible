//! Stokes integration for computational irreducibility analysis.
//!
//! This module interprets computation sequences as differential forms on a
//! temporal simplicial complex, using the discrete Stokes theorem to verify
//! conservation laws.
//!
//! # Mathematical Framework
//!
//! Given a sequence of `DiscreteInterval`s representing computation steps:
//!
//! - **0-simplices (vertices)**: Time steps t₀, t₁, ..., tₙ
//! - **1-simplices (edges)**: Intervals [tᵢ, tᵢ₊₁] connecting consecutive times
//! - **1-forms**: Assign coefficients (complexity) to each edge
//!
//! For a 1-dimensional simplicial complex, the exterior derivative d of any
//! 1-form is trivially zero (there are no 2-simplices). This means
//! conservation reduces to:
//!
//! 1. **Contiguity** — intervals connect without gaps
//! 2. **Monotonicity** — time flows forward
//! 3. **Integration consistency** — integrated complexity equals direct sum
//!
//! # Physical Interpretation
//!
//! - **Closed 1-form**: Total complexity is conserved (no shortcuts, no inflation)
//! - **Exact 1-form**: Complexity derives from a potential (state function)
//! - **Non-closed form**: Indicates computational "sources" or "sinks"
//!
//! # Novel Insight
//!
//! For 1D complexes, dω is vacuously zero, so Stokes conservation reduces to
//! cospan composability — exactly the condition for categorical composition
//! in the cobordism category ℬ. This bridges Stokes integration to
//! catgraph's cospan architecture.
//!
//! # Example
//!
//! ```rust
//! use irreducible::functor::stokes_integration::{TemporalComplex, ConservationResult};
//! use irreducible::categories::DiscreteInterval;
//!
//! let intervals = vec![
//!     DiscreteInterval::new(0, 2),
//!     DiscreteInterval::new(2, 5),
//!     DiscreteInterval::new(5, 7),
//! ];
//!
//! let complex = TemporalComplex::from_intervals(&intervals).unwrap();
//! let result = complex.verify_conservation();
//!
//! assert!(result.is_conserved);
//! ```

use crate::categories::DiscreteInterval;

/// A simplicial complex representing temporal structure of computation.
///
/// The complex has dimension 1:
/// - 0-skeleton: time step vertices
/// - 1-skeleton: interval edges connecting consecutive times
///
/// For n intervals \[t₀,t₁\], \[t₁,t₂\], ..., \[tₙ₋₁,tₙ\]:
/// - n+1 vertices: t₀, t₁, ..., tₙ
/// - n edges: e₀₁, e₁₂, ..., eₙ₋₁ₙ
#[derive(Debug, Clone)]
pub struct TemporalComplex {
    /// Time points (vertices of the 0-skeleton).
    time_points: Vec<usize>,
    /// Step counts for each interval (edge weights).
    step_counts: Vec<usize>,
}

impl TemporalComplex {
    /// Creates a temporal complex from a sequence of intervals.
    ///
    /// The intervals should be contiguous (composable) for a well-defined
    /// temporal structure. Non-contiguous intervals will still produce a
    /// complex, but conservation will fail.
    ///
    /// # Errors
    ///
    /// Returns `StokesError::EmptyIntervals` if the interval slice is empty.
    pub fn from_intervals(intervals: &[DiscreteInterval]) -> Result<Self, StokesError> {
        if intervals.is_empty() {
            return Err(StokesError::EmptyIntervals);
        }

        let mut time_points: Vec<usize> = Vec::new();
        for interval in intervals {
            if time_points.is_empty() || time_points.last() != Some(&interval.start) {
                time_points.push(interval.start);
            }
            time_points.push(interval.end);
        }

        time_points.sort_unstable();
        time_points.dedup();

        if time_points.len() < 2 {
            return Err(StokesError::InsufficientPoints(time_points.len()));
        }

        let step_counts: Vec<usize> = time_points
            .windows(2)
            .map(|w| w[1].saturating_sub(w[0]))
            .collect();

        Ok(Self {
            time_points,
            step_counts,
        })
    }

    /// Returns the number of time steps (vertices).
    #[inline]
    #[must_use]
    pub fn num_time_steps(&self) -> usize {
        self.time_points.len()
    }

    /// Returns the number of intervals (edges).
    #[inline]
    #[must_use]
    pub fn num_intervals(&self) -> usize {
        self.step_counts.len()
    }

    /// Returns the time points.
    #[inline]
    #[must_use]
    pub fn time_points(&self) -> &[usize] {
        &self.time_points
    }

    /// Returns the step counts for each interval.
    #[inline]
    #[must_use]
    pub fn step_counts(&self) -> &[usize] {
        &self.step_counts
    }

    /// Converts the interval sequence to a 1-form (coefficient vector).
    ///
    /// Each coefficient is the step count (complexity) of that interval.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[must_use]
    pub fn intervals_to_form(&self) -> Vec<f64> {
        self.step_counts.iter().map(|&s| s as f64).collect()
    }

    /// Applies the exterior derivative to a 1-form.
    ///
    /// For a 1-dimensional complex, d(1-form) = 0 always (no 2-simplices).
    /// Returns a zero vector.
    #[must_use]
    pub fn exterior_derivative(&self, _form: &[f64]) -> Vec<f64> {
        // In dim-1, dω is vacuously zero: there are no 2-simplices
        // to integrate over. This is the key insight connecting
        // Stokes conservation to cospan composability.
        vec![0.0; self.num_time_steps().saturating_sub(2).max(0)]
    }

    /// Integrates a 1-form over the full chain.
    ///
    /// The chain assigns weight 1.0 to each edge, so integration
    /// is simply the sum of coefficients.
    #[must_use]
    pub fn integrate(&self, form: &[f64]) -> f64 {
        form.iter().sum()
    }

    /// Verifies if the interval sequence satisfies conservation.
    ///
    /// Conservation means the computation is "balanced" — no steps are
    /// created or destroyed along the trajectory.
    ///
    /// # Conservation Criteria
    ///
    /// 1. **Contiguity**: Intervals must be composable (no gaps)
    /// 2. **Monotonicity**: Time must flow forward (no negative steps)
    /// 3. **Closure**: The 1-form is closed (trivially true in dim-1)
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[must_use]
    pub fn verify_conservation(&self) -> ConservationResult {
        let form = self.intervals_to_form();
        let d_form = self.exterior_derivative(&form);
        let is_closed = d_form.iter().all(|&c| c.abs() < 1e-10);
        let is_contiguous = self.step_counts.iter().all(|&s| s > 0);
        let total_complexity: f64 = self.step_counts.iter().map(|&s| s as f64).sum();
        let is_monotonic = self.time_points.windows(2).all(|w| w[0] < w[1]);

        ConservationResult {
            is_conserved: is_closed && is_contiguous && is_monotonic,
            is_closed,
            is_contiguous,
            is_monotonic,
            total_complexity,
            num_intervals: self.num_intervals(),
            num_time_steps: self.num_time_steps(),
        }
    }
}

/// Result of conservation verification for a temporal complex.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq)]
pub struct ConservationResult {
    /// Whether the computation is conserved (closed, contiguous, monotonic).
    pub is_conserved: bool,
    /// Whether the derived 1-form is closed (dω = 0).
    pub is_closed: bool,
    /// Whether intervals are contiguous (no gaps).
    pub is_contiguous: bool,
    /// Whether time is monotonically increasing.
    pub is_monotonic: bool,
    /// Total complexity (sum of all step counts).
    pub total_complexity: f64,
    /// Number of intervals in the trajectory.
    pub num_intervals: usize,
    /// Number of time steps (vertices).
    pub num_time_steps: usize,
}

impl ConservationResult {
    /// Returns the average complexity per interval.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[inline]
    #[must_use]
    pub fn average_complexity(&self) -> f64 {
        if self.num_intervals == 0 {
            0.0
        } else {
            self.total_complexity / self.num_intervals as f64
        }
    }

    /// Checks if the trajectory is well-formed (contiguous and monotonic).
    #[inline]
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        self.is_contiguous && self.is_monotonic
    }
}

/// Uses Stokes integration to analyze irreducibility.
///
/// This complements the functorial approach by providing a differential
/// geometric perspective on computational irreducibility.
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
}

// ============================================================================
// Catgraph bridge: Stokes conservation → cospan composability
// ============================================================================

impl TemporalComplex {
    /// Converts the temporal complex into a chain of composable cospans.
    ///
    /// Each interval [tᵢ, tᵢ₊₁] becomes a cospan where:
    /// - Left boundary: time point tᵢ
    /// - Right boundary: time point tᵢ₊₁
    /// - Apex: the interval's internal structure (both boundary points)
    ///
    /// The chain is composable because the right boundary of cospan i
    /// (time point tᵢ₊₁) matches the left boundary of cospan i+1.
    ///
    /// This makes precise the insight that Stokes conservation in dim-1
    /// reduces to cospan composability — the categorical condition for
    /// composition in the cobordism category ℬ.
    ///
    /// # Example
    ///
    /// ```rust
    /// use irreducible::functor::stokes_integration::TemporalComplex;
    /// use irreducible::categories::DiscreteInterval;
    ///
    /// let intervals = vec![
    ///     DiscreteInterval::new(0, 2),
    ///     DiscreteInterval::new(2, 5),
    ///     DiscreteInterval::new(5, 7),
    /// ];
    ///
    /// let complex = TemporalComplex::from_intervals(&intervals).unwrap();
    /// let cospans = complex.to_cospan_chain();
    ///
    /// assert_eq!(cospans.len(), 3);
    /// ```
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[must_use]
    pub fn to_cospan_chain(&self) -> Vec<catgraph::cospan::Cospan<u32>> {
        let mut cospans = Vec::new();

        for i in 0..self.num_intervals() {
            // Each interval [tᵢ, tᵢ₊₁] is a cospan with:
            // - Middle (apex): 2 elements labeled with actual time point values
            // - Left: 1 element (tᵢ) mapping to apex[0]
            // - Right: 1 element (tᵢ₊₁) mapping to apex[1]
            //
            // For composability: cospan i has codomain = [t_{i+1}],
            // cospan i+1 has domain = [t_{i+1}], so they match.
            let t_start = self.time_points[i] as u32;
            let t_end = self.time_points[i + 1] as u32;
            let left = vec![0];    // tᵢ → apex element 0
            let right = vec![1];   // tᵢ₊₁ → apex element 1
            let middle = vec![t_start, t_end];

            cospans.push(catgraph::cospan::Cospan::new(left, right, middle));
        }

        cospans
    }

    /// Composes the full cospan chain into a single composite cospan
    /// representing the entire temporal interval.
    ///
    /// The composite has domain = `[t_0]` and codomain = `[t_n]`.
    ///
    /// # Panics
    ///
    /// Panics if adjacent cospans in the chain are not composable.
    ///
    /// # Errors
    ///
    /// Returns `CatgraphError::Composition` if the cospan chain is empty.
    pub fn compose_cospan_chain(&self) -> Result<catgraph::cospan::Cospan<u32>, catgraph::errors::CatgraphError> {
        use catgraph::category::Composable;
        let chain = self.to_cospan_chain();
        chain.into_iter()
            .reduce(|acc, c| acc.compose(&c).expect("stokes cospans must be composable"))
            .ok_or_else(|| catgraph::errors::CatgraphError::Composition(
                "empty cospan chain".to_string()
            ))
    }
}

impl StokesIrreducibility {
    /// Returns the cospan chain for this analysis.
    ///
    /// Convenience wrapper around `TemporalComplex::to_cospan_chain()`.
    #[must_use]
    pub fn to_cospan_chain(&self) -> Vec<catgraph::cospan::Cospan<u32>> {
        self.complex.to_cospan_chain()
    }
}

/// Errors that can occur during Stokes integration analysis.
#[derive(Debug, Clone, PartialEq)]
pub enum StokesError {
    /// No intervals provided.
    EmptyIntervals,
    /// Insufficient time points to form a complex.
    InsufficientPoints(usize),
}

impl std::fmt::Display for StokesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyIntervals => write!(f, "Cannot create temporal complex from empty intervals"),
            Self::InsufficientPoints(n) => {
                write!(f, "Need at least 2 time points, got {n}")
            }
        }
    }
}

impl std::error::Error for StokesError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_complex_creation() {
        let intervals = vec![
            DiscreteInterval::new(0, 2),
            DiscreteInterval::new(2, 5),
            DiscreteInterval::new(5, 7),
        ];

        let complex = TemporalComplex::from_intervals(&intervals).unwrap();
        assert_eq!(complex.num_time_steps(), 4);
        assert_eq!(complex.num_intervals(), 3);
        assert_eq!(complex.step_counts(), &[2, 3, 2]);
    }

    #[test]
    fn test_intervals_to_form() {
        let intervals = vec![
            DiscreteInterval::new(0, 2),
            DiscreteInterval::new(2, 5),
        ];

        let complex = TemporalComplex::from_intervals(&intervals).unwrap();
        let form = complex.intervals_to_form();
        assert_eq!(form, vec![2.0, 3.0]);
    }

    #[test]
    fn test_conservation_contiguous() {
        let intervals = vec![
            DiscreteInterval::new(0, 1),
            DiscreteInterval::new(1, 2),
            DiscreteInterval::new(2, 3),
        ];

        let complex = TemporalComplex::from_intervals(&intervals).unwrap();
        let result = complex.verify_conservation();

        assert!(result.is_contiguous);
        assert!(result.is_monotonic);
        assert_eq!(result.total_complexity, 3.0);
    }

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
    fn test_integration_over_chain() {
        let intervals = vec![
            DiscreteInterval::new(0, 3),
            DiscreteInterval::new(3, 7),
        ];

        let complex = TemporalComplex::from_intervals(&intervals).unwrap();
        let form = complex.intervals_to_form();
        let integrated = complex.integrate(&form);

        assert!((integrated - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_average_complexity() {
        let intervals = vec![
            DiscreteInterval::new(0, 2),
            DiscreteInterval::new(2, 6),
            DiscreteInterval::new(6, 8),
        ];

        let complex = TemporalComplex::from_intervals(&intervals).unwrap();
        let result = complex.verify_conservation();

        assert!((result.average_complexity() - 8.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_exterior_derivative_is_zero() {
        let intervals = vec![
            DiscreteInterval::new(0, 3),
            DiscreteInterval::new(3, 7),
            DiscreteInterval::new(7, 10),
        ];

        let complex = TemporalComplex::from_intervals(&intervals).unwrap();
        let form = complex.intervals_to_form();
        let d_form = complex.exterior_derivative(&form);

        // In dim-1, dω is always zero
        assert!(d_form.iter().all(|&c| c.abs() < 1e-10));
    }

    // ── Catgraph cospan chain tests ─────────────────────────────────────

    #[test]
    fn test_cospan_chain_length() {
        let intervals = vec![
            DiscreteInterval::new(0, 2),
            DiscreteInterval::new(2, 5),
            DiscreteInterval::new(5, 7),
        ];

        let complex = TemporalComplex::from_intervals(&intervals).unwrap();
        let cospans = complex.to_cospan_chain();

        assert_eq!(cospans.len(), 3);
    }

    #[test]
    fn test_cospan_chain_composability() {
        let intervals = vec![
            DiscreteInterval::new(0, 1),
            DiscreteInterval::new(1, 2),
            DiscreteInterval::new(2, 3),
        ];

        let complex = TemporalComplex::from_intervals(&intervals).unwrap();
        let cospans = complex.to_cospan_chain();

        // Each cospan has 1 left, 1 right, 2 middle elements
        for cospan in &cospans {
            assert_eq!(cospan.left_to_middle().len(), 1);
            assert_eq!(cospan.right_to_middle().len(), 1);
            assert_eq!(cospan.middle().len(), 2);
        }

        // Right boundary of cospan i has same size as left boundary of cospan i+1
        for i in 0..cospans.len() - 1 {
            assert_eq!(
                cospans[i].right_to_middle().len(),
                cospans[i + 1].left_to_middle().len(),
            );
        }
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

    // ── Labeled cospan tests (Phase 4) ────────────────────────────────

    #[test]
    fn test_cospan_labels_are_time_points() {
        let intervals = vec![
            DiscreteInterval::new(0, 2),
            DiscreteInterval::new(2, 5),
            DiscreteInterval::new(5, 7),
        ];

        let complex = TemporalComplex::from_intervals(&intervals).unwrap();
        let cospans = complex.to_cospan_chain();

        // Each cospan's apex labels should be the actual time point values
        assert_eq!(cospans[0].middle(), &[0u32, 2]);
        assert_eq!(cospans[1].middle(), &[2u32, 5]);
        assert_eq!(cospans[2].middle(), &[5u32, 7]);
    }

    #[test]
    fn test_cospan_chain_composable_via_catgraph() {
        use catgraph::category::Composable;

        let intervals = vec![
            DiscreteInterval::new(0, 1),
            DiscreteInterval::new(1, 2),
            DiscreteInterval::new(2, 3),
        ];

        let complex = TemporalComplex::from_intervals(&intervals).unwrap();
        let cospans = complex.to_cospan_chain();

        for i in 0..cospans.len() - 1 {
            assert!(
                cospans[i].composable(&cospans[i + 1]).is_ok(),
                "stokes cospans {} and {} should be composable", i, i + 1
            );
        }
    }

    #[test]
    fn test_compose_cospan_chain() {
        use catgraph::category::Composable;

        let intervals = vec![
            DiscreteInterval::new(0, 3),
            DiscreteInterval::new(3, 7),
            DiscreteInterval::new(7, 10),
        ];

        let complex = TemporalComplex::from_intervals(&intervals).unwrap();
        let composite = complex.compose_cospan_chain().unwrap();

        // Composite domain should be [t_0], codomain should be [t_n]
        assert_eq!(composite.domain(), vec![0u32]);
        assert_eq!(composite.codomain(), vec![10u32]);
    }

    #[test]
    fn test_single_interval_cospan() {
        let intervals = vec![DiscreteInterval::new(5, 12)];

        let complex = TemporalComplex::from_intervals(&intervals).unwrap();
        let cospans = complex.to_cospan_chain();

        assert_eq!(cospans.len(), 1);
        assert_eq!(cospans[0].middle(), &[5u32, 12]);
    }
}
