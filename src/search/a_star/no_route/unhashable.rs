use std::{collections::BinaryHeap, ops::Add};

use num::Zero;

use crate::{
    CostSearchable, ExplorationManager, OrderedSearchable, Scoreable, StateCumulativeCost,
};

/// A* based, solution-only yielding, unoptimized search space manager.
pub struct Manager<S>
where
    S: Scoreable,
{
    fringe: BinaryHeap<OrderedSearchable<StateCumulativeCost<S, S::Score>, S::Score>>,
}

impl<S> ExplorationManager<S> for Manager<S>
where
    S: CostSearchable,
    S::Score: Add<S::Score, Output = S::Score> + Zero + Clone,
{
    type YieldResult = S;

    type FringeItem = StateCumulativeCost<S, S::Score>;

    type CurrentStateContext = S::Score;

    type NextStatesIterItem = (S, S::Score);

    fn initialize(initial_state: S) -> Self {
        let score = initial_state.score();
        Self {
            fringe: BinaryHeap::from([OrderedSearchable {
                score,
                state: StateCumulativeCost {
                    state: initial_state,
                    cumulative_cost: S::Score::zero(),
                },
            }]),
        }
    }

    fn pop_state(&mut self) -> Option<Self::FringeItem> {
        self.fringe.pop().map(|s| s.state)
    }

    fn prepare_result_from(&self, item: Self::FringeItem) -> Self::YieldResult {
        item.state
    }

    fn valid_state(&mut self, _item: &Self::FringeItem) -> bool {
        true
    }

    fn place_state(&mut self, item: Self::FringeItem) {
        let score = item.state.score() + item.cumulative_cost.clone();
        self.fringe.push(OrderedSearchable { state: item, score })
    }

    fn register_current_state(&mut self, item: &Self::FringeItem) -> Self::CurrentStateContext {
        item.cumulative_cost.clone()
    }

    fn prepare_state(
        &self,
        cumulative_cost: &Self::CurrentStateContext,
        (state, traversal_cost): Self::NextStatesIterItem,
    ) -> Self::FringeItem {
        StateCumulativeCost {
            state,
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

    impl CostSearchable for Pos {
        fn next_states_with_costs(&self) -> impl Iterator<Item = (Self, Self::Score)> {
            let &Pos(x, y) = self;
            [Pos(x - 1, y), Pos(x, y - 1), Pos(x + 1, y), Pos(x, y + 1)]
                .into_iter()
                .map(|s| (s, 1))
        }
    }

    let mut searcher: Searcher<Manager<_>, _> = Searcher::new(Pos(0, 0));
    assert_eq!(searcher.next(), Some(Pos(5, 5)));
}