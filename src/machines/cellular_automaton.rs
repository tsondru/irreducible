//! Elementary Cellular Automaton implementation for irreducibility analysis.
//!
//! This module implements 1D elementary cellular automata (ECA) with integration
//! into the irreducibility functor framework.
//!
//! ## Wolfram's Elementary Cellular Automata
//!
//! An ECA is defined by:
//! - A 1D grid of cells, each with state 0 or 1
//! - A rule number (0-255) that determines the next state based on 3-cell neighborhoods
//! - Periodic boundary conditions (wrap-around)
//!
//! ## Famous Rules
//!
//! - **Rule 30**: Chaotic, used for random number generation. Believed to be irreducible.
//! - **Rule 110**: Turing complete! Known to be computationally universal.
//! - **Rule 90**: Produces Sierpinski triangle pattern.
//! - **Rule 184**: Models traffic flow, particle systems.
//!
//! ## Irreducibility in CAs
//!
//! A CA evolution is irreducible if:
//! 1. No global state repeats (no cycles in phase space)
//! 2. The sequence of intervals under Z' is contiguous
//!
//! Rule 30 with a single initial cell is conjectured to be irreducible
//! (no known shortcut exists to compute generation n without computing 1..n-1).

use super::trace::{self, IrreducibilityTrace};
use crate::categories::DiscreteInterval;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};

/// A single generation (global state) of the cellular automaton.
///
/// This is an object in category 𝒯.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Generation {
    /// The cell states (true = 1, false = 0)
    cells: Vec<bool>,
    /// The generation number (time step)
    pub step: usize,
}

impl Generation {
    /// Create a new generation.
    #[must_use]
    pub fn new(cells: Vec<bool>, step: usize) -> Self {
        Self { cells, step }
    }

    /// Get the width of the grid.
    #[must_use]
    pub fn width(&self) -> usize {
        self.cells.len()
    }

    /// Get cell state at position (with wrap-around).
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    #[must_use]
    pub fn get(&self, pos: isize) -> bool {
        let width = self.cells.len() as isize;
        let normalized = ((pos % width) + width) % width;
        self.cells[normalized as usize]
    }

    /// Count the number of live (1) cells.
    #[must_use]
    pub fn population(&self) -> usize {
        self.cells.iter().filter(|&&c| c).count()
    }

    /// Compute a fingerprint hash for cycle detection.
    #[must_use]
    pub fn fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.cells.hash(&mut hasher);
        hasher.finish()
    }

    /// Get the cells as a slice.
    #[must_use]
    pub fn cells(&self) -> &[bool] {
        &self.cells
    }

    /// Convert to a string representation.
    #[must_use]
    pub fn to_pattern(&self) -> String {
        self.cells
            .iter()
            .map(|&c| if c { '█' } else { ' ' })
            .collect()
    }

    /// Convert to binary string.
    #[must_use]
    pub fn to_binary(&self) -> String {
        self.cells
            .iter()
            .map(|&c| if c { '1' } else { '0' })
            .collect()
    }
}

impl fmt::Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gen {}: {}", self.step, self.to_pattern())
    }
}

/// A transition from one generation to the next.
///
/// This is a morphism in category 𝒯.
#[derive(Clone, Debug)]
pub struct CATransition {
    /// Generation before the transition
    pub from: Generation,
    /// Generation after the transition
    pub to: Generation,
    /// The step number (same as from.step)
    pub step: usize,
}

impl CATransition {
    /// Map this transition to a discrete interval.
    #[must_use]
    pub fn to_interval(&self) -> DiscreteInterval {
        DiscreteInterval::new(self.step, self.step + 1)
    }
}

/// An elementary cellular automaton (1D, 2-state, 3-neighbor).
#[derive(Clone, Debug)]
pub struct ElementaryCA {
    /// The rule number (0-255)
    pub rule: u8,
    /// Grid width
    pub width: usize,
}

impl ElementaryCA {
    /// Create a new elementary CA with the given rule and width.
    #[must_use]
    pub fn new(rule: u8, width: usize) -> Self {
        Self { rule, width }
    }

