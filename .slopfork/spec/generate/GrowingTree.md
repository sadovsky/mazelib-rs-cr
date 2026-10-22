# Spec: generate/GrowingTree

Provenance: documented (docs/MAZE_GEN_ALGOS.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Builds a perfect maze with the growing-tree algorithm. It keeps an active list of cells, and the rule for which active cell to extend sets the maze's character: newest gives backtracker style, random gives Prim's style.

## Public interface
- `GrowingTree(h: int, w: int, backtrack_chance: float = 1.0)`
- `backtrack_chance`: on each step, the probability of extending from the most recently added active cell. Otherwise (probability `1 - backtrack_chance`) a uniformly random active cell is used. Not validated. Values of 1 or more always pick the newest, and values of 0 or less always pick at random.
- `generate() -> grid`

## Behavior
1. Size validation follows _common.md.
2. Start with one uniformly random cell, opened and placed in the active list.
3. Repeat while the active list is not empty. Select an active cell by the rule above. If it has unvisited (still wall) orthogonal neighbors, choose one uniformly, open it and the slot between, and append it to the active list. If it has none, remove the selected cell from the active list.
4. `backtrack_chance = 1.0` (default) behaves like the recursive backtracker. `0.0` behaves like a randomized Prim's growth. In between is a mix.

## Invariants
- perfect, cells_open, corners_walls, border_walls, shape, alphabet, for any backtrack_chance.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| h < 3 or w < 3 | AssertionError | no |

## Edge cases
| Input | Expected |
|---|---|
| backtrack_chance 0.0 (v1) | Prim's-like texture, perfect |
| backtrack_chance 0.5 (v2) | mixed, perfect |
| backtrack_chance outside [0,1] | saturates as described, never an error |

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |
| v1 | `{"backtrack_chance": 0.0}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |
| v2 | `{"backtrack_chance": 0.5}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/GrowingTree/v1/h3w5/s0`: seed 0, h=3, w=5, params `{"backtrack_chance": 0.0}`
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.X.......X
        X.X.XXX.X.X
        X...X...X.X
        X.X.XXXXX.X
        X.X.X.....X
        XXXXXXXXXXX

- Case `gen/GrowingTree/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/GrowingTree/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind AssertionError.

## Open questions
None yet.
