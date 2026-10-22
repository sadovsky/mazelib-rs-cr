# Spec: transmute/CuldeSacFiller

Provenance: documented (docs/MAZE_TRANSMUTE_ALGOS.md "Cul-de-sac Filler", docs/API.md transmuting example) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

Read `transmute/_common.md` first (vocabulary: cell, slot, degree, corridor cell, junction).

## Purpose
Cul-de-sac filling (also called loop filling) breaks loops that hang off the rest of the maze. Such a loop is a ring of corridor that leaves a junction and comes back to the same junction without meeting anything else. One cell of the ring is turned into a wall, which turns the loop into two ordinary dead-end branches. DeadEndFiller can then remove those. The docs note that it works well for simple loops and multi-loops but probably not for big empty rooms.

## Public interface
- `CuldeSacFiller()`: no options.
- `transmute(grid, start, end)`, in place (protocol algo `CuldeSacFiller`, `params: {}`).

## Behavior
1. Scan the cells in row-major order (top row first, left to right). Skip a cell if it equals `start` or `end`, or if it is a wall. Consider only **corridor cells** (open, degree exactly 2). Degrees are re-evaluated against the grid as modified so far.
2. For a corridor cell X, walk away from X in each of its two directions, one cell move at a time. Pass through corridor cells, always continuing away from the cell you came from. Stop at the first cell whose degree is not 2 (a junction or a dead end). Call the two stopping cells A and B.
3. If A and B are the same cell, X lies on a loop that leaves that junction and returns to it. **Seal X**: turn the cell X itself into a wall. Only the cell changes. Its slots are left as they are, so they become open stubs.
4. Pure ring with no junction at all (a closed corridor where every cell has degree 2): the walk comes back around to X. In that case the original treats each walk as stopping at the last cell before X. Those two cells differ, so nothing is sealed. Isolated rings stay intact.
5. Deterministic. It uses no randomness (the result does not depend on which direction is walked first).
6. **Port safety rule (required)**: the original does not check whether the start or end lies on the loop being broken. Doing so could lengthen the shortest route. That never happened in the goldens, where every CuldeSacFiller case leaves the grid unchanged, but the `cr` invariants require `keeps_shortest_length` on every input. The port must therefore also skip X when either walk from X, or X itself, passes through a protected square. The protected squares are `start`, `end`, and, for an outer entrance, its entry square (the cell just inside the border). With that rule, sealing X only cuts a loop that no shortest route uses. A route that enters a hanging loop has to leave by the same junction, so a shortest route never needs one. A port may additionally verify the start-to-end distance before and after each seal and undo any seal that changes it.
7. Never touches border squares, posts or slots. On a perfect maze there are no loops, so the grid is unchanged (`perfect_if_input_perfect` holds trivially).

## Invariants (cr)
`border_unchanged`, `keeps_route`, `keeps_shortest_length`, `only_adds_walls`, `perfect_if_input_perfect`, `shape`.

## Errors
None of its own. With start/end absent it simply protects nothing.

## Edge cases
| Input | Expected |
|---|---|
| perfect maze | unchanged |
| open room (blocks of junction cells) | essentially unchanged: room cells are junctions, and a degree-2 room corner has two different junction neighbours |
| 2x2 block of cells attached to the maze only through one of its cells | the other three cells form a hanging loop. The first of them in scan order is sealed (unless protected), and the other two become a dead-end branch |
| loop containing the start or end | not sealed (port safety rule) |
| isolated ring | unchanged |

## Worked examples
- Goldens: all 24 `trans/CuldeSacFiller/...` cases return the input grid unchanged. That covers the perfect Prims and DungeonRooms 6x6 grids, plus DungeonRooms 10x10 (rooms) and CellularAutomaton 6x6 (none of its loops hangs off a single junction). Example: `trans/CuldeSacFiller/v0/CellularAutomaton6x6/oo/s0`, start (12,9), end (0,9). Output equals input.
- Documentation example (docs/API.md, "Optional: Transmuting a Maze"): a 7x7 grid with a loop through (1,3), (1,5), (3,5), (3,3) hanging off the junction (1,3). The loop cell (1,5) is sealed. In the rendered output, row 1 changes from `S     #` to `S    ##`. (3,5) and (3,3) are then dead ends, which the subsequent DeadEndFiller in that example removes.

## Open questions
None yet.
