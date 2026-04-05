//! Non-deterministic Turing Machine (NTM) for multiway evolution.
//!
//! A Non-deterministic Turing Machine extends the standard TM by allowing
//! multiple transitions for each (state, symbol) pair. This creates
//! branching computation, ideal for studying multicomputational irreducibility.
//!
//! ## Theory
//!
//! In category 𝒯 with symmetric monoidal structure:
//! - Each fork represents a tensor product of morphisms
//! - The multiway graph captures all parallel branches
//! - Multicomputational irreducibility = Z' is symmetric monoidal functor
//!
//! ## Example
//!
//! ```rust
//! use irreducible::{NondeterministicTM, machines::Direction};
//!
//! let ntm = NondeterministicTM::builder()
//!     .initial_state(0)
//!     .blank('_')
//!     .transition(0, '0', vec![
//!         (1, '0', Direction::Right),
//!         (2, '1', Direction::Left),
//!     ])
//!     .accept_states(vec![1, 2])
//!     .build();
//!
//! let evolution = ntm.run_multiway("0", 10, 100);
//! ```

use std::collections::HashMap;

use crate::machines::{Configuration, Direction, State, Symbol};

use super::evolution_graph::{run_multiway_bfs, MultiwayEvolutionGraph};

/// Non-deterministic transition function type.
///
/// Maps (state, symbol) -> Vec<(`new_state`, `write_symbol`, direction)>
/// Multiple outcomes create branching in the multiway graph.
pub type NTMTransitionFn = HashMap<(State, Symbol), Vec<(State, Symbol, Direction)>>;

/// A Non-deterministic Turing Machine.
///
/// Unlike a standard TM where each (state, symbol) maps to exactly one
/// outcome, an NTM can have multiple possible transitions, creating
/// branching computation.
#[derive(Clone, Debug)]
pub struct NondeterministicTM {
    /// All states.
    pub states: Vec<State>,
    /// The initial state.
    pub initial_state: State,
    /// Accepting (halting) states.
    pub accept_states: Vec<State>,
    /// Rejecting (halting) states.
    pub reject_states: Vec<State>,
    /// The blank symbol.
    pub blank: Symbol,
    /// The non-deterministic transition function.
    pub transitions: NTMTransitionFn,
}

