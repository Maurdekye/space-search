//! # space-search
//!
//! A library providing basic utilities for performing generic depth-first, breadth-first, and heuristic-guided search space exploration algorithms.
//!
//! Implement [`Searchable`] to perform breadth-first or depth-first searching, and implement [`ScoredSearchable`] to perform heuristically guided search space exploration. Pass them to the [`Searcher`] and [`ScoredSearcher`] structs respectively to create iterators that will search the space for a solution.
//!
//! Implement `Eq + Hash + Clone` for your search space state type to benefit from prior explored state checking optimization; if youre unable to, then use the [`UnhashableSearcher`] or [`ScoredUnhashableSearcher`] iterators, which do not require these additional bounds, but will likely explore the space much less efficiently.
//!
//! When implementing [`ScoredSearcher`], make sure that higher scoring states are closer to a solution.
//!
//! ```
//! use space_search::*;
//! use std::{vec, hash::Hash};
//!
//! #[derive(Clone, Debug, PartialEq, Eq, Hash)]
//! struct Pos(i32, i32);
//!
//! impl Searchable for Pos {
//!     type NextMoveIterator = vec::IntoIter<Pos>;
//!
//!     fn next_states(&self) -> Self::NextMoveIterator {
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
//! let mut searcher = Searcher::new(Pos(0, 0));
//! assert_eq!(searcher.next(), Some(Pos(5, 5)));
//! ```

use std::{
    collections::{BinaryHeap, HashSet, VecDeque},
    hash::Hash,
};

/// Basic trait for depth-first and breadth-first search space exploration.
pub trait Searchable {
    type NextMoveIterator: Iterator<Item = Self>;

    /// Yield all adjacent explorable states reachable from this state.
    fn next_states(&self) -> Self::NextMoveIterator;

    /// Return `true` if this state is a solution state.
    fn is_solution(&self) -> bool;
}

/// Optimized breadth-first / depth-first state space exploration iterator.
pub struct Searcher<S> {
    explored: HashSet<S>,
    fringe: VecDeque<S>,

    /// Toggle depth-first searching on. By default, breadth-first search is used.
    /// Enable this flag to perform depth-first search instead.
    pub depth_first: bool,
}

impl<S> Searcher<S> {
    /// Create a new search iterator from an initial state.
    pub fn new(initial_state: S) -> Self {
        Self {
            explored: HashSet::new(),
            fringe: VecDeque::from([initial_state]),
            depth_first: false,
        }
    }

    /// Create a new search iterator from a default initial state.
    pub fn new_with_default() -> Self
    where
        S: Default,
    {
        Self::new(Default::default())
    }
}

impl<S> Iterator for Searcher<S>
where
    S: Searchable + Clone + Hash + Eq,
{
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current_state = self.fringe.pop_back()?;

            if current_state.is_solution() {
                return Some(current_state);
            }

            for state in current_state.next_states() {
                if !self.explored.contains(&state) {
                    self.explored.insert(state.clone());
                    if self.depth_first {
                        self.fringe.push_back(state);
                    } else {
                        self.fringe.push_front(state);
                    }
                }
            }
        }
    }
}

/// Unoptimized breadth-first / depth-first search space exploration iterator.
/// 
/// Use this instead of [`Searcher`] if implementing `Clone + Eq + Hash` for your [`Searchable`] type
/// is infeasible or impractical for whatever reason, or if you're running into memory
/// limitations from the optimized implementation. 
pub struct UnhashableSearcher<S> {
    fringe: VecDeque<S>,

    /// Toggle depth-first searching on. By default, breadth-first search is used.
    /// Enable this flag to perform depth-first search instead.
    pub depth_first: bool,
}

impl<S> UnhashableSearcher<S> {
    /// Create a new unoptimized iterator from an initial state.
    pub fn new(initial_state: S) -> Self {
        Self {
            fringe: VecDeque::from([initial_state]),
            depth_first: false,
        }
    }

    /// Create a new unoptimized iterator from a default state.
    pub fn new_with_default() -> Self
    where
        S: Default,
    {
        Self::new(Default::default())
    }
}

impl<S> Iterator for UnhashableSearcher<S>
where
    S: Searchable,
{
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current_state = self.fringe.pop_back()?;

            if current_state.is_solution() {
                return Some(current_state);
            }

            for state in current_state.next_states() {
                if self.depth_first {
                    self.fringe.push_back(state);
                } else {
                    self.fringe.push_front(state);
                }
            }
        }
    }
}

/// Trait for search space exploration guided by a heuristic. 
/// 
/// New states are explored in the order of
/// highest-scoring first, biasing the search exploration in the direction of a solution. Ensure the scores
/// returned by `score(self)` are increasing with the proximity to a solution.
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

/// Optimized heuristic-guided search space exploration iterator.
pub struct ScoredSearcher<S: ScoredSearchable> {
    explored: HashSet<S>,
    fringe: BinaryHeap<OrderedSearchable<S, S::Score>>,
}

impl<S: ScoredSearchable> ScoredSearcher<S> {
    /// Create a new guided search iterator from an initial state.
    pub fn new(initial_state: S) -> Self {
        let score = initial_state.score();
        Self {
            explored: HashSet::new(),
            fringe: BinaryHeap::from([OrderedSearchable {
                state: initial_state,
                score,
            }]),
        }
    }

    /// Create a new guided search iterator from a default state.
    pub fn new_with_default() -> Self
    where
        S: Default,
    {
        Self::new(Default::default())
    }
}

impl<S: ScoredSearchable> Iterator for ScoredSearcher<S>
where
    S: Clone + Hash + Eq,
{
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current_state = self.fringe.pop()?.state;

            if current_state.is_solution() {
                return Some(current_state);
            }

            for state in current_state.next_states() {
                if !self.explored.contains(&state) {
                    self.explored.insert(state.clone());
                    let score = state.score();
                    self.fringe.push(OrderedSearchable { state, score });
                }
            }
        }
    }
}

/// Unoptimized heuristic-guided search space exploration iterator.
/// 
/// Use this instead of [`ScoredSearcher`] if implementing `Clone + Eq + Hash` for your [`ScoredSearchable`] type
/// is infeasible or impractical for whatever reason, or if you're running into memory
/// limitations from the optimized implementation. 
pub struct ScoredUnhashableSearcher<S: ScoredSearchable> {
    fringe: BinaryHeap<OrderedSearchable<S, S::Score>>,
}

impl<S: ScoredSearchable> ScoredUnhashableSearcher<S> {
    /// Create a new unoptimizd guided search iterator from an initial state.
    pub fn new(initial_state: S) -> Self {
        let score = initial_state.score();
        Self {
            fringe: BinaryHeap::from([OrderedSearchable {
                state: initial_state,
                score,
            }]),
        }
    }

    /// Create a new unoptimizd guided search iterator from a default state.
    pub fn new_with_default() -> Self
    where
        S: Default,
    {
        Self::new(Default::default())
    }
}

impl<S: ScoredSearchable> Iterator for ScoredUnhashableSearcher<S> {
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current_state = self.fringe.pop()?.state;

            if current_state.is_solution() {
                return Some(current_state);
            }

            for state in current_state.next_states() {
                let score = state.score();
                self.fringe.push(OrderedSearchable { state, score });
            }
        }
    }
}
