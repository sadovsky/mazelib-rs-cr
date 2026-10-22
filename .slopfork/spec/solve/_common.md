# Spec: solve/_common (shared solver contract)

Provenance: documented (docs/MAZE_SOLVE_ALGOS.md, docs/API.md, goldens PROTOCOL.md) + observed (goldens `solve` set, `invariants.json`, `KNOWN_BUGS.json`) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

## Purpose
A solver takes a grid plus a start and an end position and returns a list of solutions. Each solution is a route through the open squares of the grid. This file fixes what all seven solvers share: the vocabulary, input validation and error kinds, what a "solution" is, how outer (border) and inner entrances are treated, how the facade's `prune` flag changes the output, what happens when the end can't be reached, and which invariants each solver must meet under profile `cr`. The per-algorithm specs in this directory add only what is specific to each one.

## Public interface
- Solver classes: `BacktrackingSolver()`, `Chain(turn: string = 'right')`, `Collision()`, `RandomMouse()`, `ShortestPath()`, `ShortestPaths()`, `Tremaux()`. Only Chain takes an option.
- `solve(grid: grid, start: cell, end: cell) -> list<list<cell>>`, called by the facade's `Maze.solve()`. `cell` = `(int row, int col)`, 0-based, row 0 at the top.
- Facade: `Maze.solver`, `Maze.start`, `Maze.end`, `Maze.prune: bool` (default true), `Maze.solve()` stores the result in `Maze.solutions`. See `.slopfork/spec/maze.md` items 14 to 17.
- Adapter protocol: op `solve`, `args = {algo, params, grid, start, end, prune}`, result `{solutions: list<list<cell>>}`. Chain is the only algo with params (`turn`).

## Vocabulary
Grid of H rows and W columns, 1 = wall, 0 = open (protocol strings: `#` wall, `.` open). Same encoding as `generate/_common.md`.
- **Cell**: a position whose row and column are both odd.
- **Wall slot**: a position with one odd and one even coordinate. It sits between two orthogonally adjacent cells.
- **Post**: both coordinates even.
- **Border**: row 0, row H-1, column 0, column W-1.
- **Outer entrance**: a border position (not a corner). In generated mazes the border is all wall, so an outer start or end is a **wall** square. The grid is not opened there.
- **Inner entrance**: a cell that is not on the border.
- **Entry square** of an entrance: for an inner entrance, the entrance itself. For an outer entrance, the one interior square directly inside it: row 0 gives `(1, c)`, row H-1 gives `(H-2, c)`, column 0 gives `(r, 1)`, column W-1 gives `(r, W-2)`. For a properly aligned outer entrance (odd column on a top/bottom side, odd row on a left/right side) the entry square is a cell.
- **Cell move**: a step from one cell to an orthogonal cell two positions away, allowed when both the wall slot in between and the target cell are open. All of the original's solvers move this way. A move adds two squares to a walk: the slot, then the cell.
- **Arrived**: a walk has reached the end when its current square equals `end` or is orthogonally adjacent to it (Manhattan distance at most 1). For an inner end this means standing on the end cell. For an outer end it means standing on the end's entry square.
- **Distance** d(start, end): the number of unit steps (4-neighbour moves) in the shortest route from `start` to `end` through open squares, where `start` and `end` themselves may be walls. This is the definition the goldens checker uses. On every grid in the goldens it equals the distance along cell moves (plus one step for each outer entrance). A grid with open posts could in principle allow a unit-step shortcut that cell moves can't take; see Design notes.

## What a solution is
1. A solution is a list of squares lying **strictly between** `start` and `end`. Neither `start` nor `end` appears in it.
2. The first square is orthogonally adjacent to `start`, the last is orthogonally adjacent to `end`, and each square is orthogonally adjacent to the one before it. Every square in it is open. This is the checker's `paths_valid` property: the chain start, path..., end must consist of unit steps, and every path square must be open.
3. For an outer start, the first square of a valid solution is therefore the start's entry square. For an inner start, it is the wall slot next to the start cell. The same holds, mirrored, at the end.
4. A **shortest** solution has exactly d(start, end) - 1 squares.
5. If start and end are orthogonally adjacent (d = 1), the only valid solution is the empty list `[]`, and the checker accepts it. The entrance helper never places entrances this close, but a user can.
6. An empty solution list `[]` means "no solution found".

