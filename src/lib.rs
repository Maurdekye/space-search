//! # space-search
//!
//! A library providing basic generic depth-first, breadth-first, and heuristic-guided search space exploration algorithms.
//!
//! Implement [`Searchable`] to perform breadth-first or depth-first searching, and implement [`ScoredSearchable`] to perform heuristically guided search space exploration. Pass them to [`Searcher`] to create an iterator that will search for a solution.
//! 
//! [`Searcher`] requires that you specify a `Manager` type that determines the strategy, return result, and optimization of the search algorithm. Choose one of the searchers defined in the hierarchy of the [`search`] module to fit your individual needs. 
//!
//! * Implement [`ScoredSearchable`] to utilize the `guided` search strategy based managers, which will prioritize searching states with a lower associated score first. If implementing [`ScoredSearchable`] is too complex or unnecessary for your use case, then you may use the `unguided` search managers, which explore the space naively in a depth-first or breadth-first manner, toggleable by a flag on the manager itself.
//! * Implement [`Eq`]` + `[`Hash`]` + `[`Clone`] for your [`Searchable`] type to benefit from prior explored state checking optimization using a `hashable` manager; if youre unable to, then use an `unhashable` manager, which does not require these additional bounds, but will likely explore the space much less efficiently.
//! * Use a `route` based manager to yield results consisting of the sequence of steps taken from the starting state to the ending state. Use a `no_route` manager to just yield the solution state alone.
//! 
//! When implementing [`ScoredSearchable`], make sure that lower scoring states are closer to a solution.
//!
//! ```
//! use space_search::*;
//! use std::{vec, hash::Hash};
//!
//! #[derive(Clone, Debug, PartialEq, Eq, Hash)]
//! struct Pos(i32, i32);
//!
//! impl Searchable for Pos {
//!     type NextStatesIter = vec::IntoIter<Pos>;
//!
//!     fn next_states(&self) -> Self::NextStatesIter {
//!         let &Pos(x, y) = self;
//!         vec![
//!             Pos(x - 1, y),
//!             Pos(x, y - 1),
//!             Pos(x + 1, y),
//!             Pos(x, y + 1),
//!         ].into_iter()
//!     }
//!
//!     fn is_solution(&self) -> bool {
//!         let &Pos(x, y) = self;
//!         x == 5 && y == 5
//!     }
//! }
//!
//! let mut searcher: Searcher<search::unguided::no_route::hashable::Manager<_>, _> = Searcher::new(Pos(0, 0));
//! assert_eq!(searcher.next(), Some(Pos(5, 5)));
//! ```

use std::marker::PhantomData;

pub mod search;

/// Basic trait for depth-first and breadth-first search space exploration.
pub trait Searchable {
    type NextStatesIter: Iterator<Item = Self>;

    /// Yield all adjacent explorable states reachable from this state.
    fn next_states(&self) -> Self::NextStatesIter;

    /// Return `true` if this state is a solution state.
    fn is_solution(&self) -> bool;
}

/// Trait for search space exploration guided by a heuristic.
///
/// New states are explored in the order of
/// lowest-scoring first, biasing the search exploration in the direction of a solution. Ensure the scores
/// returned by [`ScoredSearchable::score`] are decreasing with the proximity to a solution.
pub trait ScoredSearchable: Searchable {
    type Score: Ord;

    /// Score function used for heuristic exploration. New states are explored in the order of
    /// lowest-scoring first; ensure the scores
    /// returned by this function decreate with the proximity to a solution.
    fn score(&self) -> Self::Score;
}

struct OrderedSearchable<T, C> {
    state: T,
    score: C,
}

impl<S> From<S> for OrderedSearchable<S, S::Score>
where
    S: ScoredSearchable,
{
    fn from(state: S) -> Self {
        let score = state.score();
        OrderedSearchable { state, score }
    }
}

impl<S> From<StateParentPair<S>> for OrderedSearchable<StateParentPair<S>, S::Score>
where
    S: ScoredSearchable,
{
    fn from(pair: StateParentPair<S>) -> Self {
        let score = pair.0.score();
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

pub struct NoContext<S>(S);

impl<S> AsRef<S> for NoContext<S> {
    fn as_ref(&self) -> &S {
        &self.0
    }
}

#[derive(Clone)]
pub struct StateParentPair<S>(S, Option<usize>);

impl<S> AsRef<S> for StateParentPair<S> {
    fn as_ref(&self) -> &S {
        &self.0
    }
}

/// Newtype wrapper for f32 that implements Ord using [`f32::total_cmp`].
///
/// You may use this type as the score type when implementing [`ScoredSearchable`],
/// as it requires the trait [`Ord`] to be implemented.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct OrdF32(f32);

impl Eq for OrdF32 {}

impl Ord for OrdF32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl From<OrdF32> for f32 {
    fn from(OrdF32(value): OrdF32) -> Self {
        value
    }
}

impl From<f32> for OrdF32 {
    fn from(value: f32) -> Self {
        OrdF32(value)
    }
}

/// Newtype wrapper for f64 that implements Ord using [`f64::total_cmp`].
///
/// You may use this type as the score type when implementing [`ScoredSearchable`],
/// as it requires the trait [`Ord`] to be implemented.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct OrdF64(f64);

impl Eq for OrdF64 {}

impl Ord for OrdF64 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl From<OrdF64> for f64 {
    fn from(OrdF64(value): OrdF64) -> Self {
        value
    }
}

impl From<f64> for OrdF64 {
    fn from(value: f64) -> Self {
        OrdF64(value)
    }
}

pub trait ExplorationManager<S> {
    type YieldResult;
    type FringeItem: AsRef<S>;
    type CurrentStateContext;

    fn initialize(initial_state: S) -> Self;
    fn pop_state(&mut self) -> Option<Self::FringeItem>;
    fn prepare_result_from(&self, item: Self::FringeItem) -> Self::YieldResult;
    fn valid_state(&mut self, item: &Self::FringeItem) -> bool;
    fn place_state(&mut self, item: Self::FringeItem);
    fn register_current_state(&mut self, item: &Self::FringeItem) -> Self::CurrentStateContext;
    fn prepare_state(&self, context: &Self::CurrentStateContext, state: S) -> Self::FringeItem;
}

/// Optimized breadth-first / depth-first state space exploration iterator.
pub struct Searcher<M, S> {
    pub manager: M,
    _marker: PhantomData<S>,
}

impl<M, S> Searcher<M, S> {
    /// Create a new search iterator from an initial state.
    pub fn new(initial_state: S) -> Self
    where
        M: ExplorationManager<S>,
    {
        Self {
            manager: M::initialize(initial_state),
            _marker: PhantomData,
        }
    }

    /// Create a new search iterator from a default initial state.
    pub fn new_with_default() -> Self
    where
        S: Default,
        M: ExplorationManager<S>,
    {
        Self::new(Default::default())
    }
}

impl<M, S> Iterator for Searcher<M, S>
where
    M: ExplorationManager<S>,
    S: Searchable,
{
    type Item = M::YieldResult;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current_state = self.manager.pop_state()?;

            if current_state.as_ref().is_solution() {
                return Some(self.manager.prepare_result_from(current_state));
            }

            let context = self.manager.register_current_state(&current_state);

            for state in current_state.as_ref().next_states() {
                let new_item = self.manager.prepare_state(&context, state);
                if self.manager.valid_state(&new_item) {
                    self.manager.place_state(new_item);
                }
            }
        }
    }
}
