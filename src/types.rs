//! Core type definitions for computational causality structures.
//!
//! These types represent the fundamental building blocks for causal modeling
//! without being tied to any specific database or storage backend.
//!
//! ## Rich Domain Metadata (Phase 4)
//!
//! The [`ComputationContext`] and [`ComputationDomain`] types enable domain-aware
//! causal graphs where each causaloid node carries information about the specific
//! computational state it represents.
//!
//! ```text
//! Causaloid<f64, f64, (), ComputationContext>
//!                          ↑
//!                          └── Per-node context with:
//!                              - Domain-specific state (TM/CA/Multiway)
//!                              - Step number
//!                              - Complexity estimate
//!                              - Custom metadata
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Rich Domain Metadata (Phase 4)
// ============================================================================

/// Domain-specific state information for computational systems.
///
/// Each variant captures the essential state of a specific computation model
/// at a particular step, enabling domain-aware causal analysis.
///
/// # Example
///
/// ```rust
/// use irreducible::ComputationDomain;
///
/// // Turing machine at state 2, head at position 5
/// let tm_domain = ComputationDomain::TuringMachine { state: 2, head_pos: 5 };
///
/// // Rule 30 cellular automaton with 15 live cells
/// let ca_domain = ComputationDomain::CellularAutomaton { rule: 30, population: 15 };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComputationDomain {
    /// Turing machine state.
    ///
    /// - `state`: Current machine state (0 = halt in many conventions)
    /// - `head_pos`: Head position on the tape (negative for left of origin)
    TuringMachine {
        /// Current state number
        state: u32,
        /// Head position (can be negative)
        head_pos: i64,
    },

    /// 1D Elementary cellular automaton state.
    ///
    /// - `rule`: Wolfram rule number (0-255)
    /// - `population`: Number of live (1) cells
    CellularAutomaton {
        /// Wolfram rule number (0-255)
        rule: u8,
        /// Number of live cells
        population: usize,
    },

    /// 2D Cellular automaton state (Game of Life, etc.).
    ///
    /// - `width`, `height`: Grid dimensions
    /// - `live_cells`: Number of live cells
    /// - `rule_birth`, `rule_survive`: B/S rule encoding
    ///
    /// **Note:** Forward-declared for domain metadata. No corresponding machine
    /// implementation exists yet — see deferred work in CLAUDE.md.
    CellularAutomaton2D {
        /// Grid width
        width: usize,
        /// Grid height
        height: usize,
        /// Number of live cells
        live_cells: usize,
        /// Birth rule as bitfield (bit i = born with i neighbors)
        rule_birth: u16,
        /// Survival rule as bitfield (bit i = survive with i neighbors)
        rule_survive: u16,
    },

    /// Langton's Ant state.
    ///
    /// - `rule`: Rule string (e.g., "RL", "LLRR")
    /// - `direction`: Current facing direction (0=N, 1=E, 2=S, 3=W)
    /// - `position`: (x, y) position on grid
    ///
    /// **Note:** Forward-declared for domain metadata. No corresponding machine
    /// implementation exists yet — see deferred work in CLAUDE.md.
    LangtonsAnt {
        /// Rule string (e.g., "RL" for classic ant)
        rule: String,
        /// Direction: 0=North, 1=East, 2=South, 3=West
        direction: u8,
        /// Position (x, y)
        position: (i64, i64),
    },

    /// Multiway system state.
    ///
    /// - `branch_id`: Unique identifier for this branch
    /// - `depth`: Distance from root in the evolution graph
    /// - `state_hash`: Hash of the current state for quick comparison
    Multiway {
        /// Branch identifier
        branch_id: u32,
        /// Depth in evolution graph
        depth: usize,
        /// State fingerprint for quick comparison
        state_hash: u64,
    },

    /// Non-deterministic Turing machine state.
    ///
    /// Extends `TuringMachine` with branch information.
    NondeterministicTM {
        /// Current state number
        state: u32,
        /// Head position
        head_pos: i64,
        /// Branch this execution path belongs to
        branch_id: u32,
        /// Number of available transitions (branching factor)
        choices: usize,
    },

    /// String rewriting system state.
    StringRewrite {
        /// Current string length
        string_length: usize,
        /// Number of applicable rules at this state
        applicable_rules: usize,
        /// Branch identifier
        branch_id: u32,
    },

    /// Generic/unknown domain (fallback).
    Generic {
        /// Domain name for identification
        name: String,
        /// Arbitrary key-value metadata
        data: HashMap<String, String>,
    },
}

