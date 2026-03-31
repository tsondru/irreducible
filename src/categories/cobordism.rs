//! Cobordism category ℬ implementation.
//!
//! Objects in ℬ are natural numbers (time steps).
//! Morphisms in ℬ are discrete intervals [n, m] ∩ ℕ representing
//! computational time spans.
//!
//! The composition of morphisms is interval union (when contiguous).

use std::fmt;

/// A discrete interval [start, end] ∩ ℕ.
///
/// This represents a morphism in the cobordism category ℬ.
/// The cardinality |\[n,m\] ∩ ℕ| = m - n + 1 gives the computational complexity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DiscreteInterval {
    /// Start of the interval (inclusive)
    pub start: usize,
    /// End of the interval (inclusive)
    pub end: usize,
}

impl DiscreteInterval {
    /// Create a new discrete interval [start, end].
    ///
    /// # Panics
    /// Panics if start > end.
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end, "interval start must be <= end");
        Self { start, end }
    }

    /// Create a singleton interval [n, n].
    #[must_use]
    pub fn singleton(n: usize) -> Self {
        Self { start: n, end: n }
    }

    /// Create an identity morphism at time step n.
    ///
    /// The identity is represented as a singleton [n, n].
    #[must_use]
    pub fn identity(n: usize) -> Self {
        Self::singleton(n)
    }

    /// The cardinality (complexity) of this interval: |\[n,m\] ∩ ℕ| = m - n + 1
    #[must_use]
    pub fn cardinality(&self) -> usize {
        self.end - self.start + 1
    }

    /// The number of steps in this interval: m - n
    #[must_use]
    pub fn steps(&self) -> usize {
        self.end - self.start
    }

    /// Check if this interval is a singleton (identity morphism).
    #[must_use]
    pub fn is_identity(&self) -> bool {
        self.start == self.end
    }

    /// Compose two contiguous intervals: \[a,b\] ∘ \[b,c\] = \[a,c\]
    ///
    /// Returns None if the intervals are not contiguous.
    /// Note: composition is right-to-left (mathematical convention).
    #[must_use]
    pub fn compose(self, other: Self) -> Option<Self> {
        // Composition: f: b→c composed with g: a→b gives f∘g: a→c
        // So other.end should equal self.start
        if other.end == self.start {
            Some(Self {
                start: other.start,
                end: self.end,
            })
        } else {
            None
        }
    }

    /// Sequential composition (left-to-right): \[a,b\] then \[b,c\] = \[a,c\]
    ///
    /// This is the more intuitive order for computational steps.
    #[must_use]
    pub fn then(self, next: Self) -> Option<Self> {
        if self.end == next.start {
            Some(Self {
                start: self.start,
                end: next.end,
            })
        } else {
            None
        }
    }

    /// Check if two intervals can be composed.
    #[must_use]
    pub fn is_composable_with(&self, other: &Self) -> bool {
        self.end == other.start
    }

    /// Check if this interval contains a time step.
    #[must_use]
    pub fn contains(&self, step: usize) -> bool {
        step >= self.start && step <= self.end
    }

    /// Check if this interval contains another interval.
    #[must_use]
    pub fn contains_interval(&self, other: &Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    /// Get the intersection of two intervals, if non-empty.
    #[must_use]
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }
}

impl fmt::Display for DiscreteInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.start, self.end)
    }
}

/// Parallel intervals for multiway systems.
///
/// In multiway computation, multiple branches execute simultaneously.
/// This structure captures the tensor product structure of parallel computation.
#[derive(Clone, Debug, Default)]
pub struct ParallelIntervals {
    /// The collection of branch intervals
    pub branches: Vec<DiscreteInterval>,
}

impl ParallelIntervals {
    /// Create an empty parallel interval structure.
    #[must_use]
    pub fn new() -> Self {
        Self {
            branches: Vec::new(),
        }
    }

    /// Create from a single branch.
    #[must_use]
    pub fn from_branch(interval: DiscreteInterval) -> Self {
        Self {
            branches: vec![interval],
        }
    }

