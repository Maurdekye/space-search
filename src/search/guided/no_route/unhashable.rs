use std::collections::BinaryHeap;

use crate::{ExplorationManager, NoContext, OrderedSearchable, ScoredSearchable};

pub struct Manager<S>
where
    S: ScoredSearchable,
{
    fringe: BinaryHeap<OrderedSearchable<S, S::Score>>,
}

impl<S> ExplorationManager<S> for Manager<S>
where
    S: ScoredSearchable,
{
    type YieldResult = S;

    type FringeItem = NoContext<S>;

    type CurrentStateContext = ();

    fn initialize(initial_state: S) -> Self {
        Self {
            fringe: BinaryHeap::from([initial_state.into()]),
        }
    }

    fn pop_state(&mut self) -> Option<NoContext<S>> {
        self.fringe.pop().map(|o| NoContext(o.state))
    }

    fn prepare_result_from(&self, NoContext(state): Self::FringeItem) -> Self::YieldResult {
        state
    }

    fn valid_state(&mut self, _item: &Self::FringeItem) -> bool {
        true
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

#[test]
fn test() {
    use crate::*;
    use std::vec;

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

    impl ScoredSearchable for Pos {
        type Score = i32;

        fn score(&self) -> Self::Score {
            let &Pos(x, y) = self;
            (x - 5).abs() + (y - 5).abs()
        }
    }

    let mut searcher: Searcher<Manager<_>, _> = Searcher::new(Pos(0, 0));
    assert_eq!(searcher.next(), Some(Pos(5, 5)));
}