Example (goldens, grid G1 below, start (3,0) on the west border, end (1,6) on the east border, d = 8): the valid shortest solution is R1: up column 1 from (3,1) to (1,1), then along row 1 to (1,5), which is 7 squares. (3,1) is the start's entry square and (1,5) is the end's entry square.

## Behavior
7. Validation, in this order, before any solving:
   - The grid must be present (AssertionError).
   - `start` and `end` must both be present (AssertionError). Through the facade this is checked first, by `Maze.solve()`.
   - `start[0]` must be in `0..H-1`, `start[1]` in `0..W-1`, and the same for `end`. Otherwise AssertionError. Negative coordinates fail too.
   - No other validation happens. In particular the original does not check that `start`/`end` are open, aligned or on a cell, nor that they differ.
8. Solvers never modify the caller's grid. Some algorithms (Collision) work on a private copy that they mark up.
9. Randomized solvers (BacktrackingSolver, Chain, RandomMouse, Tremaux, and tie-breaking in the others) must draw only from the library's seeded RNG streams, so that one seed always gives one result. Under `cr`, how and in what order the RNG is used is the port's own design. Solver output is never compared exactly under `cr`.
10. The solver returns a list of solutions. How many it returns, and their order, is per algorithm (see the per-algorithm specs). Summary from the goldens:

| Solver | Number of solutions (original) |
|---|---|
| BacktrackingSolver, Chain, RandomMouse, Tremaux | always exactly 1 |
| Collision | 1 or more, all the same (shortest) length |
| ShortestPath | 0 or 1 (0 is a defect, see its spec) |
| ShortestPaths | 1 or more, sorted shortest first (up to 992 seen) |

### Raw output and the `prune` flag
11. The facade's `prune` (default true) post-processes each solution independently. The list keeps the same number of solutions in the same order, and empty solutions are kept. Each solution goes through two steps, described exactly in `maze.md` item 16:
    - **Loop erasure**: while the path has more than 2 squares and some square occurs twice, find the earliest position whose square occurs again later, and cut everything from there up to (not including) its next occurrence. Each cut removes a back-and-forth excursion or a loop, leaving one copy of the square. The result has no repeated squares and is still a unit-step chain.
    - **Endpoint trim**: if more than one square remains, drop the first square when it equals `start`, then drop the last square when it equals `end`.
12. So pruning removes backtracks, loops and revisits (duplicates inside one path), and it strips `start`/`end` if a raw walk included them. It does **not** remove duplicate solutions, re-sort the list, or drop longer solutions.
13. A solution that is already a valid simple path not containing `start` or `end` passes through pruning unchanged. A port whose solvers emit clean paths therefore behaves the same with and without pruning. That is allowed.
14. "Raw" (prune = false) output of the four walkers (BacktrackingSolver, Chain, RandomMouse, Tremaux) is, in the original, the full walk including backtracks. Under `cr` the only requirement on that raw output is `nonempty_when_reachable` (below). It need **not** be a valid path. The recommended raw form, which matches the original, is:
    - It begins at the start's entry square, except that Tremaux begins at `start` itself and then its entry square when the start is outer.
    - It contains every square visited, slot and cell, in visiting order, including revisits.
    - It ends at the first square where the walker has arrived. For an inner end, that is the end cell itself. For an outer end, it is the end's entry square.
    - For an inner start, the first square is the start cell itself.
    - Loop erasure plus endpoint trim then turns this into a valid solution.
15. Collision, ShortestPath and ShortestPaths must emit valid paths even when prune is false (their noprune invariants include `paths_valid`).

### Unreachable end
16. The goldens contain no unreachable case. Original behavior, for reference: the four random walkers never terminate when the end is unreachable but the start has somewhere to go, and fail with IndexError when the start's entry square has no cell move at all. Collision returns `[]`. ShortestPath returns `[]`, or AssertionError if the start's entry square has no cell move. ShortestPaths fails with AssertionError ("no valid solutions").
17. Port contract: every solver must **terminate** on every valid input. When the end is unreachable, the walkers, Collision and ShortestPath return `[]`. ShortestPaths may return `[]` or an AssertionError-mapped error; the recommendation is to keep the original's AssertionError for parity. Returning an error for the walkers instead of hanging is a documented deviation (record it in `metrics.json` deviations).

