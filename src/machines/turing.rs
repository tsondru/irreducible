//! Deterministic Turing machine definition and execution.
//!
//! A [`TuringMachine`] is specified by a finite state set, a transition function
//! δ: Q × Σ → Q × Σ × {L, R, S}, and distinguished initial/accept/reject states.
//! Execution produces an [`ExecutionHistory`] whose interval sequence under Z'
//! determines computational irreducibility.
//!
//! Well-known instances: [`TuringMachine::busy_beaver_2_2`] (irreducible, 6 steps),
//! [`TuringMachine::binary_incrementer`], [`TuringMachine::infinite_left_mover`] (reducible).

use super::trace::{self, IrreducibilityTrace};
use super::{BuilderError, Configuration, Direction, State, Symbol, Transition};
use catgraph::interval::DiscreteInterval;
use std::collections::HashMap;

/// Transition function type: (state, symbol) -> (`new_state`, `write_symbol`, direction)
pub type TransitionFn = HashMap<(State, Symbol), (State, Symbol, Direction)>;

/// A deterministic Turing machine.
///
/// The TM is defined by:
/// - A finite set of states
/// - An initial state
/// - Accept and reject states (halting)
/// - A blank symbol
/// - A transition function δ: Q × Σ → Q × Σ × {L, R, S}
#[derive(Clone, Debug)]
pub struct TuringMachine {
    /// All states (for documentation; not strictly needed)
    pub states: Vec<State>,
    /// The initial state
    pub initial_state: State,
    /// Accepting (halting) states
    pub accept_states: Vec<State>,
    /// Rejecting (halting) states
    pub reject_states: Vec<State>,
    /// The blank symbol
    pub blank: Symbol,
    /// The transition function
    pub transitions: TransitionFn,
}

impl TuringMachine {
    /// Create a new Turing machine.
    #[must_use]
    pub fn new(
        states: Vec<State>,
        initial_state: State,
        accept_states: Vec<State>,
        reject_states: Vec<State>,
        blank: Symbol,
        transitions: TransitionFn,
    ) -> Self {
        Self {
            states,
            initial_state,
            accept_states,
            reject_states,
            blank,
            transitions,
        }
    }

    /// Create a builder for constructing a Turing machine.
    #[must_use]
    pub fn builder() -> TuringMachineBuilder {
        TuringMachineBuilder::new()
    }

    /// Create the initial configuration for a given input.
    #[must_use]
    pub fn initial_config(&self, input: &str) -> Configuration {
        Configuration::initial(input, self.initial_state, self.blank)
    }

    /// Check if a state is a halting state (accept or reject).
    #[must_use]
    pub fn is_halting_state(&self, state: State) -> bool {
        self.accept_states.contains(&state) || self.reject_states.contains(&state)
    }

    /// Check if a configuration is in a halting state.
    #[must_use]
    pub fn is_halted(&self, config: &Configuration) -> bool {
        self.is_halting_state(config.state)
    }

    /// Execute a single step from the given configuration.
    ///
    /// Returns `None` if the machine is halted or no transition is defined.
    /// Returns `Some((new_config, transition))` otherwise.
    #[must_use]
    pub fn step(&self, config: &Configuration, step_num: usize) -> Option<(Configuration, Transition)> {
        if self.is_halted(config) {
            return None;
        }

        let current_symbol = config.current_symbol();
        let key = (config.state, current_symbol);

        let (new_state, write_symbol, direction) = self.transitions.get(&key)?;

        // Create new tape with the written symbol
        let mut new_tape = config.tape.clone();
        new_tape.write(config.head, *write_symbol);

        // Create new configuration
        let new_config = Configuration {
            tape: new_tape,
            state: *new_state,
            head: config.head + direction.delta(),
        };

        // Record the transition
        let transition = Transition::new(config.clone(), new_config.clone(), step_num);

        Some((new_config, transition))
    }

