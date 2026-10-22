# Spec: solve/ShortestPaths

Provenance: documented (docs/MAZE_SOLVE_ALGOS.md, "Shortest Paths Finder") + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

Read `solve/_common.md` first.

## Purpose
A breadth-first search that keeps branching until every branch has either reached the end or died. It returns several solutions, ordered shortest first. The docs say it finds all unique, non-looped solutions and works on imperfect mazes.

## Public interface
- `ShortestPaths()`: no options.
- `solve(grid, start, end) -> list<list<cell>>` (protocol algo `ShortestPaths`, `params: {}`).

## Behavior
1. Returns a **non-empty** list of distinct solutions when the end is reachable. The **first is a shortest** solution (d - 1 squares), and the list is in non-decreasing order of length.
2. Every solution must be a valid path (`paths_valid`) whether or not prune is set.
3. Unreachable end, or no route found: the original raises AssertionError ("no valid solutions"). Keep this for parity (recommended).
4. If the start's entry square has no cell move, the original raises AssertionError.
5. What the original returns, observed in the goldens:
   - On perfect mazes with outer entrances: exactly 1 solution.
   - With inner entrances: 2. The second is an artifact, a route that first steps away from the end, bounces off a dead end back over the start, and then follows the first route. It is valid only in the weak sense of the checker (the pruned form starts beside the start, steps onto the start square and continues).
   - On CellularAutomaton6x6: 2 to 4.
   - On DungeonRooms10x10: 179 and 992 solutions, including longer detours.
   - The raw list is sorted by length. Pruning then shortens some entries without re-sorting, so the pruned list need not be sorted (seen on DungeonRooms10x10/ii), but its first entry stays the shortest.
6. Port contract (cr): items 1 to 4. The number of solutions beyond the first is the port's choice. Recommended levels, either one conforming:
   - Minimal: all distinct shortest solutions, capped at a fixed number (for example 1000, the original's order of magnitude) because open rooms have combinatorially many.
   - Fuller, closer to the docs: distinct simple routes (no square repeated, never containing `start` or `end`) in non-decreasing length order, up to the same cap. Do not reproduce the "bounce over the start" artifacts.
   Duplicates must not appear. Among equal-length solutions any order is fine.
7. Because the port's solutions are already clean, `prune` should not change them (`_common.md` item 13).

## Invariants (cr)
With and without prune: `first_is_shortest`, `nonempty_when_reachable`, `paths_valid`. (`all_shortest` is not required; longer alternatives are allowed.)

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| common solver validation (`_common.md`) | AssertionError | no |
| start's entry square has no cell move | AssertionError | no |
| no solution found (end unreachable) | AssertionError | no |

## Edge cases
| Input | Expected |
|---|---|
| perfect maze | the unique route (original: plus an artifact when entrances are inner; port: need not) |
| maze with several routes | shortest first, then (optionally) others by length |
| open rooms | capped list, first shortest |
| end unreachable | AssertionError |

## Worked examples (goldens)
Grid G1 is the 7x7 `grid` field of case `solve/BacktrackingSolver/Prims3x3/oo/p/s0` in the goldens' `cases/solve.jsonl` (a 3x3-cell Prims maze; every Prims3x3 solve and transmute case uses it). In words: row 1 is open from column 1 to 5; column 1 is open from row 1 to row 5; row 5 is open from column 1 to 5; column 5 is open from row 3 to row 5; and (3,2), (3,3) form a stub off column 1. Everything else is wall.
- `solve/ShortestPaths/Prims3x3/oo/p/s0`: start (3,0), end (1,6). Result one solution R1: the 7 squares going up column 1 from (3,1) to (1,1) and then along row 1 to (1,5).
- `solve/ShortestPaths/Prims3x3/ii/p/s0`: start (5,3), end (3,1). The original gave two solutions: `(5,2), (5,1), (4,1)` and the artifact `(5,4), (5,3), (5,2), (5,1), (4,1)`. A port returning only `[[ (5,2), (5,1), (4,1) ]]` conforms.
- `solve/ShortestPaths/Prims3x3/ii/np/s0`: same, but the artifact is unpruned (7 squares): east to the dead end (5,5), back over the start, then west and north like the first solution.
- `solve/ShortestPaths/CellularAutomaton6x6/oo/p/s0` (grid C1 in `Collision.md`, start (12,9), end (0,9)): 2 solutions, both 23 squares.
- `solve/ShortestPaths/DungeonRooms10x10/oo/p/s0`: 179 solutions with lengths 37, 41, 41, 41, 41, 41, 41, 45, ... The first (37) is the shortest.

## Open questions
None yet.
