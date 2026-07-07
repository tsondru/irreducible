//! Fluent builder for [`super::PetriNetMachine`].
//!
//! Collects places and transitions incrementally. [`PetriBuilder::try_build`]
//! validates that every arc's place index is within the declared place set
//! and returns [`BuilderError::PetriArcOutOfBounds`] otherwise.

use std::fmt::Debug;

use catgraph_applied::petri_net::{PetriNet, Transition};
use rust_decimal::Decimal;

use crate::machines::BuilderError;

use super::machine::PetriNetMachine;

/// Fluent builder for [`PetriNetMachine`].
///
/// Places are declared in order and referenced by 0-indexed position in the
/// transition `pre` / `post` arc lists.
#[derive(Clone, Debug)]
pub struct PetriBuilder<Lambda> {
    places: Vec<Lambda>,
    transitions: Vec<Transition>,
}

impl<Lambda> Default for PetriBuilder<Lambda> {
    fn default() -> Self {
        Self {
            places: Vec::new(),
            transitions: Vec::new(),
        }
    }
}

impl<Lambda> PetriBuilder<Lambda>
where
    Lambda: Sized + Eq + Copy + Debug,
{
    /// Create a new empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a place with the given Lambda label. Returns the new place index
    /// implicitly via [`PetriNet::place_count`] if needed.
    #[must_use]
    pub fn place(mut self, label: Lambda) -> Self {
        self.places.push(label);
        self
    }

    /// Append a transition. `pre` / `post` are `(place_index, weight)` pairs.
    #[must_use]
    pub fn transition(mut self, pre: Vec<(usize, Decimal)>, post: Vec<(usize, Decimal)>) -> Self {
        self.transitions.push(Transition::new(pre, post));
        self
    }

    /// Build the machine, validating arc bounds.
    ///
    /// # Errors
    ///
    /// Returns [`BuilderError::PetriArcOutOfBounds`] if any arc references a
    /// place index `>= self.places.len()`.
    pub fn try_build(self) -> Result<PetriNetMachine<Lambda>, BuilderError> {
        let place_count = self.places.len();
        for t in &self.transitions {
            for &(p, _) in t.pre() {
                if p >= place_count {
                    return Err(BuilderError::PetriArcOutOfBounds {
                        place: p,
                        place_count,
                    });
                }
            }
            for &(p, _) in t.post() {
                if p >= place_count {
                    return Err(BuilderError::PetriArcOutOfBounds {
                        place: p,
                        place_count,
                    });
                }
            }
        }
        Ok(PetriNetMachine::new(PetriNet::new(
            self.places,
            self.transitions,
        )))
    }

    /// Build the machine, panicking on error.
    ///
    /// # Panics
    ///
    /// Panics if any arc place index is out of bounds. Use
    /// [`Self::try_build`] to recover from this error.
    #[must_use]
    pub fn build(self) -> PetriNetMachine<Lambda> {
        self.try_build()
            .expect("PetriBuilder::build() requires all arc place indices to be in range")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_build_happy_path() {
        let result: Result<PetriNetMachine<char>, _> = PetriBuilder::new()
            .place('A')
            .place('B')
            .transition(vec![(0, Decimal::ONE)], vec![(1, Decimal::ONE)])
            .try_build();
        assert!(result.is_ok());
        let machine = result.unwrap();
        assert_eq!(machine.net().place_count(), 2);
    }

    #[test]
    fn try_build_rejects_out_of_range_pre_arc() {
        let result: Result<PetriNetMachine<char>, _> = PetriBuilder::new()
            .place('A')
            .transition(vec![(5, Decimal::ONE)], vec![])
            .try_build();
        match result {
            Err(BuilderError::PetriArcOutOfBounds { place, place_count }) => {
                assert_eq!(place, 5);
                assert_eq!(place_count, 1);
            }
            other => panic!("expected PetriArcOutOfBounds, got {other:?}"),
        }
    }

    #[test]
    fn try_build_rejects_out_of_range_post_arc() {
        let result: Result<PetriNetMachine<char>, _> = PetriBuilder::new()
            .place('A')
            .transition(vec![], vec![(3, Decimal::ONE)])
            .try_build();
        assert!(matches!(
            result,
            Err(BuilderError::PetriArcOutOfBounds {
                place: 3,
                place_count: 1
            })
        ));
    }
}
