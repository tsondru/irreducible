//! Turing machine transitions (morphisms in category 𝒯).
//!
//! A [`Transition`] records one step of computation: the source and target
//! [`Configuration`], plus the step number. The functor Z' maps each
//! transition to a unit interval [step, step + 1] in the cobordism category ℬ.
//!
//! [`Direction`] encodes the three possible head movements (Left, Right, Stay).

use super::Configuration;
use catgraph::{interval::DiscreteInterval, complexity::StepCount};
use std::fmt;

/// Direction the head moves after a transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Move left (decrement head position)
    Left,
    /// Move right (increment head position)
    Right,
    /// Stay in place
    Stay,
}

impl Direction {
    /// Get the head position delta for this direction.
    #[must_use]
    pub fn delta(&self) -> isize {
        match self {
            Direction::Left => -1,
            Direction::Right => 1,
            Direction::Stay => 0,
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Left => write!(f, "L"),
            Direction::Right => write!(f, "R"),
            Direction::Stay => write!(f, "S"),
        }
    }
}

/// A single Turing machine transition.
///
/// A transition is a *morphism* in category 𝒯, representing one step
/// of computation from one configuration to another.
///
/// The functor Z': 𝒯 → ℬ maps this transition to a discrete interval
/// [step, step + 1] in the cobordism category.
#[derive(Clone, Debug)]
pub struct Transition {
    /// The configuration before this transition
    pub from_config: Configuration,
    /// The configuration after this transition
    pub to_config: Configuration,
    /// The step number when this transition occurred (0-indexed)
    pub step: usize,
}

impl Transition {
    /// Create a new transition record.
    #[must_use]
    pub fn new(from_config: Configuration, to_config: Configuration, step: usize) -> Self {
        Self {
            from_config,
            to_config,
            step,
        }
    }

    /// Map this transition to a discrete interval in category ℬ.
    ///
    /// This is the action of the functor Z' on morphisms.
    /// A single elementary transition maps to [step, step + 1].
    #[must_use]
    pub fn to_interval(&self) -> DiscreteInterval {
        DiscreteInterval::new(self.step, self.step + 1)
    }

    /// Get the complexity of this transition.
    ///
    /// An elementary transition always has complexity 1.
    #[must_use]
    pub fn complexity(&self) -> StepCount {
        StepCount(1)
    }

    /// Check if this transition creates a cycle.
    ///
    /// A cycle occurs when the target configuration equals a previous
    /// configuration, indicating the computation will repeat.
    #[must_use]
    pub fn creates_cycle_with(&self, earlier_config: &Configuration) -> bool {
        self.to_config == *earlier_config
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Step {}: {} → {}",
            self.step, self.from_config, self.to_config
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::Tape;

    fn make_config(input: &str, state: u32, head: isize) -> Configuration {
        Configuration::new(Tape::from_input(input, '_'), state, head)
    }

    #[test]
    fn test_direction_delta() {
        assert_eq!(Direction::Left.delta(), -1);
        assert_eq!(Direction::Right.delta(), 1);
        assert_eq!(Direction::Stay.delta(), 0);
    }

    #[test]
    fn test_direction_display() {
        assert_eq!(format!("{}", Direction::Left), "L");
        assert_eq!(format!("{}", Direction::Right), "R");
        assert_eq!(format!("{}", Direction::Stay), "S");
    }

    #[test]
    fn test_transition_new() {
        let from = make_config("ab", 0, 0);
        let to = make_config("ab", 1, 1);
        let t = Transition::new(from.clone(), to.clone(), 5);

        assert_eq!(t.step, 5);
        assert_eq!(t.from_config, from);
        assert_eq!(t.to_config, to);
    }

    #[test]
    fn test_transition_to_interval() {
        let from = make_config("ab", 0, 0);
        let to = make_config("ab", 1, 1);
        let t = Transition::new(from, to, 3);

        let interval = t.to_interval();
        assert_eq!(interval.start, 3);
        assert_eq!(interval.end, 4);
    }

    #[test]
    fn test_transition_complexity() {
        let from = make_config("ab", 0, 0);
        let to = make_config("ab", 1, 1);
        let t = Transition::new(from, to, 0);

        assert_eq!(t.complexity(), StepCount(1));
    }

    #[test]
    fn test_transition_creates_cycle() {
        let config_a = make_config("ab", 0, 0);
        let config_b = make_config("ab", 1, 1);
        let config_a_again = make_config("ab", 0, 0);

        let t = Transition::new(config_b, config_a_again.clone(), 5);
        assert!(t.creates_cycle_with(&config_a));
    }

    #[test]
    fn test_transition_display() {
        let from = make_config("a", 0, 0);
        let to = make_config("b", 1, 0);
        let t = Transition::new(from, to, 7);

        let display = format!("{}", t);
        assert!(display.contains("Step 7"));
    }
}
