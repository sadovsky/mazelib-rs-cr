# Spec: transmute/DeadEndFiller

Provenance: documented (docs/MAZE_TRANSMUTE_ALGOS.md "Dead End Filler", docs/API.md transmuting example) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

Read `transmute/_common.md` first (vocabulary: cell, slot, degree, dead end).

## Purpose
Dead-end filling, the classic maze-simplification method: find a dead end and wall it off. Each sealed dead end may expose a new one further up its corridor, so repeated passes eat dead-end passages back to their junctions. Run enough passes on a perfect maze and only the start-to-end route stays open (docs). In heavily braided mazes (all loops) it does little.

## Public interface
- `DeadEndFiller(iterations: int = 1)`: the number of passes.
- `transmute(grid, start, end)`, in place (protocol algo `DeadEndFiller`, params `{}` or `{"iterations": n}`).

## Behavior
1. **Parameter normalization**: if `iterations` > 0, the pass count is `iterations` truncated toward zero to an integer (so 2.7 gives 2 and 0.5 gives 0, which means no passes). If `iterations` <= 0, the pass count is **100**.
2. **Entrance protection**: for the whole operation, the squares at `start` and `end` are treated as open. Afterwards they keep their original values (they are never changed). A cell is never sealed if it equals `start` or `end` or is orthogonally adjacent to either. For an inner entrance that protects the entrance cell. For an outer entrance it protects the entrance's entry square (the cell just inside the border).
3. **One pass** seals exactly **one** dead end, or none if there are no dead ends. It scans the cells in row-major order (top row first, each row left to right), skipping protected cells, and takes the first open cell whose degree is 0 or 1. Sealing it turns that cell **and its four orthogonally adjacent squares** into walls. The slot to its only neighbour is closed; the other adjacent squares are normally already walls. Border squares are never changed: a port must leave a border square as it is even if it is adjacent to the sealed cell. In the goldens such squares are already walls, so this matches the recorded outputs.
4. Passes repeat until the pass count is used up, or until a pass finds no dead end. Later passes see the walls added by earlier ones, so a corridor ending in a dead end is consumed one cell per pass, from the dead-end side.
5. Consequences: `iterations = 1` (the default) changes at most one cell plus its slot. That is 2 squares in every golden default case, the cell and the slot to its single neighbour. `iterations = 3` changes at most 3 cells (6 squares in every golden case). A large count (the docs use 999) seals every dead-end branch, leaving only the route between the entrances plus any loops.
6. Deterministic. It uses no randomness, so following items 1 to 4 reproduces the golden grids exactly.
7. Robustness requirement (port): the transformation must keep `keeps_shortest_length` and `perfect_if_input_perfect` on any input. On grids where every cell is open and the border is solid (all goldens), "degree <= 1" is the same as "at most one open orthogonally adjacent square". A port may use the second test, which also stays safe on unusual grids with dangling slots or open posts, and it gives identical results on the goldens.

## Invariants (cr)
For both the default and `{"iterations": 3}`: `border_unchanged`, `keeps_route`, `keeps_shortest_length`, `only_adds_walls`, `perfect_if_input_perfect`, `shape`.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| start or end absent | TypeError (recommended parity mapping; see `_common.md` item 7) | no |
| non-numeric `iterations` | not covered; any invalid-input error | no |

## Edge cases
| Input | Expected |
|---|---|
| `iterations` = 0 or negative | 100 passes |
| `iterations` = 0.5 | 0 passes (grid unchanged) |
| no dead ends (all junctions/corridors, or every dead end protected) | grid unchanged |
| dead end directly beside an inner entrance cell | can be sealed (it is two squares away, not adjacent) |
| entry square of an outer entrance is a dead end | protected, never sealed |

## Worked examples (goldens)
Grid G1 is the Prims3x3 grid described in `solve/_common.md` (the `grid` of `trans/DeadEndFiller/v1/Prims3x3/oo/s0`). It has dead ends at (1,5), (3,3) and (3,5).
- `trans/DeadEndFiller/v1/Prims3x3/oo/s0`: start (3,0), end (1,6), default iterations. (1,5) is protected (next to the end) and (3,1) is protected (next to the start). The first unprotected dead end in scan order is (3,3), so (3,3) and the slot (3,2) become wall. Every other square is unchanged. Output rows 3 and 4 are `#.###.#`; all other rows are as in the input.
- `trans/DeadEndFiller/v2/Prims3x3/oo/s0`: same, `{"iterations": 3}`. Sealed in turn: (3,3) with its slot (3,2), then (3,5) with (4,5), then (5,5) with (5,4). That is 6 squares, all wall-ward.
- `trans/DeadEndFiller/v2/Prims3x3/ii/s0`: start (5,3), end (3,1), `{"iterations": 3}`. Sealed in turn: (1,5), (1,3), then (1,1) (its only neighbour is the end cell, which is two squares away, so it is not protected). The output's rows 1 and 2 are solid wall, and rows 3 to 6 are unchanged.
- `trans/DeadEndFiller/v1/Prims3x3/ii/s0`: default. Only (1,5) and its slot (1,4) are sealed.

## Open questions
None yet.
