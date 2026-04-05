//! Bi-infinite tape for Turing machines.
//!
//! The [`Tape`] uses a sparse `HashMap<isize, Symbol>` representation: only
//! non-blank cells are stored, so an unbounded tape costs O(k) where k is the
//! number of written cells. Reading an uninitialized position returns the blank
//! symbol; writing the blank symbol removes the position from storage.

use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

/// A symbol on the tape.
pub type Symbol = char;

/// An infinite tape that extends in both directions.
///
/// Uses a sparse representation internally (H`ashMap)` for efficiency,
/// with a default blank symbol for uninitialized positions.
#[derive(Clone, Debug)]
pub struct Tape {
    /// Sparse storage: position -> symbol
    cells: HashMap<isize, Symbol>,
    /// The blank symbol (default for unvisited cells)
    blank: Symbol,
}

impl Tape {
    /// Create a new empty tape with the given blank symbol.
    #[must_use]
    pub fn new(blank: Symbol) -> Self {
        Self {
            cells: HashMap::new(),
            blank,
        }
    }

    /// Create a tape initialized with input starting at position 0.
    ///
    /// The input string is written to positions 0, 1, 2, ...
    #[allow(clippy::cast_possible_wrap)]
    #[must_use]
    pub fn from_input(input: &str, blank: Symbol) -> Self {
        let mut tape = Self::new(blank);
        for (i, ch) in input.chars().enumerate() {
            tape.write(i as isize, ch);
        }
        tape
    }

    /// Read the symbol at a given position.
    ///
    /// Returns the blank symbol for positions that haven't been written.
    #[must_use]
    pub fn read(&self, pos: isize) -> Symbol {
        *self.cells.get(&pos).unwrap_or(&self.blank)
    }

    /// Write a symbol at a given position.
    ///
    /// If the symbol equals the blank, the position is removed from storage
    /// (sparse representation optimization).
    pub fn write(&mut self, pos: isize, symbol: Symbol) {
        if symbol == self.blank {
            self.cells.remove(&pos);
        } else {
            self.cells.insert(pos, symbol);
        }
    }

    /// Get the blank symbol for this tape.
    #[must_use]
    pub fn blank(&self) -> Symbol {
        self.blank
    }

    /// Get the range of non-blank positions (min, max).
    ///
    /// Returns None if the tape is empty.
    ///
    /// # Panics
    ///
    /// Cannot panic: the `unwrap()` calls are guarded by the `is_empty()` check.
    #[must_use]
    pub fn bounds(&self) -> Option<(isize, isize)> {
        if self.cells.is_empty() {
            None
        } else {
            let min = *self.cells.keys().min().unwrap();
            let max = *self.cells.keys().max().unwrap();
            Some((min, max))
        }
    }

    /// Get a string representation of the tape contents.
    ///
    /// Shows the range from min to max non-blank position.
    #[must_use]
    pub fn content_string(&self) -> String {
        match self.bounds() {
            None => String::new(),
            Some((min, max)) => (min..=max).map(|i| self.read(i)).collect(),
        }
    }

    /// Compute a hash fingerprint of the tape contents.
    ///
    /// Used for cycle detection in irreducibility analysis.
    #[must_use]
    pub fn fingerprint(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();

        // Sort keys for deterministic hashing
        let mut positions: Vec<_> = self.cells.keys().copied().collect();
        positions.sort_unstable();

        for pos in positions {
            pos.hash(&mut hasher);
            self.cells[&pos].hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Count the number of non-blank cells.
    #[must_use]
    pub fn non_blank_count(&self) -> usize {
        self.cells.len()
    }
}

impl PartialEq for Tape {
    fn eq(&self, other: &Self) -> bool {
        self.blank == other.blank && self.cells == other.cells
    }
}

impl Eq for Tape {}

impl Hash for Tape {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.blank.hash(state);
        // Sort for deterministic hashing
        let mut items: Vec<_> = self.cells.iter().collect();
        items.sort_by_key(|(k, _)| *k);
        for (pos, sym) in items {
            pos.hash(state);
            sym.hash(state);
        }
    }
}

impl fmt::Display for Tape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.bounds() {
            None => write!(f, "[empty]"),
            Some((min, max)) => {
                write!(f, "[")?;
                for i in min..=max {
                    write!(f, "{}", self.read(i))?;
                }
                write!(f, "]")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tape_new() {
        let tape = Tape::new('_');
        assert_eq!(tape.blank(), '_');
        assert_eq!(tape.read(0), '_');
        assert_eq!(tape.read(-100), '_');
        assert_eq!(tape.read(100), '_');
    }

    #[test]
    fn test_tape_read_write() {
        let mut tape = Tape::new('_');
        tape.write(0, '1');
        tape.write(-1, '0');
        tape.write(5, 'X');

        assert_eq!(tape.read(0), '1');
        assert_eq!(tape.read(-1), '0');
        assert_eq!(tape.read(5), 'X');
        assert_eq!(tape.read(1), '_'); // unwritten
    }

    #[test]
    fn test_tape_from_input() {
        let tape = Tape::from_input("101", '_');
        assert_eq!(tape.read(0), '1');
        assert_eq!(tape.read(1), '0');
        assert_eq!(tape.read(2), '1');
        assert_eq!(tape.read(3), '_');
        assert_eq!(tape.read(-1), '_');
    }

    #[test]
    fn test_tape_sparse_blank() {
        let mut tape = Tape::new('_');
        tape.write(0, '1');
        assert_eq!(tape.non_blank_count(), 1);

        // Writing blank removes the cell
        tape.write(0, '_');
        assert_eq!(tape.non_blank_count(), 0);
    }

    #[test]
    fn test_tape_bounds() {
        let mut tape = Tape::new('_');
        assert!(tape.bounds().is_none());

        tape.write(-5, 'A');
        tape.write(10, 'B');
        assert_eq!(tape.bounds(), Some((-5, 10)));
    }

    #[test]
    fn test_tape_content_string() {
        let tape = Tape::from_input("hello", '_');
        assert_eq!(tape.content_string(), "hello");
    }

    #[test]
    fn test_tape_fingerprint_deterministic() {
        let tape1 = Tape::from_input("abc", '_');
        let tape2 = Tape::from_input("abc", '_');
        assert_eq!(tape1.fingerprint(), tape2.fingerprint());
    }

    #[test]
    fn test_tape_fingerprint_different() {
        let tape1 = Tape::from_input("abc", '_');
        let tape2 = Tape::from_input("abd", '_');
        assert_ne!(tape1.fingerprint(), tape2.fingerprint());
    }

    #[test]
    fn test_tape_equality() {
        let tape1 = Tape::from_input("abc", '_');
        let tape2 = Tape::from_input("abc", '_');
        let tape3 = Tape::from_input("abc", '#');
        assert_eq!(tape1, tape2);
        assert_ne!(tape1, tape3); // different blank
    }

    #[test]
    fn test_tape_display() {
        let tape = Tape::from_input("01", '_');
        assert_eq!(format!("{}", tape), "[01]");
    }
}
