# Spec: generate/TrivialMaze

Provenance: documented (docs/API.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Builds simple unicursal mazes: a single path with no branches that visits every cell, either as a serpentine (boustrophedon) or as an inward rectangular spiral.

## Public interface
- `TrivialMaze(h: int, w: int, maze_type: string = "spiral")`
- `maze_type`: `"spiral"` (default) or `"serpentine"`. Parsed as in _common.md, so anything not equal to serpentine after lowercasing and trimming means spiral.
- `generate() -> grid`

## Behavior
1. Size validation follows _common.md.
2. The only randomness is one fair coin flip per `generate()` call, choosing the orientation. Given that choice the maze is fully determined. The two outcomes below are exactly the shapes the original produces, and a port should reproduce them for both values of the coin.
3. Serpentine, vertical orientation:
   - Every cell column (odd grid column) is opened top to bottom, from grid row 1 to row `2h-1`.
   - Adjacent columns are joined alternately at the top and at the bottom. On grid row 1, open the slots at grid columns 2, 6, 10, ... (columns congruent to 2 mod 4, below `2w`). On grid row `2h-1`, open the slots at grid columns 4, 8, 12, ... (congruent to 0 mod 4, below `2w`).
4. Serpentine, horizontal orientation: the same with rows and columns exchanged. Every cell row is opened left to right. In grid column 1, open the slots at grid rows 2, 6, 10, .... In grid column `2w-1`, open the slots at grid rows 4, 8, 12, ....
5. Spiral: start at the top-left cell (grid `(1,1)`). Walk straight while the next cell in the current direction is inside the maze and unvisited, opening each slot and cell as you go. When blocked, turn to the next direction in the cycle. End when the current cell has no unvisited neighbor at all. The coin chooses between two cycles:
   - first move down, then turn in the order down, right, up, left. The path runs down the left edge, along the bottom, up the right edge, back along the top, and so on inward.
   - first move right, then turn in the order right, down, left, up. The path runs along the top, down the right edge, along the bottom, up the left edge, and so on inward.
   Both orientations cover every cell.

## Invariants
- perfect, cells_open, corners_walls, border_walls, shape, alphabet.
- Unicursal: the passage graph is a simple path. Exactly two cells have degree 1 and every other cell has degree 2.
- Serpentine, vertical: one endpoint is the bottom-left cell, grid `(2h-1, 1)`. Serpentine, horizontal: one endpoint is the top-right cell, grid `(1, 2w-1)`. In both, the top-left cell has degree 2. Spiral: one endpoint is the top-left cell, grid `(1,1)`, and the other is near the middle.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| h < 3 or w < 3 | AssertionError | no |
| unknown maze_type | none; spiral | n/a |

## Edge cases
| Input | Expected |
|---|---|
| 3x5 serpentine vertical | see worked example (5 vertical corridors) |
| odd vs even number of columns | the final connection lands on the top or the bottom row accordingly. Still a single path |

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |
| v1 | `{"maze_type": "serpentine"}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/TrivialMaze/v0/h3w5/s0`: seed 0, h=3, w=5, params `{}`. spiral, down-first cycle
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.X.......X
        X.X.XXXXX.X
        X.X.....X.X
        X.XXXXXXX.X
        X.........X
        XXXXXXXXXXX

- Case `gen/TrivialMaze/v0/h3w5/s1`: seed 1, h=3, w=5, params `{}`. spiral, right-first cycle
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.........X
        XXXXXXXXX.X
        X.......X.X
        X.XXXXXXX.X
        X.........X
        XXXXXXXXXXX

- Case `gen/TrivialMaze/v1/h3w5/s0`: seed 0, h=3, w=5, params `{"maze_type": "serpentine"}`. serpentine, vertical
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X...X...X.X
        X.X.X.X.X.X
        X.X.X.X.X.X
        X.X.X.X.X.X
        X.X...X...X
        XXXXXXXXXXX

- Case `gen/TrivialMaze/v1/h3w5/s1`: seed 1, h=3, w=5, params `{"maze_type": "serpentine"}`. serpentine, horizontal
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.........X
        X.XXXXXXXXX
        X.........X
        XXXXXXXXX.X
        X.........X
        XXXXXXXXXXX

- Case `gen/TrivialMaze/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/TrivialMaze/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind AssertionError.

## Open questions
None yet.
