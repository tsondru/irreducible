//! Turing machine configuration (instantaneous description).

use super::{State, Symbol, Tape};
use std::fmt;
use std::hash::{Hash, Hasher};

/// An instantaneous description of a Turing machine.
///
/// A configuration captures the complete state of a TM at a moment in time:
/// - The tape contents
/// - The current state
/// - The head position
///
/// Configurations are the *objects* in category 𝒯 (the category of computations).
#[derive(Clone, Debug)]
pub struct Configuration {
    /// The tape contents
    pub tape: Tape,
    /// The current state of the machine
    pub state: State,
    /// The position of the read/write head
    pub head: isize,
}

impl Configuration {
    /// Create a new configuration.
    #[must_use]
    pub fn new(tape: Tape, state: State, head: isize) -> Self {
        Self { tape, state, head }
    }

    /// Create an initial configuration for input on a tape.
    ///
    /// The head starts at position 0, in the given initial state.
    #[must_use]
    pub fn initial(input: &str, initial_state: State, blank: Symbol) -> Self {
        Self {
            tape: Tape::from_input(input, blank),
            state: initial_state,
            head: 0,
        }
    }

    /// Read the symbol under the head.
    #[must_use]
    pub fn current_symbol(&self) -> Symbol {
        self.tape.read(self.head)
    }

    /// Compute a fingerprint hash for this configuration.
    ///
    /// Used for cycle detection: if two configurations have the same
    /// fingerprint, the computation will loop, indicating potential
    /// reducibility (the loop can be "shortcut").
    #[must_use]
    pub fn fingerprint(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// Create a normalized representation for comparison.
    ///
    /// Shifts the tape so the leftmost non-blank is at position 0,
    /// and adjusts the head position accordingly. This allows comparing
    /// configurations that are equivalent up to translation.
    #[must_use]
    pub fn normalized(&self) -> Self {
        match self.tape.bounds() {
            None => {
                // Empty tape: just reset head to 0
                Self {
                    tape: self.tape.clone(),
                    state: self.state,
                    head: 0,
                }
            }
            Some((min, _)) => {
                // Shift everything by -min
                let mut new_tape = Tape::new(self.tape.blank());
                if let Some((min_pos, max_pos)) = self.tape.bounds() {
                    for pos in min_pos..=max_pos {
                        let sym = self.tape.read(pos);
                        if sym != self.tape.blank() {
                            new_tape.write(pos - min, sym);
                        }
                    }
                }
                Self {
                    tape: new_tape,
                    state: self.state,
                    head: self.head - min,
                }
            }
        }
    }
}

impl PartialEq for Configuration {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state && self.head == other.head && self.tape == other.tape
    }
}

impl Eq for Configuration {}

impl Hash for Configuration {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.state.hash(state);
        self.head.hash(state);
        self.tape.hash(state);
    }
}

impl fmt::Display for Configuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show tape with head position marked
        let bounds = self.tape.bounds();

        match bounds {
            None => write!(f, "q{}: [_]^", self.state),
            Some((min, max)) => {
                write!(f, "q{}: ", self.state)?;
                for i in min..=max {
                    if i == self.head {
                        write!(f, "[{}]", self.tape.read(i))?;
                    } else {
                        write!(f, "{}", self.tape.read(i))?;
                    }
                }
                // If head is outside the written area
                if self.head < min || self.head > max {
                    write!(f, " (head at {})", self.head)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_new() {
        let tape = Tape::from_input("abc", '_');
        let config = Configuration::new(tape, 0, 1);
        assert_eq!(config.state, 0);
        assert_eq!(config.head, 1);
        assert_eq!(config.current_symbol(), 'b');
    }

    #[test]
    fn test_configuration_initial() {
        let config = Configuration::initial("101", 5, '_');
        assert_eq!(config.state, 5);
        assert_eq!(config.head, 0);
        assert_eq!(config.current_symbol(), '1');
    }

    #[test]
    fn test_configuration_fingerprint_deterministic() {
        let config1 = Configuration::initial("abc", 0, '_');
        let config2 = Configuration::initial("abc", 0, '_');
        assert_eq!(config1.fingerprint(), config2.fingerprint());
    }

    #[test]
    fn test_configuration_fingerprint_different_state() {
        let config1 = Configuration::initial("abc", 0, '_');
        let config2 = Configuration::initial("abc", 1, '_');
        assert_ne!(config1.fingerprint(), config2.fingerprint());
    }

    #[test]
    fn test_configuration_fingerprint_different_head() {
        let tape1 = Tape::from_input("abc", '_');
        let tape2 = Tape::from_input("abc", '_');
        let config1 = Configuration::new(tape1, 0, 0);
        let config2 = Configuration::new(tape2, 0, 1);
        assert_ne!(config1.fingerprint(), config2.fingerprint());
    }

    #[test]
    fn test_configuration_equality() {
        let config1 = Configuration::initial("abc", 0, '_');
        let config2 = Configuration::initial("abc", 0, '_');
        let config3 = Configuration::initial("abd", 0, '_');
        assert_eq!(config1, config2);
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_configuration_normalized() {
        // Create config with tape starting at position -5
        let mut tape = Tape::new('_');
        tape.write(-5, 'a');
        tape.write(-4, 'b');
        let config = Configuration::new(tape, 0, -5);

        let normalized = config.normalized();
        assert_eq!(normalized.head, 0); // shifted from -5
        assert_eq!(normalized.tape.read(0), 'a');
        assert_eq!(normalized.tape.read(1), 'b');
    }

    #[test]
    fn test_configuration_display() {
        let config = Configuration::initial("ab", 3, '_');
        let display = format!("{}", config);
        assert!(display.contains("q3"));
        assert!(display.contains("[a]")); // head at position 0
    }
}
