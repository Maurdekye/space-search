use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};

use crate::{ExplorationManager, NoContext, Searchable};

/// unguided, solution-only yielding, prior state exploration culling search manager.
pub struct Manager<S> {
    explored: HashSet<S>,
    fringe: VecDeque<S>,

    /// Toggle depth-first searching on. By default, breadth-first search is used.
    /// Enable this flag to perform depth-first search instead.
    pub depth_first: bool,
}

impl<S> ExplorationManager<S> for Manager<S>
where
    S: Searchable + Clone + Eq + Hash,
{
    type YieldResult = S;

    type FringeItem = NoContext<S>;

    type CurrentStateContext = ();

    fn initialize(initial_state: S) -> Self {
        Self {
            explored: HashSet::from([initial_state.clone()]),
            fringe: VecDeque::from([initial_state]),
            depth_first: false,
        }
    }

    fn pop_state(&mut self) -> Option<NoContext<S>> {
        let state = match self.depth_first {
            true => self.fringe.pop_back(),
            false => self.fringe.pop_front(),
        };
        state.map(|s| NoContext(s))
    }

    fn prepare_result_from(&self, NoContext(item): Self::FringeItem) -> Self::YieldResult {
        item
    }

    fn valid_state(&mut self, NoContext(state): &Self::FringeItem) -> bool {
        if !self.explored.contains(state) {
            self.explored.insert(state.clone());
            true
        } else {
            false
        }
    }

    fn place_state(&mut self, NoContext(item): Self::FringeItem) {
        self.fringe.push_back(item);
    }

    fn register_current_state(&mut self, _item: &Self::FringeItem) -> Self::CurrentStateContext {
        ()
    }

    fn prepare_state(&self, _context: &Self::CurrentStateContext, state: S) -> Self::FringeItem {
        NoContext(state)
    }
}

#[test]
fn test() {
    use crate::*;
    use std::{hash::Hash, vec};

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct Pos(i32, i32);

    impl Searchable for Pos {
        type NextStatesIter = vec::IntoIter<Pos>;

        fn next_states(&self) -> Self::NextStatesIter {
            let &Pos(x, y) = self;
            vec![Pos(x - 1, y), Pos(x, y - 1), Pos(x + 1, y), Pos(x, y + 1)].into_iter()
        }

        fn is_solution(&self) -> bool {
            let &Pos(x, y) = self;
            x == 5 && y == 5
        }
    }

    let mut searcher: Searcher<Manager<_>, _> =
        Searcher::new(Pos(0, 0));
    assert_eq!(searcher.next(), Some(Pos(5, 5)));
}