    /// Add a branch to this parallel structure.
    pub fn add_branch(&mut self, interval: DiscreteInterval) {
        self.branches.push(interval);
    }

    /// Tensor product ⊗: combine two parallel computations.
    ///
    /// This corresponds to the monoidal structure on the cobordism category.
    #[must_use]
    pub fn tensor(mut self, other: Self) -> Self {
        self.branches.extend(other.branches);
        self
    }

    /// Total multicomputational complexity (sum of all branch complexities).
    #[must_use]
    pub fn total_complexity(&self) -> usize {
        self.branches.iter().map(DiscreteInterval::cardinality).sum()
    }

    /// Maximum branch complexity (wall-clock time for parallel execution).
    #[must_use]
    pub fn max_complexity(&self) -> usize {
        self.branches.iter().map(DiscreteInterval::cardinality).max().unwrap_or(0)
    }

    /// Number of parallel branches.
    #[must_use]
    pub fn branch_count(&self) -> usize {
        self.branches.len()
    }

    /// Check if this represents a singleway (non-branching) computation.
    #[must_use]
    pub fn is_singleway(&self) -> bool {
        self.branches.len() <= 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discrete_interval_new() {
        let interval = DiscreteInterval::new(2, 5);
        assert_eq!(interval.start, 2);
        assert_eq!(interval.end, 5);
    }

    #[test]
    fn test_discrete_interval_cardinality() {
        let interval = DiscreteInterval::new(2, 5);
        assert_eq!(interval.cardinality(), 4); // 2, 3, 4, 5
    }

    #[test]
    fn test_discrete_interval_singleton() {
        let interval = DiscreteInterval::singleton(3);
        assert_eq!(interval.cardinality(), 1);
        assert!(interval.is_identity());
    }

    #[test]
    fn test_discrete_interval_compose() {
        let ab = DiscreteInterval::new(0, 2);
        let bc = DiscreteInterval::new(2, 5);
        let ac = bc.compose(ab);
        assert!(ac.is_some());
        let ac = ac.unwrap();
        assert_eq!(ac.start, 0);
        assert_eq!(ac.end, 5);
    }

    #[test]
    fn test_discrete_interval_compose_non_contiguous() {
        let ab = DiscreteInterval::new(0, 2);
        let cd = DiscreteInterval::new(3, 5); // gap between 2 and 3
        let result = cd.compose(ab);
        assert!(result.is_none());
    }

    #[test]
    fn test_discrete_interval_then() {
        let ab = DiscreteInterval::new(0, 2);
        let bc = DiscreteInterval::new(2, 5);
        let ac = ab.then(bc);
        assert!(ac.is_some());
        let ac = ac.unwrap();
        assert_eq!(ac.start, 0);
        assert_eq!(ac.end, 5);
    }

    #[test]
    fn test_discrete_interval_contains() {
        let interval = DiscreteInterval::new(2, 5);
        assert!(!interval.contains(1));
        assert!(interval.contains(2));
        assert!(interval.contains(3));
        assert!(interval.contains(5));
        assert!(!interval.contains(6));
    }

    #[test]
    fn test_parallel_intervals_tensor() {
        let branch1 = ParallelIntervals::from_branch(DiscreteInterval::new(0, 3));
        let branch2 = ParallelIntervals::from_branch(DiscreteInterval::new(0, 5));
        let combined = branch1.tensor(branch2);
        assert_eq!(combined.branch_count(), 2);
        assert_eq!(combined.total_complexity(), 4 + 6); // 10
        assert_eq!(combined.max_complexity(), 6);
    }

    #[test]
    fn test_parallel_intervals_singleway() {
        let single = ParallelIntervals::from_branch(DiscreteInterval::new(0, 3));
        assert!(single.is_singleway());

        let multi = single.tensor(ParallelIntervals::from_branch(DiscreteInterval::new(0, 2)));
        assert!(!multi.is_singleway());
    }
}