impl NondeterministicTM {
    /// Create a new NTM.
    #[must_use]
    pub fn new(
        states: Vec<State>,
        initial_state: State,
        accept_states: Vec<State>,
        reject_states: Vec<State>,
        blank: Symbol,
        transitions: NTMTransitionFn,
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

    /// Create a builder for constructing an NTM.
    #[must_use]
    pub fn builder() -> NTMBuilder {
        NTMBuilder::new()
    }

    /// Convert from a deterministic `TuringMachine`.
    ///
    /// This wraps each single transition in a Vec.
    #[must_use]
    pub fn from_deterministic(tm: &crate::machines::TuringMachine) -> Self {
        let transitions: NTMTransitionFn = tm
            .transitions
            .iter()
            .map(|((state, symbol), (ns, ws, dir))| ((*state, *symbol), vec![(*ns, *ws, *dir)]))
            .collect();

        Self::new(
            tm.states.clone(),
            tm.initial_state,
            tm.accept_states.clone(),
            tm.reject_states.clone(),
            tm.blank,
            transitions,
        )
    }

    /// Create the initial configuration for a given input.
    #[must_use]
    pub fn initial_config(&self, input: &str) -> Configuration {
        Configuration::initial(input, self.initial_state, self.blank)
    }

    /// Check if a state is a halting state.
    #[must_use]
    pub fn is_halting_state(&self, state: State) -> bool {
        self.accept_states.contains(&state) || self.reject_states.contains(&state)
    }

    /// Check if a configuration is in a halting state.
    #[must_use]
    pub fn is_halted(&self, config: &Configuration) -> bool {
        self.is_halting_state(config.state)
    }

    /// Get all possible next configurations from the current one.
    ///
    /// Returns a Vec of (Configuration, `NTMTransitionData`) for each
    /// possible non-deterministic choice.
    #[must_use]
    pub fn possible_steps(&self, config: &Configuration) -> Vec<(Configuration, NTMTransitionData)> {
        if self.is_halted(config) {
            return Vec::new();
        }

        let current_symbol = config.current_symbol();
        let key = (config.state, current_symbol);

        let Some(transitions) = self.transitions.get(&key) else {
            return Vec::new();
        };

        transitions
            .iter()
            .enumerate()
            .map(|(rule_idx, (new_state, write_symbol, direction))| {
                // Create new tape with the written symbol
                let mut new_tape = config.tape.clone();
                new_tape.write(config.head, *write_symbol);

                // Create new configuration
                let new_config = Configuration {
                    tape: new_tape,
                    state: *new_state,
                    head: config.head + direction.delta(),
                };

                let transition_data = NTMTransitionData {
                    from_state: config.state,
                    read_symbol: current_symbol,
                    to_state: *new_state,
                    write_symbol: *write_symbol,
                    direction: *direction,
                    rule_index: rule_idx,
                };

                (new_config, transition_data)
            })
            .collect()
    }

    /// Run multiway evolution using breadth-first exploration.
    ///
    /// Explores all branches simultaneously, creating a `MultiwayEvolutionGraph`.
    ///
    /// # Arguments
    /// * `input` - Initial tape content
    /// * `max_steps` - Maximum number of steps per branch
    /// * `max_branches` - Maximum total branches to explore
    #[must_use]
    pub fn run_multiway(
        &self,
        input: &str,
        max_steps: usize,
        max_branches: usize,
    ) -> MultiwayEvolutionGraph<Configuration, NTMTransitionData> {
        let initial_config = self.initial_config(input);

        run_multiway_bfs(
            initial_config,
            |config| {
                self.possible_steps(config)
                    .into_iter()
                    .map(|(cfg, data)| {
                        let rule_index = data.rule_index;
                        (cfg, data, rule_index)
                    })
                    .collect()
            },
            max_steps,
            max_branches,
        )
    }

    /// Check if any branch reaches an accept state.
    #[must_use]
    pub fn accepts(&self, input: &str, max_steps: usize, max_branches: usize) -> bool {
        let evolution = self.run_multiway(input, max_steps, max_branches);

        evolution.leaves().iter().any(|&leaf_id| {
            evolution
                .get_node(&leaf_id)
                .is_some_and(|node| self.accept_states.contains(&node.state.state))
        })
    }

    // === Example NTMs ===

    /// Simple branching example: on '0', can go left or right.
    ///
    /// Good for testing basic multiway evolution.
    #[must_use]
    pub fn simple_branching_example() -> Self {
        let mut transitions = NTMTransitionFn::new();

        // State 0 on '0': two choices
        transitions.insert(
            (0, '0'),
            vec![
                (1, '1', Direction::Right), // Branch 1: go right
                (2, '1', Direction::Left),  // Branch 2: go left
            ],
        );

        // States 1 and 2 just move to halt
        transitions.insert((1, '_'), vec![(3, '_', Direction::Stay)]);
        transitions.insert((2, '_'), vec![(3, '_', Direction::Stay)]);

        Self::new(
            vec![0, 1, 2, 3],
            0,
            vec![3],  // accept
            vec![],
            '_',
            transitions,
        )
    }

    /// Three-way branching: on 'A', can choose among three directions.
    #[must_use]
    pub fn three_way_branching() -> Self {
        let mut transitions = NTMTransitionFn::new();

        transitions.insert(
            (0, 'A'),
            vec![
                (1, 'X', Direction::Right),
                (1, 'Y', Direction::Left),
                (1, 'Z', Direction::Stay),
            ],
        );

        transitions.insert((1, '_'), vec![(2, '_', Direction::Stay)]);

        Self::new(
            vec![0, 1, 2],
            0,
            vec![2],
            vec![],
            '_',
            transitions,
        )
    }

    /// Guess-and-verify pattern: NTM for "does the input contain '1'?"
    ///
    /// Non-deterministically guesses a position, verifies it's '1'.
    #[must_use]
    pub fn contains_one() -> Self {
        let mut transitions = NTMTransitionFn::new();

        // State 0: at each position, either check or skip
        transitions.insert(
            (0, '0'),
            vec![
                (0, '0', Direction::Right), // Skip this position
            ],
        );
        transitions.insert(
            (0, '1'),
            vec![
                (0, '1', Direction::Right), // Skip this position
                (1, '1', Direction::Stay),  // Found it! Accept.
            ],
        );
        transitions.insert((0, '_'), vec![(2, '_', Direction::Stay)]); // End of input, reject

        Self::new(
            vec![0, 1, 2],
            0,
            vec![1],  // accept
            vec![2],  // reject
            '_',
            transitions,
        )
    }

    /// Cascading branches: each step can branch into two.
    ///
    /// Creates exponential branching for testing limits.
    #[must_use]
    pub fn exponential_branching() -> Self {
        let mut transitions = NTMTransitionFn::new();

        // State 0: always branch into two
        for sym in ['_', '0', '1'] {
            transitions.insert(
                (0, sym),
                vec![
                    (0, 'A', Direction::Right),
                    (0, 'B', Direction::Right),
                ],
            );
        }

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

/// Builder for constructing non-deterministic Turing machines step-by-step.
///
/// Collects states, non-deterministic transitions, and symbols incrementally.
/// Required fields: `initial_state` and `blank` symbol (panics on `build()`
/// if missing). Supports both non-deterministic and deterministic transitions.
#[derive(Clone, Debug, Default)]
pub struct NTMBuilder {
    states: Vec<State>,
    initial_state: Option<State>,
    accept_states: Vec<State>,
    reject_states: Vec<State>,
    blank: Option<Symbol>,
    transitions: NTMTransitionFn,
}

impl NTMBuilder {
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

    /// Add a non-deterministic transition.
    ///
    /// The `outcomes` Vec contains all possible (state, symbol, direction) results.
    #[must_use]
    pub fn transition(
        mut self,
        from_state: State,
        read_symbol: Symbol,
        outcomes: Vec<(State, Symbol, Direction)>,
    ) -> Self {
        self.transitions.insert((from_state, read_symbol), outcomes);
        self
    }

    /// Add a deterministic transition (single outcome).
    #[must_use]
    pub fn deterministic_transition(
        mut self,
        from_state: State,
        read_symbol: Symbol,
        to_state: State,
        write_symbol: Symbol,
        direction: Direction,
    ) -> Self {
        self.transitions.insert(
            (from_state, read_symbol),
            vec![(to_state, write_symbol, direction)],
        );
        self
    }

    /// Build the NTM.
    ///
    /// # Panics
    ///
    /// Panics if `initial_state` has not been set on the builder.
    #[must_use]
    pub fn build(self) -> NondeterministicTM {
        NondeterministicTM::new(
            self.states,
            self.initial_state.expect("initial_state must be set"),
            self.accept_states,
            self.reject_states,
            self.blank.expect("blank symbol must be set"),
            self.transitions,
        )
    }
}

/// Transition data for NTM edges in the multiway evolution graph.
///
/// Records the full delta: source state/symbol, target state/symbol,
/// head direction, and which non-deterministic choice was taken (`rule_index`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NTMTransitionData {
    /// State before transition.
    pub from_state: State,
    /// Symbol read.
    pub read_symbol: Symbol,
    /// State after transition.
    pub to_state: State,
    /// Symbol written.
    pub write_symbol: Symbol,
    /// Direction moved.
    pub direction: Direction,
    /// Which rule among the non-deterministic choices (0-indexed).
    pub rule_index: usize,
}

impl NTMTransitionData {
    /// Create a new transition data record.
    #[must_use]
    pub fn new(
        from_state: State,
        read_symbol: Symbol,
        to_state: State,
        write_symbol: Symbol,
        direction: Direction,
        rule_index: usize,
    ) -> Self {
        Self {
            from_state,
            read_symbol,
            to_state,
            write_symbol,
            direction,
            rule_index,
        }
    }
}

impl std::fmt::Display for NTMTransitionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}) -> ({}, {}, {:?}) [rule {}]",
            self.from_state,
            self.read_symbol,
            self.to_state,
            self.write_symbol,
            self.direction,
            self.rule_index
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::{Tape, TuringMachine};

    #[test]
    fn test_ntm_builder() {
        let ntm = NondeterministicTM::builder()
            .states(vec![0, 1, 2])
            .initial_state(0)
            .accept_states(vec![2])
            .blank('_')
            .transition(0, 'a', vec![(1, 'b', Direction::Right), (2, 'c', Direction::Left)])
            .build();

        assert_eq!(ntm.initial_state, 0);
        assert_eq!(ntm.blank, '_');
        assert!(ntm.transitions.get(&(0, 'a')).unwrap().len() == 2);
    }

    #[test]
    fn test_ntm_from_deterministic() {
        let tm = TuringMachine::busy_beaver_2_2();
        let ntm = NondeterministicTM::from_deterministic(&tm);

        // Should have same structure but with single-element Vecs
        assert_eq!(ntm.initial_state, tm.initial_state);
        assert_eq!(ntm.blank, tm.blank);

        // Each transition should have exactly one outcome
        for outcomes in ntm.transitions.values() {
            assert_eq!(outcomes.len(), 1);
        }
    }

    #[test]
    fn test_ntm_possible_steps() {
        let ntm = NondeterministicTM::simple_branching_example();
        let config = ntm.initial_config("0");

        let steps = ntm.possible_steps(&config);
        assert_eq!(steps.len(), 2); // Two branches
    }

    #[test]
    fn test_ntm_run_multiway_simple() {
        let ntm = NondeterministicTM::simple_branching_example();
        let evolution = ntm.run_multiway("0", 10, 100);

        // Should have root + branches
        let stats = evolution.statistics();
        assert!(stats.total_nodes >= 3);
        assert!(stats.fork_count >= 1);
    }

    #[test]
    fn test_ntm_run_multiway_deterministic() {
        // When converted from deterministic, should not fork
        let tm = TuringMachine::busy_beaver_2_2();
        let ntm = NondeterministicTM::from_deterministic(&tm);
        let evolution = ntm.run_multiway("", 20, 100);

        let stats = evolution.statistics();
        assert_eq!(stats.fork_count, 0); // No branching
        assert_eq!(stats.max_depth, 6);  // BB(2) takes 6 steps
    }

    #[test]
    fn test_ntm_three_way_branching() {
        let ntm = NondeterministicTM::three_way_branching();
        let evolution = ntm.run_multiway("A", 5, 100);

        let stats = evolution.statistics();
        assert!(stats.fork_count >= 1);
        // After first step, should have 3 branches
        let step1_nodes = evolution.nodes_at_step(1);
        assert_eq!(step1_nodes.len(), 3);
    }

    #[test]
    fn test_ntm_accepts() {
        let ntm = NondeterministicTM::contains_one();

        // Should accept strings containing '1'
        assert!(ntm.accepts("001", 10, 100));
        assert!(ntm.accepts("1", 10, 100));
        assert!(ntm.accepts("111", 10, 100));

        // Should reject strings without '1'
        assert!(!ntm.accepts("000", 10, 100));
        assert!(!ntm.accepts("", 10, 100));
    }

    #[test]
    fn test_ntm_max_branches_limit() {
        let ntm = NondeterministicTM::exponential_branching();
        let evolution = ntm.run_multiway("", 5, 20);

        let stats = evolution.statistics();
        // Should have some nodes (initial + some branches)
        assert!(stats.total_nodes >= 1);
        // Leaf count is bounded by our exploration limit
        assert!(stats.leaf_count <= 50);
    }

    #[test]
    fn test_ntm_transition_data_display() {
        let data = NTMTransitionData::new(0, 'a', 1, 'b', Direction::Right, 0);
        let display = format!("{}", data);
        assert!(display.contains("(0, a)"));
        assert!(display.contains("(1, b, Right)"));
    }

    #[test]
    fn test_ntm_halting() {
        let ntm = NondeterministicTM::simple_branching_example();
        let initial = ntm.initial_config("0");

        assert!(!ntm.is_halted(&initial));

        // After reaching state 3, should be halted
        let halted_config = Configuration {
            tape: Tape::new('_'),
            state: 3,
            head: 0,
        };
        assert!(ntm.is_halted(&halted_config));
    }
}
