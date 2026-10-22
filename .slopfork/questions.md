# Builder questions

## rng
- Q1 (spec rng item 14): When the seed is out of range for NumPy, is the Python stream seeded before the error is raised? Conservative choice: `Rng::set_seed` checks the range first and leaves both streams unchanged on error. The case fails either way, so the goldens can't tell the difference. (`// SLOPFORK-Q:` in src/rng.rs)
- Q2 (spec rng item 5): The spec only covers `randbelow(n)` for n >= 1. Conservative choice: `randbelow(0)` / `below(0)` return 0 without drawing, so there's no panic or infinite loop.

## generate (AldousBroder, Backtracking, BinaryTree, Division, Ellers, GrowingTree, HuntAndKill)
- Q-gen-1 (spec BinaryTree item "random skew at construction"): the Rust constructor takes no RNG, so an unrecognized/absent skew is drawn on the first `generate()` call and then reused for that object. Under profile cr this is indistinguishable. (`// SLOPFORK-Q:` in src/generate/binary_tree.rs)
- Q-gen-2 (spec Ellers invariants): "with xskew = 1, every row is one horizontal corridor" conflicts with "cells already in the same set are never joined" whenever one set drops down through several cells (yskew > 0). We follow the no-loop rule (required for `perfect`); the full-corridor property then holds for row 0 always, and for every row when yskew = 0. (`// SLOPFORK-Q:` in src/generate/ellers.rs tests)

## generate/CellularAutomaton
- Q3 (spec item 3): The spec says huge `density`/`complexity` values are absolute counts and are never rejected. That leaves room for effectively unbounded work (e.g. 1e300). Conservative choice: seed count is capped at 16x the number of posts, and steps per seed at the number of posts. A growth can't take more steps than there are open posts, so the steps cap doesn't change any result. NaN and negative values give 0. (`// SLOPFORK-Q:` in src/generate/cellular_automaton.rs)

## generate/DungeonRooms
- Q4 (spec item 3): For rooms that run past the grid, the spec allows clipping or skipping. Conservative choice: clip to the interior (rows 1..2h-1, cols 1..2w-1) and then apply the odd-corner door rule to the clipped rectangle. The frame is never opened. (`// SLOPFORK-Q:` in src/generate/dungeon_rooms.rs)
- Q5: A supplied `grid` with even dimensions (e.g. 10x10, which gives a 4x4 size) is copied as-is, so the output keeps the supplied shape. Only the `(2h+1)x(2w+1)` region is worked on, and the supplied frame is not rewritten. No golden supplies `grid`.

## transmute (CuldeSacFiller, DeadEndFiller, Perturbation)
- Q-tr-1 (spec transmute/_common item 7, DeadEndFiller Errors): with start or end absent, the recommended mapping is TypeError, but `MazeError` has no TypeError-like variant. Conservative choice: DeadEndFiller proceeds and protects only the entrances given (the spec's other allowed option). Not covered by the goldens. (`// SLOPFORK-Q:` in src/transmute/dead_end.rs)
- Q-tr-2 (spec DeadEndFiller item 3 vs item 7): sealing "the cell and its four adjacent squares" could cut a route on unusual grids (open posts). We seal the cell, then close each non-border adjacent square only if it has become a dead-end square itself (at most one open neighbour). On well-formed grids that is exactly the slot to the single neighbour, so all golden grids match exactly.
- Q-tr-3 (spec _common item 7): start/end outside the grid return `MazeError::InvalidInput` from all three transmuters.

## maze
- Q-maze-1 (spec maze item 29/30): per-maze candidate ranking. Choice: prefer a candidate whose first path is non-empty, then the most solution paths; earliest attempt wins ties (matches the original's ranking whenever every attempt is solvable). Not observable under cr beyond the invariants.
- Q-maze-2 (spec maze item 13): degenerate grids. Choice: `generate_entrances` returns `InvalidValue` up front when an axis has no odd interior index, or when a mode needing an inner cell has fewer than 2 inner cells; placement retries are additionally capped at 10,000 (then `InvalidValue`), so it never loops forever. No grid gives `InvalidInput`.

## solve
- Q-solve-1 (ShortestPath item 5 / ShortestPaths item 4): when the start's entry square has no cell move but start and end are orthogonally adjacent (d = 1), is the AssertionError still raised? Not covered by the goldens. Conservative choice: the entry-move check runs first (the documented validation order), so the error wins. (`// SLOPFORK-Q:` in src/solve/shortest_path.rs and src/solve/shortest_paths.rs)
- Q-solve-2 (_common edge cases): for start == end or d = 1, every solver (except the entry-move error above) returns a single empty solution. Walkers whose entry square is outside the grid or a wall, or whose end is reachable by unit steps but not by cell moves (open posts), return one shortest unit-step route instead of walking, so the pruned result stays valid. Not covered by the goldens.
