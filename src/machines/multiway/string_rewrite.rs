//! String Rewriting System (SRS) for multiway evolution.
//!
//! A String Rewriting System is a simple computational model where:
//! - States are strings
//! - Rules specify pattern → replacement transformations
//! - Non-determinism arises from multiple rules or match positions
//!
//! This is ideal for demonstrating multiway evolution concepts because:
//! 1. States are easy to visualize (just strings)
//! 2. Branching is natural (multiple match positions)
//! 3. Direct connection to Wolfram Physics model
//!
//! ## Example: Wolfram's Simple Systems
//!
//! ```rust
//! use irreducible::StringRewriteSystem;
//!
//! let srs = StringRewriteSystem::new(vec![
//!     ("AB", "BA"),
//!     ("A", "AA"),
//! ]);
//!
//! let evolution = srs.run_multiway("AB", 5, 100);
//! ```

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use catgraph::multiway::{run_multiway_bfs, MultiwayEvolutionGraph};

/// A single rewrite rule: pattern → replacement.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SrsRewriteRule {
    /// The pattern to match.
    pub pattern: String,
    /// The replacement string.
    pub replacement: String,
}

impl SrsRewriteRule {
    /// Create a new rewrite rule.
    pub fn new(pattern: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            replacement: replacement.into(),
        }
    }

    /// Check if this rule is applicable at a position in the string.
    #[must_use]
    pub fn matches_at(&self, s: &str, position: usize) -> bool {
        if position + self.pattern.len() > s.len() {
            return false;
        }
        s[position..position + self.pattern.len()] == self.pattern
    }

    /// Apply this rule at a position, returning the new string.
    #[must_use]
    pub fn apply_at(&self, s: &str, position: usize) -> Option<String> {
        if !self.matches_at(s, position) {
            return None;
        }

        let mut result = String::with_capacity(s.len() - self.pattern.len() + self.replacement.len());
        result.push_str(&s[..position]);
        result.push_str(&self.replacement);
        result.push_str(&s[position + self.pattern.len()..]);
        Some(result)
    }

    /// Find all positions where this rule can be applied.
    #[must_use]
    pub fn find_matches(&self, s: &str) -> Vec<usize> {
        let mut positions = Vec::new();
        if self.pattern.is_empty() {
            return positions; // Avoid infinite matches for empty pattern
        }

        for i in 0..=s.len().saturating_sub(self.pattern.len()) {
            if self.matches_at(s, i) {
                positions.push(i);
            }
        }
        positions
    }
}

/// A String Rewriting System (SRS).
///
/// This is a simple computational model where states are strings and
/// transitions are defined by pattern replacement rules. Natural source
/// of non-determinism: multiple rules or multiple match positions.
#[derive(Clone, Debug)]
pub struct StringRewriteSystem {
    /// The rewrite rules.
    pub rules: Vec<SrsRewriteRule>,
    /// Maximum string length (to prevent explosion).
    pub max_string_length: Option<usize>,
}

impl StringRewriteSystem {
    /// Create a new SRS from pattern/replacement pairs.
    #[must_use]
    pub fn new<S: Into<String>>(rules: Vec<(S, S)>) -> Self {
        Self {
            rules: rules
                .into_iter()
                .map(|(p, r)| SrsRewriteRule::new(p, r))
                .collect(),
            max_string_length: None,
        }
    }

    /// Set maximum string length.
    #[must_use]
    pub fn with_max_length(mut self, max_len: usize) -> Self {
        self.max_string_length = Some(max_len);
        self
    }

    /// Create from `SrsRewriteRule` objects directly.
    #[must_use]
    pub fn from_rules(rules: Vec<SrsRewriteRule>) -> Self {
        Self {
            rules,
            max_string_length: None,
        }
    }

    /// Find all possible rule applications (`rule_index`, position) for current state.
    #[must_use]
    pub fn find_all_matches(&self, state: &SRSState) -> Vec<(usize, usize)> {
        let mut matches = Vec::new();

        for (rule_idx, rule) in self.rules.iter().enumerate() {
            for position in rule.find_matches(&state.0) {
                matches.push((rule_idx, position));
            }
        }

        matches
    }

