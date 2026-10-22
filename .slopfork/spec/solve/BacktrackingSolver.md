# Spec: solve/BacktrackingSolver

Provenance: documented (docs/MAZE_SOLVE_ALGOS.md, "Recursive Backtracker") + observed (goldens) + derived-from-source (cleanroom spec-writer) · Audit: clean @ 2026-10-22

Read `solve/_common.md` first for the vocabulary (cell move, entry square, arrived, distance), the solution format, pruning and errors.

## Purpose
A random-walk solver that never immediately reverses unless it has to. The documented algorithm: pick a random direction and follow it, and backtrack only when you hit a dead end. It returns one solution, generally not the shortest, and it works on imperfect mazes (loops, rooms).

## Public interface
- `BacktrackingSolver()`: no options.
- `solve(grid, start, end) -> list<list<cell>>` (via `Maze.solve()`; protocol algo name `BacktrackingSolver`, `params: {}`).

## Behavior
1. Always returns a list with exactly **one** solution when the end is reachable.
2. The walk starts on the start's entry square (`_common.md`) and moves only by cell moves.
3. At each step the walker considers the cell moves available from its current cell. If there is more than one, it excludes the cell it arrived from (so it never reverses at a junction or in a corridor). It then picks uniformly at random among the rest. If the only available move is back the way it came (a dead end), it takes it.
4. There is no longer-term memory. The walker can revisit cells and can go around a loop more than once, so the raw walk may be long. It terminates with probability 1 on a connected maze.
5. It stops as soon as it has arrived (current square equal or adjacent to `end`).
6. Raw output (prune = false): the whole walk, both slots and cells, including backtracks and revisits. It starts with the entry square (which is the start cell itself for an inner start) and ends with the arrival square. With prune = true, loop erasure plus endpoint trim (`_common.md` item 11) gives a valid simple path.
7. Distribution note: the solution after pruning is the loop-erased trace of a non-reversing random walk. On a perfect maze that is always the unique route. On mazes with loops it can be any simple route, and it is not necessarily the shortest.
8. When the end is unreachable the original hangs, or raises IndexError if the start's entry square has no move at all. The port must terminate and return `[]` (see `_common.md` item 17). A simple way is to bound the walk, or check reachability first.

## Invariants (cr)
- With prune: `nonempty_when_reachable`, `paths_valid`.
- Without prune: `nonempty_when_reachable` (the raw walk is non-empty).
- Exactly one solution whenever reachable (recommended; it matches the original).

## Errors
As in `_common.md` (AssertionError for a missing grid, start or end, or an entrance outside the grid). No others.

## Edge cases
| Input | Expected |
|---|---|
| perfect maze, any entrances | after pruning, the unique route |
| inner start, inner end | raw walk begins with `start` and ends with `end`; pruning trims both |
| end unreachable | `[]` (port); the original hangs |

## Worked examples (goldens)
Grid G1 rows: `#######`, `#.....#`, `#.#####`, `#...#.#`, `#.###.#`, `#.....#`, `#######`.
- `solve/BacktrackingSolver/Prims3x3/oo/p/s0`: start (3,0), end (1,6), prune on. Result `[[ (3,1), (2,1), (1,1), (1,2), (1,3), (1,4), (1,5) ]]`.
- `solve/BacktrackingSolver/Prims3x3/oo/np/s0`: same maze, prune off. Result: one 27-square raw walk. From (3,1) it heads down column 1 and along row 5 to (5,5), up into the dead end (3,5), back along row 5 to (5,1), up to (3,1), into the dead end (3,3) and back, then up column 1 and along row 1 to (1,5). It reverses only at the dead ends (3,5) and (3,3).
- `solve/BacktrackingSolver/Prims3x3/ii/np/s0`: start (5,3), end (3,1). Raw `[[ (5,3), (5,2), (5,1), (4,1), (3,1) ]]`, which prunes to `[[ (5,2), (5,1), (4,1) ]]`.

## Open questions
None yet.
