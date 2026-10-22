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

