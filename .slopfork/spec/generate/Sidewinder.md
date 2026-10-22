# Spec: generate/Sidewinder

Provenance: documented (docs/MAZE_GEN_ALGOS.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Builds a perfect maze with the Sidewinder algorithm, row by row. Each row is split into horizontal runs, and each run connects upward through one of its cells.

## Public interface
- `Sidewinder(h: int, w: int, skew: float = 0.5)`
- `skew` sets the balance between continuing a run east and closing it with a passage north. The probability of carving east at a cell is `1 - skew`. Below 0.5 the maze leans east-west, with long horizontal runs. Above 0.5 it leans north-south, with short runs and many vertical passages. Not validated: values of 1 or more never carve east, and values below 0 always carve east except at the last column.
- `generate() -> grid`

## Behavior
1. Size validation follows _common.md.
2. "North" here means toward grid row 0, the top. The top cell row (grid row 1) is one fully open corridor spanning all `w` cells, since it cannot carve north.
3. Each later row is processed left to right, keeping a current run:
   - Add the cell to the run.
   - With probability `1 - skew`, and only if the cell is not in the last column, open the slot to its east. The run continues.
   - Otherwise choose a cell of the run uniformly, open the slot directly above it (connecting to the row above), and start a new, empty run.
   - The last cell of a row always closes its run.
4. Every run in rows 2 to h has exactly one upward opening.

## Invariants
- perfect, cells_open, corners_walls, border_walls, shape, alphabet, for every skew.
- Grid row 1 is open from column 1 through column `2w-1`.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| h < 3 or w < 3 | AssertionError | no |

## Edge cases
| Input | Expected |
|---|---|
| skew 0.1 (v1) | long horizontal runs |
| skew 0.9 (v2) | mostly vertical passages |
| skew 1.0 | every cell opens north, so all columns hang from the top corridor. Still perfect |

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |
| v1 | `{"skew": 0.1}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |
| v2 | `{"skew": 0.9}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/Sidewinder/v1/h4w7/s0`: seed 0, h=4, w=7, params `{"skew": 0.1}`. skew 0.1: long runs
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXXXXXX
        X.............X
        XXXXX.XXXXXXXXX
        X.............X
        XXXXXXXXXXXXX.X
        X.............X
        XXXXXXXXXXXXX.X
        X.............X
        XXXXXXXXXXXXXXX

- Case `gen/Sidewinder/v2/h4w7/s0`: seed 0, h=4, w=7, params `{"skew": 0.9}`. skew 0.9: short runs
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXXXXXX
        X.............X
        X.X.X.X.XXX.X.X
        X.X.X.X...X.X.X
        X.X.XXX.X.X.X.X
        X.X...X.X.X.X.X
        X.XXX.X.X.X.X.X
        X.X...X.X.X.X.X
        XXXXXXXXXXXXXXX

- Case `gen/Sidewinder/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/Sidewinder/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind AssertionError.

## Open questions
None yet.
