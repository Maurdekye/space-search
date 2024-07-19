use std::{
    collections::{BinaryHeap, HashSet, VecDeque},
    hash::Hash,
};

pub trait Searchable {
    type NextMoveIterator: Iterator<Item = Self>;

    fn next_states(&self) -> Self::NextMoveIterator;
    fn is_solution(&self) -> bool;
}

pub struct Search<S> {
    explored: HashSet<S>,
    fringe: VecDeque<S>,
    pub depth_first: bool,
}

impl<S> Search<S> {
    pub fn new(initial_state: S) -> Self {
        Self {
            explored: HashSet::new(),
            fringe: VecDeque::from([initial_state]),
            depth_first: false,
        }
    }

    pub fn new_with_default() -> Self
    where
        S: Default,
    {
        Self::new(Default::default())
    }
}

impl<S> Iterator for Search<S>
where
    S: Searchable + Clone + Hash + Eq,
{
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(current_state) = self.fringe.pop_back() else {
                return None;
            };

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

pub struct SearchUnhashable<S> {
    fringe: VecDeque<S>,
    pub depth_first: bool,
}

impl<S> SearchUnhashable<S> {
    pub fn new(initial_state: S) -> Self {
        Self {
            fringe: VecDeque::from([initial_state]),
            depth_first: false,
        }
    }

    pub fn new_with_default() -> Self
    where
        S: Default,
    {
        Self::new(Default::default())
    }
}

impl<S> Iterator for SearchUnhashable<S>
where
    S: Searchable,
{
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(current_state) = self.fringe.pop_back() else {
                return None;
            };

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

pub trait ScoredSearchable: Searchable {
    type Score: Ord;

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

pub struct ScoredSearch<S: ScoredSearchable> {
    explored: HashSet<S>,
    fringe: BinaryHeap<OrderedSearchable<S, S::Score>>,
}

impl<S: ScoredSearchable> ScoredSearch<S> {
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

    pub fn new_with_default() -> Self
    where
        S: Default,
    {
        Self::new(Default::default())
    }
}

impl<S: ScoredSearchable> Iterator for ScoredSearch<S>
where
    S: Clone + Hash + Eq,
{
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(current_state) = self.fringe.pop() else {
                return None;
            };

            if current_state.state.is_solution() {
                return Some(current_state.state);
            }

            for state in current_state.state.next_states() {
                if !self.explored.contains(&state) {
                    self.explored.insert(state.clone());
                    let score = state.score();
                    self.fringe.push(OrderedSearchable { state, score });
                }
            }
        }
    }
}

pub struct ScoredSearchUnhashable<S: ScoredSearchable> {
    fringe: BinaryHeap<OrderedSearchable<S, S::Score>>,
}

impl<S: ScoredSearchable> ScoredSearchUnhashable<S> {
    pub fn new(initial_state: S) -> Self {
        let score = initial_state.score();
        Self {
            fringe: BinaryHeap::from([OrderedSearchable {
                state: initial_state,
                score,
            }]),
        }
    }

    pub fn new_with_default() -> Self
    where
        S: Default,
    {
        Self::new(Default::default())
    }
}

impl<S: ScoredSearchable> Iterator for ScoredSearchUnhashable<S> {
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(current_state) = self.fringe.pop() else {
                return None;
            };

            if current_state.state.is_solution() {
                return Some(current_state.state);
            }

            for state in current_state.state.next_states() {
                let score = state.score();
                self.fringe.push(OrderedSearchable { state, score });
            }
        }
    }
}