    /// Create an initial generation with a single cell in the center.
    #[must_use]
    pub fn single_cell_initial(&self) -> Generation {
        let mut cells = vec![false; self.width];
        cells[self.width / 2] = true;
        Generation::new(cells, 0)
    }

    /// Create an initial generation from a pattern string.
    ///
    /// '1', '#', or '█' → true; others → false
    #[must_use]
    pub fn from_pattern(&self, pattern: &str) -> Generation {
        let cells: Vec<bool> = pattern
            .chars()
            .map(|c| c == '1' || c == '#' || c == '█')
            .collect();

        // Pad or truncate to width
        let mut result = vec![false; self.width];
        let offset = (self.width.saturating_sub(cells.len())) / 2;
        for (i, &c) in cells.iter().take(self.width).enumerate() {
            if offset + i < self.width {
                result[offset + i] = c;
            }
        }

        Generation::new(result, 0)
    }

    /// Create a random initial generation.
    #[must_use]
    pub fn random_initial(&self, seed: u64) -> Generation {
        // Simple PRNG for reproducibility
        let mut state = seed;
        let cells: Vec<bool> = (0..self.width)
            .map(|_| {
                state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                (state >> 63) != 0
            })
            .collect();
        Generation::new(cells, 0)
    }

    /// Apply the rule to a 3-cell neighborhood.
    ///
    /// Neighborhood is encoded as: left*4 + center*2 + right
    fn apply_rule(&self, neighborhood: u8) -> bool {
        // The rule byte encodes all 8 possible outcomes
        // neighborhood 7 (111) → bit 7, neighborhood 0 (000) → bit 0
        ((self.rule >> neighborhood) & 1) != 0
    }

    /// Compute the next generation.
    #[allow(clippy::cast_possible_wrap)]
    #[must_use]
    pub fn step(&self, current: &Generation) -> Generation {
        let cells: Vec<bool> = (0..self.width)
            .map(|i| {
                let left = u8::from(current.get(i as isize - 1));
                let center = u8::from(current.get(i as isize));
                let right = u8::from(current.get(i as isize + 1));
                let neighborhood = (left << 2) | (center << 1) | right;
                self.apply_rule(neighborhood)
            })
            .collect();

        Generation::new(cells, current.step + 1)
    }

    /// Run the CA for a given number of steps.
    #[must_use]
    pub fn run(&self, initial: Generation, steps: usize) -> CAExecutionHistory {
        let mut current = initial.clone();
        let mut transitions = Vec::with_capacity(steps);

        for _ in 0..steps {
            let next = self.step(&current);
            transitions.push(CATransition {
                from: current.clone(),
                to: next.clone(),
                step: current.step,
            });
            current = next;
        }

        CAExecutionHistory {
            rule: self.rule,
            width: self.width,
            initial,
            transitions,
            final_gen: current,
        }
    }

    /// Run until a cycle is detected or `max_steps` reached.
    #[must_use]
    pub fn run_until_cycle(&self, initial: Generation, max_steps: usize) -> CAExecutionHistory {
        use std::collections::HashMap;

        let mut current = initial.clone();
        let mut transitions = Vec::new();
        let mut seen: HashMap<u64, usize> = HashMap::new();
        seen.insert(current.fingerprint(), 0);

        for step in 0..max_steps {
            let next = self.step(&current);
            transitions.push(CATransition {
                from: current.clone(),
                to: next.clone(),
                step,
            });

            let fp = next.fingerprint();
            if seen.contains_key(&fp) {
                // Cycle detected
                current = next;
                break;
            }
            seen.insert(fp, step + 1);
            current = next;
        }

        CAExecutionHistory {
            rule: self.rule,
            width: self.width,
            initial,
            transitions,
            final_gen: current,
        }
    }

    // === Well-known rules ===

    /// Rule 30: Chaotic, used for random number generation.
    #[must_use]
    pub fn rule_30(width: usize) -> Self {
        Self::new(30, width)
    }

    /// Rule 110: Turing complete!
    #[must_use]
    pub fn rule_110(width: usize) -> Self {
        Self::new(110, width)
    }

