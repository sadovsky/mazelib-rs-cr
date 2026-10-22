# Spec: generate/Ellers

Provenance: documented (docs/MAZE_GEN_ALGOS.md) + observed (goldens, KNOWN_BUGS.json) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Builds a perfect maze with Eller's algorithm, one row at a time, tracking which cells are already connected as disjoint sets.

## Public interface
- `Ellers(h: int, w: int, xskew: float = 0.5, yskew: float = 0.5)`
- `xskew` is the probability of joining two horizontally adjacent cells of a row when they are in different sets.
- `yskew` is the probability of an additional downward connection from a cell, on top of the one each set requires.
- Both are clamped to [0, 1] at construction: values below 0 become 0 and values above 1 become 1. Out-of-range values never raise.
- `generate() -> grid`

## Behavior
1. Size validation follows _common.md.
2. Process rows from top to bottom. Every cell of a row that is not yet in a set (i.e. was not reached from the row above) gets a fresh set of its own.
3. Every row except the last:
   - Horizontal joins: for each pair of horizontally adjacent cells, left to right, with probability `xskew`, if the two cells are in different sets, open the slot between them and merge their sets. Cells already in the same set are never joined, since that would create a loop.
   - Vertical joins: every set present in the row gets at least one downward connection. Choose one of its cells uniformly, open the slot below it, and put the cell below into the same set. Each other cell of the row also opens downward with probability `yskew`, if it has not already. The original skips this extra chance for the last column; profile cr does not care.
4. Last row: for each pair of horizontally adjacent cells, left to right, if they are in different sets, open the slot between them and merge. After this every cell is in one set. No downward connections are made from the last row.
5. Effect of the parameters: high `xskew` gives long horizontal corridors, low `xskew` short ones. High `yskew` gives many vertical passages, low `yskew` few (but always at least one per set).
6. Set labels: the number of distinct labels handed out over a run can exceed 127 (12x17 with `xskew 0.9` does). Use a wide integer type, or a union-find over cell indices. The labels must never wrap.

## Invariants
- perfect, cells_open, corners_walls, border_walls, shape, alphabet, for every xskew/yskew including 0 and 1.
- With `xskew = 0`, every row except the last has no horizontal openings. Every cell is connected through vertical passages, and the last row is fully joined.
- With `xskew = 1`, every row is one horizontal corridor (all cells end up in one set), and each row connects downward through at least one slot.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| h < 3 or w < 3 | AssertionError | no |
| xskew/yskew out of range | none; clamped | n/a |

## Known original bug (must be fixed)
- `gen/Ellers/v2/h12w17/s0` through `s7` (h=12, w=17, `xskew 0.9`, `yskew 0.1`): the original raised OverflowError because a set label passed the signed 8-bit range. KNOWN_BUGS.json lists these cases. The port must return a valid perfect `25 x 35` maze for each, and any error there is a check failure.

## Edge cases
| Input | Expected |
|---|---|
| 3x3, defaults | 7x7 perfect |
| xskew 0.1, yskew 0.9 (v1) | mostly vertical corridors, perfect |
| xskew 0.9, yskew 0.1 (v2), every size including 12x17 | mostly horizontal corridors, perfect, never an error |

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |
| v1 | `{"xskew": 0.1, "yskew": 0.9}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |
| v2 | `{"xskew": 0.9, "yskew": 0.1}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/Ellers/v0/h3w5/s0`: seed 0, h=3, w=5, params `{}`
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.X.X.....X
        X.X.XXX.XXX
        X.......X.X
        X.X.XXX.X.X
        X.X...X...X
        XXXXXXXXXXX

- Case `gen/Ellers/v2/h4w7/s0`: seed 0, h=4, w=7, params `{"xskew": 0.9, "yskew": 0.1}`. xskew 0.9: long horizontal runs
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXXXXXX
        X.............X
        XXXXXXXXXXXXX.X
        X.......X.....X
        X.X.XXXXXXX.XXX
        X.X...........X
        X.XXX.XXXXXXXXX
        X...X.........X
        XXXXXXXXXXXXXXX

- Case `gen/Ellers/v2/h12w17/s0`: seed 0, h=12, w=17, params `{"xskew": 0.9, "yskew": 0.1}`. KNOWN BUG: port must instead return a valid perfect 25x35 maze
  Original result: error kind OverflowError.
- Case `gen/Ellers/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/Ellers/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind AssertionError.

## Open questions
None yet.
