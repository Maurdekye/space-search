# space-search

A library providing basic utilities for performing generic depth-first, breadth-first, and heuristic-guided search space exploration algorithms.

Implement `Searchable` to perform breadth-first or depth-first searching, and implement `ScoredSearchable` to perform heuristically guided search space exploration. Pass them to the `Search` and `ScoredSearch` structs respectively to create iterators that will search the space for a solution.

Implement `Eq + Hash + Clone` for your search space state type to benefit from prior explored state checking optimization; if youre unable to, then use the `SearchUnhashable` or `ScoredSearchUnhashable` iterators, which do not require these additional bounds, but will likely explore the space much less efficiently.

When implementing `ScoredSearch`, make sure that higher scoring states are closer to a solution.