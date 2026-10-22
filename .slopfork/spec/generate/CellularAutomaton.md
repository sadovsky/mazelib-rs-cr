# Spec: generate/CellularAutomaton

Provenance: documented (docs/MAZE_GEN_ALGOS.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Builds an imperfect maze by growing wall segments into an open field: the "complexity and density" wall-growing method popularized by the maze-generation literature (the well-known short example on Wikipedia's maze generation page). The result has loops and wide-open areas. It is always connected and every cell is open.

## Public interface
- `CellularAutomaton(h: int, w: int, complexity: float = 1.0, density: float = 1.0)`
- `generate() -> grid`
- The documented range for both floats is [0, 1]. Values above 1 are allowed and mean absolute counts (see below). The goldens use `complexity 2.0`.

## Behavior
1. Size validation follows _common.md.
2. Start from a grid whose interior is entirely open, framed by walls.
3. Effective parameters, computed per call to `generate`:
   - Seed count basis `D`: if `density <= 1`, then `D = density * h * w`. Otherwise `D = density`, taken as an absolute number.
   - Growth length `C`: if `complexity <= 1`, then `C = complexity * (h + w)`. Otherwise `C = complexity`, taken as absolute.
   - The number of wall seeds is `floor(2 * D)`. The step budget per seed is `floor(C)`.
   - The original stores the scaled values back into the object, so a second `generate` on the same object can rescale a value that was still at most 1 after scaling. Do not reproduce this. Compute the effective values fresh on each call. No golden depends on it.
4. Seeds sit on posts (even/even grid positions). Roughly the first half of the seeds (seed index below `D`) start on a post of the outer frame, which makes many walls hang off the border. To pick such a post, first choose top/bottom versus left/right 50/50, then one of the two sides, then a uniformly random even position along it. The remaining seeds start at a uniformly random post anywhere in the grid, frame included.
5. Each seed sets its post to wall. Then, for up to `C` steps, the wall may extend from the current post by two squares to an orthogonally neighboring post. The neighbor must lie inside the grid and still be open. Extending sets that post and the wall slot between them to wall, and the extended post becomes the current one. The original extends only while the current post touches between one and three wall posts, so a fresh isolated interior seed stays a single dot, and a post boxed in on all four sides stops. The open neighbor is chosen uniformly. Profile cr does not require this exact survival rule.
6. Walls never extend into a post that is already wall. That is what keeps the maze connected (see Invariants).

## Invariants
- shape, alphabet, border_walls.
- cells_open: odd/odd positions are never touched, because seeds and extensions only write posts (even/even) and the slots between two posts (one even and one odd coordinate).
- connected: every wall component is either the frame plus trees hanging from it, or a free-standing tree, because extensions only ever join a new open post to the current wall component. No new enclosed region can form, so all open squares stay one component. A port that changes the growth rule must keep this "extend only into open posts" property.
- Not required: acyclic, perfect, corners_walls. Loops and open posts are expected.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| h < 3 or w < 3 | AssertionError | no |
| any float value (negative, zero, large) | never an error | n/a |

## Edge cases
| Input | Expected |
|---|---|
| density 0 or complexity 0 | few or no walls inside the frame. Still valid: connected, cells_open |
| complexity 2.0, density 0.2 (variant v2) | `floor(0.4*h*w)` seeds, at most 2 extension steps each |
| 3x3 | 7x7, connected, all cells open |

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | alphabet, border_walls, cells_open, connected, shape |
| v1 | `{"complexity": 0.5, "density": 0.5}` | alphabet, border_walls, cells_open, connected, shape |
| v2 | `{"complexity": 2.0, "density": 0.2}` | alphabet, border_walls, cells_open, connected, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/CellularAutomaton/v0/h3w5/s0`: seed 0, h=3, w=5, params `{}`
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.........X
        XXX.X.XXX.X
        X.X.X.X.X.X
        X.XXX.X.X.X
        X.....X...X
        XXXXXXXXXXX

- Case `gen/CellularAutomaton/v2/h4w7/s1`: seed 1, h=4, w=7, params `{"complexity": 2.0, "density": 0.2}`. note loops and open posts
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXXXXXX
        X...........X.X
        X.....XXX.XXX.X
        X.....X.......X
        XXXXX.X.....X.X
        X...........X.X
        XXX.XXX.....X.X
        X.....X.....X.X
        XXXXXXXXXXXXXXX

- Case `gen/CellularAutomaton/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/CellularAutomaton/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind AssertionError.

## Open questions
None yet.