## Invariants (profile `cr`, from `invariants.json`)
Checked per case by `checkers/invariants.py` using the properties defined in "What a solution is":
- `paths_valid`: every returned solution satisfies item 2.
- `first_is_shortest`: if the list is non-empty and the end is reachable, the first solution has d - 1 squares.
- `all_shortest`: every solution has d - 1 squares (when reachable).
- `nonempty_when_reachable`: if the end is reachable, the list is non-empty and every solution is non-empty (an empty solution is accepted only when d = 1).

| Key | Required properties |
|---|---|
| BacktrackingSolver, Chain (either turn), RandomMouse, Tremaux, with prune | nonempty_when_reachable, paths_valid |
| the same four, without prune | nonempty_when_reachable |
| Collision, prune or not | all_shortest, first_is_shortest, nonempty_when_reachable, paths_valid |
| ShortestPath, prune or not | all_shortest, first_is_shortest, paths_valid (non-emptiness is not required, but recommended) |
| ShortestPaths, prune or not | first_is_shortest, nonempty_when_reachable, paths_valid |

Known bugs (goldens `KNOWN_BUGS.json`): the original's Collision crashes with TypeError on the CellularAutomaton6x6 outer-entrance cases and on all DungeonRooms10x10 cases (mazes with loops or open rooms). There the port must return a valid result that satisfies Collision's invariants. It must not return an error.

## Errors
| Condition | Error kind | Contractual message? |
|---|---|---|
| no solver set (facade) | AssertionError | no |
| start or end not set | AssertionError | no |
| grid not set | AssertionError | no |
| start or end row outside 0..H-1, or column outside 0..W-1 | AssertionError | no |
| ShortestPath / ShortestPaths: start's entry square has no cell move | AssertionError | no |
| ShortestPaths: no solution found | AssertionError (recommended) | no |

The goldens' `solve` set contains no invalid-input cases. All 384 cases are valid input, and 12 of them are the Collision known bugs. Messages are never compared.

## Edge cases
| Input | Expected |
|---|---|
| start == end | not covered; return `[]` or a single empty solution, never panic |
| start adjacent to end (d = 1) | a single empty solution `[]` inside the list is the valid answer |
| outer entrance on a corner, or misaligned, or inner entrance not on a cell | not covered; implementation-defined, never panic |
| start/end on an open border square | treat like any outer entrance |
| end unreachable | see item 17 |

## Worked examples (goldens `solve` set)
Grid G1 is the 7x7 `grid` field of case `solve/BacktrackingSolver/Prims3x3/oo/p/s0` in the goldens' `cases/solve.jsonl` (a 3x3-cell Prims maze; every Prims3x3 solve and transmute case uses it). In words: row 1 is open from column 1 to 5; column 1 is open from row 1 to row 5; row 5 is open from column 1 to 5; column 5 is open from row 3 to row 5; and (3,2), (3,3) form a stub off column 1. Everything else is wall.

- Outer entrances, start (3,0), end (1,6), d = 8. Every solver with prune returns exactly one solution, R1, for both seeds 0 and 1 (cases `solve/<Algo>/Prims3x3/oo/p/s0`).
- Inner entrances, start (5,3), end (3,1), d = 4. The shortest solution is (5,2), (5,1), (4,1) (cases `.../Prims3x3/ii/p/s0`).
- Raw walk example, `solve/BacktrackingSolver/Prims3x3/ii/np/s0`: (5,3), (5,2), (5,1), (4,1), (3,1). It starts on the inner start and ends on the inner end. Pruning trims both ends to (5,2), (5,1), (4,1).
- Raw walk with a backtrack, `solve/BacktrackingSolver/Prims3x3/oo/np/s0` (27 squares): it begins at (3,1), runs down to (5,1), along row 5 to (5,5), up to the dead end (3,5), back down and along row 5 to (5,1), back up to (3,1), into the dead end (3,3) and back, then (2,1), (1,1), and along row 1 to (1,5). Loop erasure reduces it to the 7-square solution above.
- Tremaux raw walk with an outer start, `solve/Tremaux/Prims3x3/oo/np/s0`: it begins (3,0), (3,1), ..., so the start itself is the first square and pruning trims it.

## Design notes for the port (non-normative)
- A breadth-first search over the unit-step graph of open squares (start and end allowed as endpoints even when they are walls) yields paths that satisfy `paths_valid` and the shortness properties on any grid, including grids with open posts. A search over cell moves gives the same answers on all golden grids.
- The walkers may keep the original's cell-move semantics. On golden grids, cell moves reach everything unit steps reach.

## Open questions
None yet.
