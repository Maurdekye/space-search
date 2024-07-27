use std::{
    collections::{BinaryHeap, HashSet},
    hash::Hash,
};

use crate::{ExplorationManager, NoContext, OrderedSearchable, ScoredSearchable};

pub struct Manager<S>
where
    S: ScoredSearchable,
{
    explored: HashSet<S>,
    fringe: BinaryHeap<OrderedSearchable<S, S::Score>>,
}

impl<S> ExplorationManager<S> for Manager<S>
where
    S: ScoredSearchable + Clone + Hash + Eq,
{
    type YieldResult = S;

    type FringeItem = NoContext<S>;

    type CurrentStateContext = ();

    fn initialize(initial_state: S) -> Self {
        Self {
            explored: HashSet::from([initial_state.clone()]),
            fringe: BinaryHeap::from([initial_state.into()]),
        }
    }

    fn pop_state(&mut self) -> Option<Self::FringeItem> {
        self.fringe.pop().map(|o| NoContext(o.state))
    }

    fn prepare_result_from(&self, NoContext(state): Self::FringeItem) -> Self::YieldResult {
        state
    }

    fn valid_state(&mut self, NoContext(state): &Self::FringeItem) -> bool {
        if !self.explored.contains(state) {
            self.explored.insert(state.clone());
            true
        } else {
            false
        }
    }

    fn place_state(&mut self, NoContext(state): Self::FringeItem) {
        self.fringe.push(state.into())
    }

    fn register_current_state(&mut self, _item: &Self::FringeItem) -> Self::CurrentStateContext {
        ()
    }

    fn prepare_state(&self, _context: &Self::CurrentStateContext, state: S) -> Self::FringeItem {
        NoContext(state)
    }
}
