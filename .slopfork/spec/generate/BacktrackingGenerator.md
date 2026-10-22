# Spec: generate/BacktrackingGenerator

Provenance: documented (docs/MAZE_GEN_ALGOS.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Builds a perfect maze with the recursive-backtracker algorithm, also called randomized depth-first search.

## Public interface
- `BacktrackingGenerator(h: int, w: int)`, with no options. The documented parameter names are swapped, but the first positional argument is still rows; see _common.md.
- `generate() -> grid`

## Behavior
1. Size validation and grid encoding follow _common.md.
2. Choose a starting cell uniformly at random, mark it visited, and push it onto a stack.
3. While the stack is not empty, look at the cell on top. If it has unvisited orthogonal neighbors, pick one uniformly at random, open the wall slot between them, mark the neighbor visited and push it. If it has none, pop it.
4. The result is a depth-first spanning tree with long winding corridors and relatively few dead ends.
5. Recursion depth can reach `h*w`. The port should use an explicit stack rather than call recursion, so large sizes cannot overflow the call stack.

## Invariants
- perfect, cells_open, corners_walls, border_walls, shape, alphabet.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| h < 3 or w < 3 | AssertionError | no |

## Edge cases
| Input | Expected |
|---|---|
| 3x3 | 7x7 perfect maze |
| 3x5 / 5x3 | correct non-square shape, perfect |

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/BacktrackingGenerator/v0/h3w5/s0`: seed 0, h=3, w=5, params `{}`
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.....X...X
        X.XXX.XXX.X
        X...X...X.X
        XXX.XXXXX.X
        X.........X
        XXXXXXXXXXX

- Case `gen/BacktrackingGenerator/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/BacktrackingGenerator/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind AssertionError.

## Open questions
None yet.
