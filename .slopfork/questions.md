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
