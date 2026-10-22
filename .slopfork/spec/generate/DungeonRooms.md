# Spec: generate/DungeonRooms

Provenance: documented (docs/MAZE_GEN_ALGOS.md, docs/EXAMPLES.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
A Hunt-and-Kill variant for game-style dungeons. Optional rectangular rooms are carved out first, the rest is filled in with Hunt-and-Kill walks, and any pieces left disconnected are then joined. The output is connected with every cell open. It is generally not perfect: rooms contain loops and open posts, and the joining step may add extra openings.

## Public interface
- `DungeonRooms(h: int, w: int, rooms: optional<list<room>> = none, grid: optional<grid> = none, hunt_order: string = "random")`
- `room` is a pair `[top-left, bottom-right]`. Each corner is a grid coordinate `[row, col]`: a position in the `(2h+1) x (2w+1)` grid, not a cell index. Both corners are inclusive. The docs advise odd coordinates, meaning room corners sit on cells.
- `grid` is a pre-built grid (1 = wall, 0 = open; the docs describe it as all wall, possibly with rooms already open). When given, it overrides `h` and `w`.
- `hunt_order` is `"random"` (default) or `"serpentine"`, parsed as in _common.md, so unknown strings mean random.
- `generate() -> grid`
- Adapter: `params.rooms` is a JSON list of `[[r1,c1],[r2,c2]]`, and `params.grid` is in protocol Grid encoding (`#`/`.`). The goldens only exercise `rooms` and `hunt_order`; `grid` is never supplied.

## Behavior
1. Size and validation (see the table in _common.md):
   - If `grid` is given, the maze size becomes `h = (rows - 1) div 2` and `w = (cols - 1) div 2` of that grid. The positional `h` and `w` are ignored. The 3x3 minimum is then checked (AssertionError).
   - If `grid` is not given and `h < 0` or `w < 0`, raise ValueError. This happens before the minimum-size assertion. Otherwise `h < 3` or `w < 3` raises AssertionError.
2. Starting grid for each `generate()` call: a fresh copy of the supplied grid, or an all-wall `(2h+1) x (2w+1)` grid. The caller's grid is never modified, and repeated `generate()` calls start from the same baseline.
3. Room carving, for each room in order:
   - Every grid position with row in `[r1, r2]` and column in `[c1, c2]` (inclusive) becomes open. Cells, slots and posts inside the rectangle are all opened, so a room is a solid open area.
   - Door: if all four corner coordinates are odd, open one door slot in the room's surrounding wall ring: a position just outside the room on the row above, the row below, the column to its left or the column to its right. Only sides that are not against the outer frame are eligible. A side counts as against the frame when the room's top row is 1 (or 2), its left column is 1 (or 2), its bottom row is `2h-1` or more, or its right column is `2w-1` or more. Choose the door uniformly among cell-aligned slots on the eligible sides. If any corner coordinate is even, no door is carved. Doors are a convenience only: the joining step (item 6) guarantees connectivity anyway, so a port is free to skip doors or place them differently.
   - Malformed rooms (not a pair of pairs, or coordinates outside the grid) are silently ignored. Rooms with `r2 < r1` or `c2 < c1` carve nothing. The original may leave a partly carved room behind when coordinates run past the grid. The port should instead clip the rectangle to the interior (rows 1 to 2h-1, columns 1 to 2w-1) or skip the room, and must never open the outer frame. The goldens do not exercise any of these cases.
4. Start cell: choose uniformly at random among the cells that are still wall after carving, and open it. If no cell is still wall (rooms or the supplied grid already cover every cell), `generate()` raises AssertionError. The original samples at random with a large retry budget. The port should decide deterministically: raise exactly when no wall cell exists.
5. Hunt-and-Kill fill, the same as HuntAndKill.md:
   - Walk: from the current open cell, repeatedly pick a uniformly random orthogonal neighbor cell that is still wall, open it together with the slot between, and move there. Stop when there is none.
   - Hunt with `"random"`: pick random cells and walk from any one that is open. Room cells count as open, so walks also grow outward from rooms.
   - Hunt with `"serpentine"`: scan cells in row-major order for an open cell with an unvisited (wall) neighbor and walk from the first one found. Stop when none remains.
   - In both modes the port must end with every cell open; see the random-hunt note in _common.md. Every cell is reachable because walks can start from any open region.
6. Joining: find the connected regions of cells. Two cells are linked when both are open and the slot between them is open. While more than one region exists, pick a wall slot that separates two different regions, open it, and merge them. In the original the slot is chosen at random on the boundary of the first region. The port may join in any order, but it must open only slots between two cells. Posts outside rooms must stay wall.
7. Because the region detection in the original is approximate, it sometimes opens a slot between two regions that were already connected. That is why loops can appear even with no rooms (variants v0 and v1 are not required to be acyclic). A port that yields a perfect maze when no rooms are given is acceptable.

## Invariants
- All variants: shape, alphabet, border_walls, cells_open, connected.
- No rooms (v0 default, v1 serpentine): also corners_walls. Outside rooms, only cell-to-cell slots are ever opened.
- With rooms (v2): corners_walls is not required, because posts inside rooms are open. Every position inside each in-bounds room is open.
- Not required: acyclic, perfect.
- Monte Carlo cases (`monte_carlo.jsonl`) use DungeonRooms 5x5 with default params and check the v0 invariants on the chosen grid.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| no grid, h < 0 or w < 0 | ValueError | no ("negative dimensions are not allowed") |
| no grid, otherwise h < 3 or w < 3 | AssertionError | no |
| grid given, derived size below 3x3 | AssertionError | no |
| after carving, no cell is still wall | AssertionError (from generate) | no (message free-form, e.g. "no unvisited cell to start from") |
| malformed or out-of-range room | none; room ignored (or clipped) | n/a |

## Edge cases
| Input | Expected |
|---|---|
| rooms `[[[3,3],[5,5]]]` at 3x3 (grid 7x7) | room touches the inner side of the bottom and right frame. Door only on the top or left side. Frame stays wall |
| rooms with even corners, e.g. `[[2,2],[4,4]]` | rectangle carved (wall rows and posts included), no door. The joining step still connects it |
| room covering every cell, e.g. `[[1,1],[2h-1,2w-1]]` | AssertionError from generate |
| `hunt_order: "Serpentine"` | serpentine |
| `hunt_order: "bogus"` | random |
| supplied grid of shape 9x9 with an open 3x3 block at rows/cols 5 to 7 (docs example) | maze size 4x4. The block is kept open and the rest filled. Output shape 9x9 |
| `h=-1, w=4` | ValueError |
| `h=0, w=3` | AssertionError |

The goldens use one room per size for v2, always with odd corners: 3x3, 3x5 and 5x3 use `[[3,3],[5,5]]`; 4x7 uses `[[3,5],[5,7]]`; 8x8 uses `[[5,5],[7,7]]`; 12x17 uses `[[9,11],[11,13]]`. The solve, transmute and entrances sets also use DungeonRooms 6x6 with no params, and 10x10 with room `[[3,3],[9,9]]`.

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | alphabet, border_walls, cells_open, connected, corners_walls, shape |
| v1 | `{"hunt_order": "serpentine"}` | alphabet, border_walls, cells_open, connected, corners_walls, shape |
| v2 | `{"rooms": [[[3, 3], [5, 5]]]}` | alphabet, border_walls, cells_open, connected, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/DungeonRooms/v0/h3w5/s0`: seed 0, h=3, w=5, params `{}`
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.X...X...X
        X.X.X.X.XXX
        X...X.X...X
        X.XXX.XXX.X
        X...X.....X
        XXXXXXXXXXX

- Case `gen/DungeonRooms/v2/h3w3/s0`: seed 0, h=3, w=3, params `{"rooms": [[[3, 3], [5, 5]]]}`. room rows/cols 3-5 fully open, touching the frame's inner side
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXX
        X.....X
        X.XXXXX
        X.X...X
        X.X...X
        X.....X
        XXXXXXX

- Case `gen/DungeonRooms/v2/h4w7/s0`: seed 0, h=4, w=7, params `{"rooms": [[[3, 5], [5, 7]]]}`. room rows 3-5, cols 5-7
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXXXXXX
        X.............X
        X.XXX.XXXXXXX.X
        X...X...X.....X
        XXX.X...XXXXXXX
        X.X.X.....X...X
        X.X.XXX.X.X.X.X
        X...X...X...X.X
        XXXXXXXXXXXXXXX

- Case `gen/DungeonRooms/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/DungeonRooms/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind ValueError.

## Open questions
None yet.
