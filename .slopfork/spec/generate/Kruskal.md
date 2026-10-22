# Spec: generate/Kruskal

Provenance: documented (docs/MAZE_GEN_ALGOS.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
Builds a perfect maze with randomized Kruskal's algorithm: wall slots are taken in random order, and a slot is opened whenever it joins two separate trees.

## Public interface
- `Kruskal(h: int, w: int)`, with no options.
- `generate() -> grid`

## Behavior
1. Size validation follows _common.md.
2. Every cell starts open and in its own tree. Every interior wall slot between two orthogonally adjacent cells is a candidate edge: `(h-1)*w` vertical-adjacency slots plus `h*(w-1)` horizontal-adjacency slots.
3. Shuffle the candidates into a uniformly random order. The original uses the NumPy-compatible stream for this shuffle; under profile cr either stream is fine.
4. Take candidates in that order. If the two cells a candidate separates are in different trees, open the slot and merge the trees. Otherwise leave it wall. Stop once one tree remains, or when the candidates run out (same result).
5. A union-find with path compression is the expected implementation. The original's list-of-lists bookkeeping is quadratic and not part of the contract.

## Invariants
- perfect, cells_open, corners_walls, border_walls, shape, alphabet.
- Exactly `h*w - 1` slots are opened.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| h < 3 or w < 3 | AssertionError | no |

## Edge cases
| Input | Expected |
|---|---|
| 3x3 | 7x7 perfect (12 candidates, 8 opened) |
| 12x17 | perfect. Must run fast |

## Required invariants (from invariants.json)

| Variant | Params | Must hold |
|---|---|---|
| v0 | `{}` | acyclic, alphabet, border_walls, cells_open, connected, corners_walls, perfect, shape |

Goldens sizes for every variant: 3x3, 3x5, 5x3, 4x7, 8x8, 12x17 (h x w), seeds 0-7. Invalid-size cases: see _common.md.

## Worked examples

- Case `gen/Kruskal/v0/h3w5/s0`: seed 0, h=3, w=5, params `{}`
  Original output (one valid instance; profile cr does not require reproducing it). Rendered with `X` = wall (protocol `#`) and `.` = open:

        XXXXXXXXXXX
        X.....X...X
        XXX.XXXXX.X
        X...X.....X
        X.X.XXX.XXX
        X.X.......X
        XXXXXXXXXXX

- Case `gen/Kruskal/invalid/h2w5`: seed 0, h=2, w=5, params `{}`
  Original result: error kind AssertionError.
- Case `gen/Kruskal/invalid/h-1w4`: seed 0, h=-1, w=4, params `{}`
  Original result: error kind AssertionError.

## Open questions
None yet.
