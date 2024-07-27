use std::collections::VecDeque;

use crate::{ExplorationManager, Searchable, StateParentPair};

/// unguided, solution-route yielding, unoptimized search manager.
pub struct Manager<S> {
    fringe: VecDeque<StateParentPair<S>>,
    parents: Vec<StateParentPair<S>>,
    pub depth_first: bool,
}

impl<S> ExplorationManager<S> for Manager<S>
where
    S: Searchable + Clone,
{
    type YieldResult = Vec<S>;

    type FringeItem = StateParentPair<S>;

    type CurrentStateContext = usize;

    fn initialize(initial_state: S) -> Self {
        let initial_pair = StateParentPair(initial_state, None);
        Self {
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

    fn valid_state(&mut self, _item: &Self::FringeItem) -> bool {
        true
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

#[test]
fn test() {
    use crate::*;
    use std::vec;

    #[derive(Clone, Debug, PartialEq)]
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