    /// Rule 90: Produces Sierpinski triangle.
    #[must_use]
    pub fn rule_90(width: usize) -> Self {
        Self::new(90, width)
    }

    /// Rule 184: Traffic flow / particle model.
    #[must_use]
    pub fn rule_184(width: usize) -> Self {
        Self::new(184, width)
    }
}

/// Complete execution history of a cellular automaton run.
///
/// Stores the rule number, grid width, initial and final generations,
/// and all intermediate transitions. Implements [`IrreducibilityTrace`]
/// for generic irreducibility analysis via [`analyze_trace`](super::trace::analyze_trace).
#[derive(Clone, Debug)]
pub struct CAExecutionHistory {
    /// The rule number
    pub rule: u8,
    /// Grid width
    pub width: usize,
    /// Initial generation
    pub initial: Generation,
    /// All transitions
    pub transitions: Vec<CATransition>,
    /// Final generation
    pub final_gen: Generation,
}

impl IrreducibilityTrace for CAExecutionHistory {
    fn state_fingerprints(&self) -> Vec<u64> {
        let mut fps = Vec::with_capacity(self.transitions.len() + 1);
        fps.push(self.initial.fingerprint());
        for t in &self.transitions {
            fps.push(t.to.fingerprint());
        }
        fps
    }

    fn to_intervals(&self) -> Vec<DiscreteInterval> {
        self.transitions.iter().map(CATransition::to_interval).collect()
    }

    fn step_count(&self) -> usize {
        self.transitions.len()
    }

    fn halted(&self) -> bool {
        // CAs don't have an explicit halt state; they always run for the
        // requested number of steps (or until cycle detection stops them).
        false
    }
}

impl CAExecutionHistory {
    /// Get the number of steps executed.
    #[must_use]
    pub fn step_count(&self) -> usize {
        IrreducibilityTrace::step_count(self)
    }

    /// Convert to a sequence of discrete intervals.
    #[must_use]
    pub fn to_intervals(&self) -> Vec<DiscreteInterval> {
        IrreducibilityTrace::to_intervals(self)
    }

    /// Get the total interval.
    #[must_use]
    pub fn total_interval(&self) -> Option<DiscreteInterval> {
        if self.transitions.is_empty() {
            None
        } else {
            Some(DiscreteInterval::new(0, self.transitions.len()))
        }
    }

    /// Check if this execution is irreducible.
    #[must_use]
    pub fn is_irreducible(&self) -> bool {
        let ta = trace::analyze_trace(self);
        ta.is_irreducible
    }

    /// Analyze irreducibility.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    #[must_use]
    pub fn analyze_irreducibility(&self) -> CAIrreducibilityAnalysis {
        let ta = trace::analyze_trace(self);
        let cycles: Vec<CACycle> = ta
            .repeats
            .iter()
            .map(|r| CACycle {
                start_step: r.start_step,
                end_step: r.end_step,
                cycle_length: r.cycle_length,
            })
            .collect();

        // Population statistics
        let populations: Vec<usize> = std::iter::once(&self.initial)
            .chain(self.transitions.iter().map(|t| &t.to))
            .map(Generation::population)
            .collect();

        let avg_population = if populations.is_empty() {
            0.0
        } else {
            populations.iter().sum::<usize>() as f64 / populations.len() as f64
        };

        CAIrreducibilityAnalysis {
            rule: self.rule,
            width: self.width,
            is_irreducible: ta.is_irreducible,
            is_sequence_contiguous: ta.is_sequence_contiguous,
            total_interval: ta.total_interval,
            cycles,
            step_count: ta.step_count,
            initial_population: self.initial.population(),
            final_population: self.final_gen.population(),
            avg_population,
        }
    }

    /// Find cycles (repeated generations) in the execution.
    #[must_use]
    pub fn find_cycles(&self) -> Vec<CACycle> {
        let fps = IrreducibilityTrace::state_fingerprints(self);
        let repeats = trace::detect_repeats(fps.iter().copied().enumerate());
        repeats
            .into_iter()
            .map(|r| CACycle {
                start_step: r.start_step,
                end_step: r.end_step,
                cycle_length: r.cycle_length,
            })
            .collect()
    }

