# Spec: generate/Wilsons

Provenance: documented (docs/MAZE_GEN_ALGOS.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Builds a perfect maze with Wilson's algorithm: loop-erased random walks that produce a uniform spanning tree.

## Public interface
- `Wilsons(h: int, w: int, hunt_order: string = "random")`
- `hunt_order` decides how the starting cell of each walk is chosen: `"random"` (default) or `"serpentine"`. Parsed as in _common.md.
- `generate() -> grid`

## Behavior
1. Size validation follows _common.md.
2. Put one uniformly random cell in the tree (open it).
3. Repeat until every cell is in the tree:
   - Pick a starting cell that is not in the tree. With `"serpentine"`, take the first such cell in row-major order. With `"random"`, pick cells uniformly at random; a pick that is already in the tree adds nothing and another pick follows.
   - Random walk from it, one step at a time to a uniformly chosen in-bounds orthogonal neighbor cell, until the walk hits a cell in the tree.
   - Loop erasure: retrace the walk from its start, using the last exit direction recorded at each visited cell. Overwriting the direction on revisits erases loops. Add every cell along the retraced path, and the slots between consecutive cells, to the tree, ending at the tree cell that was hit.
4. Termination with `"random"`: the original counts cells added and stops once `h*w` cells are in the tree, so it always completes. The port must also guarantee completion. Random picks that keep landing in the tree must not hang the generator; falling back to scanning for a remaining cell is fine.
5. Distribution: a uniform spanning tree of the `h x w` grid graph, whatever the hunt order.

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
| serpentine (v1) | perfect |
| 12x17 | perfect, reasonable time |

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |
| v1 | `{"hunt_order": "serpentine"}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/Wilsons/v1/h3w5/s0`: seed 0, h=3, w=5, params `{"hunt_order": "serpentine"}`
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.X.......X
        X.X.X.XXX.X
        X.X.X.X...X
        X.X.XXXXX.X
        X.....X...X
        XXXXXXXXXXX

- Case `gen/Wilsons/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/Wilsons/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind AssertionError.

## Open questions
None yet.
