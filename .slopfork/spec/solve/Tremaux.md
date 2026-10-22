# Spec: solve/Tremaux

Provenance: documented (docs/MAZE_SOLVE_ALGOS.md, "Trémaux's Algorithm") + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

Read `solve/_common.md` first.

## Purpose
Trémaux's classic marking algorithm (19th century, described by Lucas 1882; a depth-first strategy a human can carry out inside the maze). The walker marks each cell it visits and uses the marks to prefer unexplored passages and to avoid re-walking explored ones. It returns one solution, generally not the shortest, and it works on imperfect mazes.

## Public interface
- `Tremaux()`: no options.
- `solve(grid, start, end) -> list<list<cell>>` (protocol algo `Tremaux`, `params: {}`).

## Behavior
1. Always returns exactly **one** solution when the end is reachable.
2. The walk is recorded starting **at `start` itself**. If the start is an outer entrance, its entry square comes next. So for an outer start the raw walk begins with the border square, then the entry square. For an inner start it begins with the start cell. Each recorded cell gets a visit mark.
3. The walker moves by cell moves until it has arrived (current square equal or adjacent to `end`). Each move records the slot and then the new cell, and increments that cell's visit count.
4. Choice rule at each cell, with candidates being the available cell moves. This is a faithful reading of the documented rules, and the original behaves this way:
   - With only one candidate, take it (dead end: turn around; corridor: continue).
   - Otherwise, group the candidates by their visit count, treating 2 or more visits as "2".
   - If some candidates were never visited, pick uniformly among those.
   - Otherwise, if some were visited exactly once, pick uniformly among those. If that group has more than one member, first remove the cell the walker just came from.
   - Otherwise, pick among the twice-visited ones, again removing the cell just come from when others remain.
5. The original keeps the raw walk and does **not** perform the documented "follow the cells marked once back to the start" step. The facade's pruning plays that role. With prune = true the result is the loop-erased walk with `start`/`end` trimmed, which is a valid simple path. Under `cr`, a port may instead extract the route the documented way (cells marked exactly once), as long as the invariants hold.
6. Raw output (prune = false): the whole walk as described in items 2 and 3. It ends at the arrival square: the end cell for an inner end, the end's entry square for an outer end.
7. Unreachable end: the original keeps walking forever. The port must terminate and return `[]`. Trémaux's rule naturally detects this: the walker gets back to the start with every passage marked twice.

## Invariants (cr)
- With prune: `nonempty_when_reachable`, `paths_valid`.
- Without prune: `nonempty_when_reachable`.

## Errors
As in `_common.md`. No others. (With an isolated entry square the original fails with IndexError; the port should return `[]`.)

## Edge cases
| Input | Expected |
|---|---|
| outer start | raw walk's first square is `start` (a border wall square), second is the entry square |
| inner start | raw walk's first square is `start` |
| end unreachable | `[]` (port) |

## Worked examples (goldens)
Grid G1 rows: `#######`, `#.....#`, `#.#####`, `#...#.#`, `#.###.#`, `#.....#`, `#######`.
- `solve/Tremaux/Prims3x3/oo/np/s0`: start (3,0), end (1,6). One raw walk of 28 squares: (3,0), (3,1), (4,1), (5,1), then along row 5 to (5,5), up to the dead end (3,5) and back to (5,5), back along row 5 to (5,1), up to (3,1), east into the dead end (3,3) and back to (3,1), then (2,1), (1,1), (1,2), (1,3), (1,4), (1,5).
- `solve/Tremaux/Prims3x3/oo/p/s0`: pruned `[[ (3,1), (2,1), (1,1), (1,2), (1,3), (1,4), (1,5) ]]`.
- `solve/Tremaux/Prims3x3/ii/np/s0`: start (5,3), end (3,1). Raw `[[ (5,3), (5,2), (5,1), (4,1), (3,1) ]]`.

## Open questions
None yet.
