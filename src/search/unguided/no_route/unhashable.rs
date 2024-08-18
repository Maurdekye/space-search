use std::collections::VecDeque;

use crate::{ExplorationManager, NoContext, Searchable};

/// unguided, solution-only yielding, unoptimized culling search manager.
pub struct Manager<S> {
    fringe: VecDeque<S>,

    /// Toggle depth-first searching on. By default, breadth-first search is used.
    /// Enable this flag to perform depth-first search instead.
    pub depth_first: bool,
}

impl<S> ExplorationManager for Manager<S>
where
    S: Searchable,
{
    type State = S;
    type YieldResult = S;

    type FringeItem = NoContext<S>;

    type CurrentStateContext = ();

    type NextStatesIterItem = S;

    fn initialize(initial_state: S) -> Self {
        Self {
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

    fn valid_state(&mut self, _item: &Self::FringeItem) -> bool {
        true
    }

    fn place_state(&mut self, NoContext(item): Self::FringeItem) {
        self.fringe.push_back(item);
    }

    fn register_current_state(&mut self, _item: &Self::FringeItem) -> Self::CurrentStateContext {}

    fn prepare_state(&self, _context: &Self::CurrentStateContext, state: S) -> Self::FringeItem {
        NoContext(state)
    }

    fn next_states_iter(current_state: &S) -> impl Iterator<Item = Self::NextStatesIterItem> {
        current_state.next_states()
    }
}

#[test]
fn test() {
    use crate::*;

    #[derive(Clone, Debug, PartialEq)]
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
    assert_eq!(searcher.next(), Some(Pos(5, 5)));
}