impl Default for ComputationDomain {
    fn default() -> Self {
        Self::Generic {
            name: "unknown".to_string(),
            data: HashMap::new(),
        }
    }
}

impl ComputationDomain {
    /// Get the domain name as a string.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::TuringMachine { .. } => "TuringMachine",
            Self::CellularAutomaton { .. } => "CellularAutomaton",
            Self::CellularAutomaton2D { .. } => "CellularAutomaton2D",
            Self::LangtonsAnt { .. } => "LangtonsAnt",
            Self::Multiway { .. } => "Multiway",
            Self::NondeterministicTM { .. } => "NondeterministicTM",
            Self::StringRewrite { .. } => "StringRewrite",
            Self::Generic { name, .. } => name,
        }
    }

    /// Check if this domain represents a multiway (branching) system.
    #[must_use]
    pub fn is_multiway(&self) -> bool {
        matches!(
            self,
            Self::Multiway { .. } | Self::NondeterministicTM { .. } | Self::StringRewrite { .. }
        )
    }
}

/// Rich context for causaloid nodes in computational systems.
///
/// This struct is used as the `CTX` type parameter in `Causaloid<f64, f64, (), ComputationContext>`,
/// enabling each node in a causal graph to carry domain-specific metadata.
///
/// # Example
///
/// ```rust
/// use irreducible::{ComputationContext, ComputationDomain};
///
/// let ctx = ComputationContext::new(
///     ComputationDomain::TuringMachine { state: 1, head_pos: 3 },
///     5,  // step number
/// );
///
/// assert_eq!(ctx.step, 5);
/// assert!(ctx.domain.name() == "TuringMachine");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComputationContext {
    /// Domain-specific state information.
    pub domain: ComputationDomain,

    /// Step number in the computation (0-indexed).
    pub step: usize,

    /// Estimated complexity at this step (optional).
    ///
    /// For Turing machines: could be tape usage or state × position product.
    /// For cellular automata: could be population or entropy.
    pub complexity_estimated: Option<f64>,

    /// Optional custom metadata for domain-specific extensions.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl ComputationContext {
    /// Create a new computation context.
    #[must_use]
    pub fn new(domain: ComputationDomain, step: usize) -> Self {
        Self {
            domain,
            step,
            complexity_estimated: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a context with complexity estimate.
    #[must_use]
    pub fn with_complexity(domain: ComputationDomain, step: usize, complexity: f64) -> Self {
        Self {
            domain,
            step,
            complexity_estimated: Some(complexity),
            metadata: HashMap::new(),
        }
    }

    /// Add a metadata entry.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set the complexity estimate.
    pub fn set_complexity(&mut self, complexity: f64) {
        self.complexity_estimated = Some(complexity);
    }

    /// Get a metadata value by key.
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(String::as_str)
    }
}

impl Default for ComputationContext {
    fn default() -> Self {
        Self::new(ComputationDomain::default(), 0)
    }
}

/// Causal effect data container for persistence and serialization.
///
/// Represents the result of evaluating a causal function, capturing
/// both the computed value and any side effects (errors, logs).
///
/// This is a serializable representation of causal effects, suitable for
/// storage in databases (e.g., `SurrealDB`). It complements but is distinct
/// from `deep_causality`'s `PropagatingEffect` which is used for computation.
///
/// # Naming Note
///
/// This type is intentionally named `CausalEffect` (not `PropagatingEffect`)
/// to avoid confusion with `deep_causality::PropagatingEffect`, which has
/// a monadic API (`pure()`, `bind()`, `intervene()`) for causal computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEffect<T> {
    /// The computed effect value, if successful
    pub value: Option<T>,
    /// Whether an error occurred during computation
    pub has_error: bool,
    /// Error message if `has_error` is true
    pub error_message: Option<String>,
    /// Log entries from the computation
    pub log_entries: Vec<String>,
}

