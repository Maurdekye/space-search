//! # space-search
//!
//! A library providing basic generic depth-first, breadth-first, heuristic-guided, and A* based search space exploration algorithms.
//!
//! Implement [`Searchable`] + [`SolutionIdentifiable`] to perform breadth-first or depth-first searching. Implement [`Scoreable`] as well to perform heuristically guided search space exploration. Finally, additionally implement [`CostSearchable`] to perform A* based search exploration. Pass them to [`Searcher`] to create an iterator that will search for a solution.
//!
//! [`Searcher`] requires that you specify a `Manager` type that determines the strategy, return result, and optimization of the search algorithm. Choose one of the searchers defined in the hierarchy of the [`search`] module to fit your individual needs.
//!
//! * Implement [`Scoreable`] to utilize the `guided` search strategy based managers, which will prioritize searching states with a lower associated score first. Additionally, implement [`CostSearchable`] to make use of the A* based search managers in the `a_star` module. If implementing [`Scoreable`] is too complex or unnecessary for your use case, then you may use the `unguided` search managers, which explore the space naively in a depth-first or breadth-first manner, toggleable by a flag on the manager itself.
//! * Use a `route` based manager to yield results consisting of the sequence of steps taken from the starting state to the ending state. Use a `no_route` manager to just yield the solution state alone. Route based managers require that your state type implement [`Clone`].
//! * Implement [`Eq`] + [`std::hash::Hash`] + [`Clone`] for your [`Searchable`] type to benefit from prior explored state checking optimization using a `hashable` manager; if youre unable to, then use an `unhashable` manager, which does not require these additional bounds, but will likely explore the space much less efficiently unless cyclic traversal is not an inherent property of your search space.
//!
//! When implementing [`Scoreable`], make sure that lower scoring states are closer to a solution.
//!
//! ```
//! use space_search::*;
//! use std::{vec, hash::Hash};
//!
//! #[derive(Clone, Debug, PartialEq, Eq, Hash)]
//! struct Pos(i32, i32);
//!
//! impl Searchable for Pos {
//!     fn next_states(&self) -> impl Iterator<Item = Self> {
//!         let &Pos(x, y) = self;
//!         [
//!             Pos(x - 1, y),
//!             Pos(x, y - 1),
//!             Pos(x + 1, y),
//!             Pos(x, y + 1),
//!         ].into_iter()
//!     }
//! }
//!
//! impl SolutionIdentifiable for Pos {
//!     fn is_solution(&self) -> bool {
//!         let &Pos(x, y) = self;
//!         x == 5 && y == 5
//!     }
//! }
//!
//! let mut searcher: Searcher<search::unguided::no_route::hashable::Manager<_>, _> = Searcher::new(Pos(0, 0));
//! assert_eq!(searcher.next(), Some(Pos(5, 5)));
//! ```

use std::collections::VecDeque;

pub mod search;

/// Basic trait for depth-first and breadth-first search space exploration.
///
/// Implement this + [`SolutionIdentifiable`] for your state type to perform search-space exploration.
pub trait Searchable: Sized {
    /// Yield all adjacent explorable states reachable from this state.
    fn next_states(&self) -> impl Iterator<Item = Self>;
}

/// Trait that allows a state to be identified as a solution.
///
/// Implement this + [`Searchable`] for your state type to perform search-space exploration.
pub trait SolutionIdentifiable {
    /// Return `true` if this state is a solution state.
    fn is_solution(&self) -> bool;
}

/// Trait for search space exploration guided by a heuristic.
///
/// Implement this + [`Searchable`] + [`SolutionIdentifiable`] to perform heuristically-guided search-space exploration.
///
/// New states are explored in the order of
/// lowest-scoring first, biasing the search exploration in the direction of a solution. Ensure the scores
/// returned by [`Scoreable::score`] are decreasing with the proximity to a solution.
pub trait Scoreable {
    /// Type used to represent a state's score.
    /// Common types can be [`i32`], [`usize`], an ordered float type, or your own type implementing [`Ord`].
    type Score: Ord;

    /// Score function used for heuristic exploration. New states are explored in the order of
    /// lowest-scoring first; ensure the scores
    /// returned by this function decrease with the proximity to a solution.
    fn score(&self) -> Self::Score;
}

/// Trait for search space exploration guided by a cost function & heuristic.
///
/// Implement this + [`Scoreable`] + [`SolutionIdentifiable`] to perform A* guided search-space exploration.
///
/// [`Searchable`] is automatically implemented if this trait is implemented.
pub trait CostSearchable: Scoreable + Sized {
    /// Yield all adjacent explorable states reachable from this state, paired with the associated cost
    /// of traversing from the current state each new state.
    fn next_states_with_costs(&self) -> impl Iterator<Item = (Self, Self::Score)>;
}

// this doesnt work unfortunately :(
// we can only have one blanket impl or the other

// /// Marker trait to auto-implement [`CostSearchable`] for any type that already implements [`Scoreable`] + [`Searchable`]; the cost
// /// is assumed to be 1 for all states returned by [`Searchable::next_states`].
// pub trait CostSearchableForSearchable {}

// impl<T> CostSearchable for T
// where
//     T: Scoreable + Searchable + CostSearchableForSearchable,
//     T::Score: One,
// {
//     fn next_states_with_costs(&self) -> impl Iterator<Item = (Self, Self::Score)> {
//         self.next_states().map(|s| (s, T::Score::one()))
//     }
// }

impl<T> Searchable for T
where
    T: CostSearchable,
{
    fn next_states(&self) -> impl Iterator<Item = Self> {
        self.next_states_with_costs().map(|(s, _)| s)
    }
}

