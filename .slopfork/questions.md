# Builder questions

## rng
- Q1 (spec rng item 14): When the seed is out of range for NumPy, is the Python stream seeded before the error is raised? Conservative choice: `Rng::set_seed` checks the range first and leaves both streams unchanged on error. The case fails either way, so the goldens can't tell the difference. (`// SLOPFORK-Q:` in src/rng.rs)
  - Answer: the original seeds the Python stream first and then rejects the NumPy seed, so the Python stream is reseeded and the NumPy stream is unchanged after the error. Your check-first choice is also acceptable (unobservable in the goldens). Spec rng.md item 14 updated. No change required.
- Q2 (spec rng item 5): The spec only covers `randbelow(n)` for n >= 1. Conservative choice: `randbelow(0)` / `below(0)` return 0 without drawing, so there's no panic or infinite loop.
  - Answer: randbelow(0) is never reached on valid input (empty choice / empty randrange raise first; the original would not terminate there). Returning 0 without drawing is fine. Spec rng.md item 15 added. No change.

## generate (AldousBroder, Backtracking, BinaryTree, Division, Ellers, GrowingTree, HuntAndKill)
- Q-gen-1 (spec BinaryTree item "random skew at construction"): the Rust constructor takes no RNG, so an unrecognized/absent skew is drawn on the first `generate()` call and then reused for that object. Under profile cr this is indistinguishable. (`// SLOPFORK-Q:` in src/generate/binary_tree.rs)
  - Answer: accepted. The original draws the skew at construction; deferring to the first generate() and reusing it is indistinguishable under cr. No change.
- Q-gen-2 (spec Ellers invariants): "with xskew = 1, every row is one horizontal corridor" conflicts with "cells already in the same set are never joined" whenever one set drops down through several cells (yskew > 0). We follow the no-loop rule (required for `perfect`); the full-corridor property then holds for row 0 always, and for every row when yskew = 0. (`// SLOPFORK-Q:` in src/generate/ellers.rs tests)
  - Answer: your reading is correct; the no-loop rule wins. Only the top row is guaranteed to be a full corridor with xskew = 1 (every row when yskew = 0). The same caveat applies to the last row with xskew = 0 (pairs already in one set stay closed). Spec Ellers.md Invariants corrected. No change.

## generate/CellularAutomaton
- Q3 (spec item 3): The spec says huge `density`/`complexity` values are absolute counts and are never rejected. That leaves room for effectively unbounded work (e.g. 1e300). Conservative choice: seed count is capped at 16x the number of posts, and steps per seed at the number of posts. A growth can't take more steps than there are open posts, so the steps cap doesn't change any result. NaN and negative values give 0. (`// SLOPFORK-Q:` in src/generate/cellular_automaton.rs)
  - Answer: caps accepted as a resource bound. Negative finite values give 0 seeds/steps, as you do. For non-finite values the original raises at generate(): ValueError for NaN, OverflowError for infinities; treating them as 0 is acceptable under cr (not in the goldens), raising would match the original. Spec CellularAutomaton.md Open questions updated. Optional change only.

## generate/DungeonRooms
- Q4 (spec item 3): For rooms that run past the grid, the spec allows clipping or skipping. Conservative choice: clip to the interior (rows 1..2h-1, cols 1..2w-1) and then apply the odd-corner door rule to the clipped rectangle. The frame is never opened. (`// SLOPFORK-Q:` in src/generate/dungeon_rooms.rs)
  - Answer: accepted (clip to interior, door rule on the clipped rectangle, frame never opened). Spec DungeonRooms.md Open questions updated. No change.
- Q5: A supplied `grid` with even dimensions (e.g. 10x10, which gives a 4x4 size) is copied as-is, so the output keeps the supplied shape. Only the `(2h+1)x(2w+1)` region is worked on, and the supplied frame is not rewritten. No golden supplies `grid`.
  - Answer: matches the original: supplied shape kept, only the (2h+1)x(2w+1) region worked on, extra row/column and supplied frame untouched. Spec DungeonRooms.md Open questions updated. No change.

## transmute (CuldeSacFiller, DeadEndFiller, Perturbation)
- Q-tr-1 (spec transmute/_common item 7, DeadEndFiller Errors): with start or end absent, the recommended mapping is TypeError, but `MazeError` has no TypeError-like variant. Conservative choice: DeadEndFiller proceeds and protects only the entrances given (the spec's other allowed option). Not covered by the goldens. (`// SLOPFORK-Q:` in src/transmute/dead_end.rs)
  - Answer: accepted (proceed, protect only the entrances given). Exact parity would need a typed error mapped to TypeError in the adapter; optional, not in the goldens. Spec transmute/_common.md updated. No change required.
- Q-tr-2 (spec DeadEndFiller item 3 vs item 7): sealing "the cell and its four adjacent squares" could cut a route on unusual grids (open posts). We seal the cell, then close each non-border adjacent square only if it has become a dead-end square itself (at most one open neighbour). On well-formed grids that is exactly the slot to the single neighbour, so all golden grids match exactly.
  - Answer: accepted. On well-formed grids it is identical to sealing the cell's four neighbouring squares, and it is safer on open-post grids. No change.
- Q-tr-3 (spec _common item 7): start/end outside the grid return `MazeError::InvalidInput` from all three transmuters.
  - Answer: acceptable under cr. For reference, in the original CuldeSacFiller and Perturbation run normally with out-of-grid entrances (they protect nothing), and DeadEndFiller fails (IndexError past the far edge; negative positions wrap, do not copy). Spec transmute/_common.md item 7 updated. Optional change only (proceed for CuldeSacFiller/Perturbation).

## maze
- Q-maze-1 (spec maze item 29/30): per-maze candidate ranking. Choice: prefer a candidate whose first path is non-empty, then the most solution paths; earliest attempt wins ties (matches the original's ranking whenever every attempt is solvable). Not observable under cr beyond the invariants.
  - Answer: accepted. The original keeps, per maze, the first attempt with the most solution paths, then sorts by the reducer on the first path; your ranking agrees whenever every attempt is solvable (otherwise the original can fail). No change.
- Q-maze-2 (spec maze item 13): degenerate grids. Choice: `generate_entrances` returns `InvalidValue` up front when an axis has no odd interior index, or when a mode needing an inner cell has fewer than 2 inner cells; placement retries are additionally capped at 10,000 (then `InvalidValue`), so it never loops forever. No grid gives `InvalidInput`.
  - Answer: accepted (the original has no guard there and can loop forever). No change.

## solve
- Q-solve-1 (ShortestPath item 5 / ShortestPaths item 4): when the start's entry square has no cell move but start and end are orthogonally adjacent (d = 1), is the AssertionError still raised? Not covered by the goldens. Conservative choice: the entry-move check runs first (the documented validation order), so the error wins. (`// SLOPFORK-Q:` in src/solve/shortest_path.rs and src/solve/shortest_paths.rs)
  - Answer: confirmed. In the original the entry-move check runs before any end/adjacency check, so the AssertionError wins; same for ShortestPaths. Spec ShortestPath.md Open questions updated. No change.
- Q-solve-2 (_common edge cases): for start == end or d = 1, every solver (except the entry-move error above) returns a single empty solution. Walkers whose entry square is outside the grid or a wall, or whose end is reachable by unit steps but not by cell moves (open posts), return one shortest unit-step route instead of walking, so the pruned result stays valid. Not covered by the goldens.
  - Answer: accepted as port design; the original's behaviour on these inputs varies by solver and is not covered by the goldens or required by cr. No change.
