# Spec: generate/AldousBroder

Provenance: documented (docs/MAZE_GEN_ALGOS.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Builds a perfect maze using the Aldous-Broder random-walk algorithm, which finds a spanning tree of the cell graph by wandering at random.

## Public interface
- `AldousBroder(h: int, w: int)`, with no options.
- `generate() -> grid`

## Behavior
1. Size validation and grid encoding follow _common.md.
2. The walk starts at a cell chosen uniformly at random and marks it visited. The walk then steps repeatedly to an orthogonally adjacent cell. Whenever it steps into a cell that has not been visited, the wall slot between the two cells is opened and the new cell is marked visited. Stepping into a visited cell opens nothing.
3. The walk stops once all `h*w` cells have been visited.
4. Distribution: the published algorithm picks the next step uniformly among all in-bounds neighbors, which gives a uniform spanning tree. The original deviates: whenever the current cell has unvisited neighbors, it moves to one of them chosen uniformly, and it moves to a uniformly random visited neighbor only when every neighbor is visited. Under profile cr either variant is acceptable. The port is encouraged to keep the original's "prefer unvisited" rule, because it ends in roughly linear time and the textbook walk can be slow on large grids.

## Invariants
- perfect (spanning tree over all cells), cells_open, corners_walls, border_walls, shape, alphabet.
- Exactly `h*w - 1` wall slots are open.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| h < 3 or w < 3 | AssertionError | no |

## Edge cases
| Input | Expected |
|---|---|
| 3x3 (smallest) | 7x7 grid, perfect |
| non-square, e.g. 3x5 or 5x3 | `(2h+1) x (2w+1)`, perfect |

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/AldousBroder/v0/h3w3/s0`: seed 0, h=3, w=3, params `{}`
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXX
        X.....X
        X.XXX.X
        X...X.X
        XXXXX.X
        X.....X
        XXXXXXX

- Case `gen/AldousBroder/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/AldousBroder/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind AssertionError.

## Open questions
None yet.