    /// Get all generations (initial + after each transition).
    #[must_use]
    pub fn all_generations(&self) -> Vec<&Generation> {
        std::iter::once(&self.initial)
            .chain(self.transitions.iter().map(|t| &t.to))
            .collect()
    }

    /// Print the evolution pattern.
    pub fn print_evolution(&self) {
        println!("{}", self.initial);
        for t in &self.transitions {
            println!("{}", t.to);
        }
    }
}

/// A cycle detected in the CA evolution (repeated global state).
///
/// When the same generation fingerprint appears at `start_step` and
/// `end_step`, the evolution has entered a periodic orbit with the
/// given `cycle_length`. Any cycle implies reducibility.
#[derive(Clone, Debug)]
pub struct CACycle {
    /// Step where the cycle starts
    pub start_step: usize,
    /// Step where the same state reappears
    pub end_step: usize,
    /// Length of the cycle
    pub cycle_length: usize,
}

impl fmt::Display for CACycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cycle: step {} → step {} (period {})",
            self.start_step, self.end_step, self.cycle_length
        )
    }
}

/// Result of irreducibility analysis for a cellular automaton evolution.
///
/// Extends the generic [`TraceAnalysis`](super::trace::TraceAnalysis) with
/// CA-specific metrics: rule number, grid width, and population statistics
/// (initial, final, average live cell counts).
#[derive(Clone, Debug)]
pub struct CAIrreducibilityAnalysis {
    /// Rule number
    pub rule: u8,
    /// Grid width
    pub width: usize,
    /// Whether the evolution is irreducible
    pub is_irreducible: bool,
    /// Whether intervals are contiguous
    pub is_sequence_contiguous: bool,
    /// Total interval
    pub total_interval: Option<DiscreteInterval>,
    /// Detected cycles
    pub cycles: Vec<CACycle>,
    /// Total steps
    pub step_count: usize,
    /// Initial population
    pub initial_population: usize,
    /// Final population
    pub final_population: usize,
    /// Average population
    pub avg_population: f64,
}

