use std::{collections::BinaryHeap, ops::Add};

use num::Zero;

use crate::{
    prepare_result_from_state_parent_map, CostSearchable, ExplorationManager, OrderedSearchable,
    Scoreable, StateParent, StateParentCumulativeCost,
};

/// A* based, solution-route yielding, unoptimized search manager.
pub struct Manager<S>
where
    S: Scoreable,
{
    fringe: BinaryHeap<OrderedSearchable<StateParentCumulativeCost<S, S::Score>, S::Score>>,
    parents: Vec<StateParent<S>>,
}

impl<S> ExplorationManager for Manager<S>
where
    S: CostSearchable + Clone,
    S::Score: Add<S::Score, Output = S::Score> + Zero + Clone,
{
    type State = S;
    type YieldResult = Vec<S>;

    type FringeItem = StateParentCumulativeCost<S, S::Score>;

    type CurrentStateContext = (usize, S::Score);

    type NextStatesIterItem = (S, S::Score);

    fn initialize(initial_state: S) -> Self {
        let score = initial_state.score();
        let initial_item = StateParentCumulativeCost {
            state: initial_state.clone(),
            parent: None,
            cumulative_cost: S::Score::zero(),
        };
        Self {
            fringe: BinaryHeap::from([OrderedSearchable {
                score,
                state: initial_item.clone(),
            }]),
            parents: vec![initial_item.into()],
        }
    }

    fn pop_state(&mut self) -> Option<Self::FringeItem> {
        self.fringe.pop().map(|s| s.state)
    }

    fn prepare_result_from(&self, item: Self::FringeItem) -> Self::YieldResult {
        prepare_result_from_state_parent_map(&self.parents, item.into())
    }

    fn valid_state(&mut self, _item: &Self::FringeItem) -> bool {
        true
    }

    fn place_state(&mut self, item: Self::FringeItem) {
        let score = item.state.score() + item.cumulative_cost.clone();
        self.fringe.push(OrderedSearchable { state: item, score })
    }

    fn register_current_state(&mut self, item: &Self::FringeItem) -> Self::CurrentStateContext {
        self.parents.push(item.clone().into());
        (self.parents.len() - 1, item.cumulative_cost.clone())
    }

    fn prepare_state(
        &self,
        (parent, cumulative_cost): &Self::CurrentStateContext,
        (state, traversal_cost): Self::NextStatesIterItem,
    ) -> Self::FringeItem {
        StateParentCumulativeCost {
            state,
            parent: Some(*parent),
            cumulative_cost: cumulative_cost.clone() + traversal_cost,
        }
    }

    fn next_states_iter(current_state: &S) -> impl Iterator<Item = Self::NextStatesIterItem> {
        current_state.next_states_with_costs()
    }
}

#[test]
fn test() {
    use crate::*;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct Pos(i32, i32);

    impl CostSearchable for Pos {
        fn next_states_with_costs(&self) -> impl Iterator<Item = (Self, Self::Score)> {
            let &Pos(x, y) = self;
            [Pos(x - 1, y), Pos(x, y - 1), Pos(x + 1, y), Pos(x, y + 1)]
                .into_iter()
                .map(|s| (s, 1))
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
            Pos(1, 1),
            Pos(2, 1),
            Pos(3, 1),
            Pos(3, 2),
            Pos(3, 3),
            Pos(3, 4),
            Pos(4, 4),
            Pos(4, 5),
            Pos(5, 5)
        ])
    );
}
