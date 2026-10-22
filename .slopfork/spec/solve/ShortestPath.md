# Spec: solve/ShortestPath

Provenance: documented (docs/MAZE_SOLVE_ALGOS.md, "Shortest Path Finder") + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

Read `solve/_common.md` first.

## Purpose
Breadth-first search from the start that stops at the first route to reach the end, which is therefore a shortest one. The docs describe it as a breadth-first search cut short when the first solution is found. It works on imperfect mazes.

## Public interface
- `ShortestPath()`: no options.
- `solve(grid, start, end) -> list<list<cell>>` (protocol algo `ShortestPath`, `params: {}`).

## Behavior
1. Intended contract: return a list with exactly **one** solution, a shortest one (d(start, end) - 1 squares), when the end is reachable. Return `[]` when it is not.
2. Output is in final form (strictly between start and end). `prune` makes no difference.
3. When several shortest routes exist, which one is returned is unspecified (any tie-break, seeded RNG or fixed order).
4. **Observed defect in the original**: it sometimes returns `[]` although a route exists. In the goldens this happens on **every** case with inner entrances (all 24 `ShortestPath/*/ii/*` cases give `{"solutions": []}`). Every outer/outer case returns one shortest solution. The `cr` invariants for ShortestPath do not include `nonempty_when_reachable`, so both behaviors pass the check. **Ports should return the true shortest path** (recommended; it matches the documentation). Record the fix as a deviation ("ShortestPath returns a solution where the original returned none").
5. Validation beyond `_common.md`: if the start's entry square has no cell move at all, the original raises AssertionError ("input maze is invalid"). Keep this for parity.
6. The search must terminate on every input. Visited-set BFS does.

## Invariants (cr)
With and without prune: `all_shortest`, `first_is_shortest`, `paths_valid`. (`nonempty_when_reachable` is not required because of item 4, but the recommended behavior satisfies it.)

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| common solver validation (`_common.md`) | AssertionError | no |
| start's entry square has no cell move | AssertionError | no |

## Edge cases
| Input | Expected |
|---|---|
| inner start and inner end | original `[]`; port should return the one shortest solution |
| two equally short routes | one of them |
| end unreachable | `[]` |

## Worked examples (goldens)
Grid G1 is the 7x7 `grid` field of case `solve/BacktrackingSolver/Prims3x3/oo/p/s0` in the goldens' `cases/solve.jsonl` (a 3x3-cell Prims maze; every Prims3x3 solve and transmute case uses it). In words: row 1 is open from column 1 to 5; column 1 is open from row 1 to row 5; row 5 is open from column 1 to 5; column 5 is open from row 3 to row 5; and (3,2), (3,3) form a stub off column 1. Everything else is wall.
- `solve/ShortestPath/Prims3x3/oo/p/s0` (and `/np/`): start (3,0), end (1,6). Result one solution R1: the 7 squares going up column 1 from (3,1) to (1,1) and then along row 1 to (1,5).
- `solve/ShortestPath/Prims3x3/ii/p/s0`: start (5,3), end (3,1). The original returned `[]`. The recommended port result is `[[ (5,2), (5,1), (4,1) ]]`, and the checker accepts both.
- Lengths of the single solution on the other outer/outer goldens: Prims6x6 27, Prims10x10 31, DungeonRooms6x6 27, DungeonRooms10x10 37, CellularAutomaton6x6 23.

## Open questions
None yet.
