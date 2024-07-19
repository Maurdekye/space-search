# space-search

A library providing basic utilities for performing generic depth-first, breadth-first, and heuristic-guided search space exploration algorithms.

Implement `Searchable` to perform breadth-first or depth-first searching, and implement `ScoredSearchable` to perform heuristically guided search space exploration. Pass them to the `Search` and `ScoredSearch` structs respectively to create iterators that will search the space for a solution.

Implement `Eq + Hash + Clone` for your search space state type to benefit from prior explored state checking optimization; if youre unable to, then use the `SearchUnhashable` or `ScoredSearchUnhashable` iterators, which do not require these additional bounds, but will likely explore the space much less efficiently.

When implementing `ScoredSearch`, make sure that higher scoring states are closer to a solution.

```rust
use space_search::*;
use std::{vec, hash::Hash};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Pos(i32, i32);

impl Searchable for Pos {
    type NextMoveIterator = vec::IntoIter<Pos>;

    fn next_states(&self) -> Self::NextMoveIterator {
        let &Pos(x, y) = self;
        vec![
            Pos(x - 1, y),
            Pos(x, y - 1),
            Pos(x + 1, y),
            Pos(x, y + 1),
        ].into_iter()
    }

    fn is_solution(&self) -> bool {
        let &Pos(x, y) = self;
        x == 5 && y == 5
    }
}

let mut searcher = Search::new(Pos(0, 0));
assert_eq!(searcher.next(), Some(Pos(5, 5)));
```

## Todo

* search path memorization
* A* shortest path algorithm implementation