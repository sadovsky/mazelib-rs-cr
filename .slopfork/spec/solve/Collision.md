# Spec: solve/Collision

Provenance: documented (docs/MAZE_SOLVE_ALGOS.md, "Collision Solver") + observed (goldens, KNOWN_BUGS.json) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

Read `solve/_common.md` first.

## Purpose
A flood solver. Water spreads from the start in all directions at the same speed. Where two fronts meet, a wall is built at the meeting point, and dead-end fronts dry up. The flood repeats until nothing changes, and the fronts that reached the end are the solutions. The docs say it finds shortest solutions and "doesn't always work against imperfect mazes". The original crashes on several such mazes (see Known bugs). The port must not.

## Public interface
- `Collision()`: no options.
- `solve(grid, start, end) -> list<list<cell>>` (protocol algo `Collision`, `params: {}`).

## Behavior
1. Returns a list of **one or more distinct solutions, all of the same minimal length** d(start, end) - 1, whenever the end is reachable. The goldens show 1 solution in 32 successful cases and 2 solutions (two distinct shortest routes) in 4 cases.
2. The output is already in final form: each solution lies strictly between `start` and `end`, begins next to `start` (the entry square for an outer start, the slot next to the start cell for an inner start) and ends next to `end`. The `prune` flag therefore makes no difference. The goldens show identical prune and noprune results for every case.
3. Order of multiple solutions: unspecified. In the goldens the order of the two equal-length solutions depends on the seed. Duplicates are not allowed.
4. The solver works on a private copy of the grid. Walls it builds at collision points never reach the caller's grid.
5. Unreachable end: returns `[]`.
6. Mechanism (documented, for orientation; not contractual under `cr`): advance all fronts one cell move per time step from the start's entry square. A front that reaches a junction splits. A front never steps straight back. A front with no move is a dead end and stops. A front that arrives at the end stops and is kept. When two or more fronts land on the same cell at the same step, that cell is walled off (in the private copy) and those fronts stop. The whole flood is re-run on the marked-up copy until a run produces no new set of paths. The surviving arrived fronts are the solutions.
7. **Port requirement**: the result must satisfy all four invariants on every input, including mazes with loops and open rooms. Implementing the documented flood is optional. Any method that yields distinct shortest paths is conforming, for example a breadth-first search that records predecessors and enumerates shortest routes. Recommended:
   - Return every distinct shortest solution, capped at a fixed small number (for example 64) to bound output on open rooms, where the count of shortest routes is combinatorial.
   - Always return at least one.
   - Use the RNG (seeded) or a fixed order for tie-breaking. Either is fine.

## Invariants (cr)
With and without prune: `all_shortest`, `first_is_shortest`, `nonempty_when_reachable`, `paths_valid`.

## Known bugs (original)
`KNOWN_BUGS.json` lists 12 cases where the original raises TypeError: `solve/Collision/CellularAutomaton6x6/oo/{p,np}/{s0,s1}` and `solve/Collision/DungeonRooms10x10/{oo,ii}/{p,np}/{s0,s1}`. The crash happens on mazes with loops or open rooms, when several flood fronts meet at once. On these cases the port must return a successful result meeting the invariants above. Returning any error fails the check.

## Errors
As in `_common.md`. No others. Never TypeError.

## Edge cases
| Input | Expected |
|---|---|
| perfect maze | exactly one solution (the unique route) |
| maze with two equally short routes | both routes, in any order (recommended) or at least one |
| open room between start and end | at least one shortest route; capped list recommended |
| end unreachable | `[]` |

## Worked examples (goldens)
Grid G1 is the 7x7 `grid` field of case `solve/BacktrackingSolver/Prims3x3/oo/p/s0` in the goldens' `cases/solve.jsonl` (a 3x3-cell Prims maze; every Prims3x3 solve and transmute case uses it). In words: row 1 is open from column 1 to 5; column 1 is open from row 1 to row 5; row 5 is open from column 1 to 5; column 5 is open from row 3 to row 5; and (3,2), (3,3) form a stub off column 1. Everything else is wall.
- `solve/Collision/Prims3x3/oo/np/s0` (and `/p/`): start (3,0), end (1,6). Result one solution R1: the 7 squares going up column 1 from (3,1) to (1,1) and then along row 1 to (1,5).
- `solve/Collision/Prims3x3/ii/np/s0`: start (5,3), end (3,1). Result `[[ (5,2), (5,1), (4,1) ]]`. This is already trimmed without pruning.

Grid C1 (CellularAutomaton 6x6), rows: `#############`, `#...........#`, `#.###.###.###`, `#.#...#.#...#`, `#.#.###.###.#`, `#.#.#.......#`, `###.#####.###`, `#.......#...#`, `###.###.###.#`, `#.#.......#.#`, `#.#######.#.#`, `#.........#.#`, `#############`.
- `solve/Collision/CellularAutomaton6x6/ii/p/s0`: start (3,9), end (9,7). Two solutions, each 19 squares. Both share the prefix (2,9), (1,9), (1,8), (1,7), (1,6), (1,5), (2,5), (3,5), (3,4), (3,3), (4,3), (5,3), (6,3), (7,3). From there one continues (7,4), (7,5), (7,6), (7,7), (8,7), and the other continues (8,3), (9,3), (9,4), (9,5), (9,6). The seed s1 case lists the same two in the opposite order.
- `solve/Collision/CellularAutomaton6x6/oo/p/s0`: start (12,9), end (0,9) on the same grid C1. Known bug: the original raised TypeError. The port must return valid shortest solutions here.

## Open questions
None yet.
