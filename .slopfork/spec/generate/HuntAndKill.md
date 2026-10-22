# Spec: generate/HuntAndKill

Provenance: documented (docs/MAZE_GEN_ALGOS.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Builds a perfect maze with the hunt-and-kill algorithm: random walks that carve through unvisited cells, and a hunt for a new starting point whenever a walk gets stuck.

## Public interface
- `HuntAndKill(h: int, w: int, hunt_order: string = "random")`
- `hunt_order`: `"serpentine"` (the classic row-by-row scan) or `"random"` (default). Parsed as in _common.md, so unknown strings mean random.
- `generate() -> grid`

## Behavior
1. Size validation follows _common.md.
2. Open a uniformly random starting cell.
3. Walk (the "kill" phase): from the current cell, while it has orthogonal neighbor cells that are still wall (unvisited), pick one uniformly, open it and the slot between, and move there. The walk ends when the current cell has no unvisited neighbor.
4. Hunt, `"serpentine"`: scan cells in row-major order (top row first, left to right) for a visited cell that has at least one unvisited neighbor. Walk from the first one found. When the scan finds nothing, the maze is complete.
5. Hunt, `"random"` (the library default, said to give harder mazes): pick cells uniformly at random. When the picked cell is visited, walk from it, which does nothing if it has no unvisited neighbor. The original makes a fixed number of picks, one per grid square (`(2h+1)*(2w+1)` trials), and then stops, so in rare cases it could leave a cell unvisited. The port must guarantee completion. For example, do the random picks and then, if any cell is still unvisited, finish with serpentine hunts. The random flavor should dominate the texture.
6. Every walk starts from an already-visited cell and only enters unvisited cells, so the carved passages always form a single tree.

## Invariants
- perfect, cells_open, corners_walls, border_walls, shape, alphabet, for both hunt orders.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| h < 3 or w < 3 | AssertionError | no |
| unknown hunt_order | none; random | n/a |

## Edge cases
| Input | Expected |
|---|---|
| `hunt_order: "serpentine"` (v1) | perfect |
| `hunt_order: " SERPENTINE "` | serpentine |
| 3x3 random | perfect. Every cell must be visited even though random picks may miss |

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |
| v1 | `{"hunt_order": "serpentine"}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/HuntAndKill/v1/h3w5/s0`: seed 0, h=3, w=5, params `{"hunt_order": "serpentine"}`
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.........X
        X.XXXXXXX.X
        X.X.....X.X
        X.X.XXXXX.X
        X...X.....X
        XXXXXXXXXXX

- Case `gen/HuntAndKill/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/HuntAndKill/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind AssertionError.

## Open questions
None yet.
