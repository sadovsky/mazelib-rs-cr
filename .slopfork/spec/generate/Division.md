# Spec: generate/Division

Provenance: documented (docs/MAZE_GEN_ALGOS.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Builds a perfect maze with the recursive-division algorithm. It starts from an empty chamber and keeps splitting chambers with a wall that has a single door.

## Public interface
- `Division(h: int, w: int)`, with no options.
- `generate() -> grid`

## Behavior
1. Size validation follows _common.md.
2. Start with the outer frame as wall and the whole interior open. The first chamber is the full interior: grid rows 1 to 2h-1 and grid columns 1 to 2w-1.
3. To divide a chamber:
   - If it is taller than it is wide, cut it with a horizontal wall. If wider than tall, use a vertical wall. If square, choose the orientation uniformly (50/50).
   - The wall lies on an even grid line strictly inside the chamber and spans the chamber's full extent in the other direction. The line is chosen uniformly among the eligible even lines.
   - Exactly one door is reopened in that wall, at an odd coordinate, so the door is a slot between two cells. It is chosen uniformly among the chamber's cell positions along the wall.
   - The two sub-chambers on either side of the wall are divided in turn.
4. A chamber stops dividing once it is a single cell thick in either direction, i.e. when no even line lies strictly inside it in the cut direction.
5. The docs' remark about alternating cut orientation is carried out by the "cut across the longer side" rule. Square chambers are random.
6. The port should use an explicit work stack, not recursion. The processing order is unconstrained.

## Invariants
- perfect, cells_open, corners_walls, border_walls, shape, alphabet.
- Every wall placed is a straight segment along an even grid line. Each segment has exactly one door when placed, and later cuts never reopen it.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| h < 3 or w < 3 | AssertionError | no |

## Edge cases
| Input | Expected |
|---|---|
| 3x3 | 7x7 perfect |
| 3x5 | the first cut is vertical, because the chamber is wider than tall |
| 5x3 | the first cut is horizontal |

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/Division/v0/h3w5/s0`: seed 0, h=3, w=5, params `{}`
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.X.X.X.X.X
        X.X.X.X.X.X
        X.........X
        X.XXX.XXX.X
        X.X.....X.X
        XXXXXXXXXXX

- Case `gen/Division/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/Division/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind AssertionError.

## Open questions
None yet.
