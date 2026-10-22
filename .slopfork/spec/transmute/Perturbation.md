# Spec: transmute/Perturbation

Provenance: documented (docs/MAZE_TRANSMUTE_ALGOS.md "Perturbation", docs/EXAMPLES.md) + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

Read `transmute/_common.md` first.

## Purpose
First-order perturbation of a maze. Add a few random walls that block current passages, then reconnect every part of the maze that got cut off by opening walls at random. Repeat a set number of times. Starting from a perfect maze this usually yields another perfect maze that differs slightly. Many rounds drift far from the original. The docs also suggest using it to turn a non-maze (a spiral, a drawing) into a maze.

## Public interface
- `Perturbation(repeat: int = 1, new_walls: int = 1)`. Documented domain for both: integers 1, 2, ... `repeat` is the number of rounds. `new_walls` is the number of walls added per round.
- `transmute(grid, start, end)`, in place (protocol algo `Perturbation`, params `{}` or e.g. `{"new_walls": 2, "repeat": 3}`). Start and end are not used.

## Behavior
1. The transmuter runs `repeat` rounds. With `repeat` <= 0 nothing happens at all (grid unchanged). Each round has two phases.
2. **Add walls**: `new_walls` times, pick a random **interior wall slot** (a slot not on the border, between two orthogonally adjacent cells) that is currently open, and make it a wall. The original picks a random interior slot position. If that slot is already a wall it redraws, and it gives up on this wall after about 2 x H x W failed draws (as when every slot is already a wall). Under `cr` the selection distribution is the port's choice, for example uniform over the currently open interior slots, skipping when there are none. With `new_walls` <= 0 no walls are added, but the reconnect phase still runs.
3. **Reconnect**: compute the connected components of the open cells, where two cells are connected when the slot between them is open. While there is more than one component, open one wall slot that separates a cell of one component from a cell of a different component, and merge them. Repeat until a single component remains. The original grows from its first component by picking random cells of it and opening a slot to any adjacent other component. Any random choice of joining slots is conforming. Each opening must join two **different** components.
4. Only interior wall slots ever change. Cells, posts and the border are never touched, and neither are the start/end squares (which are border squares or cells).
5. Consequences:
   - After every round, all open cells form one connected component, so `keeps_route` holds whenever the start and end each sit on or next to an open cell.
   - Because each reconnect opening joins two different components, a perfect input stays perfect for any parameters. The original guarantees this only for the defaults: with `{"new_walls": 2, "repeat": 3}` it sometimes opened more slots than it closed, creating loops. The port should keep perfection always (recommended; stronger than required).
   - With the defaults on a perfect maze, exactly one slot closes and exactly one slot opens. They can be the same slot, in which case the grid is unchanged.
6. Cells that are walls (not present in any golden input) are not part of any component and must never be used as join points. The original can loop forever on such grids. The port must terminate. If two components can't be joined because only wall cells lie between them, leave them disconnected.
7. Wall slots are opened, so `only_adds_walls` and `keeps_shortest_length` are **not** expected (the route can get longer or shorter).

## Invariants (cr)
- Default: `border_unchanged`, `keeps_route`, `perfect_if_input_perfect`, `shape`.
- `{"new_walls": 2, "repeat": 3}`: `border_unchanged`, `keeps_route`, `shape`.

## Errors
None of its own in the goldens. Non-integer parameters are not covered; a port may reject them as invalid input.

## Edge cases
| Input | Expected |
|---|---|
| `repeat` = 0 or negative | grid unchanged |
| `new_walls` = 0 | no walls added; a disconnected input still gets reconnected |
| no open interior slot | no wall added that round; reconnect still runs |
| input already disconnected (e.g. a spiral drawing) | becomes connected (if possible) after the first round |
| very large `new_walls` | many walls, then a spanning reconnection; the result is still connected |

## Worked examples (goldens)
Grid G1 is the Prims3x3 grid described in `solve/_common.md`.
- `trans/Perturbation/v3/Prims3x3/oo/s0` (defaults): one slot closed, (1,4), and one opened, (2,5). Closing (1,4) cut the cell (1,5) off from the rest. Opening (2,5) reattached it from below via (3,5). The output is still perfect: rows 1 to 3 become `#...#.#`, `#.###.#`, `#...#.#`, and rows 4 to 6 are unchanged. The `ii` variant with the same seed gives the same grid, since entrances don't matter.
- `trans/Perturbation/v4/Prims3x3/oo/s0` (`{"new_walls": 2, "repeat": 3}`): net effect two slots closed, (1,4) and (4,5), and two opened, (2,5) and (3,4).
- `trans/Perturbation/v4/Prims10x10/oo/s0`: net 5 closed and 6 opened. The original produced a loop here, which is why `perfect_if_input_perfect` is not required for this parameter set.
- Under `cr` none of these grids is compared exactly. Only the invariants above are checked.

## Open questions
None yet.