impl<T> Default for CausalEffect<T> {
    fn default() -> Self {
        Self {
            value: None,
            has_error: false,
            error_message: None,
            log_entries: Vec::new(),
        }
    }
}

impl<T> CausalEffect<T> {
    /// Create a successful effect with a value
    pub fn success(value: T) -> Self {
        Self {
            value: Some(value),
            has_error: false,
            error_message: None,
            log_entries: Vec::new(),
        }
    }

    /// Create a failed effect with an error message
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            value: None,
            has_error: true,
            error_message: Some(message.into()),
            log_entries: Vec::new(),
        }
    }

    /// Add a log entry to this effect
    #[must_use]
    pub fn with_log(mut self, entry: impl Into<String>) -> Self {
        self.log_entries.push(entry.into());
        self
    }

    /// Check if this effect is successful (has value, no error)
    pub fn is_success(&self) -> bool {
        self.value.is_some() && !self.has_error
    }

    /// Map the value if successful, preserving error state
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> CausalEffect<U> {
        CausalEffect {
            value: self.value.map(f),
            has_error: self.has_error,
            error_message: self.error_message,
            log_entries: self.log_entries,
        }
    }
}

impl<T: Serialize> CausalEffect<T> {
    /// Serialize this effect to JSON.
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if serialization fails.
    pub fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

impl<T: for<'de> Deserialize<'de>> CausalEffect<T> {
    /// Deserialize an effect from JSON.
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if deserialization fails.
    pub fn from_json(value: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_effect_success() {
        let effect = CausalEffect::success(42);
        assert!(effect.is_success());
        assert_eq!(effect.value, Some(42));
        assert!(!effect.has_error);
    }

    #[test]
    fn test_causal_effect_error() {
        let effect: CausalEffect<i32> = CausalEffect::error("something went wrong");
        assert!(!effect.is_success());
        assert_eq!(effect.value, None);
        assert!(effect.has_error);
        assert_eq!(effect.error_message, Some("something went wrong".to_string()));
    }

    #[test]
    fn test_causal_effect_map() {
        let effect = CausalEffect::success(10);
        let doubled = effect.map(|x| x * 2);
        assert_eq!(doubled.value, Some(20));
    }

    #[test]
    fn test_causal_effect_with_log() {
        let effect = CausalEffect::success(42)
            .with_log("step 1")
            .with_log("step 2");
        assert_eq!(effect.log_entries.len(), 2);
    }

    // ========================================================================
    // ComputationDomain Tests
    // ========================================================================

    #[test]
    fn test_computation_domain_turing_machine() {
        let domain = ComputationDomain::TuringMachine {
            state: 2,
            head_pos: -5,
        };
        assert_eq!(domain.name(), "TuringMachine");
        assert!(!domain.is_multiway());
    }

    #[test]
    fn test_computation_domain_cellular_automaton() {
        let domain = ComputationDomain::CellularAutomaton {
            rule: 30,
            population: 15,
        };
        assert_eq!(domain.name(), "CellularAutomaton");
        assert!(!domain.is_multiway());
    }

    #[test]
    fn test_computation_domain_cellular_automaton_2d() {
        let domain = ComputationDomain::CellularAutomaton2D {
            width: 100,
            height: 100,
            live_cells: 50,
            rule_birth: 0b0000_1000,   // B3
            rule_survive: 0b0000_1100, // S23
        };
        assert_eq!(domain.name(), "CellularAutomaton2D");
        assert!(!domain.is_multiway());
    }

    #[test]
    fn test_computation_domain_langtons_ant() {
        let domain = ComputationDomain::LangtonsAnt {
            rule: "RL".to_string(),
            direction: 0, // North
            position: (25, 25),
        };
        assert_eq!(domain.name(), "LangtonsAnt");
        assert!(!domain.is_multiway());
    }

    #[test]
    fn test_computation_domain_multiway() {
        let domain = ComputationDomain::Multiway {
            branch_id: 42,
            depth: 5,
            state_hash: 0xDEADBEEF,
        };
        assert_eq!(domain.name(), "Multiway");
        assert!(domain.is_multiway());
    }

    #[test]
    fn test_computation_domain_ntm() {
        let domain = ComputationDomain::NondeterministicTM {
            state: 1,
            head_pos: 3,
            branch_id: 7,
            choices: 3,
        };
        assert_eq!(domain.name(), "NondeterministicTM");
        assert!(domain.is_multiway());
    }

    #[test]
    fn test_computation_domain_string_rewrite() {
        let domain = ComputationDomain::StringRewrite {
            string_length: 10,
            applicable_rules: 2,
            branch_id: 5,
        };
        assert_eq!(domain.name(), "StringRewrite");
        assert!(domain.is_multiway());
    }

    #[test]
    fn test_computation_domain_generic() {
        let mut data = HashMap::new();
        data.insert("key".to_string(), "value".to_string());
        let domain = ComputationDomain::Generic {
            name: "CustomDomain".to_string(),
            data,
        };
        assert_eq!(domain.name(), "CustomDomain");
        assert!(!domain.is_multiway());
    }

    #[test]
    fn test_computation_domain_default() {
        let domain = ComputationDomain::default();
        assert_eq!(domain.name(), "unknown");
        assert!(!domain.is_multiway());
    }

    // ========================================================================
    // ComputationContext Tests
    // ========================================================================

    #[test]
    fn test_computation_context_new() {
        let ctx = ComputationContext::new(
            ComputationDomain::TuringMachine {
                state: 1,
                head_pos: 0,
            },
            5,
        );
        assert_eq!(ctx.step, 5);
        assert_eq!(ctx.domain.name(), "TuringMachine");
        assert!(ctx.complexity_estimated.is_none());
        assert!(ctx.metadata.is_empty());
    }

    #[test]
    fn test_computation_context_with_complexity() {
        let ctx = ComputationContext::with_complexity(
            ComputationDomain::CellularAutomaton {
                rule: 110,
                population: 20,
            },
            10,
            42.5,
        );
        assert_eq!(ctx.step, 10);
        assert_eq!(ctx.complexity_estimated, Some(42.5));
    }

    #[test]
    fn test_computation_context_with_metadata() {
        let ctx = ComputationContext::new(
            ComputationDomain::TuringMachine {
                state: 0,
                head_pos: 0,
            },
            0,
        )
        .with_metadata("tape_symbols", "01")
        .with_metadata("halted", "true");

        assert_eq!(ctx.get_metadata("tape_symbols"), Some("01"));
        assert_eq!(ctx.get_metadata("halted"), Some("true"));
        assert_eq!(ctx.get_metadata("missing"), None);
    }

    #[test]
    fn test_computation_context_set_complexity() {
        let mut ctx = ComputationContext::new(ComputationDomain::default(), 0);
        assert!(ctx.complexity_estimated.is_none());

        ctx.set_complexity(100.0);
        assert_eq!(ctx.complexity_estimated, Some(100.0));
    }

    #[test]
    fn test_computation_context_default() {
        let ctx = ComputationContext::default();
        assert_eq!(ctx.step, 0);
        assert!(ctx.complexity_estimated.is_none());
        assert!(ctx.metadata.is_empty());
    }

    #[test]
    fn test_computation_context_serialization() {
        let ctx = ComputationContext::with_complexity(
            ComputationDomain::TuringMachine {
                state: 2,
                head_pos: 5,
            },
            10,
            25.0,
        )
        .with_metadata("note", "test");

        // Serialize to JSON
        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("TuringMachine"));
        assert!(json.contains("\"step\":10"));

        // Deserialize back
        let ctx2: ComputationContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx, ctx2);
    }
}
