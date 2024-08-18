use std::collections::BinaryHeap;

use crate::{
    prepare_result_from_state_parent_map, ExplorationManager, OrderedSearchable, Scoreable,
    Searchable, StateParent,
};

/// guided, solution-route yielding, unoptimized search manager.
pub struct Manager<S>
where
    S: Scoreable,
{
    fringe: BinaryHeap<OrderedSearchable<StateParent<S>, S::Score>>,
    parents: Vec<StateParent<S>>,
}

impl<S> ExplorationManager for Manager<S>
where
    S: Scoreable + Searchable + Clone,
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
            fringe: BinaryHeap::from([initial_pair.clone().into()]),
            parents: vec![initial_pair],
        }
    }

    fn pop_state(&mut self) -> Option<Self::FringeItem> {
        self.fringe.pop().map(|o| o.state)
    }

    fn prepare_result_from(&self, item: Self::FringeItem) -> Self::YieldResult {
        prepare_result_from_state_parent_map(&self.parents, item)
    }

    fn valid_state(&mut self, _item: &Self::FringeItem) -> bool {
        true
    }

    fn place_state(&mut self, item: Self::FringeItem) {
        self.fringe.push(item.into());
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

    impl Scoreable for Pos {
        type Score = i32;

        fn score(&self) -> Self::Score {
            let &Pos(x, y) = self;
            (x - 5).abs() + (y - 5).abs()
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
