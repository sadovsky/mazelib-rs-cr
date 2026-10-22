# Spec: generate/Prims

Provenance: documented (docs/MAZE_GEN_ALGOS.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Builds a perfect maze with randomized Prim's algorithm: a single tree grows from a random cell by repeatedly absorbing a random frontier cell.

## Public interface
- `Prims(h: int, w: int)`, with no options.
- `generate() -> grid`

## Behavior
1. Size validation follows _common.md.
2. Open a uniformly random starting cell. The frontier is the set of unvisited cells orthogonally adjacent to a visited cell.
3. Until all `h*w` cells are visited: choose a frontier cell uniformly at random and open it. Connect it to one of its already-visited neighbors, chosen uniformly, by opening the slot between them. Then add its unvisited neighbors to the frontier, with no duplicates.
4. This is the "frontier cell" form of randomized Prim's. The "random wall" form named in the docs gives an equivalent class of output. Either is acceptable under profile cr.

## Invariants
- perfect, cells_open, corners_walls, border_walls, shape, alphabet.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| h < 3 or w < 3 | AssertionError | no |

## Edge cases
| Input | Expected |
|---|---|
| 3x3, 3x5, 5x3 | perfect, correct shape |
| Prims 4x4 / 3x3 in monte_carlo.jsonl, entrances, solve and transmute sets | these grids are inputs to other ops; the generate op itself is only invariant-checked |

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/Prims/v0/h3w5/s0`: seed 0, h=3, w=5, params `{}`
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X...X.X...X
        XXX.X.X.XXX
        X.........X
        XXX.X.X.XXX
        X...X.X...X
        XXXXXXXXXXX

- Case `gen/Prims/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/Prims/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind AssertionError.

## Open questions
None yet.