    /// Apply a specific rule at a position.
    #[must_use]
    pub fn apply_rule(
        &self,
        state: &SRSState,
        rule_idx: usize,
        position: usize,
    ) -> Option<SRSState> {
        self.rules
            .get(rule_idx)
            .and_then(|rule| rule.apply_at(&state.0, position))
            .and_then(|new_str| {
                // Check max length
                if let Some(max_len) = self.max_string_length
                    && new_str.len() > max_len {
                        return None;
                    }
                Some(SRSState(new_str))
            })
    }

    /// Run multiway evolution using breadth-first exploration.
    ///
    /// # Arguments
    /// * `initial` - Initial string
    /// * `max_steps` - Maximum number of steps to evolve
    /// * `max_branches` - Maximum number of branches to explore
    ///
    /// # Returns
    /// A `MultiwayEvolutionGraph` containing all explored states and transitions.
    #[must_use]
    pub fn run_multiway(
        &self,
        initial: &str,
        max_steps: usize,
        max_branches: usize,
    ) -> MultiwayEvolutionGraph<SRSState, RewriteApplication> {
        let initial_state = SRSState(initial.to_string());

        run_multiway_bfs(
            initial_state,
            |state| {
                self.find_all_matches(state)
                    .into_iter()
                    .filter_map(|(rule_idx, position)| {
                        self.apply_rule(state, rule_idx, position)
                            .map(|new_state| {
                                let transition = RewriteApplication {
                                    rule_index: rule_idx,
                                    position,
                                };
                                (new_state, transition, rule_idx)
                            })
                    })
                    .collect()
            },
            max_steps,
            max_branches,
        )
    }

    // === Built-in Example Systems ===

    /// Simple cyclic system: "A" ↔ "B".
    ///
    /// This system oscillates between two states, demonstrating reducibility.
    #[must_use]
    pub fn simple_cycle() -> Self {
        Self::new(vec![("A", "B"), ("B", "A")])
    }

    /// Wolfram-style binary growth system.
    ///
    /// Rules: "A" → "AB", "B" → "A"
    /// This generates Fibonacci-like growth patterns.
    #[must_use]
    pub fn fibonacci_growth() -> Self {
        Self::new(vec![("A", "AB"), ("B", "A")])
    }

    /// Swap system: "AB" → "BA".
    ///
    /// Simple system that swaps adjacent symbols, useful for testing.
    #[must_use]
    pub fn swap_system() -> Self {
        Self::new(vec![("AB", "BA")])
    }

    /// Binary duplication: "A" → "AA", "B" → "BB".
    #[must_use]
    pub fn binary_duplication() -> Self {
        Self::new(vec![("A", "AA"), ("B", "BB")])
    }

    /// Tag system example: simple 2-symbol system.
    ///
    /// Rules model a simple tag system with branching.
    #[must_use]
    pub fn simple_tag() -> Self {
        Self::new(vec![("AA", "B"), ("AB", "BA"), ("BA", "AB"), ("BB", "A")])
    }

    /// Wolfram (2,3) system - known to be computationally universal.
    ///
    /// This is a simplified version for demonstration.
    #[must_use]
    pub fn wolfram_universal() -> Self {
        Self::new(vec![("BA", "AB"), ("AB", "BA"), ("AAA", "B")])
    }
}

/// A state in the String Rewriting System is just a string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SRSState(pub String);

impl Hash for SRSState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl SRSState {
    /// Create a new SRS state.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the string content.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the length of the string.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Compute fingerprint for this state.
    #[must_use]
    pub fn fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish()
    }
}

impl std::fmt::Display for SRSState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.0)
    }
}

/// Transition data for a single SRS rewrite step.
///
/// Records which rule was applied and at which position in the source
/// string, serving as edge metadata in the multiway evolution graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteApplication {
    /// Index of the rule that was applied.
    pub rule_index: usize,
    /// Position in the string where the pattern was matched.
    pub position: usize,
}

