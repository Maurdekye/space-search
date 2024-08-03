use std::{collections::VecDeque, marker::PhantomData};

use crate::{ExplorationManager, NoContext};

/// unguided, solution-only yielding, unoptimized culling search manager.
pub struct Manager<S, N, NI, IG> {
    fringe: VecDeque<S>,

    /// Toggle depth-first searching on. By default, breadth-first search is used.
    /// Enable this flag to perform depth-first search instead.
    pub depth_first: bool,

    next_states: N,
    is_goal: IG,
    _marker_ni: PhantomData<NI>,
}

pub struct Options<State, NextStates, NextStatesIter, IsGoal>
where
    NextStates: Fn(&State) -> NextStatesIter,
    NextStatesIter: Iterator<Item = State>,
    IsGoal: Fn(&State) -> bool,
{
    start: State,
    next_states: NextStates,
    is_goal: IsGoal,
}

impl<S, N, NI, IG> ExplorationManager<S> for Manager<S, N, NI, IG>
where
    N: Fn(&S) -> NI,
    NI: Iterator<Item = S>,
    IG: Fn(&S) -> bool,
{
    type YieldResult = S;

    type FringeItem = NoContext<S>;

    type CurrentStateContext = ();

    type NextStatesIterItem = S;

    type InitializeOptions = Options<S, N, NI, IG>;

    fn initialize(
        Options {
            start,
            next_states,
            is_goal,
        }: Self::InitializeOptions,
    ) -> Self {
        Self {
            fringe: VecDeque::from([start]),
            depth_first: false,
            next_states,
            is_goal,
            _marker_ni: PhantomData,
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

    fn register_current_state(&mut self, _item: &Self::FringeItem) -> Self::CurrentStateContext {
        ()
    }

    fn prepare_state(&self, _context: &Self::CurrentStateContext, state: S) -> Self::FringeItem {
        NoContext(state)
    }

    fn next_states_iter(
        &self,
        current_state: &S,
    ) -> impl Iterator<Item = Self::NextStatesIterItem> {
        (self.next_states)(current_state)
    }

    fn is_goal(&self, state: &S) -> bool {
        self.is_goal(state)
    }
}

#[test]
fn test() {
    use crate::*;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Pos(i32, i32);

    // impl Searchable for Pos {
    //     fn next_states(&self) -> impl Iterator<Item = Self> {
    //         let &Pos(x, y) = self;
    //         [Pos(x - 1, y), Pos(x, y - 1), Pos(x + 1, y), Pos(x, y + 1)].into_iter()
    //     }
    // }

    // impl SolutionIdentifiable for Pos {
    //     fn is_solution(&self) -> bool {
    //         let &Pos(x, y) = self;
    //         x == 5 && y == 5
    //     }
    // }

    let mut searcher: Searcher<Manager<_, _, _, _>, _> = Searcher::new(Options {
        start: Pos(0, 0),
        next_states: |Pos(x, y)| {
            [
                Pos(x + 1, *y),
                Pos(x - 1, *y),
                Pos(*x, y + 1),
                Pos(*x, y - 1),
            ]
            .into_iter()
        },
        is_goal: |pos| pos == &Pos(5, 5),
    });
    assert_eq!(searcher.next(), Some(Pos(5, 5)));
}
