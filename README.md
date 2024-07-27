# space-search

A library providing basic generic depth-first, breadth-first, and heuristic-guided search space exploration algorithms.

Implement `Searchable` to perform breadth-first or depth-first searching, and implement `ScoredSearchable` to perform heuristically guided search space exploration. Pass them to `Searcher` to create an iterator that will search for a solution.

`Searcher` requires that you specify a `Manager` type that determines the strategy, return result, and optimization of the search algorithm. Choose one of the searchers defined in the hierarchy of the `search` module to fit your individual needs. 

* Implement `ScoredSearchable` to utilize the `guided` search strategy based managers, which will prioritize searching states with a lower associated score first. If implementing `ScoredSearchable` is too complex or unnecessary for your use case, then you may use the `unguided` search managers, which explore the space naively in a depth-first or breadth-first manner, toggleable by a flag on the manager itself.
* Implement `Eq + Hash + Clone` for your `Searchable` type to benefit from prior explored state checking optimization using a `hashable` manager; if youre unable to, then use an `unhashable` manager, which does not require these additional bounds, but will likely explore the space much less efficiently.
* Use a `route` based manager to yield results consisting of the sequence of steps taken from the starting state to the ending state. Use a `no_route` manager to just yield the solution state alone.

When implementing `ScoredSearchable`, make sure that lower scoring states are closer to a solution.

```rust
use space_search::*;
use std::{hash::Hash, vec};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Pos(i32, i32);

impl Searchable for Pos {
    type NextStatesIter = vec::IntoIter<Pos>;

    fn next_states(&self) -> Self::NextStatesIter {
        let &Pos(x, y) = self;
        vec![
            Pos(x - 1, y), 
            Pos(x, y - 1), 
            Pos(x + 1, y), 
            Pos(x, y + 1)
        ].into_iter()
    }

    fn is_solution(&self) -> bool {
        let &Pos(x, y) = self;
        x == 5 && y == 5
    }
}

let mut searcher: Searcher<search::unguided::no_route::hashable::Manager<_>, _> =
    Searcher::new(Pos(0, 0));
assert_eq!(searcher.next(), Some(Pos(5, 5)));
```

## Todo

* A* shortest path algorithm implementation