    /// Run the machine on input for up to `max_steps`.
    ///
    /// Returns the complete execution history.
    #[must_use]
    pub fn run(&self, input: &str, max_steps: usize) -> ExecutionHistory {
        let initial = self.initial_config(input);
        let mut current = initial.clone();
        let mut transitions = Vec::new();
        let mut halted = false;

        for step in 0..max_steps {
            if let Some((next, transition)) = self.step(&current, step) {
                transitions.push(transition);
                current = next;
            } else {
                halted = true;
                break;
            }
        }

        ExecutionHistory {
            initial,
            transitions,
            final_config: current,
            halted,
        }
    }

    /// Run with a callback for each transition.
    ///
    /// The callback receives the transition and returns `false` to stop early.
    pub fn run_with_callback<F>(
        &self,
        input: &str,
        max_steps: usize,
        mut callback: F,
    ) -> ExecutionHistory
    where
        F: FnMut(&Transition) -> bool,
    {
        let initial = self.initial_config(input);
        let mut current = initial.clone();
        let mut transitions = Vec::new();
        let mut halted = false;

        for step in 0..max_steps {
            if let Some((next, transition)) = self.step(&current, step) {
                let should_continue = callback(&transition);
                transitions.push(transition);
                current = next;
                if !should_continue {
                    break;
                }
            } else {
                halted = true;
                break;
            }
        }

        ExecutionHistory {
            initial,
            transitions,
            final_config: current,
            halted,
        }
    }

    // === Well-known Turing Machines ===

    /// Create the 2-state 2-symbol Busy Beaver.
    ///
    /// This machine produces the maximum number of 1s (4) for a 2-state machine.
    /// It runs for exactly 6 steps before halting.
    /// Known to be computationally irreducible.
    #[must_use]
    pub fn busy_beaver_2_2() -> Self {
        let mut transitions = HashMap::new();

        // State A (0):
        //   0 -> write 1, move R, go to B
        //   1 -> write 1, move L, go to B
        // State B (1):
        //   0 -> write 1, move L, go to A
        //   1 -> write 1, move R, go to HALT (2)

        transitions.insert((0, '0'), (1, '1', Direction::Right)); // A,0 -> B,1,R
        transitions.insert((0, '1'), (1, '1', Direction::Left)); // A,1 -> B,1,L
        transitions.insert((1, '0'), (0, '1', Direction::Left)); // B,0 -> A,1,L
        transitions.insert((1, '1'), (2, '1', Direction::Right)); // B,1 -> HALT,1,R

        Self::new(
            vec![0, 1, 2],
            0,           // initial: A
            vec![2],     // accept: HALT
            vec![],      // no reject states
            '0',         // blank = 0
            transitions,
        )
    }

    /// Create a simple binary incrementer.
    ///
    /// Increments a binary number by 1.
    /// Example: "1011" -> "1100"
    #[must_use]
    pub fn binary_incrementer() -> Self {
        let mut transitions = HashMap::new();

        // State 0: scan right to find end
        transitions.insert((0, '0'), (0, '0', Direction::Right));
        transitions.insert((0, '1'), (0, '1', Direction::Right));
        transitions.insert((0, '_'), (1, '_', Direction::Left));

        // State 1: increment (add 1 with carry)
        transitions.insert((1, '0'), (2, '1', Direction::Left)); // 0 -> 1, done
        transitions.insert((1, '1'), (1, '0', Direction::Left)); // 1 -> 0, carry
        transitions.insert((1, '_'), (2, '1', Direction::Stay)); // prepend 1

        // State 2: halt (accept)

        Self::new(
            vec![0, 1, 2],
            0,
            vec![2],
            vec![],
            '_',
            transitions,
        )
    }

