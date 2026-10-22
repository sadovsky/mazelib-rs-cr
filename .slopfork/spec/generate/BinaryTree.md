# Spec: generate/BinaryTree

Provenance: documented (docs/MAZE_GEN_ALGOS.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Builds a perfect maze with the binary-tree algorithm. Every cell opens a wall toward one of two fixed directions, chosen by the skew, so the maze is strongly biased toward one corner.

## Public interface
- `BinaryTree(h: int, w: int, skew: optional<string> = none)`
- `skew` is one of `"NW"`, `"NE"`, `"SW"`, `"SE"`, matched exactly (case-sensitive, no trimming). The docs give `"NW"` as the default. The observed behavior is different: when the skew is omitted, null, or anything other than those four exact strings, one of the four is chosen uniformly at random once, at construction, from the seeded RNG. Every later `generate()` call on the same object reuses that choice.
- `generate() -> grid`

## Behavior
1. Size validation comes first (_common.md). The random skew choice, when needed, happens after validation, so invalid sizes raise without drawing.
2. Each skew allows two carving directions, one vertical and one horizontal. In grid terms, with row 0 at the top, the observed mapping is:
   - `"NW"`: toward the next row down (row + 1) or toward the column to the left (col - 1).
   - `"NE"`: toward row + 1 or toward col + 1.
   - `"SW"`: toward row - 1 or toward col - 1.
   - `"SE"`: toward row - 1 or toward col + 1.
   The letter N therefore corresponds to increasing row index in this library; the original treats row index as growing northward. Reproduce this mapping. Profile cr does not check it, but the tx ports and users who compare pictures will see it.
3. For every cell, take whichever of its two allowed directions stay inside the maze. If both do, open the wall slot in one of them, chosen uniformly (50/50). If only one does, open that one. If neither does, the cell opens nothing; this happens for exactly one corner cell, the root of the tree.
4. Resulting shape: the cells along the edge that can only move horizontally form one straight corridor, and the cells along the edge that can only move vertically form another. For `"NW"` these are the bottom row (grid row `2h-1`) and the leftmost column (grid column 1), and they meet at the bottom-left cell. For `"NE"`: bottom row and rightmost column. For `"SW"`: top row and leftmost column. For `"SE"`: top row and rightmost column.

## Invariants
- perfect, cells_open, corners_walls, border_walls, shape, alphabet, for every skew including the random one.
- Exactly `h*w - 1` wall slots are open. Every cell except the root opens exactly one slot, in one of its two allowed directions.
- The two edge corridors described in item 4 are fully open.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| h < 3 or w < 3 | AssertionError | no |
| unrecognized skew | none; picks a random skew | n/a |

## Edge cases
| Input | Expected |
|---|---|
| skew omitted or null (variant v0) | random one of the four, fixed per object |
| skew `"nw"` (lowercase) | treated as unrecognized, so random |
| 3x3 | 7x7 perfect |

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |
| v1 | `{"skew": "NW"}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |
| v2 | `{"skew": "NE"}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |
| v3 | `{"skew": "SW"}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |
| v4 | `{"skew": "SE"}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/BinaryTree/v1/h3w5/s0`: seed 0, h=3, w=5, params `{"skew": "NW"}`. skew NW: bottom row and left column are full corridors
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X...X.....X
        X.XXX.XXXXX
        X.......X.X
        X.XXXXXXX.X
        X.........X
        XXXXXXXXXXX

- Case `gen/BinaryTree/v2/h3w5/s0`: seed 0, h=3, w=5, params `{"skew": "NE"}`. skew NE: bottom row and right column
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.....X...X
        XXXXX.XXX.X
        X.........X
        XXXXXXXXX.X
        X.........X
        XXXXXXXXXXX

- Case `gen/BinaryTree/v3/h3w5/s0`: seed 0, h=3, w=5, params `{"skew": "SW"}`. skew SW: top row and left column
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.........X
        X.XXXXXXXXX
        X.........X
        X.X.XXX.X.X
        X.X...X.X.X
        XXXXXXXXXXX

- Case `gen/BinaryTree/v4/h3w5/s0`: seed 0, h=3, w=5, params `{"skew": "SE"}`. skew SE: top row and right column
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.........X
        XXXXXXXXX.X
        X.........X
        X.X.XXX.X.X
        X.X.X...X.X
        XXXXXXXXXXX

- Case `gen/BinaryTree/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/BinaryTree/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind AssertionError.

## Open questions
None yet.
