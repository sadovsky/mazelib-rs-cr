# Builder questions

## rng
- Q1 (spec rng item 14): When the seed is out of range for NumPy, is the Python stream seeded before the error is raised? Conservative choice: `Rng::set_seed` checks the range first and leaves both streams unchanged on error. The case fails either way, so the goldens can't tell the difference. (`// SLOPFORK-Q:` in src/rng.rs)
- Q2 (spec rng item 5): The spec only covers `randbelow(n)` for n >= 1. Conservative choice: `randbelow(0)` / `below(0)` return 0 without drawing, so there's no panic or infinite loop.

## generate (AldousBroder, Backtracking, BinaryTree, Division, Ellers, GrowingTree, HuntAndKill)
- Q-gen-1 (spec BinaryTree item "random skew at construction"): the Rust constructor takes no RNG, so an unrecognized/absent skew is drawn on the first `generate()` call and then reused for that object. Under profile cr this is indistinguishable. (`// SLOPFORK-Q:` in src/generate/binary_tree.rs)
- Q-gen-2 (spec Ellers invariants): "with xskew = 1, every row is one horizontal corridor" conflicts with "cells already in the same set are never joined" whenever one set drops down through several cells (yskew > 0). We follow the no-loop rule (required for `perfect`); the full-corridor property then holds for row 0 always, and for every row when yskew = 0. (`// SLOPFORK-Q:` in src/generate/ellers.rs tests)