    /// Create a simple left-mover that loops forever.
    ///
    /// This machine demonstrates reducibility: it enters a cycle.
    #[must_use]
    pub fn infinite_left_mover() -> Self {
        let mut transitions = HashMap::new();

        // Always move left and stay in state 0
        transitions.insert((0, '0'), (0, '0', Direction::Left));
        transitions.insert((0, '1'), (0, '1', Direction::Left));
        transitions.insert((0, '_'), (0, '_', Direction::Left));

        Self::new(
            vec![0],
            0,
            vec![],
            vec![],
            '_',
            transitions,
        )
    }
}

/// Builder for constructing Turing machines step-by-step.
///
/// Collects states, transition rules, and symbols incrementally before
/// producing a validated [`TuringMachine`]. Required fields: `initial_state`
/// and `blank` symbol (panics on `build()` if missing).
#[derive(Clone, Debug, Default)]
pub struct TuringMachineBuilder {
    states: Vec<State>,
    initial_state: Option<State>,
    accept_states: Vec<State>,
    reject_states: Vec<State>,
    blank: Option<Symbol>,
    transitions: TransitionFn,
}

impl TuringMachineBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the states.
    #[must_use]
    pub fn states(mut self, states: Vec<State>) -> Self {
        self.states = states;
        self
    }

    /// Set the initial state.
    #[must_use]
    pub fn initial_state(mut self, state: State) -> Self {
        self.initial_state = Some(state);
        self
    }

    /// Set the accept states.
    #[must_use]
    pub fn accept_states(mut self, states: Vec<State>) -> Self {
        self.accept_states = states;
        self
    }

    /// Set the reject states.
    #[must_use]
    pub fn reject_states(mut self, states: Vec<State>) -> Self {
        self.reject_states = states;
        self
    }

    /// Set the blank symbol.
    #[must_use]
    pub fn blank(mut self, symbol: Symbol) -> Self {
        self.blank = Some(symbol);
        self
    }

    /// Add a transition.
    #[must_use]
    pub fn transition(
        mut self,
        from_state: State,
        read_symbol: Symbol,
        to_state: State,
        write_symbol: Symbol,
        direction: Direction,
    ) -> Self {
        self.transitions
            .insert((from_state, read_symbol), (to_state, write_symbol, direction));
        self
    }

    /// Build the Turing machine, returning an error if required fields are missing.
    ///
    /// # Errors
    /// Returns [`BuilderError::MissingInitialState`] or [`BuilderError::MissingBlank`]
    /// if the corresponding field was not set.
    pub fn try_build(self) -> Result<TuringMachine, BuilderError> {
        Ok(TuringMachine::new(
            self.states,
            self.initial_state
                .ok_or(BuilderError::MissingInitialState)?,
            self.accept_states,
            self.reject_states,
            self.blank.ok_or(BuilderError::MissingBlank)?,
            self.transitions,
        ))
    }

    /// Build the Turing machine.
    ///
    /// # Panics
    /// Panics if `initial_state` or blank symbol is not set.
    #[must_use]
    pub fn build(self) -> TuringMachine {
        self.try_build()
            .expect("builder requires initial_state and blank to be set")
    }
}

/// Complete execution history of a Turing machine run.
///
/// Contains all transitions from initial to final configuration,
/// enabling irreducibility analysis.
#[derive(Clone, Debug)]
pub struct ExecutionHistory {
    /// The initial configuration
    pub initial: Configuration,
    /// All transitions that occurred
    pub transitions: Vec<Transition>,
    /// The final configuration
    pub final_config: Configuration,
    /// Whether the machine halted (vs. hit `max_steps`)
    pub halted: bool,
}

impl IrreducibilityTrace for ExecutionHistory {
    fn state_fingerprints(&self) -> Vec<u64> {
        let mut fps = Vec::with_capacity(self.transitions.len() + 1);
        fps.push(self.initial.fingerprint());
        for t in &self.transitions {
            fps.push(t.to_config.fingerprint());
        }
        fps
    }

    fn to_intervals(&self) -> Vec<DiscreteInterval> {
        self.transitions.iter().map(Transition::to_interval).collect()
    }

