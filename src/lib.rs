//! # space-search
//!
//! A library providing basic utilities for performing generic depth-first, breadth-first, and heuristic-guided search space exploration algorithms.
//!
//! Implement [`Searchable`] to perform breadth-first or depth-first searching, and implement [`ScoredSearchable`] to perform heuristically guided search space exploration. Pass them to the [`Searcher`] and [`ScoredSearcher`] structs respectively to create iterators that will search the space for a solution.
//!
//! Implement `Eq + Hash + Clone` for your search space state type to benefit from prior explored state checking optimization; if youre unable to, then use the [`UnhashableSearcher`] or [`ScoredUnhashableSearcher`] iterators, which do not require these additional bounds, but will likely explore the space much less efficiently.
//!
//! When implementing [`ScoredSearchable`], make sure that higher scoring states are closer to a solution.
//!
//! ---
//!
//! The rationale behind why all these different state space search iterators are unique structs instead of just different configurations of the same underlying struct is because they each require different, unique type constraints. Forcing all the different types of iterators to conform to the same set of type constraints would be counterproductive, and in my opinion, attempting to consolidate all of them into the same struct while maintaining the current type constraint system would be extremely uninutitive to read and maintain. This added redundancy keeps the codebase relatively simple and easy to interpret. Think of it like a toolkit with a bunch of different sizes of the same tool, as opposed to one tool handle with a swappable head.
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
//! let mut searcher: Searcher<search::unguided::no_route::hashable::Manager, _> = Searcher::new(Pos(0, 0));
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

// intentionally left unimplemented
// pub struct UnhashableRouteSearcher;

/// Trait for search space exploration guided by a heuristic.
///
/// New states are explored in the order of
/// highest-scoring first, biasing the search exploration in the direction of a solution. Ensure the scores
/// returned by [`ScoredSearchable::score`] are increasing with the proximity to a solution.
pub trait ScoredSearchable: Searchable {
    type Score: Ord;

    /// Score function used for heuristic exploration. New states are explored in the order of
    /// highest-scoring first; ensure the scores
    /// returned by this function increase with the proximity to a solution.
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
        self.score.partial_cmp(&other.score)
    }
}

impl<T, C> Ord for OrderedSearchable<T, C>
where
    C: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
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
