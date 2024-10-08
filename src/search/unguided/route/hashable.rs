use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};

use crate::{prepare_result_from_state_parent_map, ExplorationManager, Searchable, StateParent};

/// unguided, solution-route yielding, prior state exploration culling search manager.
pub struct Manager<S> {
    explored: HashSet<S>,
    fringe: VecDeque<StateParent<S>>,
    parents: Vec<StateParent<S>>,

    /// Toggle depth-first searching on. By default, breadth-first search is used.
    /// Enable this flag to perform depth-first search instead.
    pub depth_first: bool,
}

impl<S> ExplorationManager for Manager<S>
where
    S: Searchable + Clone + Eq + Hash,
{
    type State = S;
    type YieldResult = Vec<S>;

    type FringeItem = StateParent<S>;

    type CurrentStateContext = usize;

    type NextStatesIterItem = S;

    fn initialize(initial_state: S) -> Self {
        let initial_pair = StateParent {
            state: initial_state.clone(),
            parent: None,
        };
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

    fn prepare_result_from(&self, item: Self::FringeItem) -> Self::YieldResult {
        prepare_result_from_state_parent_map(&self.parents, item)
    }

    fn valid_state(&mut self, StateParent { state, parent: _ }: &Self::FringeItem) -> bool {
        if !self.explored.contains(state) {
            self.explored.insert(state.clone());
            true
        } else {
            false
        }
    }

    fn place_state(&mut self, item: Self::FringeItem) {
        self.fringe.push_back(item);
    }

    fn register_current_state(&mut self, item: &Self::FringeItem) -> Self::CurrentStateContext {
        self.parents.push(item.clone());
        self.parents.len() - 1
    }

    fn prepare_state(&self, context: &Self::CurrentStateContext, state: S) -> Self::FringeItem {
        StateParent {
            state,
            parent: Some(*context),
        }
    }

    fn next_states_iter(current_state: &S) -> impl Iterator<Item = Self::NextStatesIterItem> {
        current_state.next_states()
    }
}

#[test]
fn test() {
    use crate::*;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct Pos(i32, i32);

    impl Searchable for Pos {
        fn next_states(&self) -> impl Iterator<Item = Self> {
            let &Pos(x, y) = self;
            [Pos(x - 1, y), Pos(x, y - 1), Pos(x + 1, y), Pos(x, y + 1)].into_iter()
        }
    }

    impl SolutionIdentifiable for Pos {
        fn is_solution(&self) -> bool {
            let &Pos(x, y) = self;
            x == 5 && y == 5
        }
    }

    let mut searcher: Searcher<Manager<_>> = Searcher::new(Pos(0, 0));
    assert_eq!(
        searcher.next(),
        Some(vec![
            Pos(0, 0),
            Pos(1, 0),
            Pos(2, 0),
            Pos(3, 0),
            Pos(4, 0),
            Pos(5, 0),
            Pos(5, 1),
            Pos(5, 2),
            Pos(5, 3),
            Pos(5, 4),
            Pos(5, 5)
        ])
    );
}