    fn step_count(&self) -> usize {
        self.transitions.len()
    }

    fn halted(&self) -> bool {
        self.halted
    }
}

impl ExecutionHistory {
    /// Get the number of steps executed.
    #[must_use]
    pub fn step_count(&self) -> usize {
        IrreducibilityTrace::step_count(self)
    }

    /// Convert the execution to a sequence of discrete intervals.
    ///
    /// This is the image of the transitions under the functor Z'.
    #[must_use]
    pub fn to_intervals(&self) -> Vec<DiscreteInterval> {
        IrreducibilityTrace::to_intervals(self)
    }

    /// Get the total interval [0, n] for n steps.
    #[must_use]
    pub fn total_interval(&self) -> Option<DiscreteInterval> {
        if self.transitions.is_empty() {
            None
        } else {
            Some(DiscreteInterval::new(0, self.transitions.len()))
        }
    }

    /// Check if this execution is irreducible.
    ///
    /// An execution is irreducible if:
    /// 1. The sequence of intervals is contiguous (no gaps)
    /// 2. No configuration repeats (no cycles/shortcuts)
    ///
    /// This is the core test of Gorard's functoriality criterion.
    #[must_use]
    pub fn is_irreducible(&self) -> bool {
        let ta = trace::analyze_trace(self);
        ta.is_irreducible
    }

    /// Perform full irreducibility analysis.
    #[must_use]
    pub fn analyze_irreducibility(&self) -> IrreducibilityAnalysis {
        let ta = trace::analyze_trace(self);
        let shortcuts: Vec<Shortcut> = ta
            .repeats
            .iter()
            .map(|r| Shortcut {
                from: r.start_step,
                to: r.end_step,
                cycle_length: r.cycle_length,
            })
            .collect();

        IrreducibilityAnalysis {
            is_irreducible: ta.is_irreducible,
            is_sequence_contiguous: ta.is_sequence_contiguous,
            total_interval: ta.total_interval,
            shortcuts,
            complexity_ratio: ta.complexity_ratio,
            step_count: ta.step_count,
        }
    }

    /// Find shortcuts (repeated configurations) in the execution.
    ///
    /// A shortcut exists when the same configuration appears twice,
    /// meaning the computation could "jump" from the first occurrence
    /// to the second, skipping the intermediate steps.
    #[must_use]
    pub fn find_shortcuts(&self) -> Vec<Shortcut> {
        let fps = IrreducibilityTrace::state_fingerprints(self);
        let repeats = trace::detect_repeats(fps.iter().copied().enumerate());
        repeats
            .into_iter()
            .map(|r| Shortcut {
                from: r.start_step,
                to: r.end_step,
                cycle_length: r.cycle_length,
            })
            .collect()
    }

    /// Get all configurations in order (initial + after each transition).
    #[must_use]
    pub fn all_configurations(&self) -> Vec<&Configuration> {
        let mut configs = vec![&self.initial];
        for t in &self.transitions {
            configs.push(&t.to_config);
        }
        configs
    }
}

/// Result of irreducibility analysis for a Turing machine execution.
///
/// Combines interval contiguity, cycle detection, and complexity ratio
/// into a single verdict. Shortcuts indicate repeated configurations
/// that could be "jumped over", breaking functoriality of Z'.
#[derive(Clone, Debug)]
pub struct IrreducibilityAnalysis {
    /// Whether the computation is fully irreducible
    pub is_irreducible: bool,
    /// Whether the interval sequence is contiguous
    pub is_sequence_contiguous: bool,
    /// The total interval [0, n] if composable
    pub total_interval: Option<DiscreteInterval>,
    /// Any shortcuts (repeated configurations) found
    pub shortcuts: Vec<Shortcut>,
    /// Ratio of actual steps to minimum required
    pub complexity_ratio: f64,
    /// Total number of steps
    pub step_count: usize,
}

