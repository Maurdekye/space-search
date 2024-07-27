use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};

use crate::{ExplorationManager, Searchable, StateParentPair};

pub struct Manager<S> {
    explored: HashSet<S>,
    fringe: VecDeque<StateParentPair<S>>,
    parents: Vec<StateParentPair<S>>,

    /// Toggle depth-first searching on. By default, breadth-first search is used.
    /// Enable this flag to perform depth-first search instead.
    pub depth_first: bool,
}

impl<S> ExplorationManager<S> for Manager<S>
where
    S: Searchable + Clone + Eq + Hash,
{
    type YieldResult = Vec<S>;

    type FringeItem = StateParentPair<S>;

    type CurrentStateContext = usize;

    fn initialize(initial_state: S) -> Self {
        let initial_pair = StateParentPair(initial_state.clone(), None);
        Self {
            explored: HashSet::from([initial_state]),
            fringe: VecDeque::from([initial_pair.clone()]),
            parents: vec![initial_pair],
            depth_first: false,
        }
    }

    fn pop_state(&mut self) -> Option<Self::FringeItem> {
        match self.depth_first {
            true => self.fringe.pop_back(),
            false => self.fringe.pop_front(),
        }
    }

    fn prepare_result_from(
        &self,
        StateParentPair(mut state, mut maybe_parent_index): Self::FringeItem,
    ) -> Self::YieldResult {
        let mut result = VecDeque::new();
        while let Some(parent_index) = maybe_parent_index {
            result.push_front(state);
            let StateParentPair(new_state, new_parent_index) = self
                .parents
                .get(parent_index)
                .expect("Parent state will always exist if parent index exists")
                .clone();
            state = new_state;
            maybe_parent_index = new_parent_index;
        }
        result.push_front(state);
        result.into()
    }

    fn valid_state(&mut self, StateParentPair(state, _): &Self::FringeItem) -> bool {
        if !self.explored.contains(state) {
            self.explored.insert(state.clone());
            false
        } else {
            true
        }
    }

    fn place_state(&mut self, item: Self::FringeItem) {
        self.fringe.push_back(item);
    }

    fn register_current_state(&mut self, item: &Self::FringeItem) -> Self::CurrentStateContext {
        self.parents.push(item.clone());
        return self.parents.len() - 1;
    }

    fn prepare_state(&self, context: &Self::CurrentStateContext, state: S) -> Self::FringeItem {
        StateParentPair(state, Some(*context))
    }
}