impl fmt::Display for CAIrreducibilityAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CA Irreducibility Analysis (Rule {}):", self.rule)?;
        writeln!(f, "  Grid width: {}", self.width)?;
        writeln!(f, "  Steps: {}", self.step_count)?;
        writeln!(f, "  Is irreducible: {}", self.is_irreducible)?;
        writeln!(f, "  Sequence contiguous: {}", self.is_sequence_contiguous)?;
        if let Some(ref interval) = self.total_interval {
            writeln!(f, "  Total interval: {interval}")?;
        }
        writeln!(f, "  Cycles found: {}", self.cycles.len())?;
        for cycle in &self.cycles {
            writeln!(f, "    - {cycle}")?;
        }
        writeln!(f, "  Population: {} → {} (avg: {:.1})",
            self.initial_population, self.final_population, self.avg_population)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_new() {
        let generation = Generation::new(vec![true, false, true], 5);
        assert_eq!(generation.width(), 3);
        assert_eq!(generation.step, 5);
        assert_eq!(generation.population(), 2);
    }

    #[test]
    fn test_generation_get_wraparound() {
        let generation = Generation::new(vec![true, false, false], 0);
        assert!(generation.get(0));
        assert!(!generation.get(1));
        assert!(!generation.get(2));
        assert!(generation.get(3)); // wraps to 0
        assert!(generation.get(-3)); // wraps to 0
    }

    #[test]
    fn test_generation_fingerprint() {
        let generation1 = Generation::new(vec![true, false, true], 0);
        let generation2 = Generation::new(vec![true, false, true], 5); // different step
        let generation3 = Generation::new(vec![true, true, true], 0);

        // Same cells = same fingerprint (step not included)
        assert_eq!(generation1.fingerprint(), generation2.fingerprint());
        assert_ne!(generation1.fingerprint(), generation3.fingerprint());
    }

    #[test]
    fn test_ca_single_cell_initial() {
        let ca = ElementaryCA::new(30, 5);
        let initial = ca.single_cell_initial();
        assert_eq!(initial.cells(), &[false, false, true, false, false]);
    }

    #[test]
    fn test_ca_rule_30_step() {
        let ca = ElementaryCA::rule_30(5);
        let initial = ca.single_cell_initial();
        let next = ca.step(&initial);

        // Rule 30: 001 → 1, 010 → 1, 100 → 1
        // Initial: 00100
        // After:   01110
        assert_eq!(next.cells(), &[false, true, true, true, false]);
    }

    #[test]
    fn test_ca_run() {
        let ca = ElementaryCA::rule_30(7);
        let initial = ca.single_cell_initial();
        let history = ca.run(initial, 3);

        assert_eq!(history.step_count(), 3);
        assert_eq!(history.transitions.len(), 3);
    }

    #[test]
    fn test_ca_to_intervals() {
        let ca = ElementaryCA::rule_30(5);
        let initial = ca.single_cell_initial();
        let history = ca.run(initial, 5);

        let intervals = history.to_intervals();
        assert_eq!(intervals.len(), 5);
        assert_eq!(intervals[0], DiscreteInterval::new(0, 1));
        assert_eq!(intervals[4], DiscreteInterval::new(4, 5));
    }

    #[test]
    fn test_ca_irreducibility_short_run() {
        let ca = ElementaryCA::rule_30(11);
        let initial = ca.single_cell_initial();
        let history = ca.run(initial, 10);

        // Rule 30 with single cell should be irreducible for short runs
        let analysis = history.analyze_irreducibility();
        assert!(analysis.is_irreducible);
        assert!(analysis.cycles.is_empty());
    }

    #[test]
    fn test_ca_cycle_detection() {
        // Create a simple CA that will cycle quickly
        // Rule 0: all neighborhoods → 0 (everything dies)
        let ca = ElementaryCA::new(0, 5);
        let initial = Generation::new(vec![true, false, true, false, true], 0);
        let history = ca.run(initial, 10);

        // After first step, all cells die. Then it stays dead.
        let analysis = history.analyze_irreducibility();
        assert!(!analysis.is_irreducible); // Should have cycles (all-zero repeats)
        assert!(!analysis.cycles.is_empty());
    }

    #[test]
    fn test_ca_rule_110() {
        let ca = ElementaryCA::rule_110(21);
        let initial = ca.single_cell_initial();
        let history = ca.run(initial, 20);

        let analysis = history.analyze_irreducibility();
        // Rule 110 is Turing complete - likely irreducible for short runs
        println!("{}", analysis);
    }

    #[test]
    fn test_generation_display() {
        let generation = Generation::new(vec![true, false, true, true], 3);
        let s = format!("{}", generation);
        assert!(s.contains("Gen 3"));
    }

    #[test]
    fn test_ca_from_pattern() {
        let ca = ElementaryCA::new(30, 7);
        let generation = ca.from_pattern("101");
        // Pattern centered: 0010100
        assert_eq!(generation.population(), 2);
    }

    #[test]
    fn test_ca_random_initial() {
        let ca = ElementaryCA::new(30, 10);
        let generation1 = ca.random_initial(42);
        let generation2 = ca.random_initial(42);
        let generation3 = ca.random_initial(123);

        assert_eq!(generation1.cells(), generation2.cells()); // Same seed = same result
        assert_ne!(generation1.cells(), generation3.cells()); // Different seeds
    }

    #[test]
    fn test_ca_run_until_cycle() {
        // Rule 0 should quickly reach a fixed point (all zeros)
        let ca = ElementaryCA::new(0, 5);
        let initial = Generation::new(vec![true, true, true, true, true], 0);
        let history = ca.run_until_cycle(initial, 100);

        // Should stop early due to cycle detection
        assert!(history.step_count() < 100);
    }

    #[test]
    fn test_analysis_display() {
        let ca = ElementaryCA::rule_30(11);
        let initial = ca.single_cell_initial();
        let history = ca.run(initial, 5);
        let analysis = history.analyze_irreducibility();

        let s = format!("{}", analysis);
        assert!(s.contains("Rule 30"));
        assert!(s.contains("Is irreducible"));
    }
}