/// Internal.
///
/// Used to represent states paired with their scores in guided exploration managers.
struct OrderedSearchable<T, C> {
    state: T,
    score: C,
}

impl<S> From<S> for OrderedSearchable<S, S::Score>
where
    S: Scoreable,
{
    fn from(state: S) -> Self {
        let score = state.score();
        OrderedSearchable { state, score }
    }
}

impl<S> From<StateParent<S>> for OrderedSearchable<StateParent<S>, S::Score>
where
    S: Scoreable,
{
    fn from(pair: StateParent<S>) -> Self {
        let score = pair.state.score();
        OrderedSearchable { state: pair, score }
    }
}

impl<T, C> PartialEq for OrderedSearchable<T, C>
where
    C: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl<T, C> Eq for OrderedSearchable<T, C> where C: Eq {}

impl<T, C> PartialOrd for OrderedSearchable<T, C>
where
    C: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.score.partial_cmp(&self.score)
    }
}

impl<T, C> Ord for OrderedSearchable<T, C>
where
    C: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.score.cmp(&self.score)
    }
}

/// Internal.
///
/// Used to represent states with no additional context in solution-only yielding managers.
pub struct NoContext<S>(S);

impl<S> AsRef<S> for NoContext<S> {
    fn as_ref(&self) -> &S {
        &self.0
    }
}

/// Internal.
///
/// Used to represent states with the added context of their parent state
/// in solution-route yielding managers.
#[derive(Clone)]
pub struct StateParent<S> {
    state: S,
    parent: Option<usize>,
}

impl<S> AsRef<S> for StateParent<S> {
    fn as_ref(&self) -> &S {
        &self.state
    }
}

/// Internal.
///
/// Used to represent states with the added context of their
/// cumulative rolling cost for A* based managers.
pub struct StateCumulativeCost<S, C> {
    state: S,
    cumulative_cost: C,
}

impl<S, C> AsRef<S> for StateCumulativeCost<S, C> {
    fn as_ref(&self) -> &S {
        &self.state
    }
}

/// Internal.
///
/// Used to represent states with the added context of their parent state &
/// cumulative rolling cost for A* based solution route-yielding managers.
#[derive(Clone)]
pub struct StateParentCumulativeCost<S, C> {
    state: S,
    parent: Option<usize>,
    cumulative_cost: C,
}

impl<S, C> AsRef<S> for StateParentCumulativeCost<S, C> {
    fn as_ref(&self) -> &S {
        &self.state
    }
}

impl<S, C> From<StateParentCumulativeCost<S, C>> for StateParent<S> {
    fn from(
        StateParentCumulativeCost { state, parent, .. }: StateParentCumulativeCost<S, C>,
    ) -> Self {
        StateParent { state, parent }
    }
}

/// Internal.
///
/// Trait abstracting all exploration managers' common functionality.
pub trait ExplorationManager {
    type State;
    type YieldResult;
    type FringeItem: AsRef<Self::State>;
    type CurrentStateContext;
    type NextStatesIterItem;

    fn initialize(initial_state: Self::State) -> Self;
    fn pop_state(&mut self) -> Option<Self::FringeItem>;
    fn prepare_result_from(&self, item: Self::FringeItem) -> Self::YieldResult;
    fn valid_state(&mut self, item: &Self::FringeItem) -> bool;
    fn place_state(&mut self, item: Self::FringeItem);
    fn register_current_state(&mut self, item: &Self::FringeItem) -> Self::CurrentStateContext;
    fn prepare_state(
        &self,
        context: &Self::CurrentStateContext,
        state: Self::NextStatesIterItem,
    ) -> Self::FringeItem;
    fn next_states_iter(
        current_state: &Self::State,
    ) -> impl Iterator<Item = Self::NextStatesIterItem>;
}

/// State space exploration iterator.
///
/// Create an instance of this to explore a search space.
pub struct Searcher<M> {
    pub manager: M,
}

impl<M> Searcher<M> {
    /// Create a new search iterator from an initial state.
    pub fn new(initial_state: M::State) -> Self
    where
        M: ExplorationManager,
    {
        Self {
            manager: M::initialize(initial_state),
        }
    }

    /// Create a new search iterator from a default initial state.
    pub fn new_with_default() -> Self
    where
        M: ExplorationManager,
        M::State: Default,
    {
        Self::new(Default::default())
    }
}

impl<M> Iterator for Searcher<M>
where
    M: ExplorationManager,
    M::State: SolutionIdentifiable,
{
    type Item = M::YieldResult;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current_state = self.manager.pop_state()?;

            if current_state.as_ref().is_solution() {
                return Some(self.manager.prepare_result_from(current_state));
            }

            let context = self.manager.register_current_state(&current_state);

            for item in M::next_states_iter(current_state.as_ref()) {
                let new_item = self.manager.prepare_state(&context, item);
                if self.manager.valid_state(&new_item) {
                    self.manager.place_state(new_item);
                }
            }
        }
    }
}

fn prepare_result_from_state_parent_map<S>(
    parents: &[StateParent<S>],
    StateParent {
        mut state,
        parent: mut maybe_parent_index,
    }: StateParent<S>,
) -> Vec<S>
where
    S: Clone,
{
    let mut result = VecDeque::new();
    while let Some(parent_index) = maybe_parent_index {
        result.push_front(state);
        let StateParent {
            state: new_state,
            parent: new_parent_index,
        } = parents
            .get(parent_index)
            .expect("Parent state will always exist if parent index exists")
            .clone();
        state = new_state;
        maybe_parent_index = new_parent_index;
    }
    result.push_front(state);
    result.into()
}