impl std::fmt::Display for IrreducibilityAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Irreducibility Analysis:")?;
        writeln!(f, "  Steps: {}", self.step_count)?;
        writeln!(f, "  Is irreducible: {}", self.is_irreducible)?;
        writeln!(f, "  Sequence contiguous: {}", self.is_sequence_contiguous)?;
        if let Some(ref interval) = self.total_interval {
            writeln!(f, "  Total interval: {interval}")?;
        }
        writeln!(f, "  Shortcuts found: {}", self.shortcuts.len())?;
        for shortcut in &self.shortcuts {
            writeln!(f, "    - {shortcut}")?;
        }
        writeln!(f, "  Complexity ratio: {:.3}", self.complexity_ratio)?;
        Ok(())
    }
}

/// A shortcut in the computation (repeated configuration).
///
/// When the same configuration appears at steps `from` and `to`, the
/// intermediate computation is a cycle of length `to - from`. The
/// existence of any shortcut means the computation is reducible.
#[derive(Clone, Debug)]
pub struct Shortcut {
    /// Step number of the first occurrence
    pub from: usize,
    /// Step number of the repeated occurrence
    pub to: usize,
    /// Number of steps in the cycle
    pub cycle_length: usize,
}

impl std::fmt::Display for Shortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cycle from step {} to step {} (length {})",
            self.from, self.to, self.cycle_length
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tm_builder() {
        let tm = TuringMachine::builder()
            .states(vec![0, 1])
            .initial_state(0)
            .accept_states(vec![1])
            .blank('_')
            .transition(0, 'a', 1, 'b', Direction::Right)
            .build();

        assert_eq!(tm.initial_state, 0);
        assert_eq!(tm.blank, '_');
        assert!(tm.accept_states.contains(&1));
    }

    #[test]
    fn test_tm_initial_config() {
        let tm = TuringMachine::builder()
            .initial_state(0)
            .blank('_')
            .build();

        let config = tm.initial_config("abc");
        assert_eq!(config.state, 0);
        assert_eq!(config.head, 0);
        assert_eq!(config.current_symbol(), 'a');
    }

    #[test]
    fn test_tm_single_step() {
        let tm = TuringMachine::builder()
            .states(vec![0, 1])
            .initial_state(0)
            .accept_states(vec![1])
            .blank('_')
            .transition(0, 'a', 1, 'X', Direction::Right)
            .build();

        let config = tm.initial_config("a");
        let result = tm.step(&config, 0);

        assert!(result.is_some());
        let (new_config, transition) = result.unwrap();
        assert_eq!(new_config.state, 1);
        assert_eq!(new_config.head, 1);
        assert_eq!(new_config.tape.read(0), 'X');
        assert_eq!(transition.step, 0);
    }

    #[test]
    fn test_tm_halting() {
        let tm = TuringMachine::builder()
            .states(vec![0, 1])
            .initial_state(0)
            .accept_states(vec![1])
            .blank('_')
            .transition(0, 'a', 1, 'a', Direction::Stay)
            .build();

        let config = tm.initial_config("a");
        let (halted_config, _) = tm.step(&config, 0).unwrap();

        // Should not step further from halting state
        assert!(tm.is_halted(&halted_config));
        assert!(tm.step(&halted_config, 1).is_none());
    }

    #[test]
    fn test_tm_run() {
        let tm = TuringMachine::builder()
            .states(vec![0, 1])
            .initial_state(0)
            .accept_states(vec![1])
            .blank('_')
            .transition(0, 'a', 0, 'a', Direction::Right)
            .transition(0, 'b', 1, 'b', Direction::Stay)
            .build();

        let history = tm.run("aab", 10);
        assert!(history.halted);
        assert_eq!(history.step_count(), 3); // a->a->b->halt
    }

    #[test]
    fn test_busy_beaver_2_2() {
        let bb = TuringMachine::busy_beaver_2_2();
        let history = bb.run("", 20);

        assert!(history.halted);
        assert_eq!(history.step_count(), 6); // Known result for BB(2)

        // The tape should have four 1s
        let ones: usize = history
            .final_config
            .tape
            .content_string()
            .chars()
            .filter(|&c| c == '1')
            .count();
        assert_eq!(ones, 4);
    }

    #[test]
    fn test_busy_beaver_irreducible() {
        let bb = TuringMachine::busy_beaver_2_2();
        let history = bb.run("", 20);

        // Busy Beaver should be irreducible (no shortcuts)
        assert!(history.is_irreducible());

        let analysis = history.analyze_irreducibility();
        assert!(analysis.is_irreducible);
        assert!(analysis.shortcuts.is_empty());
        assert!((analysis.complexity_ratio - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_binary_incrementer() {
        let tm = TuringMachine::binary_incrementer();

        // Test: 1011 + 1 = 1100
        let history = tm.run("1011", 50);
        assert!(history.halted);

        let result = history.final_config.tape.content_string();
        assert_eq!(result, "1100");
    }

    #[test]
    fn test_infinite_left_mover_does_not_halt() {
        let tm = TuringMachine::infinite_left_mover();
        let history = tm.run("", 100);

        assert!(!history.halted);
    }

    #[test]
    fn test_cycling_tm() {
        // A machine that cycles between two states on blank tape
        let tm = TuringMachine::builder()
            .states(vec![0, 1])
            .initial_state(0)
            .blank('_')
            .transition(0, '_', 1, '_', Direction::Stay) // 0 -> 1
            .transition(1, '_', 0, '_', Direction::Stay) // 1 -> 0 (cycle!)
            .build();

        let history = tm.run("", 10);
        assert!(!history.halted);

        // Should detect the cycle
        let shortcuts = history.find_shortcuts();
        assert!(!shortcuts.is_empty());

        let analysis = history.analyze_irreducibility();
        assert!(!analysis.is_irreducible);
    }

    #[test]
    fn test_execution_history_to_intervals() {
        let tm = TuringMachine::builder()
            .states(vec![0, 1])
            .initial_state(0)
            .accept_states(vec![1])
            .blank('_')
            .transition(0, 'a', 0, 'a', Direction::Right)
            .transition(0, '_', 1, '_', Direction::Stay)
            .build();

        let history = tm.run("aaa", 10);
        let intervals = history.to_intervals();

        assert_eq!(intervals.len(), 4); // 3 'a' moves + 1 halt transition
        assert_eq!(intervals[0], DiscreteInterval::new(0, 1));
        assert_eq!(intervals[1], DiscreteInterval::new(1, 2));
        assert_eq!(intervals[2], DiscreteInterval::new(2, 3));
        assert_eq!(intervals[3], DiscreteInterval::new(3, 4));

        // Should be contiguous (irreducible in interval sense)
        assert!(crate::functor::IrreducibilityFunctor::is_sequence_irreducible(&intervals));
    }

    #[test]
    fn test_analysis_display() {
        let bb = TuringMachine::busy_beaver_2_2();
        let history = bb.run("", 20);
        let analysis = history.analyze_irreducibility();

        let display = format!("{}", analysis);
        assert!(display.contains("Steps: 6"));
        assert!(display.contains("Is irreducible: true"));
    }

    #[test]
    fn test_tm_try_build_missing_initial_state() {
        let result = TuringMachineBuilder::new()
            .states(vec![0])
            .blank('_')
            .try_build();
        assert!(matches!(result, Err(BuilderError::MissingInitialState)));
    }

    #[test]
    fn test_tm_try_build_missing_blank() {
        let result = TuringMachineBuilder::new()
            .states(vec![0])
            .initial_state(0)
            .try_build();
        assert!(matches!(result, Err(BuilderError::MissingBlank)));
    }
}