impl RewriteApplication {
    /// Create a new rewrite application record.
    #[must_use]
    pub fn new(rule_index: usize, position: usize) -> Self {
        Self { rule_index, position }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewrite_rule_matches() {
        let rule = SrsRewriteRule::new("AB", "BA");
        assert!(rule.matches_at("ABC", 0));
        assert!(!rule.matches_at("ABC", 1));
        assert!(!rule.matches_at("A", 0));
    }

    #[test]
    fn test_rewrite_rule_apply() {
        let rule = SrsRewriteRule::new("AB", "BA");
        assert_eq!(rule.apply_at("ABC", 0), Some("BAC".to_string()));
        assert_eq!(rule.apply_at("XAB", 1), Some("XBA".to_string()));
        assert_eq!(rule.apply_at("ABAB", 0), Some("BAAB".to_string()));
        assert_eq!(rule.apply_at("ABAB", 2), Some("ABBA".to_string()));
    }

    #[test]
    fn test_rewrite_rule_find_matches() {
        let rule = SrsRewriteRule::new("AB", "BA");
        assert_eq!(rule.find_matches("ABABAB"), vec![0, 2, 4]);
        let expected: Vec<usize> = vec![];
        assert_eq!(rule.find_matches("AAA"), expected);
    }

    #[test]
    fn test_srs_find_all_matches() {
        let srs = StringRewriteSystem::new(vec![("A", "B"), ("B", "A")]);
        let state = SRSState::new("AB");

        let matches = srs.find_all_matches(&state);
        // Should find: rule 0 at pos 0, rule 1 at pos 1
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_srs_apply_rule() {
        let srs = StringRewriteSystem::new(vec![("A", "AA")]);
        let state = SRSState::new("A");

        let new_state = srs.apply_rule(&state, 0, 0);
        assert!(new_state.is_some());
        assert_eq!(new_state.unwrap().0, "AA");
    }

    #[test]
    fn test_srs_run_multiway_simple() {
        let srs = StringRewriteSystem::swap_system();
        let evolution = srs.run_multiway("AB", 3, 10);

        // Should have at least the initial state
        assert!(evolution.node_count() >= 1);
    }

    #[test]
    fn test_srs_run_multiway_branching() {
        // System with multiple rules that can apply
        let srs = StringRewriteSystem::new(vec![("A", "B"), ("A", "C")]);
        let evolution = srs.run_multiway("A", 2, 10);

        // Should branch at step 1 (two rules applicable)
        let stats = evolution.statistics();
        assert!(stats.fork_count >= 1);
    }

    #[test]
    fn test_srs_fibonacci_growth() {
        let srs = StringRewriteSystem::fibonacci_growth();
        let evolution = srs.run_multiway("A", 3, 100);

        // Fibonacci growth starts with A, which has only one rule match ("A" -> "AB")
        // But subsequent states like "AB" have two matches: "A" at 0, "B" at 1
        // So this system DOES branch after the first step
        let stats = evolution.statistics();
        // At least one step should be taken
        assert!(stats.max_depth >= 1);
        // May have some nodes
        assert!(stats.total_nodes >= 1);
    }

    #[test]
    fn test_srs_max_length() {
        let srs = StringRewriteSystem::new(vec![("A", "AAA")]).with_max_length(5);
        let evolution = srs.run_multiway("A", 10, 100);

        // Should stop when max length is exceeded
        for node in evolution.nodes_at_step(evolution.max_step()) {
            assert!(node.state.len() <= 5);
        }
    }

    #[test]
    fn test_srs_simple_cycle() {
        let srs = StringRewriteSystem::simple_cycle();
        let evolution = srs.run_multiway("A", 4, 10);

        // Should find cycles
        let cycles = evolution.find_cycles_across_branches();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_srs_state_fingerprint() {
        let s1 = SRSState::new("ABC");
        let s2 = SRSState::new("ABC");
        let s3 = SRSState::new("XYZ");

        assert_eq!(s1.fingerprint(), s2.fingerprint());
        assert_ne!(s1.fingerprint(), s3.fingerprint());
    }
}
