use std::{
    collections::{BinaryHeap, HashSet},
    hash::Hash,
};

use crate::{ExplorationManager, NoContext, OrderedSearchable, Scoreable, Searchable};

/// guided, solution-only yielding, prior state exploration culling search manager.
pub struct Manager<S>
where
    S: Scoreable,
{
    explored: HashSet<S>,
    fringe: BinaryHeap<OrderedSearchable<S, S::Score>>,
}

impl<S> ExplorationManager<S> for Manager<S>
where
    S: Searchable + Scoreable + Clone + Hash + Eq,
{
    type YieldResult = S;

    type FringeItem = NoContext<S>;

    type CurrentStateContext = ();

    type NextStatesIterItem = S;

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

    fn next_states_iter(
        current_state: &S,
    ) -> impl Iterator<Item = Self::NextStatesIterItem> {
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

    impl Scoreable for Pos {
        type Score = i32;

        fn score(&self) -> Self::Score {
            let &Pos(x, y) = self;
            (x - 5).abs() + (y - 5).abs()
        }
    }

    let mut searcher: Searcher<Manager<_>, _> = Searcher::new(Pos(0, 0));
    assert_eq!(searcher.next(), Some(Pos(5, 5)));
}
